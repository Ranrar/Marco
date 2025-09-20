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
pub mod cache;
pub mod grammar;
pub mod parser;
pub mod parser_cache;
pub mod render_html;

// Re-export main types for the 3-function API (removed errors module)
pub use ast_builder::AstBuilder;
pub use ast_node::Node;
pub use cache::{ASTCache, global_ast_cache};
pub use grammar::{MarcoParser, Rule};
pub use parser_cache::{MarcoParserCache, global_parser_cache, ParserCacheStats};
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
    // Use new parser cache for block-level AST caching
    global_parser_cache().parse_with_cache(input)
        .map_err(|e| format!("Block-level AST caching failed: {}", e))
}

/// Enhanced incremental function: Parse text with incremental block processing
pub fn parse_to_ast_incremental(input: &str) -> Result<Vec<Node>, String> {
    // Use incremental parsing for optimal performance on document edits
    global_parser_cache().parse_with_cache_incremental(input)
        .map_err(|e| format!("Incremental block parsing failed: {}", e))
}

/// Cached function: Parse text with AST caching (replaces parse_text + build_ast) - LEGACY
pub fn parse_cached(input: &str) -> Result<Node, String> {
    // Keep using document-level cache for backward compatibility
    global_ast_cache().parse_cached(input)
        .map_err(|e| format!("Document-level parsing failed: {}", e))
}

/// Legacy function: Parse markdown text (alias for parse_text) - Used by footer.rs
pub fn parse_markdown(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, String> {
    parse_text(input)
}
