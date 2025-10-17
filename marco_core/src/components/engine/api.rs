//! Marco Engine Public API
//!
//! Provides a clean, simple API for markdown processing using the two-stage parser:
//! - `parse_markdown()` - Parse markdown to AST
//! - `render_to_html()` - Render AST to HTML
//! - `parse_and_render()` - One-step convenience function
//!
//! Uses the modular builders and renderers architecture with two-stage parsing.

use crate::components::engine::{
    ast_node::Node,
    parsers::orchestrator,
    reference_resolver::ReferenceResolver,
    renderers::{HtmlOptions, block_renderer::BlockRenderer},
};

/// Parse markdown text to AST using the two-stage parser with reference resolution
///
/// This performs a three-pass process:
/// 1. Parse blocks and inline content to AST
/// 2. Collect reference definitions from the AST
/// 3. Resolve reference links and images
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::engine::api;
///
/// let markdown = r#"
/// [google]: https://google.com "Google"
/// 
/// Visit [Google][google] for search.
/// "#;
/// 
/// let ast = api::parse_markdown(markdown).unwrap();
/// ```
///
/// # Errors
///
/// Returns error if:
/// - Pest parsing fails (invalid syntax)
/// - AST building fails (invalid structure)
pub fn parse_markdown(input: &str) -> Result<Node, String> {
    // Pass 1: Parse document to AST using orchestrator's two-stage parsing
    let mut ast = orchestrator::parse_document(input)?;
    
    // Pass 2: Collect reference definitions
    let mut resolver = ReferenceResolver::new();
    resolver.collect_definitions(&ast);
    
    // Pass 3: Resolve reference links and images
    resolver.resolve_references(&mut ast);
    
    Ok(ast)
}


/// Render AST to HTML using modular renderers
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::engine::{api, renderers::HtmlOptions};
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
/// use marco_core::components::engine::{api, renderers::HtmlOptions};
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

    // ========================================
    // Phase 4: Entity Reference End-to-End Tests
    // ========================================

    #[test]
    fn smoke_test_entity_named_end_to_end() {
        let input = "Use &nbsp; for non-breaking space and &copy; for copyright.";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Parse and render should succeed");
        let html = result.unwrap();
        
        // Should decode entities
        assert!(html.contains("\u{00A0}"), "Should contain non-breaking space");
        assert!(html.contains("©"), "Should contain copyright symbol");
        assert!(!html.contains("&nbsp;"), "Should not contain literal &nbsp;");
        assert!(!html.contains("&copy;"), "Should not contain literal &copy;");
    }

    #[test]
    fn smoke_test_entity_numeric_end_to_end() {
        let input = "Hash: &#35; and Euro: &#8364; and Rocket: &#128640;";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Parse and render should succeed");
        let html = result.unwrap();
        
        // Should decode numeric entities
        assert!(html.contains("#"), "Should contain hash from &#35;");
        assert!(html.contains("€"), "Should contain euro from &#8364;");
        assert!(html.contains("🚀"), "Should contain rocket emoji from &#128640;");
    }

    #[test]
    fn smoke_test_entity_hex_end_to_end() {
        let input = "Hash: &#x23; and Euro: &#x20AC; and Poop: &#x1F4A9;";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Parse and render should succeed");
        let html = result.unwrap();
        
        // Should decode hex entities
        assert!(html.contains("#"), "Should contain hash from &#x23;");
        assert!(html.contains("€"), "Should contain euro from &#x20AC;");
        assert!(html.contains("💩"), "Should contain poop emoji from &#x1F4A9;");
    }

    #[test]
    fn smoke_test_entity_invalid_renders_literally() {
        let input = "Invalid entity: &invalidname; should render as-is.";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Parse and render should succeed");
        let html = result.unwrap();
        
        // Invalid entities should render literally (but & will be HTML-escaped to &amp;)
        assert!(html.contains("&amp;invalidname;") || html.contains("&invalidname;"), 
                "Should contain invalid entity (possibly HTML-escaped)");
    }

    #[test]
    fn smoke_test_entity_in_emphasis() {
        let input = "*Copyright &copy; 2025*";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Parse and render should succeed");
        let html = result.unwrap();
        
        // Should decode entity inside emphasis
        assert!(html.contains("©"), "Should contain copyright symbol");
        assert!(html.contains("<em>") || html.contains("<i>"), "Should contain emphasis tag");
    }
}
