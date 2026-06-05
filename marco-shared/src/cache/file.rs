//! File content caches.
//!
//! Two types:
//! - [`SimpleFileCache`] – lightweight `Mutex<HashMap>`, no mtime; for short-lived operations (search).
//! - [`FileCache`] – `DashMap` with mtime-based invalidation; the global singleton.

use crate::cache::stats::CacheStats;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// SimpleFileCache
// ---------------------------------------------------------------------------

/// Lightweight in-memory file cache for single-operation use (e.g. the search dialog).
///
/// No mtime checking. Caller is responsible for clearing between distinct operations.
pub struct SimpleFileCache {
    inner: Mutex<HashMap<PathBuf, String>>,
}

impl SimpleFileCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, path: &Path) -> Option<String> {
        self.inner.lock().ok()?.get(path).cloned()
    }

    pub fn insert(&self, path: PathBuf, content: String) {
        if let Ok(mut map) = self.inner.lock() {
            map.insert(path, content);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut map) = self.inner.lock() {
            map.clear();
        }
    }
}

impl Default for SimpleFileCache {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FileCache
// ---------------------------------------------------------------------------

/// A single cached file entry.
pub struct CachedFile {
    pub content: String,
    pub modified: SystemTime,
}

/// Global file content cache with mtime-based invalidation.
///
/// Uses [`DashMap`] for concurrent reads without a global lock.
/// Does not evict entries proactively — when the cache is full it simply skips
/// caching the new entry (log at `debug!` level). Marco's working set is small
/// (≤ 10 open/recently opened files), so this is sufficient.
pub struct FileCache {
    inner: DashMap<PathBuf, CachedFile>,
    max_entries: usize,
    stats: Mutex<CacheStats>,
}

impl FileCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: DashMap::new(),
            max_entries,
            stats: Mutex::new(CacheStats::default()),
        }
    }

    fn canonical(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    /// Get file content. Returns `None` if the file is not cached or has been modified.
    pub fn get(&self, path: &Path) -> Option<String> {
        let key = Self::canonical(path);

        // Fast path: read mtime, check against cached value.
        let modified = match std::fs::metadata(path).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => {
                // File was deleted or became unreadable — evict any stale entry
                // so the slot can be reused once the file exists again.
                self.inner.remove(&key);
                return None;
            }
        };

        if let Some(entry) = self.inner.get(&key) {
            if entry.modified == modified {
                if let Ok(mut s) = self.stats.lock() {
                    s.hits += 1;
                }
                return Some(entry.content.clone());
            }
            // Stale entry — drop the borrow before removing.
            drop(entry);
            self.inner.remove(&key);
        }

        if let Ok(mut s) = self.stats.lock() {
            s.misses += 1;
        }
        None
    }

    /// Insert (or replace) a file's content. Does nothing if the cache is full
    /// and the key isn't already present.
    ///
    /// The caller MUST pass the `modified` timestamp captured at the same time
    /// as `content` (i.e. before reading the file). Reading mtime *after*
    /// content would create a TOCTOU race: if the file changed between the
    /// read and the insert, we'd cache stale content tagged with a newer
    /// mtime, and `get()` would never invalidate it.
    pub fn insert(&self, path: PathBuf, content: String, modified: SystemTime) {
        let key = Self::canonical(&path);

        // Allow replacements even when at capacity; only refuse new keys.
        if !self.inner.contains_key(&key) && self.inner.len() >= self.max_entries {
            log::debug!(
                "[file_cache] at capacity ({} entries), skipping insert for {}",
                self.max_entries,
                path.display()
            );
            return;
        }

        self.inner.insert(key, CachedFile { content, modified });
    }

    /// Remove a single file from the cache (call after writes).
    pub fn invalidate_file(&self, path: &Path) {
        let key = Self::canonical(path);
        self.inner.remove(&key);
    }

    /// Remove all entries.
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Return a snapshot of current statistics.
    pub fn stats(&self) -> CacheStats {
        let mut s = self.stats.lock().map(|s| s.clone()).unwrap_or_default();
        s.current_size = self.inner.len() as u64;
        s
    }
}
