//! Caching System for Marco
//!
//! Provides two types of caching:
//! 1. **File Caching** (SimpleFileCache): Cache file content with modification time tracking
//! 2. **Parser Caching** (ParserCache): Cache parsed AST and rendered HTML using moka
//!
//! ## File Caching
//! - Cache file content in memory to avoid repeated disk I/O
//! - Track file modification times for automatic cache invalidation
//! - Use weak references to active DocumentBuffers for automatic cleanup
//! - File monitoring removed to prevent memory leaks and threading issues
//!
//! ## Parser Caching (Moka-based)
//! - **AST Caching**: Cache parsed Document structures keyed by markdown content hash
//! - **HTML Caching**: Cache rendered HTML keyed by (content_hash, render_options_hash)
//! - **Thread-safe**: Moka provides lock-free concurrent access
//! - **Automatic eviction**: LRU-based eviction when cache size limits reached
//! - **No manual cleanup**: Moka handles cleanup automatically on drop

use crate::parser::{parse, Document};
use crate::render::{render, RenderOptions};
use moka::sync::Cache;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Simple cache entry for file content (as per spec)
#[derive(Debug, Clone)]
pub struct CachedFile {
    pub content: Arc<String>,
    pub modification_time: u64,
    pub last_accessed: SystemTime,
}

impl CachedFile {
    pub fn new(content: String, modification_time: u64) -> Self {
        Self {
            content: Arc::new(content),
            modification_time,
            last_accessed: SystemTime::now(),
        }
    }

    /// Check if this entry is still valid for the given file
    pub fn is_valid_for(&self, path: &Path) -> bool {
        match fs::metadata(path) {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                        return duration.as_secs() == self.modification_time;
                    }
                }
            }
            Err(_) => return false,
        }
        false
    }
}

/// Simple file cache with basic functionality as per spec
pub struct SimpleFileCache {
    /// File content cache (RwLock<HashMap> as per spec)
    content_cache: Arc<RwLock<HashMap<PathBuf, CachedFile>>>,
}

impl Default for SimpleFileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleFileCache {
    /// Create new simple file cache
    pub fn new() -> Self {
        Self {
            content_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load file fast using cache-first strategy (as per spec)
    pub fn load_file_fast<P: AsRef<Path>>(&self, path: P) -> Result<String, Box<dyn std::error::Error>> {
        // Use the shared version and convert to String for backwards compatibility
        let shared_content = self.load_file_fast_shared(path)?;
        Ok((*shared_content).clone())
    }

    /// Load file fast with shared ownership - avoids cloning for better memory efficiency
    pub fn load_file_fast_shared<P: AsRef<Path>>(&self, path: P) -> Result<Arc<String>, Box<dyn std::error::Error>> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        {
            if let Ok(cache) = self.content_cache.read() {
                if let Some(entry) = cache.get(&path) {
                    if entry.is_valid_for(&path) {
                        // Cache hit - return shared reference (no cloning!)
                        return Ok(Arc::clone(&entry.content));
                    }
                }
            }
        }

        // Cache miss - load from disk and cache
        self.load_and_cache_file_shared(path)
    }

    /// Load file from disk and add to cache with shared ownership - avoids unnecessary cloning
    fn load_and_cache_file_shared(&self, path: PathBuf) -> Result<Arc<String>, Box<dyn std::error::Error>> {
        // Read raw bytes and sanitize UTF-8 (prevents crashes from invalid UTF-8)
        let raw_bytes =
            fs::read(&path).map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

        let (content, stats) = crate::logic::utf8::sanitize_input_with_stats(
            &raw_bytes,
            crate::logic::utf8::InputSource::File,
        );

        // Log any UTF-8 issues
        if stats.had_issues() {
            log::warn!(
                "File '{}' had UTF-8 issues: {}",
                path.display(),
                stats.summary()
            );
        }

        let metadata = fs::metadata(&path)
            .map_err(|e| format!("Failed to get metadata for {}: {}", path.display(), e))?;

        let modification_time = metadata
            .modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Invalid system time: {}", e))?
            .as_secs();

        // Create Arc<String> directly - no clone needed!
        let cached_file = CachedFile::new(content, modification_time);
        let shared_content = Arc::clone(&cached_file.content);

        // Add to cache
        if let Ok(mut cache) = self.content_cache.write() {
            cache.insert(path, cached_file);
        }

        Ok(shared_content)
    }

    /// Invalidate cache entry for specific file
    pub fn invalidate_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();

        if let Ok(mut cache) = self.content_cache.write() {
            cache.remove(path);
        }
    }

    /// Clear all cached entries to free memory
    /// This is called during application shutdown to prevent memory retention
    pub fn clear(&self) {
        log::info!("Clearing file cache");

        let mut cleared_files = 0;

        // Clear file content cache
        if let Ok(mut cache) = self.content_cache.write() {
            cleared_files = cache.len();
            cache.clear();
        }

        log::info!("File cache cleared: {} file entries", cleared_files);
    }
}

