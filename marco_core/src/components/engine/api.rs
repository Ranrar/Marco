//! Marco Engine Public API
//!
//! Provides a clean, simple API for markdown processing:
//! - `parse_markdown()` - Parse markdown to AST
//! - `render_to_html()` - Render AST to HTML
//! - `parse_and_render()` - One-step convenience function
//!
//! Uses the modular builders and renderers architecture.
//!
//! **Status**: Phase 2.5 - New modular API implementation

use crate::components::engine::{
    ast_node::Node,
    grammar::{MarcoParser, Rule},
    builders::{AstError, BlockBuilder},
    renderers::{HtmlOptions, block_renderer::BlockRenderer},
};
use pest::Parser;

/// Parse markdown text to AST using modular builders
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::marco_engine::api;
///
/// let ast = api::parse_markdown("# Hello World").unwrap();
/// ```
///
/// # Errors
///
/// Returns error if:
/// - Pest parsing fails (invalid syntax)
/// - AST building fails (invalid structure)
pub fn parse_markdown(input: &str) -> Result<Node, String> {
    // Step 1: Parse with Pest
    let pairs = MarcoParser::parse(Rule::document, input)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    // Step 2: Build AST with modular builder
    let mut builder = BlockBuilder::new();
    builder.build_document(pairs)
        .map_err(|e| format!("AST build error: {}", e))
}


/// Render AST to HTML using modular renderers
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::marco_engine::{api, renderers::HtmlOptions};
///
/// let ast = api::parse_markdown("# Hello").unwrap();
/// let html = api::render_to_html(&ast, HtmlOptions::default());
/// ```
pub fn render_to_html(ast: &Node, options: HtmlOptions) -> String {
    // Use modular block renderer (which handles inline rendering internally)
    let renderer = BlockRenderer::new(options);
    renderer.render(ast)
}

/// Parse markdown and render to HTML in one step
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::marco_engine::{api, renderers::HtmlOptions};
///
/// let html = api::parse_and_render("# Hello", HtmlOptions::default()).unwrap();
/// ```
///
/// # Errors
///
/// Returns error if parsing or AST building fails.
pub fn parse_and_render(input: &str, options: HtmlOptions) -> Result<String, String> {
    let ast = parse_markdown(input)?;
    Ok(render_to_html(&ast, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_markdown() {
        let input = "# Hello World\n\nThis is a paragraph.";
        let result = parse_markdown(input);
        assert!(result.is_ok(), "Parse should succeed");
        
        let ast = result.unwrap();
        let debug_str = format!("{:?}", ast);
        assert!(debug_str.contains("Hello World"), "Should contain heading text");
    }

    #[test]
    fn smoke_test_render_to_html() {
        let input = "# Hello\n\nSimple paragraph.";
        let ast = parse_markdown(input).expect("Parse failed");
        let html = render_to_html(&ast, HtmlOptions::default());
        
        assert!(html.contains("<h1"), "Should contain h1 tag");
        assert!(html.contains("Hello"), "Should contain heading text");
        assert!(html.contains("<p>"), "Should contain paragraph tag");
        // NOTE: Inline formatting (bold, italic, etc.) not yet implemented in new builder
        // This is a known limitation - TODO for Phase 2.7 or later
    }

    #[test]
    fn smoke_test_parse_and_render() {
        let input = "# Title\n\nParagraph text.";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Parse and render should succeed");
        let html = result.unwrap();
        assert!(html.contains("<h1"), "Should contain heading");
        assert!(html.contains("Title"), "Should contain title text");
        assert!(html.contains("<p>"), "Should contain paragraph");
        // NOTE: Lists not fully implemented yet - simplified test for now
    }
}
