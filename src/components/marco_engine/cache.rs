//! AST Caching System
//! 
//! This module provides high-performance AST caching for Marco documents with:
//! - Block-level caching for individual AST nodes
//! - Change detection based on content hashes
//! - Incremental parsing and rendering support
//! - Memory-efficient cache management
//! - Thread-safe concurrent access

use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use std::time::{SystemTime, Instant};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

use crate::components::marco_engine::ast_node::{Node, Span};
use crate::components::marco_engine::{AstBuilder, MarcoParser};
use crate::components::marco_engine::grammar::Rule;
use pest::Parser;

/// Hash type for content identification
type ContentHash = u64;

/// Cached AST node with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedASTNode {
    /// The actual AST node
    pub node: Node,
    /// Hash of the original content that produced this node
    pub content_hash: ContentHash,
    /// When this cache entry was created
    pub created_at: SystemTime,
    /// How many times this cache entry has been accessed
    pub access_count: u32,
    /// Last time this entry was accessed
    pub last_accessed: SystemTime,
    /// Size estimate in bytes for memory management
    pub size_estimate: usize,
}

impl CachedASTNode {
    pub fn new(node: Node, content_hash: ContentHash) -> Self {
        let size_estimate = estimate_node_size(&node);
        
        Self {
            node,
            content_hash,
            created_at: SystemTime::now(),
            access_count: 1,
            last_accessed: SystemTime::now(),
            size_estimate,
        }
    }
    
    /// Update access statistics
    pub fn touch(&mut self) {
        self.access_count = self.access_count.saturating_add(1);
        self.last_accessed = SystemTime::now();
    }
    
    /// Check if this cache entry is still fresh (within specified age)
    pub fn is_fresh(&self, max_age_secs: u64) -> bool {
        if let Ok(elapsed) = self.created_at.elapsed() {
            elapsed.as_secs() < max_age_secs
        } else {
            false
        }
    }
}

/// Block identifier for tracking document changes
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockId {
    /// Line range this block spans
    pub line_range: (u32, u32),
    /// Column range for the start line
    pub column_range: (u32, u32),
    /// Type hint for quick matching
    pub block_type: String,
}

impl BlockId {
    pub fn from_span_and_node(span: &Span, node: &Node) -> Self {
        Self {
            line_range: (span.line, span.line), // For now, assume single line blocks
            column_range: (span.column, span.column + (span.end - span.start)),
            block_type: node_type_hint(node),
        }
    }
    
    /// Check if this block might overlap with another based on line ranges
    pub fn overlaps_with(&self, other: &BlockId) -> bool {
        // Check if line ranges overlap
        self.line_range.0 <= other.line_range.1 && other.line_range.0 <= self.line_range.1
    }
}

/// Configuration for AST cache behavior
#[derive(Debug, Clone)]
pub struct ASTCacheConfig {
    /// Maximum number of cached AST nodes
    pub max_cached_nodes: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum age for cache entries in seconds
    pub max_age_seconds: u64,
    /// Enable automatic cleanup
    pub enable_cleanup: bool,
    /// Minimum block size to cache (avoid caching tiny text nodes)
    pub min_cache_size: usize,
}

impl Default for ASTCacheConfig {
    fn default() -> Self {
        Self {
            max_cached_nodes: 1000,
            max_memory_bytes: 50 * 1024 * 1024, // 50MB
            max_age_seconds: 300, // 5 minutes
            enable_cleanup: true,
            min_cache_size: 10, // Don't cache very small text nodes
        }
    }
}

/// High-performance AST cache with block-level granularity
pub struct ASTCache {
    /// Cache entries indexed by block ID and content hash
    cache: Arc<RwLock<HashMap<(BlockId, ContentHash), CachedASTNode>>>,
    /// Full document cache for complete ASTs
    document_cache: Arc<RwLock<HashMap<ContentHash, (Node, SystemTime)>>>,
    /// Configuration
    config: ASTCacheConfig,
    /// Statistics
    stats: Arc<RwLock<CacheStats>>,
}