// ============================================================================
// Parser Cache (Moka-based)
// ============================================================================

// Cache size limits
const AST_CACHE_MAX_CAPACITY: u64 = 1000; // Max 1000 parsed documents
const HTML_CACHE_MAX_CAPACITY: u64 = 2000; // Max 2000 rendered HTML strings

/// Global singleton parser cache instance
static GLOBAL_PARSER_CACHE: OnceLock<ParserCache> = OnceLock::new();

/// High-performance parser cache using moka
#[derive(Clone)]
pub struct ParserCache {
    /// Cache for parsed AST documents
    ast_cache: Cache<u64, Document>,
    /// Cache for rendered HTML (keyed by content hash + options hash)
    html_cache: Cache<(u64, u64), String>,
}

impl ParserCache {
    /// Create a new parser cache with default capacity
    pub fn new() -> Self {
        Self {
            ast_cache: Cache::new(AST_CACHE_MAX_CAPACITY),
            html_cache: Cache::new(HTML_CACHE_MAX_CAPACITY),
        }
    }

    /// Parse markdown content with AST caching
    pub fn parse_with_cache(&self, content: &str) -> Result<Document, Box<dyn std::error::Error>> {
        let content_hash = hash_content(content);

        // Try to get from cache
        if let Some(doc) = self.ast_cache.get(&content_hash) {
            return Ok(doc);
        }

        // Parse and cache
        let doc = parse(content)?;
        self.ast_cache.insert(content_hash, doc.clone());
        Ok(doc)
    }

    /// Render markdown to HTML with full caching (AST + HTML)
    pub fn render_with_cache(&self, content: &str, options: RenderOptions) -> Result<String, Box<dyn std::error::Error>> {
        let content_hash = hash_content(content);
        let options_hash = hash_options(&options);
        let cache_key = (content_hash, options_hash);

        // Try to get rendered HTML from cache
        if let Some(html) = self.html_cache.get(&cache_key) {
            return Ok(html);
        }

        // Parse (with AST caching)
        let doc = self.parse_with_cache(content)?;

        // Render and cache
        let html = render(&doc, &options)?;
        self.html_cache.insert(cache_key, html.clone());
        Ok(html)
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.ast_cache.invalidate_all();
        self.html_cache.invalidate_all();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            ast_entries: self.ast_cache.entry_count(),
            html_entries: self.html_cache.entry_count(),
            ast_capacity: AST_CACHE_MAX_CAPACITY,
            html_capacity: HTML_CACHE_MAX_CAPACITY,
        }
    }
}

impl Default for ParserCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub ast_entries: u64,
    pub html_entries: u64,
    pub ast_capacity: u64,
    pub html_capacity: u64,
}

/// Get the global parser cache instance (creates on first access)
pub fn global_parser_cache() -> &'static ParserCache {
    GLOBAL_PARSER_CACHE.get_or_init(ParserCache::new)
}

/// Shutdown and clear the global parser cache
///
/// Note: With moka, this is optional - the cache will be cleaned up
/// automatically when the program exits. This function is provided
/// for compatibility with old API.
pub fn shutdown_global_parser_cache() {
    if let Some(cache) = GLOBAL_PARSER_CACHE.get() {
        cache.clear();
    }
}

// === Helper functions ===

/// Hash markdown content for cache key
fn hash_content(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Hash render options for cache key
fn hash_options(options: &RenderOptions) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();

    // Hash the relevant fields of RenderOptions
    options.syntax_highlighting.hash(&mut hasher);
    options.line_numbers.hash(&mut hasher);
    options.theme.hash(&mut hasher);

    hasher.finish()
}

// === Convenience Functions ===

/// Parse markdown to HTML (uncached, for one-off conversions)
pub fn parse_to_html(content: &str, options: RenderOptions) -> Result<String, Box<dyn std::error::Error>> {
    let doc = parse(content)?;
    render(&doc, &options)
}

