//! Simple Parser Cache for Marco Engine
//!
//! Provides basic AST caching as per optimization spec:
//! - Cache parsed AST nodes to avoid repeated parsing
//! - Use content hash for cache invalidation 
//! - LRU cache for efficient O(1) eviction and access
//! - Note: HTML is NOT cached to support reference link resolution across sections

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

/// Section cache key: (section_hash, section_index)
type SectionCacheKey = (ContentHash, usize);

/// Cache size limits to prevent memory exhaustion
const MAX_SECTION_CACHE_ENTRIES: usize = 2000; // AST sections cache

/// Split document into logical sections for caching
/// Sections are split on double newlines (paragraph breaks) and headers
fn split_into_sections(content: &str) -> Vec<String> {
    // For now, split on double newlines (paragraph breaks)
    // This can be enhanced to split on headers later
    let sections: Vec<String> = content
        .split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .collect();
        
    // If no double newlines found, treat as single section
    if sections.is_empty() {
        vec![content.to_string()]
    } else {
        sections
    }
}

/// Calculate hash of content for cache key
fn calculate_hash(content: &str) -> ContentHash {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}



/// Cached section with AST (HTML is not cached to support reference resolution)
#[derive(Debug, Clone)]
pub struct CachedSection {
    pub ast: Node,
}

impl CachedSection {
    pub fn new(ast: Node) -> Self {
        Self { ast }
    }
}

/// Simple parser cache with LRU eviction (as per spec)
pub struct SimpleParserCache {
    /// Section cache: (section_hash, section_index) -> cached section data
    section_cache: Arc<RwLock<LruCache<SectionCacheKey, CachedSection>>>,
    /// Cache statistics tracking
    stats: Arc<RwLock<ParserCacheStats>>,
}

