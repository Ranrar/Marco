//! Marco Engine - Simplified grammar-centered parsing and rendering system
//!
//! This module provides a streamlined markdown processing engine with:
//! - Pest-based parsing with custom Marco syntax
//! - Direct AST building from grammar rules
//! - HTML rendering with simple pattern matching
//! - Essential 3-function API: parse → build_ast → render
//! - Block-level AST/HTML caching for performance optimization

// Import modules directly (no subfolders)
pub mod ast_builder;
pub mod ast_node;
pub mod grammar;
pub mod parser;
pub mod parser_cache;
pub mod render_html;

// Re-export main types for the 3-function API (removed errors module)
pub use ast_builder::AstBuilder;
pub use ast_node::Node;
pub use parser_cache::{global_parser_cache};
pub use grammar::{MarcoParser, Rule};
pub use render_html::{HtmlOptions, HtmlRenderer};
// Re-export parser utilities for testing and convenience
pub use parser::{ParseResult, parse_document, parse_with_rule};

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

/// Convenience function: Parse markdown to HTML (now uses block-level caching internally)
pub fn parse_to_html(input: &str) -> Result<String, String> {
    // Use new parser cache internally for better performance
    parse_to_html_cached(input)
}

/// Enhanced cached function: Parse markdown to HTML with block-level caching
pub fn parse_to_html_cached(input: &str) -> Result<String, String> {
    // Use new parser cache for block-level HTML caching
    global_parser_cache().render_with_cache(input, HtmlOptions::default())
        .map_err(|e| format!("Block-level HTML caching failed: {}", e))
}

/// Enhanced cached function: Parse text with block-level AST caching
pub fn parse_to_ast_cached(input: &str) -> Result<Node, String> {
    // Use simple parser cache for document-level AST caching
    global_parser_cache().parse_with_cache(input)
        .map_err(|e| format!("Simple AST caching failed: {}", e))
}

/// Enhanced incremental function: Parse text with incremental block processing
pub fn parse_to_ast_incremental(input: &str) -> Result<Vec<Node>, String> {
    // Simplified - just parse normally and return as single-item vec
    // Real incremental parsing removed as per spec (too complex)
    let node = global_parser_cache().parse_with_cache(input)
        .map_err(|e| format!("Incremental block parsing failed: {}", e))?;
    Ok(vec![node])
}

/// Cached function: Parse text with AST caching - NOW USING SIMPLE CACHE
pub fn parse_cached(input: &str) -> Result<Node, String> {
    // Use simple parser cache for better performance and maintainability
    global_parser_cache().parse_with_cache(input)
        .map_err(|e| format!("Simple cache parsing failed: {}", e))
}

/// Legacy function: Parse markdown text (alias for parse_text) - Used by footer.rs
pub fn parse_markdown(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, String> {
    parse_text(input)
}
