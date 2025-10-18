// Syntax highlighting: map AST nodes to SourceView5 text tags

use crate::ast::{Document, Node};
use crate::parser::Span;

#[derive(Debug, Clone)]
pub struct Highlight {
    pub span: Span,
    pub tag: HighlightTag,
}

#[derive(Debug, Clone)]
pub enum HighlightTag {
    Heading,
    Emphasis,
    Strong,
    Link,
    CodeSpan,
    CodeBlock,
}

// Generate highlights from AST
pub fn compute_highlights(document: &Document) -> Vec<Highlight> {
    log::debug!("Computing syntax highlights");
    
    // TODO: Walk AST and generate highlight ranges
    let highlights = Vec::new();
    
    log::info!("Generated {} highlights", highlights.len());
    highlights
}
