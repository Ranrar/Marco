//! Simple Parser Cache for Marco Engine
//!
//! Provides basic AST and HTML caching as per optimization spec:
//! - Cache parsed AST nodes to avoid repeated parsing
//! - Cache rendered HTML to avoid repeated rendering
//! - Use content hash for cache invalidation 
//! - HashMap-based storage for simplicity
//! - No complex block-level or incremental features

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock, LazyLock};
use anyhow::Result;

use crate::components::marco_engine::ast_node::Node;
use crate::components::marco_engine::render_html::HtmlOptions;
use crate::components::marco_engine::{parse_text, build_ast, render_html};

/// Simple content hash type
type ContentHash = u64;

/// Calculate hash of content for cache key
fn calculate_hash(content: &str) -> ContentHash {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Cached AST node - simplified structure  
#[derive(Debug, Clone)]
pub struct CachedAst {
    pub node: Node,
}

impl CachedAst {
    pub fn new(node: Node, _content_hash: ContentHash) -> Self {
        Self { node }
    }
}

/// Cached HTML - simplified structure
#[derive(Debug, Clone)]
pub struct CachedHtml {
    pub html: String,
}

impl CachedHtml {
    pub fn new(html: String, _content_hash: ContentHash, _options_hash: ContentHash) -> Self {
        Self { html }
    }
}

/// Simple parser cache with basic HashMap storage (as per spec)
pub struct SimpleParserCache {
    /// AST cache: content hash -> cached AST
    ast_cache: Arc<RwLock<HashMap<ContentHash, CachedAst>>>,
    /// HTML cache: (content hash, options hash) -> cached HTML
    html_cache: Arc<RwLock<HashMap<(ContentHash, ContentHash), CachedHtml>>>,
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
            ast_cache: Arc::new(RwLock::new(HashMap::new())),
            html_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ParserCacheStats::new())),
        }
    }
    
    /// Parse content with AST caching
    pub fn parse_with_cache(&self, content: &str) -> Result<Node> {
        let content_hash = calculate_hash(content);
        
        // Check cache first
        {
            if let Ok(cache) = self.ast_cache.read() {
                if let Some(cached) = cache.get(&content_hash) {
                    // Cache hit - increment counter and return cached AST
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
        
        // Add to cache and update entry count
        let cached_ast = CachedAst::new(ast.clone(), content_hash);
        if let Ok(mut cache) = self.ast_cache.write() {
            cache.insert(content_hash, cached_ast);
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
        
        // Check cache first
        {
            if let Ok(cache) = self.html_cache.read() {
                if let Some(cached) = cache.get(&cache_key) {
                    // Cache hit - increment counter and return cached HTML
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
        
        // Add to cache and update entry count
        let cached_html = CachedHtml::new(html.clone(), content_hash, options_hash);
        if let Ok(mut cache) = self.html_cache.write() {
            cache.insert(cache_key, cached_html);
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
    
    /// Clear all cached entries (used by test files)
    pub fn clear(&self) {
        if let Ok(mut cache) = self.ast_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.html_cache.write() {
            cache.clear();
        }
        // Reset statistics
        if let Ok(mut stats) = self.stats.write() {
            *stats = ParserCacheStats::new();
        }
    }
    
    /// Get cache statistics (used by test files)
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

    pub fn ast_hit_rate(&self) -> f64 {
        if self.ast_hits + self.ast_misses == 0 {
            0.0
        } else {
            self.ast_hits as f64 / (self.ast_hits + self.ast_misses) as f64
        }
    }
    
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

/// Type alias for backward compatibility
pub type MarcoParserCache = SimpleParserCache;

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
        
        let mut options1 = HtmlOptions::default();
        options1.syntax_highlighting = true;
        
        let mut options2 = HtmlOptions::default();
        options2.syntax_highlighting = false;
        
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
}