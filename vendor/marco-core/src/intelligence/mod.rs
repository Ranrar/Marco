//! Smart Markdown editor intelligence (in-process engine).
//!
//! This module replaces the old `lsp` naming with an intelligence-first layout:
//! - `markdown`: shared markdown model/view of parser data
//! - `analysis`: linting + diagnostics
//! - `editor`: highlighting, hover, completion
//! - `lsp_protocol`: optional protocol adapter surface

#[cfg(feature = "intelligence-diagnostics")]
pub mod analysis;
pub mod catalog;
#[cfg(any(
    feature = "intelligence-highlights",
    feature = "intelligence-completions",
    feature = "intelligence-hover"
))]
pub mod editor;
pub mod lsp_protocol;
pub mod markdown;
pub mod toc;

#[cfg(feature = "intelligence-diagnostics")]
pub use analysis::{
    compute_diagnostics, compute_diagnostics_critical, compute_diagnostics_with_options,
    compute_lints, compute_lints_detailed, compute_lints_detailed_with_options,
    compute_lints_with_options, Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticsOptions,
    DiagnosticsProfile, LintCodeBucket, LintDetailedReport, LintReport,
};
pub use catalog::{
    diagnostics_catalog, diagnostics_catalog_groups, diagnostics_catalog_settings,
    diagnostics_markdown_features, find_catalog_entry, find_catalog_entry_by_key,
    find_catalog_group, find_catalog_group_by_code, find_markdown_feature, DiagnosticsCatalog,
    DiagnosticsCatalogEntry, DiagnosticsCatalogGroup, DiagnosticsCatalogSettings,
    MarkdownFeatureCoverage,
};
#[cfg(feature = "intelligence-highlights")]
pub use editor::{compute_highlights, compute_highlights_with_source, Highlight, HighlightTag};
#[cfg(feature = "intelligence-hover")]
pub use editor::{get_hover_info, get_position_span, HoverInfo};
#[cfg(feature = "intelligence-completions")]
pub use editor::{get_markdown_completions, CompletionItem};

use crate::parser::Document;
#[cfg(feature = "intelligence-hover")]
use crate::parser::Position;

/// In-process provider for editor intelligence features.
#[derive(Default)]
pub struct MarkdownIntelligenceProvider {
    document: Option<Document>,
}

impl MarkdownIntelligenceProvider {
    /// Create a new intelligence provider with no active document.
    pub fn new() -> Self {
        log::info!("Markdown intelligence provider initialized");
        Self { document: None }
    }

    /// Replace the currently analyzed document.
    pub fn update_document(&mut self, document: Document) {
        self.document = Some(document);
    }

    /// Compute semantic highlights for the current document using source-aware markers.
    #[cfg(feature = "intelligence-highlights")]
    pub fn highlights(&self, source: &str) -> Vec<Highlight> {
        self.document
            .as_ref()
            .map(|doc| compute_highlights_with_source(doc, source))
            .unwrap_or_default()
    }

    /// Compute diagnostics for the current document.
    #[cfg(feature = "intelligence-diagnostics")]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.document
            .as_ref()
            .map(compute_diagnostics)
            .unwrap_or_default()
    }

    /// Compute diagnostics for the current document using custom options.
    #[cfg(feature = "intelligence-diagnostics")]
    pub fn diagnostics_with_options(&self, options: DiagnosticsOptions) -> Vec<Diagnostic> {
        self.document
            .as_ref()
            .map(|doc| compute_diagnostics_with_options(doc, options))
            .unwrap_or_default()
    }

    /// Resolve hover information for a source position.
    #[cfg(feature = "intelligence-hover")]
    pub fn hover(&self, position: Position) -> Option<HoverInfo> {
        self.document
            .as_ref()
            .and_then(|doc| get_hover_info(position, doc))
    }

    /// Get completion candidates for a query.
    #[cfg(feature = "intelligence-completions")]
    pub fn completions(&self, query: &str) -> Vec<CompletionItem> {
        get_markdown_completions(query)
    }
}
