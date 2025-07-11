// ===================== AST CACHE =====================
//! Thread-safe cache for parsed Markdown AST (Vec<pulldown_cmark::Event>)
//! Key: u64 (hash of markdown source)
//
// NOTE: If this file grows much larger, consider splitting each cache type into its own module.

use pulldown_cmark::Event;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

/// Type alias for a parsed Markdown AST (static lifetime)
pub type AstVec = Vec<Event<'static>>;

/// Thread-safe cache for Markdown ASTs, keyed by hash
pub struct AstCache {
    map: RwLock<HashMap<u64, AstVec>>,
}

impl AstCache {
    /// Create a new, empty AST cache
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    /// Get a cached AST by hash, if present
    pub fn get(&self, hash: u64) -> Option<AstVec> {
        self.map.read().unwrap().get(&hash).cloned()
    }

    /// Insert an AST into the cache by hash
    pub fn insert(&self, hash: u64, ast: AstVec) {
        self.map.write().unwrap().insert(hash, ast);
    }

    /// Remove a cached AST by hash
    pub fn invalidate(&self, hash: u64) {
        self.map.write().unwrap().remove(&hash);
    }

    /// Clear all cached ASTs
    pub fn clear(&self) {
        self.map.write().unwrap().clear();
    }

}

// Global singleton cache
/// Hash a markdown source string for use as a cache key
pub fn hash_source(source: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Convert Vec<Event<'a>> to Vec<Event<'static>> by owning all data
pub fn own_events<'a>(events: Vec<Event<'a>>) -> Vec<Event<'static>> {
    use pulldown_cmark::{Event, CowStr};
    events.into_iter().map(|event| match event {
        Event::Start(tag) => Event::Start(own_tag(tag)),
        Event::End(tag) => Event::End(own_tag(tag)),
        Event::Text(text) => Event::Text(CowStr::Boxed(text.into_string().into_boxed_str())),
        Event::Code(code) => Event::Code(CowStr::Boxed(code.into_string().into_boxed_str())),
        Event::Html(html) => Event::Html(CowStr::Boxed(html.into_string().into_boxed_str())),
        Event::FootnoteReference(f) => Event::FootnoteReference(CowStr::Boxed(f.into_string().into_boxed_str())),
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
    }).collect()
}

/// Convert a Tag<'a> to Tag<'static> by leaking memory for string fields.
///
/// # WARNING
/// This function leaks memory for every unique string it processes, which is a tradeoff for
/// achieving 'static lifetimes in the AST cache. This is generally acceptable for a limited
/// number of unique headings/ids/classes, but could cause unbounded memory growth if used
/// with untrusted or highly dynamic input. Documented for future maintainers.
fn own_tag<'a>(tag: pulldown_cmark::Tag<'a>) -> pulldown_cmark::Tag<'static> {
    use pulldown_cmark::{Tag, CowStr};
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading(lvl, id, classes) => Tag::Heading(
            lvl,
            id.map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str),
            classes.into_iter().map(|c| Box::leak(c.to_string().into_boxed_str()) as &'static str).collect()
        ),
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(kind) => Tag::CodeBlock(match kind {
            pulldown_cmark::CodeBlockKind::Indented => pulldown_cmark::CodeBlockKind::Indented,
            pulldown_cmark::CodeBlockKind::Fenced(lang) => pulldown_cmark::CodeBlockKind::Fenced(CowStr::Boxed(lang.into_string().into_boxed_str())),
        }),
        Tag::List(num) => Tag::List(num),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(f) => Tag::FootnoteDefinition(CowStr::Boxed(f.into_string().into_boxed_str())),
        Tag::Table(aligns) => Tag::Table(aligns),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link(lt, dest, title) => Tag::Link(lt, CowStr::Boxed(dest.into_string().into_boxed_str()), CowStr::Boxed(title.into_string().into_boxed_str())),
        Tag::Image(lt, dest, title) => Tag::Image(lt, CowStr::Boxed(dest.into_string().into_boxed_str()), CowStr::Boxed(title.into_string().into_boxed_str())),
    }
}
pub static AST_CACHE: Lazy<AstCache> = Lazy::new(|| AstCache::new());

