//! Simple Parser Cache for Marco Engine
//!
//! Provides basic AST and HTML caching as per optimization spec:
//! - Cache parsed AST nodes to avoid repeated parsing
//! - Cache rendered HTML to avoid repeated rendering
//! - Use content hash for cache invalidation 
//! - LRU cache for efficient O(1) eviction and access
//! - No complex block-level or incremental features

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock, LazyLock};
use lru::LruCache;
use std::num::NonZeroUsize;
use anyhow::Result;

use crate::components::marco_engine::ast_node::Node;
use crate::components::marco_engine::render_html::HtmlOptions;
use crate::components::marco_engine::{parse_text, build_ast, render_html};

/// Simple content hash type
type ContentHash = u64;

/// Cache size limits to prevent memory exhaustion
const MAX_AST_CACHE_ENTRIES: usize = 500;
const MAX_HTML_CACHE_ENTRIES: usize = 1000;
const MAX_CONTENT_SIZE_BYTES: usize = 1024 * 1024; // 1MB per content item

/// Calculate hash of content for cache key
fn calculate_hash(content: &str) -> ContentHash {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Cached AST node (no manual LRU tracking needed)
#[derive(Debug, Clone)]
pub struct CachedAst {
    pub node: Node,
}

impl CachedAst {
    pub fn new(node: Node, _content_hash: ContentHash) -> Self {
        Self { node }
    }
}

/// Cached HTML (no manual LRU tracking needed)
#[derive(Debug, Clone)]
pub struct CachedHtml {
    pub html: String,
}

impl CachedHtml {
    pub fn new(html: String, _content_hash: ContentHash, _options_hash: ContentHash) -> Self {
        Self { html }
    }
}

/// Simple parser cache with LRU eviction (as per spec)
pub struct SimpleParserCache {
    /// AST cache: content hash -> cached AST
    ast_cache: Arc<RwLock<LruCache<ContentHash, CachedAst>>>,
    /// HTML cache: (content hash, options hash) -> cached HTML
    html_cache: Arc<RwLock<LruCache<(ContentHash, ContentHash), CachedHtml>>>,
    /// Cache statistics tracking
    stats: Arc<RwLock<ParserCacheStats>>,
}

impl Default for SimpleParserCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleParserCache {
    /// Create new simple parser cache
    pub fn new() -> Self {
        Self {
            ast_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(MAX_AST_CACHE_ENTRIES).unwrap()))),
            html_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(MAX_HTML_CACHE_ENTRIES).unwrap()))),
            stats: Arc::new(RwLock::new(ParserCacheStats::new())),
        }
    }
    
    /// Parse content with AST caching
    pub fn parse_with_cache(&self, content: &str) -> Result<Node> {
        let content_hash = calculate_hash(content);
        
        // Check cache first - LRU cache automatically updates access order
        {
            if let Ok(mut cache) = self.ast_cache.write() {
                if let Some(cached) = cache.get(&content_hash) {
                    // Cache hit - LRU automatically moved to front
                    if let Ok(mut stats) = self.stats.write() {
                        stats.ast_hits += 1;
                    }
                    return Ok(cached.node.clone());
                }
            }
        }
        
        // Cache miss - increment counter and parse
        if let Ok(mut stats) = self.stats.write() {
            stats.ast_misses += 1;
        }
        
        let pairs = parse_text(content)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        
        let ast = build_ast(pairs)
            .map_err(|e| anyhow::anyhow!("AST build error: {}", e))?;
        
        // Add to cache - LRU automatically handles eviction when capacity exceeded
        let cached_ast = CachedAst::new(ast.clone(), content_hash);
        if let Ok(mut cache) = self.ast_cache.write() {
            // Check content size before caching
            if content.len() > MAX_CONTENT_SIZE_BYTES {
                log::warn!("Content too large for caching: {} bytes", content.len());
                return Ok(ast);
            }
            
            // LRU cache automatically evicts least recently used when capacity exceeded
            cache.put(content_hash, cached_ast);
            
            // Update stats entry count
            if let Ok(mut stats) = self.stats.write() {
                stats.ast_entries = cache.len();
            }
        }
        
        Ok(ast)
    }
    
    /// Render content with HTML caching
    pub fn render_with_cache(&self, content: &str, options: HtmlOptions) -> Result<String> {
        let content_hash = calculate_hash(content);
        let options_hash = self.hash_options(&options);
        let cache_key = (content_hash, options_hash);
        
        // Check cache first - LRU cache automatically updates access order
        {
            if let Ok(mut cache) = self.html_cache.write() {
                if let Some(cached) = cache.get(&cache_key) {
                    // Cache hit - LRU automatically moved to front
                    if let Ok(mut stats) = self.stats.write() {
                        stats.html_hits += 1;
                    }
                    return Ok(cached.html.clone());
                }
            }
        }
        
        // Cache miss - increment counter and render
        if let Ok(mut stats) = self.stats.write() {
            stats.html_misses += 1;
        }
        
        let ast = self.parse_with_cache(content)?;
        let html = render_html(&ast, options.clone());
        
        // Add to cache - LRU automatically handles eviction when capacity exceeded
        let cached_html = CachedHtml::new(html.clone(), content_hash, options_hash);
        if let Ok(mut cache) = self.html_cache.write() {
            // Check content size before caching
            if content.len() > MAX_CONTENT_SIZE_BYTES {
                log::warn!("Content too large for HTML caching: {} bytes", content.len());
                return Ok(html);
            }
            
            // LRU cache automatically evicts least recently used when capacity exceeded
            cache.put(cache_key, cached_html);
            
            // Update stats entry count
            if let Ok(mut stats) = self.stats.write() {
                stats.html_entries = cache.len();
            }
        }
        
        Ok(html)
    }
    
    /// Hash HTML options for cache key
    fn hash_options(&self, options: &HtmlOptions) -> ContentHash {
        let mut hasher = DefaultHasher::new();
        // Hash the relevant fields of HtmlOptions
        options.syntax_highlighting.hash(&mut hasher);
        options.css_classes.hash(&mut hasher);
        options.inline_styles.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Clear all cached entries to free memory
    /// This is called during application shutdown to prevent memory retention
    pub fn clear(&self) {
        log::info!("Clearing parser cache");
        
        let mut cleared_ast = 0;
        let mut cleared_html = 0;
        
        // Clear AST cache
        if let Ok(mut cache) = self.ast_cache.write() {
            cleared_ast = cache.len();
            cache.clear();
        }
        
        // Clear HTML cache
        if let Ok(mut cache) = self.html_cache.write() {
            cleared_html = cache.len();
            cache.clear();
        }
        
        // Reset statistics
        if let Ok(mut stats) = self.stats.write() {
            *stats = ParserCacheStats::new();
        }
        
        log::info!("Parser cache cleared: {} AST entries, {} HTML entries", cleared_ast, cleared_html);
    }
    
    /// Get cache statistics (used by test files)
    #[allow(dead_code)]
    pub fn stats(&self) -> ParserCacheStats {
        let ast_entries = if let Ok(cache) = self.ast_cache.read() {
            cache.len()
        } else {
            0
        };
        
        let html_entries = if let Ok(cache) = self.html_cache.read() {
            cache.len()
        } else {
            0
        };
        
        // Return actual statistics with current entry counts
        if let Ok(stats) = self.stats.read() {
            let mut result = stats.clone();
            result.ast_entries = ast_entries;
            result.html_entries = html_entries;
            result
        } else {
            // Fallback if stats can't be read
            ParserCacheStats {
                ast_hits: 0,
                ast_misses: 0,
                html_hits: 0,
                html_misses: 0,
                ast_entries,
                html_entries,
            }
        }
    }
}

