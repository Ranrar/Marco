//! Markdown analysis: linting + diagnostics.

pub mod diagnostics;
pub mod lint;

pub use diagnostics::{
    compute_diagnostics, compute_diagnostics_critical, compute_diagnostics_with_options,
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticsOptions, DiagnosticsProfile,
};
pub use lint::{
    compute_lints, compute_lints_detailed, compute_lints_detailed_with_options,
    compute_lints_with_options, LintCodeBucket, LintDetailedReport, LintReport,
};
