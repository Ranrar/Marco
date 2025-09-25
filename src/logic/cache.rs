//! Simple File Caching System
//!
//! Provides basic file and directory caching as per optimization spec:
//! - Cache file content in memory to avoid repeated disk I/O
//! - Track file modification times for automatic cache invalidation
//! - Cache directory listings for 30 seconds
//! - Use weak references to active DocumentBuffers for automatic cleanup
//! - File monitoring removed to prevent memory leaks and threading issues

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
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

/// Simple directory cache entry (as per spec - 30 second TTL)
#[derive(Debug, Clone)]
pub struct CachedDirectory {
    pub files: Vec<PathBuf>,
    pub directories: Vec<PathBuf>,
    pub cached_at: SystemTime,
}

impl CachedDirectory {
    pub fn new(files: Vec<PathBuf>, directories: Vec<PathBuf>) -> Self {
        Self {
            files,
            directories,
            cached_at: SystemTime::now(),
        }
    }

    /// Check if directory cache is still fresh (30 seconds as per spec)
    pub fn is_fresh(&self) -> bool {
        if let Ok(elapsed) = self.cached_at.elapsed() {
            elapsed.as_secs() < 30
        } else {
            false
        }
    }
}

/// Simple file cache with basic functionality as per spec
pub struct SimpleFileCache {
    /// File content cache (RwLock<HashMap> as per spec)
    content_cache: Arc<RwLock<HashMap<PathBuf, CachedFile>>>,
    /// Directory listing cache
    directory_cache: Arc<RwLock<HashMap<PathBuf, CachedDirectory>>>,
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
            directory_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load file fast using cache-first strategy (as per spec)
    pub fn load_file_fast<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        // Use the shared version and convert to String for backwards compatibility
        let shared_content = self.load_file_fast_shared(path)?;
        Ok((*shared_content).clone())
    }

    /// Load file fast with shared ownership - avoids cloning for better memory efficiency
    pub fn load_file_fast_shared<P: AsRef<Path>>(&self, path: P) -> Result<Arc<String>> {
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
    fn load_and_cache_file_shared(&self, path: PathBuf) -> Result<Arc<String>> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let metadata = fs::metadata(&path)
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        let modification_time = metadata
            .modified()
            .map_err(|e| anyhow::anyhow!("Failed to get modification time: {}", e))?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Invalid system time: {}", e))?
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

    /// Search files fast using cached directory listings (as per spec)
    pub fn search_files_fast<P: AsRef<Path>>(
        &self,
        path: P,
        pattern: Option<&str>,
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        {
            if let Ok(cache) = self.directory_cache.read() {
                if let Some(entry) = cache.get(&path) {
                    if entry.is_fresh() {
                        // Cache hit - apply pattern filtering and return
                        return Ok(self.apply_search_pattern(
                            &entry.files,
                            &entry.directories,
                            pattern,
                        ));
                    }
                }
            }
        }

        // Cache miss - scan directory and cache
        let (files, directories) = self.scan_and_cache_directory(path)?;
        Ok(self.apply_search_pattern(&files, &directories, pattern))
    }

    /// Apply case-insensitive pattern filtering (as per spec)
    fn apply_search_pattern(
        &self,
        files: &[PathBuf],
        directories: &[PathBuf],
        pattern: Option<&str>,
    ) -> (Vec<PathBuf>, Vec<PathBuf>) {
        match pattern {
            Some(search_pattern) if !search_pattern.is_empty() => {
                let pattern_lower = search_pattern.to_lowercase();
                let filtered_files = files
                    .iter()
                    .filter(|path| {
                        if let Some(name) = path.file_name() {
                            name.to_string_lossy()
                                .to_lowercase()
                                .contains(&pattern_lower)
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();

                let filtered_dirs = directories
                    .iter()
                    .filter(|path| {
                        if let Some(name) = path.file_name() {
                            name.to_string_lossy()
                                .to_lowercase()
                                .contains(&pattern_lower)
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();

                (filtered_files, filtered_dirs)
            }
            _ => {
                // No pattern - return all results
                (files.to_vec(), directories.to_vec())
            }
        }
    }

    /// Scan directory and add to cache
    fn scan_and_cache_directory(&self, path: PathBuf) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        let entries = fs::read_dir(&path)
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;

        let mut files = Vec::new();
        let mut directories = Vec::new();

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                directories.push(entry_path);
            } else if entry_path.is_file() {
                files.push(entry_path);
            }
        }

        // Sort for consistent output
        files.sort();
        directories.sort();

        let cached_dir = CachedDirectory::new(files.clone(), directories.clone());

        // Add to cache
        if let Ok(mut cache) = self.directory_cache.write() {
            cache.insert(path, cached_dir);
        }

        Ok((files, directories))
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
        let mut cleared_dirs = 0;
        
        // Clear file content cache
        if let Ok(mut cache) = self.content_cache.write() {
            cleared_files = cache.len();
            cache.clear();
        }
        
        // Clear directory cache
        if let Ok(mut cache) = self.directory_cache.write() {
            cleared_dirs = cache.len();
            cache.clear();
        }
        
        log::info!("File cache cleared: {} file entries, {} directory entries", cleared_files, cleared_dirs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};
    use std::io::Write;
    
    #[test]
    fn smoke_test_file_cache() {
        let cache = SimpleFileCache::new();
        
        // Create a temporary file for testing
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "Test content for file cache").expect("Failed to write temp file");
        let temp_path = temp_file.path();
        
        // Test file caching - first load should read from disk
        let content1 = cache.load_file_fast(temp_path).expect("Failed to load file");
        assert!(content1.contains("Test content for file cache"));
        
        // Second load should use cache (we can't directly verify this, but it should work)
        let content2 = cache.load_file_fast(temp_path).expect("Failed to load file");
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
        let _content = cache.load_file_fast(&file_path).expect("Failed to load file");
        
        // Note: We can't directly verify cache entries because the cache internals 
        // use RwLock and the cache might be empty due to error handling, but we can 
        // test that clear() doesn't panic and works correctly
        
        // Test cache cleanup - this is the main focus of issue #16
        cache.clear();
        
        // Verify cache still works after cleanup (should reload from disk)
        let content_after_clear = cache.load_file_fast(&file_path).expect("Cache should work after clear");
        assert!(content_after_clear.contains("Content for cleanup test"));
    }
    
    #[test]
    fn smoke_test_global_cache_cleanup() {
        // Test global cache access
        let cache = global_cache();
        
        // Create a temporary file
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("global_test.txt");
        std::fs::write(&file_path, "Global cache test content").expect("Failed to write test file");
        
        // Populate global cache
        let _content = cache.load_file_fast(&file_path).expect("Failed to load file");
        
        // Test global cleanup - this is the main focus of issue #16
        shutdown_global_cache();
        
        // Verify global cache still works after cleanup
        let content_after_shutdown = cache.load_file_fast(&file_path).expect("Global cache should work after shutdown");
        assert!(content_after_shutdown.contains("Global cache test content"));
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

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        global_cache().load_file_fast(path)
    }
}