/// Cache performance statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
    pub memory_usage: usize,
    pub cache_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_requests as f64
        }
    }
    
    pub fn memory_mb(&self) -> f64 {
        self.memory_usage as f64 / 1024.0 / 1024.0
    }
}

impl ASTCache {
    /// Create new AST cache with default configuration
    pub fn new() -> Self {
        Self::with_config(ASTCacheConfig::default())
    }
    
    /// Create AST cache with custom configuration
    pub fn with_config(config: ASTCacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            document_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// Parse document with caching support
    pub fn parse_cached(&self, content: &str) -> Result<Node> {
        let content_hash = calculate_hash(content);
        
        // Update stats
        if let Ok(mut stats) = self.stats.write() {
            stats.total_requests += 1;
        }
        
        // Check document-level cache first
        if let Ok(doc_cache) = self.document_cache.read() {
            if let Some((cached_ast, created_at)) = doc_cache.get(&content_hash) {
                if created_at.elapsed().unwrap_or_default().as_secs() < self.config.max_age_seconds {
                    // Cache hit
                    if let Ok(mut stats) = self.stats.write() {
                        stats.hits += 1;
                    }
                    log::debug!("Document cache hit for hash: {}", content_hash);
                    return Ok(cached_ast.clone());
                }
            }
        }
        
        // Cache miss - parse the document
        let ast = self.parse_with_block_cache(content, content_hash)?;
        
        // Cache the complete document
        if let Ok(mut doc_cache) = self.document_cache.write() {
            // Evict old documents if needed
            if doc_cache.len() >= 10 { // Keep only last 10 complete documents
                let oldest_key = doc_cache.iter()
                    .min_by_key(|(_, (_, created_at))| *created_at)
                    .map(|(hash, _)| *hash);
                
                if let Some(key) = oldest_key {
                    doc_cache.remove(&key);
                    if let Ok(mut stats) = self.stats.write() {
                        stats.evictions += 1;
                    }
                }
            }
            
            doc_cache.insert(content_hash, (ast.clone(), SystemTime::now()));
        }
        
        // Update stats
        if let Ok(mut stats) = self.stats.write() {
            stats.misses += 1;
        }
        
        log::debug!("Document parsed and cached with hash: {}", content_hash);
        Ok(ast)
    }
    
    /// Parse with block-level caching
    fn parse_with_block_cache(&self, content: &str, content_hash: ContentHash) -> Result<Node> {
        // For now, do regular parsing - block-level caching is complex and would require
        // significant changes to the parser. This is a foundation for future enhancement.
        let pairs = MarcoParser::parse(Rule::document, content)
            .context("Failed to parse document")?;
        
        AstBuilder::build(pairs)
            .map_err(|e| anyhow::anyhow!("Failed to build AST: {}", e))
    }
    
    /// Get cached node for specific block
    pub fn get_cached_node(&self, block_id: &BlockId, content_hash: ContentHash) -> Option<Node> {
        if let Ok(cache) = self.cache.read() {
            if let Some(mut cached_node) = cache.get(&(block_id.clone(), content_hash)).cloned() {
                // Update access stats
                cached_node.touch();
                
                // Update stats
                if let Ok(mut stats) = self.stats.write() {
                    stats.hits += 1;
                    stats.total_requests += 1;
                }
                
                return Some(cached_node.node);
            }
        }
        
        // Cache miss
        if let Ok(mut stats) = self.stats.write() {
            stats.misses += 1;
            stats.total_requests += 1;
        }
        
        None
    }
    
    /// Cache a specific AST node
    pub fn cache_node(&self, block_id: BlockId, content_hash: ContentHash, node: Node) {
        // Don't cache very small nodes
        let size_estimate = estimate_node_size(&node);
        if size_estimate < self.config.min_cache_size {
            return;
        }
        
        let cached_node = CachedASTNode::new(node, content_hash);
        
        if let Ok(mut cache) = self.cache.write() {
            // Evict entries if needed
            if cache.len() >= self.config.max_cached_nodes {
                self.evict_lru(&mut cache);
            }
            
            cache.insert((block_id, content_hash), cached_node);
            
            // Update stats
            if let Ok(mut stats) = self.stats.write() {
                stats.cache_size = cache.len();
                stats.memory_usage += size_estimate;
            }
        }
    }
    
    /// Evict least recently used entries
    fn evict_lru(&self, cache: &mut HashMap<(BlockId, ContentHash), CachedASTNode>) {
        if cache.is_empty() {
            return;
        }
        
        // Find the least recently accessed entry
        let oldest_key = cache.iter()
            .min_by_key(|(_, cached_node)| cached_node.last_accessed)
            .map(|(key, _)| key.clone());
        
        if let Some(key) = oldest_key {
            if let Some(removed) = cache.remove(&key) {
                // Update stats
                if let Ok(mut stats) = self.stats.write() {
                    stats.evictions += 1;
                    stats.memory_usage = stats.memory_usage.saturating_sub(removed.size_estimate);
                }
                
                log::debug!("Evicted cached AST node for block: {:?}", key.0);
            }
        }
    }
    
    /// Invalidate cached entries for specific line ranges
    pub fn invalidate_lines(&self, start_line: u32, end_line: u32) {
        if let Ok(mut cache) = self.cache.write() {
            let keys_to_remove: Vec<_> = cache.keys()
                .filter(|(block_id, _)| {
                    block_id.line_range.0 <= end_line && start_line <= block_id.line_range.1
                })
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                if let Some(removed) = cache.remove(&key) {
                    log::debug!("Invalidated cached node at lines {}-{}", 
                        key.0.line_range.0, key.0.line_range.1);
                    
                    // Update stats
                    if let Ok(mut stats) = self.stats.write() {
                        stats.memory_usage = stats.memory_usage.saturating_sub(removed.size_estimate);
                    }
                }
            }
            
            // Update cache size
            if let Ok(mut stats) = self.stats.write() {
                stats.cache_size = cache.len();
            }
        }
        
        // Also clear document cache to force re-parsing
        if let Ok(mut doc_cache) = self.document_cache.write() {
            doc_cache.clear();
        }
    }
    
