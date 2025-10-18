// Diagnostics: parse errors, broken links, etc.

use crate::parser::Span;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

// Compute diagnostics for document
pub fn compute_diagnostics(document: &crate::ast::Document) -> Vec<Diagnostic> {
    log::debug!("Computing diagnostics");
    
    // TODO: Walk AST and find issues
    let diagnostics = Vec::new();
    
    log::info!("Found {} diagnostics", diagnostics.len());
    diagnostics
}
