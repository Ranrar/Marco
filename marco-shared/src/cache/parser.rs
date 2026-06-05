//! AST + HTML + intelligence parser cache backed by Moka (TinyLFU eviction).
//!
//! Five layers:
//! 1. **HTML cache** – keyed by `(content_hash, options_hash)`. A hit here
//!    returns the final HTML without any parsing or rendering.
//! 2. **AST cache** – keyed by `content_hash`. A hit here skips parsing but
//!    still renders HTML from the cached AST.
//! 3. **Section HTML cache** – keyed by `(section_content_hash, section_index, options_hash)`.
//!    Used by the section render path for large documents.
//! 4. **TOC cache** – keyed by `content_hash`. Caches the extracted table-of-contents
//!    entries so the TOC panel and debounced rebuild skip re-parsing.
//! 5. **Diagnostics cache** – keyed by `content_hash`. Caches lint diagnostics so the
//!    footer and intelligence pipeline skip re-parsing on content that hasn't changed.

use crate::cache::section::DocumentSection;
use crate::cache::stats::CacheStats;
use marco_core::RenderOptions;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, Mutex};

const AST_CACHE_MAX: u64 = 128;
const HTML_CACHE_MAX: u64 = 256;
const SECTION_CACHE_MAX: u64 = 2048;
const TOC_CACHE_MAX: u64 = 64;
const DIAGNOSTICS_CACHE_MAX: u64 = 128;

fn hash_bytes(data: &[u8]) -> u64 {
    let h = blake3::hash(data);
    u64::from_le_bytes(h.as_bytes()[0..8].try_into().unwrap())
}