/// Simple cache statistics (as per spec - no complex tracking)
#[derive(Debug, Clone, Default)]
pub struct ParserCacheStats {
    pub ast_hits: usize,
    pub ast_misses: usize,
    pub html_hits: usize,
    pub html_misses: usize,
    pub ast_entries: usize,
    pub html_entries: usize,
}

impl ParserCacheStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get AST cache hit rate as percentage (0.0-1.0, used by test files)
    #[allow(dead_code)]
    pub fn ast_hit_rate(&self) -> f64 {
        if self.ast_hits + self.ast_misses == 0 {
            0.0
        } else {
            self.ast_hits as f64 / (self.ast_hits + self.ast_misses) as f64
        }
    }

    /// Get HTML cache hit rate as percentage (0.0-1.0, used by test files)
    #[allow(dead_code)]
    pub fn html_hit_rate(&self) -> f64 {
        if self.html_hits + self.html_misses == 0 {
            0.0
        } else {
            self.html_hits as f64 / (self.html_hits + self.html_misses) as f64
        }
    }
}

/// Global parser cache instance (singleton pattern as per spec)
static GLOBAL_PARSER_CACHE: LazyLock<SimpleParserCache> = LazyLock::new(|| {
    SimpleParserCache::new()
});

/// Get global parser cache instance
pub fn global_parser_cache() -> &'static SimpleParserCache {
    &GLOBAL_PARSER_CACHE
}