// ===================== CACHE BENCH (empty) =====================
// (No content from cache_bench.rs)

// ===================== CACHE INVALIDATION =====================
// Centralized cache invalidation utilities for all resource caches.
//
// # When to call each function
// - `clear_all_caches()`: Use for a full reload or when you want to reset all caches (rare).
// - `on_theme_change()`: Call after changing the theme (CSS or syntax theme) to ensure all theme-dependent caches are refreshed.
// - `on_file_save_or_reload(path)`: Call after saving or reloading a file. This will invalidate file and image caches for the given path, and clear the AST cache if the markdown buffer is reloaded.
// - `on_buffer_update()`: Call after the markdown buffer/content is updated (e.g., on edit) to clear the AST cache.
// - `invalidate_image_asset(path)`: Call when a specific image asset changes on disk.
// - `invalidate_file_cache(path)`: Call when a specific file changes on disk.
//
// Integrate these utilities into dialog save handlers, file open/reload logic, and theme change events for robust cache management.

// ===================== FILE CACHE =====================
use std::fs;
use std::path::{Path, PathBuf};

/// Thread-safe cache for file contents (text files)
static FILE_CACHE: Lazy<CacheSync<PathBuf, String>> = Lazy::new(|| CacheSync::new());

/// Get file contents from cache, reading from disk if not present.
/// Returns None if the file cannot be read.
/// Get file contents from cache, reading from disk if not present.
/// Returns None if the file cannot be read. Logs a warning on error.
pub fn get_file_contents<P: AsRef<Path>>(path: P) -> Option<String> {
    let pathbuf = path.as_ref().to_path_buf();
    let s = FILE_CACHE.get_or_insert_with(pathbuf.clone(), |p| {
        match fs::read_to_string(p) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("File cache read error: {}: {}", p.display(), e);
                String::new()
            }
        }
    });
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Invalidate a specific file in the cache
pub fn invalidate_file<P: AsRef<Path>>(path: P) {
    FILE_CACHE.invalidate(&path.as_ref().to_path_buf());
}

/// Clear all cached files
pub fn clear_file_cache() {
    FILE_CACHE.clear();
}

// ===================== IMAGE CACHE =====================
use std::collections::HashMap as StdHashMap;

/// Global image asset cache: key = asset path/name, value = image bytes
static IMAGE_CACHE: Lazy<RwLock<StdHashMap<String, Vec<u8>>>> = Lazy::new(|| RwLock::new(StdHashMap::new()));

/// Get image bytes from cache, loading from disk if not present.
/// Returns None if the file cannot be read. Logs a warning on error.
pub fn get_image(path: &str) -> Option<Vec<u8>> {
    {
        let cache = IMAGE_CACHE.read().unwrap();
        if let Some(data) = cache.get(path) {
            return Some(data.clone());
        }
    }
    // Not in cache, try to load from disk
    match fs::read(path) {
        Ok(data) => {
            let mut cache = IMAGE_CACHE.write().unwrap();
            cache.insert(path.to_string(), data.clone());
            Some(data)
        }
        Err(e) => {
            eprintln!("Image cache read error: {}: {}", path, e);
            None
        }
    }
}

/// Invalidate (remove) an image from the cache.
pub fn invalidate_image(path: &str) {
    let mut cache = IMAGE_CACHE.write().unwrap();
    cache.remove(path);
}

/// Clear the entire image cache
pub fn clear_image_cache() {
    let mut cache = IMAGE_CACHE.write().unwrap();
    cache.clear();
}

// ===================== REGEX CACHE =====================
use regex::Regex;

/// Global thread-safe cache for compiled regexes
static REGEX_CACHE: Lazy<CacheSync<String, Regex>> = Lazy::new(|| CacheSync::new());

/// Get a compiled regex from the global cache, compiling and caching if not present.
/// Panics if the pattern is invalid. Only use with trusted, static patterns.
pub fn get_regex(pattern: &str) -> Regex {
    REGEX_CACHE.get_or_insert_with(pattern.to_string(), |pat| Regex::new(pat).unwrap()).clone()
}

/// Invalidate a specific regex pattern in the cache.
pub fn invalidate_regex(pattern: &str) {
    REGEX_CACHE.invalidate(&pattern.to_string());
}

