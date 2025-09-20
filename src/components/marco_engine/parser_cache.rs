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

/// Cached AST node with metadata
#[derive(Debug, Clone)]
pub struct CachedAst {
    pub node: Node,
    pub content_hash: ContentHash,
    pub cached_at: std::time::SystemTime,
}

impl CachedAst {
    pub fn new(node: Node, content_hash: ContentHash) -> Self {
        Self {
            node,
            content_hash,
            cached_at: std::time::SystemTime::now(),
        }
    }
}

/// Cached HTML with metadata
#[derive(Debug, Clone)]
pub struct CachedHtml {
    pub html: String,
    pub content_hash: ContentHash, 
    pub options_hash: ContentHash,
    pub cached_at: std::time::SystemTime,
}

impl CachedHtml {
    pub fn new(html: String, content_hash: ContentHash, options_hash: ContentHash) -> Self {
        Self {
            html,
            content_hash,
            options_hash,
            cached_at: std::time::SystemTime::now(),
        }
    }
}

/// Simple parser cache with basic HashMap storage (as per spec)
pub struct SimpleParserCache {
    /// AST cache: content hash -> cached AST
    ast_cache: Arc<RwLock<HashMap<ContentHash, CachedAst>>>,
    /// HTML cache: (content hash, options hash) -> cached HTML
    html_cache: Arc<RwLock<HashMap<(ContentHash, ContentHash), CachedHtml>>>,
}

impl SimpleParserCache {
    /// Create new simple parser cache
    pub fn new() -> Self {
        Self {
            ast_cache: Arc::new(RwLock::new(HashMap::new())),
            html_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Parse content with AST caching
    pub fn parse_with_cache(&self, content: &str) -> Result<Node> {
        let content_hash = calculate_hash(content);
        
        // Check cache first
        {
            if let Ok(cache) = self.ast_cache.read() {
                if let Some(cached) = cache.get(&content_hash) {
                    // Cache hit - return cached AST
                    return Ok(cached.node.clone());
                }
            }
        }
        
        // Cache miss - parse and cache
        let pairs = parse_text(content)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        
        let ast = build_ast(pairs)
            .map_err(|e| anyhow::anyhow!("AST build error: {}", e))?;
        
        // Add to cache
        let cached_ast = CachedAst::new(ast.clone(), content_hash);
        if let Ok(mut cache) = self.ast_cache.write() {
            cache.insert(content_hash, cached_ast);
        }
        
        Ok(ast)
    }
    
    /// Parse content with incremental support - SIMPLIFIED (no actual incremental processing)
    pub fn parse_with_cache_incremental(&self, content: &str) -> Result<Vec<Node>> {
        // For simplicity, just parse normally and return as single-item vec
        // Real incremental parsing removed as per spec (too complex)
        let node = self.parse_with_cache(content)?;
        Ok(vec![node])
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
                    // Cache hit - return cached HTML
                    return Ok(cached.html.clone());
                }
            }
        }
        
        // Cache miss - parse, render and cache
        let ast = self.parse_with_cache(content)?;
        let html = render_html(&ast, options.clone());
        
        // Add to cache
        let cached_html = CachedHtml::new(html.clone(), content_hash, options_hash);
        if let Ok(mut cache) = self.html_cache.write() {
            cache.insert(cache_key, cached_html);
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
    
    /// Clear all cached entries
    pub fn clear(&self) {
        if let Ok(mut cache) = self.ast_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.html_cache.write() {
            cache.clear();
        }
    }
    
    /// Get cache statistics (simplified - just counts)
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
        
        ParserCacheStats {
            ast_hits: 0,          // Simplified - no hit tracking
            ast_misses: 0,        // Simplified - no miss tracking
            html_hits: 0,         // Simplified - no hit tracking
            html_misses: 0,       // Simplified - no miss tracking
            ast_entries,
            html_entries,
        }
    }
}

/// Simple cache statistics (as per spec - no complex tracking)
#[derive(Debug, Clone)]
pub struct ParserCacheStats {
    pub ast_hits: usize,
    pub ast_misses: usize,
    pub html_hits: usize,
    pub html_misses: usize,
    pub ast_entries: usize,
    pub html_entries: usize,
}

impl ParserCacheStats {
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