/// Shutdown and cleanup the global parser cache
/// This clears all cached data to prevent memory retention on application exit
pub fn shutdown_global_parser_cache() {
    global_parser_cache().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_parser_cache() {
        let cache = SimpleParserCache::new();
        
        // Test basic markdown content
        let content = "# Hello World\n\nThis is a **test** document.";
        
        // Test AST caching - first call should be cache miss
        let ast1 = cache.parse_with_cache(content).expect("Failed to parse content");
        let stats_after_first = cache.stats();
        assert_eq!(stats_after_first.ast_misses, 1);
        assert_eq!(stats_after_first.ast_hits, 0);
        assert_eq!(stats_after_first.ast_entries, 1);
        
        // Second call should be cache hit
        let ast2 = cache.parse_with_cache(content).expect("Failed to parse content");
        let stats_after_second = cache.stats();
        assert_eq!(stats_after_second.ast_misses, 1);
        assert_eq!(stats_after_second.ast_hits, 1);
        assert_eq!(stats_after_second.ast_entries, 1);
        
        // AST nodes should be identical (cloned from cache)
        assert_eq!(format!("{:?}", ast1), format!("{:?}", ast2));
        
        // Test HTML caching
        let options = HtmlOptions::default();
        
        // First render should be cache miss
        let html1 = cache.render_with_cache(content, options.clone()).expect("Failed to render HTML");
        let stats_after_html1 = cache.stats();
        assert_eq!(stats_after_html1.html_misses, 1);
        assert_eq!(stats_after_html1.html_hits, 0);
        assert_eq!(stats_after_html1.html_entries, 1);
        
        // Second render should be cache hit
        let html2 = cache.render_with_cache(content, options.clone()).expect("Failed to render HTML");
        let stats_after_html2 = cache.stats();
        assert_eq!(stats_after_html2.html_misses, 1);
        assert_eq!(stats_after_html2.html_hits, 1);
        assert_eq!(stats_after_html2.html_entries, 1);
        
        // HTML should be identical
        assert_eq!(html1, html2);
        assert!(html1.contains("Hello World"));
        assert!(html1.contains("<strong>test</strong>"));
        
        // Test cache clearing
        cache.clear();
        let stats_after_clear = cache.stats();
        assert_eq!(stats_after_clear.ast_hits, 0);
        assert_eq!(stats_after_clear.ast_misses, 0);
        assert_eq!(stats_after_clear.html_hits, 0);
        assert_eq!(stats_after_clear.html_misses, 0);
        assert_eq!(stats_after_clear.ast_entries, 0);
        assert_eq!(stats_after_clear.html_entries, 0);
    }
    
    #[test]
    fn smoke_test_hit_rates() {
        let cache = SimpleParserCache::new();
        
        let content1 = "# First Document";
        let content2 = "# Second Document";
        
        // Parse both documents once (2 misses)
        cache.parse_with_cache(content1).expect("Parse failed");
        cache.parse_with_cache(content2).expect("Parse failed");
        
        // Parse first document again (1 hit)
        cache.parse_with_cache(content1).expect("Parse failed");
        
        let stats = cache.stats();
        assert_eq!(stats.ast_misses, 2);
        assert_eq!(stats.ast_hits, 1);
        
        // Check hit rate calculation
        let expected_rate = 1.0 / 3.0; // 1 hit out of 3 total accesses
        let actual_rate = stats.ast_hit_rate();
        assert!((actual_rate - expected_rate).abs() < f64::EPSILON);
    }
    
    #[test]
    fn smoke_test_global_cache() {
        // Test global cache access
        let cache = global_parser_cache();
        
        let content = "# Global Cache Test";
        let result = cache.parse_with_cache(content);
        assert!(result.is_ok());
        
        // Stats should be accessible
        let stats = cache.stats();
        // Just verify we can access the stats without checking the value
        let _ = stats.ast_entries;
    }
    
    #[test]
    fn smoke_test_different_options() {
        let cache = SimpleParserCache::new();
        let content = "# Test Document";
        
        let options1 = HtmlOptions {
            syntax_highlighting: true,
            ..HtmlOptions::default()
        };
        
        let options2 = HtmlOptions {
            syntax_highlighting: false,
            ..HtmlOptions::default()
        };
        
        // Same content, different options should create separate cache entries
        let html1 = cache.render_with_cache(content, options1).expect("Render failed");
        let html2 = cache.render_with_cache(content, options2).expect("Render failed");
        
        let stats = cache.stats();
        assert_eq!(stats.html_entries, 2); // Should have 2 separate HTML cache entries
        assert_eq!(stats.html_misses, 2);  // Both should be cache misses
        assert_eq!(stats.html_hits, 0);    // No hits yet
        
        // HTML content might be the same but they're cached separately
        assert_eq!(html1, html2); // For this simple case, output should be same
    }

    #[test]
    fn smoke_test_global_cache_cleanup() {
        // Populate global cache
        let content = "# Global Cache Cleanup Test\n\nTesting issue #16 fix.";
        let _ast = global_parser_cache().parse_with_cache(content).expect("Parse failed");
        
        // Verify global cache has entries
        let stats_before = global_parser_cache().stats();
        assert!(stats_before.ast_entries > 0, "Global cache should have entries before cleanup");
        
        // Test global cleanup - this is the main focus of issue #16
        shutdown_global_parser_cache();
        
        // Verify global cache is empty after cleanup
        let stats_after = global_parser_cache().stats();
        assert_eq!(stats_after.ast_entries, 0, "Global cache should be empty after shutdown");
        assert_eq!(stats_after.ast_hits, 0, "Global statistics should be reset");
        assert_eq!(stats_after.ast_misses, 0, "Global statistics should be reset");
        assert_eq!(stats_after.html_entries, 0, "Global HTML cache should be empty");
        
        // Verify global cache still works after cleanup
        let _ast2 = global_parser_cache().parse_with_cache(content).expect("Parse should work after cleanup");
        let stats_final = global_parser_cache().stats();
        assert_eq!(stats_final.ast_misses, 1, "Should work normally after cleanup");
        assert!(stats_final.ast_entries > 0, "Cache should populate again after cleanup");
    }
}