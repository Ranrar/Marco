//! Lint orchestration for markdown analysis.

use crate::intelligence::analysis::{
    compute_diagnostics_with_options, DiagnosticCode, DiagnosticSeverity, DiagnosticsOptions,
};
use crate::parser::Document;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LintReport {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub hints: usize,
}

impl LintReport {
    /// Total number of diagnostics represented by this report.
    pub const fn total(&self) -> usize {
        self.errors + self.warnings + self.infos + self.hints
    }

    /// True when there are no diagnostics.
    pub const fn is_clean(&self) -> bool {
        self.total() == 0
    }

    /// True when at least one error exists.
    pub const fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// True when warnings, infos, or hints are present.
    pub const fn has_non_error_issues(&self) -> bool {
        self.warnings > 0 || self.infos > 0 || self.hints > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintCodeBucket {
    pub code: DiagnosticCode,
    pub count: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LintDetailedReport {
    pub summary: LintReport,
    pub by_code: Vec<LintCodeBucket>,
}

impl LintDetailedReport {
    /// Sum of all per-code counts.
    pub fn total_from_buckets(&self) -> usize {
        self.by_code.iter().map(|bucket| bucket.count).sum()
    }

    /// True when bucket counts are consistent with the summary total.
    pub fn is_consistent(&self) -> bool {
        self.total_from_buckets() == self.summary.total()
    }
}

/// Compute aggregate lint counts from diagnostics.
pub fn compute_lints(document: &Document) -> LintReport {
    compute_lints_with_options(document, DiagnosticsOptions::all())
}

/// Compute aggregate lint counts from diagnostics using policy options.
pub fn compute_lints_with_options(document: &Document, options: DiagnosticsOptions) -> LintReport {
    compute_lints_detailed_with_options(document, options).summary
}

/// Compute aggregate lint counts and per-code breakdown.
pub fn compute_lints_detailed(document: &Document) -> LintDetailedReport {
    compute_lints_detailed_with_options(document, DiagnosticsOptions::all())
}

/// Compute aggregate lint counts and per-code breakdown with policy options.
pub fn compute_lints_detailed_with_options(
    document: &Document,
    options: DiagnosticsOptions,
) -> LintDetailedReport {
    let mut report = LintReport::default();

    // Use a deterministic key order so output is stable for tests/logging.
    let mut counts: BTreeMap<&'static str, (DiagnosticCode, usize)> = BTreeMap::new();

    for diagnostic in compute_diagnostics_with_options(document, options) {
        match diagnostic.severity {
            DiagnosticSeverity::Error => report.errors += 1,
            DiagnosticSeverity::Warning => report.warnings += 1,
            DiagnosticSeverity::Info => report.infos += 1,
            DiagnosticSeverity::Hint => report.hints += 1,
        }

        let code_id = diagnostic.code.as_str();
        if let Some((_code, count)) = counts.get_mut(code_id) {
            *count += 1;
        } else {
            counts.insert(code_id, (diagnostic.code, 1));
        }
    }

    let by_code = counts
        .into_iter()
        .map(|(_id, (code, count))| LintCodeBucket { code, count })
        .collect();

    LintDetailedReport {
        summary: report,
        by_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Document, Node, NodeKind, Position, Span};

    fn span(line: usize, start_col: usize, end_col: usize, start_offset: usize) -> Span {
        Span {
            start: Position {
                line,
                column: start_col,
                offset: start_offset,
            },
            end: Position {
                line,
                column: end_col,
                offset: start_offset + (end_col.saturating_sub(start_col)),
            },
        }
    }

    #[test]
    fn smoke_test_lint_report_helpers() {
        let report = LintReport {
            errors: 1,
            warnings: 2,
            infos: 3,
            hints: 4,
        };

        assert_eq!(report.total(), 10);
        assert!(!report.is_clean());
        assert!(report.has_errors());
        assert!(report.has_non_error_issues());
    }

    #[test]
    fn smoke_test_compute_lints_detailed_groups_by_code() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Image {
                        url: "".to_string(),
                        alt: "".to_string(),
                    },
                    span: Some(span(1, 1, 5, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Image {
                        url: "".to_string(),
                        alt: "image".to_string(),
                    },
                    span: Some(span(2, 1, 5, 10)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let detailed = compute_lints_detailed(&doc);
        assert_eq!(detailed.summary.errors, 2);

        let empty_image_url = detailed
            .by_code
            .iter()
            .find(|bucket| bucket.code == DiagnosticCode::EmptyImageUrl)
            .expect("Expected EmptyImageUrl bucket");
        assert_eq!(empty_image_url.count, 2);
    }

    #[test]
    fn smoke_test_compute_lints_with_options_respects_critical_profile() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: Some(span(1, 1, 40, 0)),
                children: vec![Node {
                    kind: NodeKind::Link {
                        url: "http://example.com".to_string(),
                        title: None,
                    },
                    span: Some(span(1, 2, 20, 1)),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let all = compute_lints_with_options(&doc, DiagnosticsOptions::all());
        let critical = compute_lints_with_options(&doc, DiagnosticsOptions::critical_only());

        assert!(all.infos > 0);
        assert_eq!(critical.total(), 0);
    }

    #[test]
    fn smoke_test_lint_detailed_report_consistency() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Image {
                        url: "".to_string(),
                        alt: "".to_string(),
                    },
                    span: Some(span(1, 1, 5, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: Some(span(2, 1, 30, 10)),
                    children: vec![Node {
                        kind: NodeKind::Link {
                            url: "http://example.com".to_string(),
                            title: None,
                        },
                        span: Some(span(2, 2, 24, 11)),
                        children: vec![],
                    }],
                },
            ],
            ..Default::default()
        };

        let detailed = compute_lints_detailed(&doc);
        assert!(detailed.is_consistent());
    }

    #[test]
    fn smoke_test_lint_code_buckets_are_sorted_by_code_id() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 10,
                        text: "".to_string(),
                        id: None,
                    },
                    span: Some(span(1, 1, 2, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Image {
                        url: "".to_string(),
                        alt: "".to_string(),
                    },
                    span: Some(span(2, 1, 5, 10)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let detailed = compute_lints_detailed(&doc);
        let mut previous = "";
        for bucket in &detailed.by_code {
            let current = bucket.code.as_str();
            assert!(
                previous <= current,
                "lint buckets must be sorted by code id"
            );
            previous = current;
        }
    }
}
