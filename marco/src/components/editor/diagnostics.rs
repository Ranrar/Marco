//! One diagnostic type for everything the editor reports.
//!
//! Most diagnostics come from `marco_core`, which analyses the parsed document
//! and looks up its user-facing text in an embedded catalog keyed by
//! [`marco_core::intelligence::DiagnosticCode`]. Marco adds the one check that
//! crate cannot make: whether a link or image target actually exists. That is
//! a question about the filesystem, and the analysis is never told where the
//! document lives — so the *check* has to run here, even though the codes it
//! reports under (`MD206`, `MD404`) and all their text live in the catalog
//! with every other diagnostic.
//!
//! [`EditorDiagnostic`] is the common shape: catalog text already resolved, so
//! a diagnostic the parser raised and one Marco raised are indistinguishable
//! to the three places that display them — the squiggle underlines, the hover
//! popover, and the footer issues panel.

use marco_core::intelligence::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use marco_core::parser::position::{Position, Span};
use marco_shared::logic::link_path::{self, TargetOrigin};
use std::path::Path;

/// A diagnostic ready to display: severity, where it applies, and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDiagnostic {
    /// Displayed code (`MD201`, `MD404`, …).
    pub code: String,
    /// Where in the source the diagnostic applies.
    pub span: Span,
    pub severity: DiagnosticSeverity,
    /// Short headline.
    pub title: String,
    /// The specific complaint, naming the offending text where useful.
    pub message: String,
    /// Long-form explanation, when there is one.
    pub description: Option<String>,
    /// What to do about it.
    pub fix_suggestion: String,
}

impl EditorDiagnostic {
    /// Resolve a `marco_core` diagnostic's catalog text into a displayable one.
    pub fn from_core(diagnostic: &Diagnostic) -> Self {
        Self {
            code: diagnostic.code_id().to_string(),
            span: diagnostic.span,
            severity: diagnostic.severity,
            title: diagnostic
                .title_resolved()
                .unwrap_or(diagnostic.message.as_str())
                .to_string(),
            message: diagnostic.message.clone(),
            description: diagnostic
                .description_resolved()
                .map(str::to_string)
                .filter(|d| !d.trim().is_empty()),
            fix_suggestion: diagnostic.fix_suggestion_resolved().into_owned(),
        }
    }

    /// Byte length of the span, used to prefer the most specific diagnostic
    /// when several cover the same offset.
    pub fn span_len(&self) -> usize {
        self.span.end.offset.saturating_sub(self.span.start.offset)
    }
}

/// One [`EditorDiagnostic`] per local link or image destination in `text`
/// whose file is not on disk.
///
/// `document_dir` is the directory the document lives in; `None` for an
/// unsaved document, where a relative destination has nothing to resolve
/// against and so is left unjudged rather than reported broken.
pub fn missing_link_target_diagnostics(
    text: &str,
    document_dir: Option<&Path>,
) -> Vec<EditorDiagnostic> {
    link_path::find_missing_local_targets(text, document_dir)
        .into_iter()
        .map(|missing| {
            let (start_line, start_column) = link_path::line_and_byte_column(text, missing.start);
            let (end_line, end_column) = link_path::line_and_byte_column(text, missing.end);
            let code = match missing.origin {
                TargetOrigin::InlineImage => DiagnosticCode::MissingImageTarget,
                // A definition serves reference-style links and images alike,
                // and nothing in it says which — the link code is the one that
                // names the syntax the destination was actually written in.
                TargetOrigin::InlineLink | TargetOrigin::ReferenceDefinition => {
                    DiagnosticCode::MissingLinkTarget
                }
            };

            EditorDiagnostic::from_core(&Diagnostic {
                code,
                span: Span {
                    start: Position {
                        line: start_line,
                        column: start_column,
                        offset: missing.start,
                    },
                    end: Position {
                        line: end_line,
                        column: end_column,
                        offset: missing.end,
                    },
                },
                severity: code.default_severity(),
                message: code.format_message(&[("path", missing.markdown_path)]),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_missing_target_becomes_a_warning_spanning_the_destination() {
        let dir = tempfile::tempdir().expect("tempdir");
        let text = "intro\n\n[gone](./missing.md)\n";
        let diagnostics = missing_link_target_diagnostics(text, Some(dir.path()));

        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert_eq!(d.code, "MD206");
        assert_eq!(d.severity, DiagnosticSeverity::Warning);
        assert_eq!(d.span.start.line, 3);
        assert_eq!(d.span.start.column, 8);
        assert_eq!(
            &text[d.span.start.offset..d.span.end.offset],
            "./missing.md"
        );
        assert!(d.message.contains("./missing.md"));
        // Title, explanation and fix all come from the shared catalog rather
        // than from strings kept here.
        assert_eq!(d.title, "Link target not found");
        assert!(d.description.is_some());
        assert!(!d.fix_suggestion.is_empty());
    }

    #[test]
    fn smoke_broken_image_is_reported_under_the_image_code() {
        let dir = tempfile::tempdir().expect("tempdir");
        let diagnostics =
            missing_link_target_diagnostics("![alt](./missing.png)\n", Some(dir.path()));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "MD404");
        assert_eq!(diagnostics[0].title, "Image target not found");
    }

    #[test]
    fn smoke_broken_reference_definition_is_reported_under_the_link_code() {
        let dir = tempfile::tempdir().expect("tempdir");
        let diagnostics =
            missing_link_target_diagnostics("[ref]: ./missing.md\n", Some(dir.path()));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "MD206");
    }

    #[test]
    fn smoke_unsaved_document_produces_no_missing_target_diagnostics() {
        assert!(missing_link_target_diagnostics("[x](./a.md)\n", None).is_empty());
    }
}
