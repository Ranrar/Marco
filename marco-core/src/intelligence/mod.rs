//! Smart Markdown editor intelligence (in-process engine).
//!
//! This module replaces the old `lsp` naming with an intelligence-first layout:
//! - `markdown`: shared markdown model/view of parser data
//! - `analysis`: linting + diagnostics
//! - `editor`: highlighting, hover, completion
//! - `lsp_protocol`: optional protocol adapter surface

pub mod analysis;
pub mod catalog;
pub mod editor;
pub mod lsp_protocol;
pub mod markdown;
pub mod toc;

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
pub use editor::{
    compute_highlights, compute_highlights_with_source, get_hover_info, get_markdown_completions,
    get_position_span, CompletionItem, Highlight, HighlightTag, HoverInfo,
};

use crate::parser::{Document, Position};

/// In-process provider for editor intelligence features.
#[derive(Default)]
pub struct MarkdownIntelligenceProvider {
    document: Option<Document>,
}

impl MarkdownIntelligenceProvider {
    pub fn new() -> Self {
        log::info!("Markdown intelligence provider initialized");
        Self { document: None }
    }

    pub fn update_document(&mut self, document: Document) {
        self.document = Some(document);
    }

    pub fn highlights(&self, source: &str) -> Vec<Highlight> {
        self.document
            .as_ref()
            .map(|doc| compute_highlights_with_source(doc, source))
            .unwrap_or_default()
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.document
            .as_ref()
            .map(compute_diagnostics)
            .unwrap_or_default()
    }

    pub fn diagnostics_with_options(&self, options: DiagnosticsOptions) -> Vec<Diagnostic> {
        self.document
            .as_ref()
            .map(|doc| compute_diagnostics_with_options(doc, options))
            .unwrap_or_default()
    }

    pub fn hover(&self, position: Position) -> Option<HoverInfo> {
        self.document
            .as_ref()
            .and_then(|doc| get_hover_info(position, doc))
    }

    pub fn completions(&self, query: &str) -> Vec<CompletionItem> {
        get_markdown_completions(query)
    }
}