    /// Clear all cached entries
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        if let Ok(mut doc_cache) = self.document_cache.write() {
            doc_cache.clear();
        }
        if let Ok(mut stats) = self.stats.write() {
            *stats = CacheStats::default();
        }
        
        log::info!("Cleared all AST cache entries");
    }
    
    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }
    
    /// Perform maintenance cleanup
    pub fn cleanup(&self) {
        if !self.config.enable_cleanup {
            return;
        }
        
        let now = SystemTime::now();
        let max_age = std::time::Duration::from_secs(self.config.max_age_seconds);
        
        if let Ok(mut cache) = self.cache.write() {
            let keys_to_remove: Vec<_> = cache.iter()
                .filter(|(_, cached_node)| {
                    now.duration_since(cached_node.created_at).unwrap_or_default() > max_age
                })
                .map(|(key, _)| key.clone())
                .collect();
            
            for key in keys_to_remove {
                if let Some(removed) = cache.remove(&key) {
                    log::debug!("Cleaned up expired AST cache entry");
                    
                    // Update stats
                    if let Ok(mut stats) = self.stats.write() {
                        stats.memory_usage = stats.memory_usage.saturating_sub(removed.size_estimate);
                        stats.evictions += 1;
                    }
                }
            }
            
            // Update cache size
            if let Ok(mut stats) = self.stats.write() {
                stats.cache_size = cache.len();
            }
        }
        
        // Cleanup document cache too
        if let Ok(mut doc_cache) = self.document_cache.write() {
            let keys_to_remove: Vec<_> = doc_cache.iter()
                .filter(|(_, (_, created_at))| {
                    now.duration_since(*created_at).unwrap_or_default() > max_age
                })
                .map(|(key, _)| *key)
                .collect();
            
            for key in keys_to_remove {
                doc_cache.remove(&key);
            }
        }
        
        log::debug!("AST cache cleanup completed");
    }
}

/// Global AST cache instance
static GLOBAL_AST_CACHE: OnceLock<ASTCache> = OnceLock::new();

