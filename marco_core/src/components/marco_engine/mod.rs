//! Marco Engine - Modular grammar-centered parsing and rendering system
//!
//! This module provides a streamlined markdown processing engine with:
//! - Pest-based parsing with Marco grammar
//! - Modular AST building (builders/)
//! - Modular HTML rendering (renderers/)
//! - Clean API: api::parse_markdown(), api::render_to_html(), api::parse_and_render()
//! - Block-level caching for performance optimization
//!
//! **Phase 2.6**: Old monolithic files archived (ast_builder.rs, render_html.rs)

// ============================================================================
// PHASE 2 MODULAR ARCHITECTURE (PRIMARY API)
// ============================================================================

pub mod api;         // Public API functions
pub mod builders;    // AST builders (block + inline)
pub mod renderers;   // HTML renderers (block + inline)
pub mod span;        // Span utilities

// ============================================================================
// CORE COMPONENTS (KEPT)
// ============================================================================

pub mod ast_node;       // AST node definitions (CommonMark only)
pub mod grammar;        // Pest grammar
pub mod parser;         // Parser utilities
pub mod parser_cache;   // Caching layer
pub mod parsers;        // Two-stage parser orchestrator

// ============================================================================
// ARCHIVED (moved to archive/)
// ============================================================================
// - ast_builder.rs → archive/ast_builder.rs.old (replaced by builders/)
// - render_html.rs → archive/render_html.rs.old (replaced by renderers/)

// ============================================================================
// PUBLIC RE-EXPORTS
// ============================================================================

// Core types
pub use ast_node::Node;
pub use grammar::{MarcoParser, Rule};

// New API (Phase 2.5)
pub use api::{parse_markdown as parse_to_ast, render_to_html, parse_and_render};

// Caching
pub use parser_cache::global_parser_cache;

// Renderers
pub use renderers::HtmlOptions;

// ============================================================================
// SIMPLIFIED 3-FUNCTION API (LEGACY - Use api:: functions instead)
// ============================================================================

use pest::Parser;

/// Parse markdown text into Pest pairs (LEGACY - use api::parse_markdown instead)
///
/// **Deprecated**: Use `api::parse_markdown()` for new code
#[deprecated(since = "0.2.0", note = "Use api::parse_markdown() instead")]
pub fn parse_text(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, String> {
    MarcoParser::parse(Rule::document, input).map_err(|e| e.to_string())
}

/// Build AST from Pest pairs (LEGACY - integrated into api::parse_markdown)
///
/// **Deprecated**: Use `api::parse_markdown()` which does this internally
#[deprecated(since = "0.2.0", note = "Use api::parse_markdown() instead")]
pub fn build_ast(pairs: pest::iterators::Pairs<'_, Rule>) -> Result<Node, String> {
    let mut builder = builders::BlockBuilder::new();
    builder.build_document(pairs)
        .map_err(|e| format!("AST build error: {}", e))
}

/// Render AST to HTML (LEGACY - use api::render_to_html instead)
///
/// **Deprecated**: Use `api::render_to_html()` for new code
#[deprecated(since = "0.2.0", note = "Use api::render_to_html() instead")]
pub fn render_html(ast: &Node, options: HtmlOptions) -> String {
    let renderer = renderers::block_renderer::BlockRenderer::new(options);
    renderer.render(ast)
}

/// Parse markdown to HTML with caching (LEGACY)
///
/// **Deprecated**: Cache integration will be updated in future
#[deprecated(since = "0.2.0", note = "Cache integration being updated")]
pub fn parse_to_html_cached(input: &str) -> Result<String, String> {
    global_parser_cache().render_with_cache(input, HtmlOptions::default())
        .map_err(|e| format!("Block-level HTML caching failed: {}", e))
}

/// Parse markdown to Pest pairs (LEGACY - for backwards compatibility)
///
/// **Deprecated**: Use `api::parse_markdown()` instead
#[deprecated(since = "0.2.0", note = "Use api::parse_markdown() for AST")]
#[allow(dead_code)]
pub fn parse_markdown_legacy(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, String> {
    parse_text(input)
}