fn hash_options(opts: &RenderOptions) -> u64 {
    let mut hasher = DefaultHasher::new();
    opts.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------

/// Five-layer AST + HTML + intelligence cache.
pub struct ParserCache {
    ast_cache: moka::sync::Cache<u64, Arc<marco_core::Document>>,
    html_cache: moka::sync::Cache<(u64, u64), Arc<String>>,
    /// Key: (section_content_hash, options_hash)
    ///
    /// Note: `section.index` is intentionally excluded so that inserting or
    /// removing a section near the top of a document does not invalidate
    /// cache entries for later sections whose content is unchanged.
    section_html_cache: moka::sync::Cache<(u64, u64), Arc<String>>,
    /// Key: content_hash → extracted TOC entries.
    toc_cache: moka::sync::Cache<u64, Arc<Vec<marco_core::intelligence::toc::TocEntry>>>,
    /// Key: content_hash → all diagnostics (unfiltered; callers apply their own severity filter).
    diagnostics_cache: moka::sync::Cache<u64, Arc<Vec<marco_core::intelligence::Diagnostic>>>,
    stats: Mutex<CacheStats>,
}

impl ParserCache {
    pub fn new() -> Self {
        Self {
            ast_cache: moka::sync::Cache::builder()
                .max_capacity(AST_CACHE_MAX)
                .build(),
            html_cache: moka::sync::Cache::builder()
                .max_capacity(HTML_CACHE_MAX)
                .build(),
            section_html_cache: moka::sync::Cache::builder()
                .max_capacity(SECTION_CACHE_MAX)
                .build(),
            toc_cache: moka::sync::Cache::builder()
                .max_capacity(TOC_CACHE_MAX)
                .build(),
            diagnostics_cache: moka::sync::Cache::builder()
                .max_capacity(DIAGNOSTICS_CACHE_MAX)
                .build(),
            stats: Mutex::new(CacheStats::default()),
        }
    }

    /// Render `text` to HTML, using cached AST and/or HTML where available.
    ///
    /// Returns `Err` only if parsing or rendering fails — never on cache issues.
    pub fn render_with_cache(
        &self,
        text: &str,
        options: RenderOptions,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let content_hash = hash_bytes(text.as_bytes());
        let options_hash = hash_options(&options);
        let html_key = (content_hash, options_hash);

        // Layer 1: full HTML cache hit
        if let Some(html) = self.html_cache.get(&html_key) {
            if let Ok(mut s) = self.stats.lock() {
                s.hits += 1;
            }
            return Ok((*html).clone());
        }

        // Layer 2: AST cache hit — re-render only
        let html = if let Some(ast) = self.ast_cache.get(&content_hash) {
            marco_core::render(&ast, &options)?
        } else {
            // Full miss: parse + render + cache AST
            let doc = marco_core::parse(text)?;
            let html = marco_core::render(&doc, &options)?;
            self.ast_cache.insert(content_hash, Arc::new(doc));
            html
        };

        if let Ok(mut s) = self.stats.lock() {
            s.misses += 1;
        }

        self.html_cache.insert(html_key, Arc::new(html.clone()));
        Ok(html)
    }

    /// Render each section to HTML, returning an `Arc<String>` per section.
    ///
    /// Only sections whose `content_hash` has changed since the last call will
    /// actually be parsed and rendered — all others are served from the section
    /// HTML cache.  This is the hot path for large documents.
    pub fn render_sections_with_cache(
        &self,
        sections: &[DocumentSection],
        options: &RenderOptions,
    ) -> Result<Vec<Arc<String>>, Box<dyn std::error::Error>> {
        let options_hash = hash_options(options);
        let mut result = Vec::with_capacity(sections.len());
        let mut batch_hits: u64 = 0;
        let mut batch_misses: u64 = 0;

        for section in sections {
            let key = (section.content_hash, options_hash);

            let html_arc = if let Some(cached) = self.section_html_cache.get(&key) {
                batch_hits += 1;
                cached
            } else {
                batch_misses += 1;
                // Miss: parse + render this section as a standalone fragment
                let html = match marco_core::parse(&section.content) {
                    Ok(doc) => marco_core::render(&doc, options)?,
                    Err(e) => {
                        log::warn!(
                            "[section_cache] parse error for section {}: {}",
                            section.index,
                            e
                        );
                        format!("<!-- parse error in section {} -->", section.index)
                    }
                };
                let arc = Arc::new(html);
                self.section_html_cache.insert(key, Arc::clone(&arc));
                arc
            };

            result.push(html_arc);
        }

        if batch_hits > 0 || batch_misses > 0 {
            if let Ok(mut s) = self.stats.lock() {
                s.hits += batch_hits;
                s.misses += batch_misses;
            }
        }

        Ok(result)
    }

    // -----------------------------------------------------------------------
    // TOC helpers
    // -----------------------------------------------------------------------

    /// Return cached TOC entries for a document that has already been parsed
    /// (caller supplies the `Arc<Document>` and the pre-computed content hash).
    ///
    /// On a cache miss the entries are computed from `doc` and stored.
    pub fn get_or_compute_toc_for_doc(
        &self,
        doc: &Arc<marco_core::Document>,
        content_hash: u64,
    ) -> Arc<Vec<marco_core::intelligence::toc::TocEntry>> {
        if let Some(cached) = self.toc_cache.get(&content_hash) {
            return cached;
        }
        let entries = marco_core::intelligence::toc::extract_toc(doc.as_ref());
        let arc = Arc::new(entries);
        self.toc_cache.insert(content_hash, Arc::clone(&arc));
        arc
    }

    /// Get or compute TOC entries for raw `text`.
    ///
    /// Reuses the AST cache so parsing is skipped when the same text has
    /// already been parsed (e.g. by the render pipeline).  Returns an empty
    /// slice on parse failure.
    pub fn get_or_compute_toc(
        &self,
        text: &str,
    ) -> Arc<Vec<marco_core::intelligence::toc::TocEntry>> {
        let content_hash = hash_bytes(text.as_bytes());
        if let Some(cached) = self.toc_cache.get(&content_hash) {
            return cached;
        }
        match self.parse_and_cache_ast(text) {
            Ok(doc) => self.get_or_compute_toc_for_doc(&doc, content_hash),
            Err(_) => Arc::new(Vec::new()),
        }
    }

    // -----------------------------------------------------------------------
    // Diagnostics helpers
    // -----------------------------------------------------------------------

    /// Return cached diagnostics for a document that has already been parsed.
    ///
    /// Diagnostics are **unfiltered** — callers apply their own severity
    /// filter before displaying.  On a cache miss the diagnostics are computed
    /// from `doc` and stored.
    pub fn get_or_compute_diagnostics_for_doc(
        &self,
        doc: &Arc<marco_core::Document>,
        content_hash: u64,
    ) -> Arc<Vec<marco_core::intelligence::Diagnostic>> {
        if let Some(cached) = self.diagnostics_cache.get(&content_hash) {
            return cached;
        }
        let diags = marco_core::intelligence::compute_diagnostics_with_options(
            doc.as_ref(),
            marco_core::intelligence::DiagnosticsOptions::all(),
        );
        let arc = Arc::new(diags);
        self.diagnostics_cache
            .insert(content_hash, Arc::clone(&arc));
        arc
    }

    /// Get or compute diagnostics for raw `text`.
    ///
    /// Reuses the AST cache so parsing is skipped on a cache hit.
    /// Returns an empty slice on parse failure (callers that need the parse
    /// error should call [`parse_and_cache_ast`] directly).
    pub fn get_or_compute_diagnostics(
        &self,
        text: &str,
    ) -> Arc<Vec<marco_core::intelligence::Diagnostic>> {
        let content_hash = hash_bytes(text.as_bytes());
        if let Some(cached) = self.diagnostics_cache.get(&content_hash) {
            return cached;
        }
        match self.parse_and_cache_ast(text) {
            Ok(doc) => self.get_or_compute_diagnostics_for_doc(&doc, content_hash),
            Err(_) => Arc::new(Vec::new()),
        }
    }

    /// Invalidate all cached entries (e.g. on shutdown or theme change).
    pub fn clear(&self) {
        self.ast_cache.invalidate_all();
        self.html_cache.invalidate_all();
        self.section_html_cache.invalidate_all();
        self.toc_cache.invalidate_all();
        self.diagnostics_cache.invalidate_all();
    }

    /// Return a snapshot of current statistics.
    pub fn stats(&self) -> CacheStats {
        let mut s = self.stats.lock().map(|s| s.clone()).unwrap_or_default();
        // current_size = sum of live entries across all five Moka caches.
        s.current_size = self.ast_cache.entry_count()
            + self.html_cache.entry_count()
            + self.section_html_cache.entry_count()
            + self.toc_cache.entry_count()
            + self.diagnostics_cache.entry_count();
        s
    }

    /// Look up a cached AST by content hash without parsing.
    ///
    /// Returns `None` if the AST is not in the cache.  Callers that need a
    /// guaranteed AST should use [`parse_and_cache_ast`] instead.
    pub fn get_cached_ast(&self, content_hash: u64) -> Option<Arc<marco_core::Document>> {
        self.ast_cache.get(&content_hash)
    }

    /// Parse `text` (if not already cached) and store the resulting AST.
    ///
    /// Safe to call from any thread.  The AST is keyed by the blake3 content
    /// hash so concurrent calls with the same text are idempotent.
    pub fn parse_and_cache_ast(
        &self,
        text: &str,
    ) -> Result<Arc<marco_core::Document>, Box<dyn std::error::Error>> {
        let content_hash = hash_bytes(text.as_bytes());
        if let Some(ast) = self.ast_cache.get(&content_hash) {
            return Ok(ast);
        }
        let doc = marco_core::parse(text)?;
        let arc = Arc::new(doc);
        self.ast_cache.insert(content_hash, Arc::clone(&arc));
        Ok(arc)
    }
}

impl Default for ParserCache {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::section::split_into_sections;
    use marco_core::RenderOptions;

    #[test]
    fn smoke_same_input_hits_cache() {
        let cache = ParserCache::new();
        let text = "# Hello\n\nThis is a **test**.";
        let opts = RenderOptions::default();

        cache
            .render_with_cache(text, opts.clone())
            .expect("first render failed");
        cache
            .render_with_cache(text, opts)
            .expect("second render failed");

        let stats = cache.stats();
        assert_eq!(stats.misses, 1, "expected exactly 1 miss");
        assert_eq!(stats.hits, 1, "expected exactly 1 hit");
    }

    #[test]
    fn smoke_different_input_no_hits() {
        let cache = ParserCache::new();
        let opts = RenderOptions::default();

        cache
            .render_with_cache("# Doc A", opts.clone())
            .expect("first failed");
        cache
            .render_with_cache("# Doc B", opts)
            .expect("second failed");

        let stats = cache.stats();
        assert_eq!(stats.hits, 0, "expected 0 hits");
        assert_eq!(stats.misses, 2, "expected 2 misses");
    }

    #[test]
    fn smoke_different_options_same_text_is_miss() {
        let cache = ParserCache::new();
        let text = "# Test";
        let opts_a = RenderOptions {
            syntax_highlighting: true,
            ..RenderOptions::default()
        };
        let opts_b = RenderOptions {
            syntax_highlighting: false,
            ..RenderOptions::default()
        };

        cache
            .render_with_cache(text, opts_a)
            .expect("opts_a failed");
        cache
            .render_with_cache(text, opts_b)
            .expect("opts_b failed");

        let stats = cache.stats();
        assert_eq!(stats.misses, 2, "different options must miss HTML cache");
    }

    #[test]
    fn smoke_section_cache_only_rerenders_changed_section() {
        let cache = ParserCache::new();
        let opts = RenderOptions::default();

        let doc = "## Section A\n\nContent A.\n\n## Section B\n\nContent B.\n\n## Section C\n\nContent C.";
        let sections = split_into_sections(doc);
        assert_eq!(sections.len(), 3);

        // First call: all misses
        cache
            .render_sections_with_cache(&sections, &opts)
            .expect("first sections render failed");

        // Edit only section B (index 1)
        let doc2 = "## Section A\n\nContent A.\n\n## Section B\n\nCHANGED B.\n\n## Section C\n\nContent C.";
        let sections2 = split_into_sections(doc2);

        let results = cache
            .render_sections_with_cache(&sections2, &opts)
            .expect("second sections render failed");

        // Section A and C should be cache hits (same content hash)
        // We can verify by checking that the HTML for A and C is the same Arc pointer
        let first_a = cache.render_sections_with_cache(&sections, &opts).unwrap();
        assert!(
            Arc::ptr_eq(&first_a[0], &results[0]),
            "section A should be a cache hit (same Arc)"
        );
        assert!(
            Arc::ptr_eq(&first_a[2], &results[2]),
            "section C should be a cache hit (same Arc)"
        );
    }
}