/// Get global AST cache instance
pub fn global_ast_cache() -> &'static ASTCache {
    GLOBAL_AST_CACHE.get_or_init(|| ASTCache::new())
}

/// Utility functions

/// Calculate hash of content for cache keys
fn calculate_hash(content: &str) -> ContentHash {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Estimate memory size of an AST node
fn estimate_node_size(node: &Node) -> usize {
    match node {
        Node::Document { children, .. } => {
            std::mem::size_of::<Node>() + children.iter().map(estimate_node_size).sum::<usize>()
        }
        Node::Paragraph { content, .. } => {
            std::mem::size_of::<Node>() + content.iter().map(estimate_node_size).sum::<usize>()
        }
        Node::Heading { content, .. } => {
            std::mem::size_of::<Node>() + content.iter().map(estimate_node_size).sum::<usize>()
        }
        Node::Text { content, .. } => {
            std::mem::size_of::<Node>() + content.len()
        }
        Node::CodeBlock { content, language, .. } => {
            std::mem::size_of::<Node>() + content.len() + language.as_ref().map_or(0, |l| l.len())
        }
        // Add more specific estimates for other node types
        _ => std::mem::size_of::<Node>() + 50, // Default estimate
    }
}

/// Get a type hint string for a node (for BlockId)
fn node_type_hint(node: &Node) -> String {
    match node {
        Node::Document { .. } => "document".to_string(),
        Node::Paragraph { .. } => "paragraph".to_string(),
        Node::Heading { level, .. } => format!("heading-{}", level),
        Node::CodeBlock { .. } => "code-block".to_string(),
        Node::Text { .. } => "text".to_string(),
        Node::List { .. } => "list".to_string(),
        Node::ListItem { .. } => "list-item".to_string(),
        Node::BlockQuote { .. } => "blockquote".to_string(),
        Node::Table { .. } => "table".to_string(),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::marco_engine::ast_node::Node;
    
    #[test]
    fn test_ast_cache_basic() {
        let cache = ASTCache::new();
        let content = "# Hello World\n\nThis is a test document.";
        
        // First parse should be a cache miss
        let ast1 = cache.parse_cached(content).unwrap();
        
        // Second parse should be a cache hit
        let ast2 = cache.parse_cached(content).unwrap();
        
        // Should be identical
        assert_eq!(ast1, ast2);
        
        let stats = cache.stats();
        assert!(stats.hits > 0);
        assert!(stats.total_requests >= 2);
    }
    
    #[test]
    fn test_content_hash() {
        let content1 = "# Hello World";
        let content2 = "# Hello World";
        let content3 = "# Hello Mars";
        
        assert_eq!(calculate_hash(content1), calculate_hash(content2));
        assert_ne!(calculate_hash(content1), calculate_hash(content3));
    }
    
    #[test]
    fn test_block_id_overlap() {
        let span1 = Span { start: 0, end: 10, line: 1, column: 1 };
        let span2 = Span { start: 5, end: 15, line: 2, column: 1 };
        let span3 = Span { start: 20, end: 30, line: 5, column: 1 };
        
        let node = Node::text("test".to_string(), span1.clone());
        
        let block1 = BlockId::from_span_and_node(&span1, &node);
        let block2 = BlockId::from_span_and_node(&span2, &node);
        let block3 = BlockId::from_span_and_node(&span3, &node);
        
        assert!(block1.overlaps_with(&block2));
        assert!(!block1.overlaps_with(&block3));
    }
    
    #[test]
    fn test_cache_eviction() {
        let config = ASTCacheConfig {
            max_cached_nodes: 2,
            ..Default::default()
        };
        
        let cache = ASTCache::with_config(config);
        
        // Add more nodes than the cache can hold
        let span = Span { start: 0, end: 10, line: 1, column: 1 };
        for i in 0..5 {
            let node = Node::text(format!("test-{}", i), span.clone());
            let block_id = BlockId::from_span_and_node(&span, &node);
            cache.cache_node(block_id, i as u64, node);
        }
        
        let stats = cache.stats();
        assert!(stats.evictions > 0);
        assert!(stats.cache_size <= 2);
    }
}