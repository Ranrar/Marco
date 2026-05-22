//! Cache subsystem for marco-shared.
//!
//! ## Components
//!
//! - [`ParserCache`] — triple-layer AST + HTML + section cache (Moka TinyLFU).
//! - [`FileCache`] — mtime-invalidated file content cache (DashMap).
//! - [`SimpleFileCache`] — lightweight per-operation cache (search dialog).
//! - [`global_parser_cache`] — global singleton for the parser cache.
//! - [`global_cache`] — global singleton for the file cache.
//! - [`parse_to_html_cached`] / [`parse_to_html`] — render convenience functions.
//! - [`hash_content`] — blake3 content hash used as a change guard in the marco binary.
//! - [`section`] — document section splitter for section-level caching.
//!
//! ## Singletons
//!
//! Both global caches are initialized lazily on first access via [`OnceLock`].
//! There is no explicit init step — just call [`global_parser_cache`] or
//! [`global_cache`] and the cache will be ready.

pub mod cached;
mod file;
mod parser;
pub mod section;
mod stats;

pub use file::{CachedFile, FileCache, SimpleFileCache};
pub use parser::ParserCache;
pub use section::{section_for_line, split_into_sections, DocumentSection};
pub use stats::CacheStats;

use std::sync::OnceLock;

static PARSER_CACHE: OnceLock<ParserCache> = OnceLock::new();
static FILE_CACHE: OnceLock<FileCache> = OnceLock::new();

// ---------------------------------------------------------------------------
// Global singletons
// ---------------------------------------------------------------------------

/// Returns the process-wide parser cache (AST + HTML).
pub fn global_parser_cache() -> &'static ParserCache {
    PARSER_CACHE.get_or_init(ParserCache::new)
}

/// Returns the process-wide file content cache.
pub fn global_cache() -> &'static FileCache {
    FILE_CACHE.get_or_init(|| FileCache::new(64))
}

// ---------------------------------------------------------------------------
// Shutdown helpers
// ---------------------------------------------------------------------------

/// Clear the global file cache and release memory.
pub fn shutdown_global_cache() {
    if let Some(cache) = FILE_CACHE.get() {
        cache.clear();
    }
}

/// Clear the global parser cache and log statistics.
pub fn shutdown_global_parser_cache() {
    if let Some(cache) = PARSER_CACHE.get() {
        cache.clear();
        log::debug!(
            "[cache] Parser cache stats at shutdown: {:?}",
            cache.stats()
        );
    }
}

// ---------------------------------------------------------------------------
// Convenience render functions
// ---------------------------------------------------------------------------

/// Render markdown to HTML without caching.
pub fn parse_to_html(
    md: &str,
    opts: marco_core::RenderOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    let doc = marco_core::parse(md)?;
    marco_core::render(&doc, &opts)
}

/// Render markdown to HTML through the global [`ParserCache`].
pub fn parse_to_html_cached(
    md: &str,
    opts: marco_core::RenderOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    global_parser_cache().render_with_cache(md, opts)
}

// ---------------------------------------------------------------------------
// Content hashing
// ---------------------------------------------------------------------------

/// Hash markdown text to a 64-bit value for cache-key and change-guard use.
///
/// Uses the first 8 bytes of a blake3 hash — cryptographically uniform output,
/// so collision probability is negligible at Marco's document scale.
///
/// Exposed here (rather than requiring `blake3` in the marco binary) because
/// `blake3` is a dep of `marco-shared`, not of `marco`. Call this from
/// `editor/ui.rs` as `marco_shared::cache::hash_content(&text)`.
pub fn hash_content(text: &str) -> u64 {
    let h = blake3::hash(text.as_bytes());
    u64::from_le_bytes(h.as_bytes()[0..8].try_into().unwrap())
}