impl Default for SimpleParserCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleParserCache {
    /// Create new simple parser cache with sectioned caching
    pub fn new() -> Self {
        Self {
            section_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(MAX_SECTION_CACHE_ENTRIES).unwrap()))),
            stats: Arc::new(RwLock::new(ParserCacheStats::new())),
        }
    }
    
    /// Parse content with sectioned AST caching
    pub fn parse_with_cache(&self, content: &str) -> Result<Node> {
        // For small content, parse directly without sectioning overhead
        if content.len() < 1024 {
            return self.parse_directly(content);
        }
        
        let sections = split_into_sections(content);
        let mut combined_ast_nodes = Vec::new();
        let mut cache_hits = 0;
        let mut cache_misses = 0;
        
        // Process each section
        for (section_index, section_content) in sections.iter().enumerate() {
            let section_hash = calculate_hash(section_content);
            let cache_key = (section_hash, section_index);
            
            // Check section cache
            let section_ast = {
                if let Ok(mut cache) = self.section_cache.write() {
                    if let Some(cached_section) = cache.get(&cache_key) {
                        cache_hits += 1;
                        cached_section.ast.clone()
                    } else {
                        cache_misses += 1;
                        
                        // Parse section
                        let section_pairs = parse_text(section_content)
                            .map_err(|e| anyhow::anyhow!("Section parse error: {}", e))?;
                        let section_ast = build_ast(section_pairs)
                            .map_err(|e| anyhow::anyhow!("Section AST build error: {}", e))?;
                        
                        // Cache the section AST
                        let cached_section = CachedSection::new(section_ast.clone());
                        cache.put(cache_key, cached_section);
                        section_ast
                    }
                } else {
                    // Fallback if cache lock fails
                    let section_pairs = parse_text(section_content)
                        .map_err(|e| anyhow::anyhow!("Section parse error: {}", e))?;
                    build_ast(section_pairs)
                        .map_err(|e| anyhow::anyhow!("Section AST build error: {}", e))?
                }
            };
            
            combined_ast_nodes.push(section_ast);
        }
        
        // Update statistics
        if let Ok(mut stats) = self.stats.write() {
            stats.ast_hits += cache_hits;
            stats.ast_misses += cache_misses;
        }
        
        // Combine all section ASTs into a single document AST
        // For now, create a simple document node containing all sections
        use crate::components::marco_engine::ast_node::Span;
        Ok(Node::Document { 
            children: combined_ast_nodes,
            span: Span { 
                start: 0, 
                end: content.len() as u32,
                line: 1,
                column: 1
            }
        })
    }
    
    /// Parse small content directly without sectioning
    fn parse_directly(&self, content: &str) -> Result<Node> {
        let pairs = parse_text(content)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        
        let ast = build_ast(pairs)
            .map_err(|e| anyhow::anyhow!("AST build error: {}", e))?;
        
        // Update statistics
        if let Ok(mut stats) = self.stats.write() {
            stats.ast_misses += 1;
        }
        
        Ok(ast)
    }
    
    /// Render content with caching
    /// 
    /// Note: Reference links/images require full document context for resolution.
    /// This means we must:
    /// 1. Parse the full document (AST is cached per section - fast)
    /// 2. Render the complete AST in one pass (HTML not cached - required for references)
    /// 
    /// The expensive parsing operation is still cached, but HTML rendering is done
    /// fresh each time to ensure reference definitions can be resolved across the
    /// entire document, even when references and definitions are in different sections.
    pub fn render_with_cache(&self, content: &str, options: HtmlOptions) -> Result<String> {
        // For small content, render directly without sectioning overhead
        if content.len() < 1024 {
            return self.render_directly(content, options);
        }
        
        // Parse the full document (with AST caching per section)
        // This ensures all reference definitions are available
        let full_ast = self.parse_with_cache(content)?;
        
        // Render the complete AST
        // This allows reference resolution to work across the entire document
        let html = render_html(&full_ast, options);
        
        Ok(html)
    }
    
    /// Render small content directly without sectioning
    fn render_directly(&self, content: &str, options: HtmlOptions) -> Result<String> {
        let pairs = parse_text(content)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        
        let ast = build_ast(pairs)
            .map_err(|e| anyhow::anyhow!("AST build error: {}", e))?;
        
        let html = render_html(&ast, options);
        
        Ok(html)
    }
    

    
    /// Clear all cached entries to free memory
    /// This is called during application shutdown to prevent memory retention
    pub fn clear(&self) {
        log::info!("Clearing sectioned parser cache");
        
        let mut cleared_sections = 0;
        
        // Clear section cache
        if let Ok(mut cache) = self.section_cache.write() {
            cleared_sections = cache.len();
            cache.clear();
        }
        
        // Reset statistics
        if let Ok(mut stats) = self.stats.write() {
            *stats = ParserCacheStats::new();
        }
        
        log::info!("Sectioned parser cache cleared: {} section entries", cleared_sections);
    }
    
    /// Get cache statistics (used by test files)
    #[allow(dead_code)]
    pub fn stats(&self) -> ParserCacheStats {
        let section_entries = if let Ok(cache) = self.section_cache.read() {
            cache.len()
        } else {
            0
        };
        
        // Return actual statistics with current entry counts
        if let Ok(stats) = self.stats.read() {
            let mut result = stats.clone();
            result.ast_entries = section_entries;
            result
        } else {
            // Fallback if stats can't be read
            ParserCacheStats {
                ast_hits: 0,
                ast_misses: 0,
                ast_entries: section_entries,
            }
        }
    }
}

/// Simple cache statistics (as per spec - no complex tracking)
#[derive(Debug, Clone, Default)]
pub struct ParserCacheStats {
    pub ast_hits: usize,
    pub ast_misses: usize,
    pub ast_entries: usize,
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
        
        // Test basic markdown content (use large content to test sectioning)
        let content = "# Hello World\n\nThis is a **test** document with enough content to trigger sectioning. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Additional padding to exceed 1024 bytes threshold. More filler text here to reach the required minimum size. Even more content.\n\n## Section 2\n\nMore content to test multi-section documents properly and reliably. Extra text to ensure we are well above the 1024 byte minimum for sectioning. Adding even more padding here to be absolutely certain we exceed the threshold for sectioned caching to be triggered.";
        
        // Test AST caching - first call should be cache miss
        let ast1 = cache.parse_with_cache(content).expect("Failed to parse content");
        let stats_after_first = cache.stats();
        assert!(stats_after_first.ast_misses > 0, "First parse should have misses");
        assert_eq!(stats_after_first.ast_hits, 0, "First parse should have no hits");
        assert!(stats_after_first.ast_entries > 0, "Cache should have entries");
        
