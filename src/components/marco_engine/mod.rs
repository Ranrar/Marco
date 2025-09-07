//! Marco Engine - Complete AST-based parsing and rendering system
//!
//! This module provides a comprehensive markdown processing engine with:
//! - Pest-based parsing with custom Marco syntax
//! - Strongly-typed AST with visitor patterns
//! - Multiple output formats (HTML, Text, JSON)
//! - Async and parallel processing capabilities
//! - GTK integration utilities

pub mod ast;
pub mod engine;
pub mod errors;
pub mod grammar;
pub mod render;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod simple_tests;

// Re-export main types for convenience
pub use ast::{AstBuilder, Node, Span, Visitor, VisitorMut};
pub use engine::{AsyncMarcoPipeline, MarcoEngine, MarcoPipeline, ParallelMarcoPipeline};
pub use errors::MarcoError;
pub use grammar::{MarcoParser, Rule};
pub use render::{
    markdown_to_html, HtmlOptions, MarcoRenderer, MarkdownExtensions, MarkdownOptions,
    OutputFormat, TextOptions,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Quick access functions
pub fn parse_to_html(input: &str) -> Result<String, MarcoError> {
    MarcoEngine::to_html(input)
}

pub fn parse_to_text(input: &str) -> Result<String, MarcoError> {
    MarcoEngine::to_text(input)
}

pub fn parse_to_json(input: &str, pretty: bool) -> Result<String, MarcoError> {
    MarcoEngine::to_json(input, pretty)
}

/// Async versions
pub async fn parse_to_html_async(input: &str) -> Result<String, MarcoError> {
    MarcoEngine::to_html_async(input).await
}

pub async fn parse_to_text_async(input: &str) -> Result<String, MarcoError> {
    MarcoEngine::to_text_async(input).await
}

pub async fn parse_to_json_async(input: &str, pretty: bool) -> Result<String, MarcoError> {
    MarcoEngine::to_json_async(input, pretty).await
}

// Legacy compatibility functions
use pest::Parser;

/// Legacy function for parsing with a specific rule
pub fn parse_with_rule(
    input: &str,
    rule: Rule,
) -> Result<pest::iterators::Pairs<Rule>, MarcoError> {
    MarcoParser::parse(rule, input).map_err(|e| MarcoError::Parse(e.to_string()))
}

/// Legacy function for parsing a complete document
pub fn parse_document(input: &str) -> Result<Node, MarcoError> {
    let pairs =
        MarcoParser::parse(Rule::document, input).map_err(|e| MarcoError::Parse(e.to_string()))?;
    AstBuilder::build(pairs)
}

/// Legacy function for debugging pest pairs
pub fn print_pairs(pairs: pest::iterators::Pairs<Rule>, indent: usize) {
    for pair in pairs {
        println!(
            "{}{:?}: {}",
            " ".repeat(indent),
            pair.as_rule(),
            pair.as_str()
        );
        print_pairs(pair.into_inner(), indent + 2);
    }
}

/// Legacy parse_markdown function (alias for parse_document)
pub fn parse_markdown(input: &str) -> Result<Node, MarcoError> {
    parse_document(input)
}