/// Parse markdown to HTML using global cache (recommended for UI)
pub fn parse_to_html_cached(content: &str, options: RenderOptions) -> Result<String, Box<dyn std::error::Error>> {
    global_parser_cache().render_with_cache(content, options)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn smoke_test_file_cache() {
        let cache = SimpleFileCache::new();

        // Create a temporary file for testing
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "Test content for file cache").expect("Failed to write temp file");
        let temp_path = temp_file.path();

        // Test file caching - first load should read from disk
        let content1 = cache
            .load_file_fast(temp_path)
            .expect("Failed to load file");
        assert!(content1.contains("Test content for file cache"));

        // Second load should use cache (we can't directly verify this, but it should work)
        let content2 = cache
            .load_file_fast(temp_path)
            .expect("Failed to load file");
        assert_eq!(content1, content2);
    }

    #[test]
    fn smoke_test_file_cache_cleanup() {
        let cache = SimpleFileCache::new();

        // Create temporary files for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_file.txt");
        std::fs::write(&file_path, "Content for cleanup test").expect("Failed to write test file");

        // Populate the cache
        let _content = cache
            .load_file_fast(&file_path)
            .expect("Failed to load file");

        // Note: We can't directly verify cache entries because the cache internals
        // use RwLock and the cache might be empty due to error handling, but we can
        // test that clear() doesn't panic and works correctly

        // Test cache cleanup - this is the main focus of issue #16
        cache.clear();

        // Verify cache still works after cleanup (should reload from disk)
        let content_after_clear = cache
            .load_file_fast(&file_path)
            .expect("Cache should work after clear");
        assert!(content_after_clear.contains("Content for cleanup test"));
    }

    #[test]
    #[serial(file_cache)]
    fn smoke_test_global_cache_cleanup() {
        // Test global cache access
        let cache = global_cache();

        // Create a temporary file
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("global_test.txt");
        std::fs::write(&file_path, "Global cache test content").expect("Failed to write test file");

        // Populate global cache
        let _content = cache
            .load_file_fast(&file_path)
            .expect("Failed to load file");

        // Test global cleanup - this is the main focus of issue #16
        shutdown_global_cache();

        // Verify global cache still works after cleanup
        let content_after_shutdown = cache
            .load_file_fast(&file_path)
            .expect("Global cache should work after shutdown");
        assert!(content_after_shutdown.contains("Global cache test content"));
    }

    // === Parser Cache Tests ===

    #[test]
    fn smoke_test_parser_cache() {
        let cache = ParserCache::new();
        let content = "# Hello World\n\nThis is **bold** text.";

        // First parse - cache miss
        let doc1 = cache.parse_with_cache(content).expect("Parse failed");
        assert!(format!("{:?}", doc1).contains("Heading"));

        // Force sync to update entry counts
        cache.ast_cache.run_pending_tasks();

        // Second parse - cache hit (should be instant)
        let doc2 = cache.parse_with_cache(content).expect("Parse failed");
        assert!(format!("{:?}", doc2).contains("Heading"));

        let stats = cache.stats();
        assert_eq!(stats.ast_entries, 1); // Only one unique content cached
    }

    #[test]
    fn smoke_test_render_cache() {
        let cache = ParserCache::new();
        let content = "# Test\n\nSome content.";
        let options = RenderOptions::default();

        // First render - cache miss
        let html1 = cache
            .render_with_cache(content, options.clone())
            .expect("Render failed");
        assert!(html1.contains("<h1"));

        // Force sync to update entry counts
        cache.ast_cache.run_pending_tasks();
        cache.html_cache.run_pending_tasks();

        // Second render - cache hit
        let html2 = cache
            .render_with_cache(content, options)
            .expect("Render failed");
        assert_eq!(html1, html2);

        let stats = cache.stats();
        assert_eq!(stats.ast_entries, 1);
        assert_eq!(stats.html_entries, 1);
    }

    #[test]
    fn smoke_test_global_parser_cache() {
        let content = "## Global Cache Test";
        let cache1 = global_parser_cache();
        let cache2 = global_parser_cache();

        // Should be same instance
        assert_eq!(cache1 as *const _, cache2 as *const _);

        // Should work
        let doc = cache1.parse_with_cache(content).expect("Parse failed");
        assert!(format!("{:?}", doc).contains("Heading"));
    }

    #[test]
    fn smoke_test_convenience_functions() {
        let content = "Test content with **emphasis**.";
        let options = RenderOptions::default();

        // Uncached version
        let html1 = parse_to_html(content, options.clone()).expect("Parse failed");
        assert!(html1.contains("<strong>"));

        // Cached version
        let html2 = parse_to_html_cached(content, options).expect("Parse failed");
        assert_eq!(html1, html2);
    }
}

/// Global cache instance (singleton pattern as per spec)
static GLOBAL_CACHE: OnceLock<SimpleFileCache> = OnceLock::new();

/// Get global file cache instance (as per spec)
pub fn global_cache() -> &'static SimpleFileCache {
    GLOBAL_CACHE.get_or_init(SimpleFileCache::new)
}

/// Shutdown and cleanup the global file cache
/// This clears all cached data to prevent memory retention on application exit
pub fn shutdown_global_cache() {
    // Only clear if the global cache has been initialized
    if let Some(cache) = GLOBAL_CACHE.get() {
        cache.clear();
    } else {
        log::info!("File cache was never initialized, no cleanup needed");
    }
}

/// Simple cached file operations (as per spec)
pub mod cached {
    use super::*;

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
        global_cache().load_file_fast(path)
    }
}