        // Second call should be cache hit
        let ast2 = cache.parse_with_cache(content).expect("Failed to parse content");
        let stats_after_second = cache.stats();
        assert!(stats_after_second.ast_hits > 0, "Second parse should have cache hits");
        
        // AST nodes should be identical (cloned from cache)
        assert_eq!(format!("{:?}", ast1), format!("{:?}", ast2));
        
        // Test HTML rendering (note: HTML is not cached to support reference resolution)
        let options = HtmlOptions::default();
        
        // First render - AST from cache, HTML freshly generated
        let html1 = cache.render_with_cache(content, options.clone()).expect("Failed to render HTML");
        
        // Second render - AST from cache, HTML freshly generated again
        let html2 = cache.render_with_cache(content, options.clone()).expect("Failed to render HTML");
        
        // HTML should be identical
        assert_eq!(html1, html2);
        assert!(html1.contains("Hello World"));
        assert!(html1.contains("<strong>test</strong>"));
        
        // Test cache clearing
        cache.clear();
        let stats_after_clear = cache.stats();
        assert_eq!(stats_after_clear.ast_hits, 0);
        assert_eq!(stats_after_clear.ast_misses, 0);
        assert_eq!(stats_after_clear.ast_entries, 0);
    }
    
    #[test]
    fn smoke_test_hit_rates() {
        let cache = SimpleParserCache::new();
        
        // Use larger content to trigger sectioned caching (need > 1024 bytes)
        let content1 = "# First Document\n\nThis document contains enough text to trigger the sectioned caching mechanism. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Additional padding text to ensure we exceed 1024 bytes threshold for testing purposes. More filler content here.\n\n## Section 2\n\nEven more content in another section to test the sectioning mechanism properly.";
        let content2 = "# Second Document\n\nThis is a different document that also contains enough text to trigger sectioned caching. At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis praesentium voluptatum deleniti atque corrupti quos dolores et quas molestias excepturi sint occaecati cupiditate non provident, similique sunt in culpa qui officia deserunt mollitia animi, id est laborum et dolorum fuga. Et harum quidem rerum facilis est et expedita distinctio. Nam libero tempore, cum soluta nobis est eligendi optio cumque nihil impedit quo minus id quod maxime placeat facere possimus, omnis voluptas assumenda est, omnis dolor repellendus. Temporibus autem quibusdam et aut officiis debitis aut rerum necessitatibus saepe eveniet ut et voluptates repudiandae sint et molestiae non recusandae. Additional padding text to ensure we exceed 1024 bytes threshold for testing purposes. More filler content here.\n\n## Section 2\n\nEven more content in another section to test the sectioning mechanism properly.";
        
        let init_stats = cache.stats();
        
        // Parse both documents once (should be misses)
        cache.parse_with_cache(content1).expect("Parse failed");
        cache.parse_with_cache(content2).expect("Parse failed");
        
        let after_first_two = cache.stats();
        let _misses_count = after_first_two.ast_misses - init_stats.ast_misses;
        
        // Parse first document again (should have hits)
        cache.parse_with_cache(content1).expect("Parse failed");
        
        let stats = cache.stats();
        assert!(stats.ast_misses > 0, "Should have some cache misses");
        assert!(stats.ast_hits > 0, "Should have some cache hits on reparse");
        
        // Check hit rate is reasonable (should be > 0 and < 1)
        let hit_rate = stats.ast_hit_rate();
        assert!(hit_rate > 0.0, "Hit rate should be positive");
        assert!(hit_rate < 1.0, "Hit rate should be less than 1.0");
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
    fn smoke_test_sectioned_caching() {
        let cache = SimpleParserCache::new();
        let options = HtmlOptions::default();
        
        // Large document with sections that will change independently (over 1024 bytes to trigger sectioning)
        let content1 = "# Section 1\n\nThis is the first section with substantial content that needs to be long enough to trigger the sectioned caching mechanism. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\n# Section 2\n\nThis is the second section with different content that also needs to be substantial enough to demonstrate the caching behavior. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n\n# Section 3\n\nThis is the third section with even more content to ensure we exceed the minimum size threshold for sectioned caching. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.";
        
        let content2 = "# Section 1\n\nThis is the first section with substantial content that needs to be long enough to trigger the sectioned caching mechanism. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\n# Section 2 MODIFIED\n\nThis is the MODIFIED second section with completely different content that should cause a cache miss while other sections remain cached. The modification should demonstrate that only changed sections miss the cache while unchanged sections hit. This tests the core functionality of sectioned caching.\n\n# Section 3\n\nThis is the third section with even more content to ensure we exceed the minimum size threshold for sectioned caching. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.";
        
        // First render - should be all cache misses
        let _html1 = cache.render_with_cache(content1, options.clone())
            .expect("Failed to render sectioned HTML");
        let stats1 = cache.stats();
        
        // Second render of same content - should be cache hits
        let _html2 = cache.render_with_cache(content1, options.clone())
            .expect("Failed to render sectioned HTML");
        let stats2 = cache.stats();
        
        // Third render with one section changed - should have some hits and some misses
        let _html3 = cache.render_with_cache(content2, options.clone())
            .expect("Failed to render sectioned HTML");
        let stats3 = cache.stats();
        
        // Verify sectioned AST caching is working (HTML is not cached for reference resolution)
        println!("Stats1 (first render): ast_hits={}, ast_misses={}", stats1.ast_hits, stats1.ast_misses);
        println!("Stats2 (second render same): ast_hits={}, ast_misses={}", stats2.ast_hits, stats2.ast_misses);
        println!("Stats3 (third render modified): ast_hits={}, ast_misses={}", stats3.ast_hits, stats3.ast_misses);
        
        assert!(stats2.ast_hits > stats1.ast_hits, "Second render should have AST cache hits");
        assert!(stats3.ast_hits > stats2.ast_hits, "Third render should have additional AST cache hits for unchanged sections");
        assert!(stats3.ast_misses > stats2.ast_misses, "Third render should have AST cache misses for changed sections");
    }

    #[test]
    fn smoke_test_different_options() {
        let cache = SimpleParserCache::new();
        // Use larger content to trigger sectioned caching
        let content = "# Test Document\n\nThis document contains enough text to trigger the sectioned caching mechanism. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit. Additional padding text to ensure we exceed 1024 bytes threshold for testing purposes. More filler content here to reach the size requirement.\n\n## Section 2\n\nEven more content in another section to test the sectioning mechanism properly and reliably exceed the minimum threshold.";
        
        let options1 = HtmlOptions {
            syntax_highlighting: true,
            ..HtmlOptions::default()
        };
        
        let options2 = HtmlOptions {
            syntax_highlighting: false,
            ..HtmlOptions::default()
        };
        
        // Same content, different options - AST is parsed/cached, HTML is always fresh
        let html1 = cache.render_with_cache(content, options1).expect("Render failed");
        let html2 = cache.render_with_cache(content, options2).expect("Render failed");
        
        let stats = cache.stats();
        // Note: HTML is not cached (reference resolution requires full document)
        // AST sections are cached
        assert!(stats.ast_entries > 0, "AST sections should be cached");
        
        // HTML content might be the same but they're always freshly rendered
        assert_eq!(html1, html2); // For this simple case, output should be same
    }

    #[test]
    fn smoke_test_global_cache_cleanup() {
        // Populate global cache with large content to trigger sectioning
        let content = "# Global Cache Cleanup Test\n\nTesting issue #16 fix. This document contains enough text to trigger the sectioned caching mechanism. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicando. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Additional padding text to ensure we exceed 1024 bytes threshold for testing purposes. More filler content here to reach the size requirement.\n\n## Section 2\n\nEven more content in another section to test the sectioning mechanism properly and reliably exceed the minimum threshold.";
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
        
        // Verify global cache still works after cleanup
        let _ast2 = global_parser_cache().parse_with_cache(content).expect("Parse should work after cleanup");
        let stats_final = global_parser_cache().stats();
        assert!(stats_final.ast_misses > 0, "Should work normally after cleanup");
        assert!(stats_final.ast_entries > 0, "Cache should populate again after cleanup");
    }
}