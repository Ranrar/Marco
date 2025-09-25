//! Simple File Caching System
//!
//! Provides basic file and directory caching as per optimization spec:
//! - Cache file content in memory to avoid repeated disk I/O
//! - Track file modification times for automatic cache invalidation
//! - Cache directory listings for 30 seconds
//! - Use weak references to active DocumentBuffers for automatic cleanup
//! - Monitor external file changes with gio::FileMonitor

use anyhow::{Context, Result};
use gio::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

/// Simple cache entry for file content (as per spec)
#[derive(Debug, Clone)]
pub struct CachedFile {
    pub content: String,
    pub modification_time: u64,
    pub last_accessed: SystemTime,
}

impl CachedFile {
    pub fn new(content: String, modification_time: u64) -> Self {
        Self {
            content,
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
    /// Weak references to open files for automatic cleanup (as per spec)
    open_files: Arc<RwLock<HashMap<PathBuf, Vec<Weak<dyn std::any::Any + Send + Sync>>>>>,
}

impl SimpleFileCache {
    /// Create new simple file cache
    pub fn new() -> Self {
        Self {
            content_cache: Arc::new(RwLock::new(HashMap::new())),
            directory_cache: Arc::new(RwLock::new(HashMap::new())),
            open_files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load file fast using cache-first strategy (as per spec)
    pub fn load_file_fast<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        {
            if let Ok(cache) = self.content_cache.read() {
                if let Some(entry) = cache.get(&path) {
                    if entry.is_valid_for(&path) {
                        // Cache hit - return content
                        return Ok(entry.content.clone());
                    }
                }
            }
        }

        // Cache miss - load from disk and cache
        self.load_and_cache_file(path)
    }

    /// Load file from disk and add to cache
    fn load_and_cache_file(&self, path: PathBuf) -> Result<String> {
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

        let cached_file = CachedFile::new(content.clone(), modification_time);

        // Add to cache
        if let Ok(mut cache) = self.content_cache.write() {
            cache.insert(path, cached_file);
        }

        Ok(content)
    }

    /// Register a weak reference to an object that represents an open file (as per spec)
    pub fn register_open_file<T>(&self, path: &Path, reference: Weak<T>)
    where
        T: Send + Sync + 'static,
    {
        let path = path.to_path_buf();
        if let Ok(mut open_files) = self.open_files.write() {
            let weak_ref: Weak<dyn std::any::Any + Send + Sync> = reference;
            open_files
                .entry(path)
                .or_insert_with(Vec::new)
                .push(weak_ref);
        }
    }

    /// Check if a file is currently open (has valid weak references)
    pub fn is_file_open(&self, path: &Path) -> bool {
        if let Ok(mut open_files) = self.open_files.write() {
            if let Some(refs) = open_files.get_mut(path) {
                // Remove expired weak references
                refs.retain(|weak_ref| weak_ref.upgrade().is_some());

                if refs.is_empty() {
                    // No more valid references, remove the entry
                    open_files.remove(path);
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            false
        }
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

    /// Async version for GTK-safe operation (as per spec)
    pub fn load_file_fast_async<P, F>(&self, path: P, callback: F)
    where
        P: AsRef<Path> + Send + 'static,
        F: Fn(Result<String>) + 'static,
    {
        let path = path.as_ref().to_path_buf();

        // Check cache first on main thread
        {
            if let Ok(cache) = self.content_cache.read() {
                if let Some(entry) = cache.get(&path) {
                    if entry.is_valid_for(&path) {
                        // Cache hit - return immediately
                        callback(Ok(entry.content.clone()));
                        return;
                    }
                }
            }
        }

        // Cache miss - load in background using glib::spawn_local (as per spec)
        let content_cache = Arc::clone(&self.content_cache);

        glib::spawn_future_local(async move {
            let result = gio::spawn_blocking({
                let path = path.clone();
                move || -> Result<String> {
                    let content = fs::read_to_string(&path)
                        .with_context(|| format!("Failed to read file: {}", path.display()))?;

                    let metadata = fs::metadata(&path).with_context(|| {
                        format!("Failed to get metadata for: {}", path.display())
                    })?;

                    let modification_time = metadata
                        .modified()
                        .map_err(|e| anyhow::anyhow!("Failed to get modification time: {}", e))?
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| anyhow::anyhow!("Invalid system time: {}", e))?
                        .as_secs();

                    let cached_file = CachedFile::new(content.clone(), modification_time);

                    // Add to cache
                    if let Ok(mut cache) = content_cache.write() {
                        cache.insert(path, cached_file);
                    }

                    Ok(content)
                }
            })
            .await;

            // Return result via GTK-safe callback (as per spec)
            glib::idle_add_local_once(move || match result {
                Ok(Ok(content)) => callback(Ok(content)),
                Ok(Err(e)) => callback(Err(e)),
                Err(e) => callback(Err(anyhow::anyhow!("Spawn error: {:?}", e))),
            });
        });
    }

    /// Create file monitor for external changes (as per spec)
    pub fn create_file_monitor(
        path: &Path,
        content_cache: Arc<RwLock<HashMap<PathBuf, CachedFile>>>,
    ) -> Result<gio::FileMonitor> {
        let file = gio::File::for_path(path);

        match file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
            Ok(monitor) => {
                // Set up change callback for cache invalidation
                monitor.connect_changed(move |_, file_obj, _other_file, event| {
                    match event {
                        gio::FileMonitorEvent::Changed
                        | gio::FileMonitorEvent::Deleted
                        | gio::FileMonitorEvent::Moved => {
                            // Invalidate cache entry (as per spec)
                            if let Some(changed_path) = file_obj.path() {
                                if let Ok(mut cache) = content_cache.write() {
                                    if cache.remove(&changed_path).is_some() {
                                        log::debug!(
                                            "Cache invalidated due to external change: {}",
                                            changed_path.display()
                                        );
                                    }
                                }
                            }
                        }
                        _ => {
                            // Other events - ignore
                        }
                    }
                });

                Ok(monitor)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to create file monitor: {}", e)),
        }
    }

    /// Invalidate cache entry for specific file
    pub fn invalidate_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();

        if let Ok(mut cache) = self.content_cache.write() {
            cache.remove(path);
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        if let Ok(mut cache) = self.content_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.directory_cache.write() {
            cache.clear();
        }
    }
}

/// Global cache instance (singleton pattern as per spec)
static GLOBAL_CACHE: OnceLock<SimpleFileCache> = OnceLock::new();

/// Get global file cache instance (as per spec)
pub fn global_cache() -> &'static SimpleFileCache {
    GLOBAL_CACHE.get_or_init(|| SimpleFileCache::new())
}

/// Simple cached file operations (as per spec)
pub mod cached {
    use super::*;

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        global_cache().load_file_fast(path)
    }
}