/// Clear all cached regexes.
pub fn clear_regex_cache() {
    REGEX_CACHE.clear();
}

// ===================== CACHE INVALIDATION (impl) =====================
/// Invalidate all caches (use with caution, e.g., on full reload)
pub fn clear_all_caches() {
    clear_regex_cache();
    clear_file_cache();
    AST_CACHE.clear();
    clear_image_cache();
}

/// Invalidate caches on theme change
pub fn on_theme_change() {
    clear_regex_cache();
    AST_CACHE.clear();
    // Add more as needed (e.g., CSS theme cache)
}

/// Invalidate caches on file save or reload
pub fn on_file_save_or_reload(path: &str) {
    invalidate_file(path);
    invalidate_image(path);
    AST_CACHE.clear(); // If markdown buffer is reloaded
}

/// Invalidate caches on buffer/content update
pub fn on_buffer_update() {
    AST_CACHE.clear();
}

/// Invalidate image asset cache for a specific asset
pub fn invalidate_image_asset(path: &str) {
    invalidate_image(path);
}

/// Invalidate file cache for a specific file
pub fn invalidate_file_cache(path: &str) {
    invalidate_file(path);
}
// ...existing code...
// use std::sync::RwLock; // Already imported above
/// Thread-safe cache with manual invalidation (for use with Rayon, etc.)
pub struct CacheSync<K, V>
where
    K: Eq + Hash + Clone,
{
    map: RwLock<HashMap<K, V>>,
}

impl<K, V> CacheSync<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Create a new, empty cache
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    /// Get a value from the cache, or insert it using the provided loader function
    pub fn get_or_insert_with<F>(&self, key: K, loader: F) -> V
    where
        F: FnOnce(&K) -> V,
        V: Clone,
    {
        {
            let map = self.map.read().unwrap();
            if let Some(val) = map.get(&key) {
                return val.clone();
            }
        }
        let val = loader(&key);
        let mut map = self.map.write().unwrap();
        map.insert(key.clone(), val.clone());
        val
    }

    /// Get a value from the cache if present (read-only, does not insert)
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.map.read().unwrap().get(key).cloned()
    }

    /// Insert a value into the cache manually
    pub fn insert(&self, key: K, value: V) {
        self.map.write().unwrap().insert(key, value);
    }

    /// Invalidate a specific key
    pub fn invalidate(&self, key: &K) {
        self.map.write().unwrap().remove(key);
    }

    /// Invalidate all cached values
    pub fn clear(&self) {
        self.map.write().unwrap().clear();
    }
}

/// Simple cache utility for storing and invalidating expensive-to-load resources (e.g., themes, CSS)
/// Use for editor/view theme and CSS caching.

// use std::collections::HashMap; // Already imported above
// use std::hash::Hash; // Already imported above
use std::cell::RefCell;

/// Generic cache with manual invalidation
pub struct Cache<K, V>
where
    K: Eq + Hash + Clone,
{
    map: RefCell<HashMap<K, V>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Create a new, empty cache
    pub fn new() -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
        }
    }

    /// Get a value from the cache, or insert it using the provided loader function
    /// Get a value from the cache, or insert it using the provided loader function
    pub fn get_or_insert_with<F>(&self, key: K, loader: F) -> V
    where
        F: FnOnce(&K) -> V,
        V: Clone,
    {
        let mut map = self.map.borrow_mut();
        if let Some(val) = map.get(&key) {
            return val.clone();
        }
        let val = loader(&key);
        map.insert(key.clone(), val.clone());
        val
    }

    /// Get a value from the cache if present (read-only, does not insert)
    /// Get a value from the cache if present (read-only, does not insert)
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.map.borrow().get(key).cloned()
    }

    /// Insert a value into the cache manually
    /// Insert a value into the cache manually
    pub fn insert(&self, key: K, value: V) {
        self.map.borrow_mut().insert(key, value);
    }

    /// Invalidate a specific key
    /// Invalidate a specific key
    pub fn invalidate(&self, key: &K) {
        self.map.borrow_mut().remove(key);
    }

    /// Invalidate all cached values
    /// Invalidate all cached values
    pub fn clear(&self) {
        self.map.borrow_mut().clear();
    }
}
