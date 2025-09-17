//! Marco Engine - Simplified grammar-centered parsing and rendering system
//!
//! This module provides a streamlined markdown processing engine with:
//! - Pest-based parsing with custom Marco syntax
//! - Direct AST building from grammar rules
//! - HTML rendering with simple pattern matching
//! - Essential 3-function API: parse → build_ast → render

// Import modules directly (no subfolders)
pub mod ast_builder;
pub mod ast_node;
pub mod grammar;
pub mod parser;
pub mod render_html;

// Re-export main types for the 3-function API (removed errors module)
pub use ast_builder::AstBuilder;
pub use ast_node::Node;
pub use grammar::{MarcoParser, Rule};
pub use render_html::{HtmlOptions, HtmlRenderer};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// SIMPLIFIED 3-FUNCTION API
// ============================================================================

use pest::Parser;

/// Core function 1: Parse markdown text into Pest pairs
pub fn parse_text(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, String> {
    MarcoParser::parse(Rule::document, input).map_err(|e| e.to_string())
}

/// Core function 2: Build AST from Pest pairs
pub fn build_ast(pairs: pest::iterators::Pairs<'_, Rule>) -> Result<Node, String> {
    AstBuilder::build(pairs)
}

/// Core function 3: Render AST to HTML
pub fn render_html(ast: &Node, options: HtmlOptions) -> String {
    HtmlRenderer::new(options).render(ast)
}

/// Convenience function: Parse markdown to HTML (combines all 3 functions)
pub fn parse_to_html(input: &str) -> Result<String, String> {
    let pairs = parse_text(input)?;
    let ast = build_ast(pairs)?;
    Ok(render_html(&ast, HtmlOptions::default()))
}

/// Legacy function: Parse markdown text (alias for parse_text) - Used by footer.rs
pub fn parse_markdown(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, String> {
    parse_text(input)
}
