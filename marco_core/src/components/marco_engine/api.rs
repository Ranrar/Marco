//! Marco Engine Public API
//!
//! Provides a clean, simple API for markdown processing:
//! - `parse_markdown()` - Parse markdown to AST (TODO: Phase 2.5)
//! - `render_to_html()` - Render AST to HTML (TODO: Phase 2.5)
//! - `parse_and_render()` - One-step convenience function (TODO: Phase 2.5)
//!
//! Uses the two-stage parser architecture internally.
//!
//! **Status**: Phase 2.1 stub - functions will be implemented in Phase 2.5

use crate::components::marco_engine::{
    ast_node::Node,
    renderers::HtmlOptions,
};

/// Parse markdown text to AST using two-stage parser
///
/// **TODO**: Phase 2.5 - Currently returns placeholder error
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::marco_engine::api;
///
/// let ast = api::parse_markdown("# Hello World").unwrap();
/// ```
pub fn parse_markdown(_input: &str) -> Result<Node, String> {
    // TODO: Phase 2.5 - Implement using:
    // orchestrator::parse_document(input) → returns AST instead of HTML
    // For now, return error to avoid breaking compilation
    Err("Phase 2.5: parse_markdown not yet implemented".to_string())
}

/// Render AST to HTML
///
/// **TODO**: Phase 2.5 - Currently returns placeholder
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::marco_engine::{api, renderers::HtmlOptions};
///
/// let ast = api::parse_markdown("# Hello").unwrap();
/// let html = api::render_to_html(&ast, HtmlOptions::default());
/// ```
pub fn render_to_html(_ast: &Node, _options: HtmlOptions) -> String {
    // TODO: Phase 2.5 - Implement using:
    // let block_renderer = BlockRenderer::new(options.clone());
    // let inline_renderer = InlineRenderer::new(options);
    // block_renderer.render_with_inline(&inline_renderer, ast)
    String::new()
}

/// Parse markdown and render to HTML in one step
///
/// **TODO**: Phase 2.5 - Currently uses old API
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::marco_engine::{api, renderers::HtmlOptions};
///
/// let html = api::parse_and_render("# Hello", HtmlOptions::default()).unwrap();
/// ```
pub fn parse_and_render(input: &str, _options: HtmlOptions) -> Result<String, String> {
    // TODO: Phase 2.5 - Use new parse_markdown + render_to_html
    // For now, use existing orchestrator (returns HTML directly)
    use crate::components::marco_engine::parsers::orchestrator;
    orchestrator::parse_document(input)
}

/// Parse markdown to HTML with caching (TODO: implement)
///
/// **TODO**: Phase 2.5 - Integrate with cache.rs
///
/// # Example
///
/// ```rust,no_run
/// use marco_core::components::marco_engine::{api, renderers::HtmlOptions};
///
/// let html = api::parse_and_render_cached("# Hello", HtmlOptions::default()).unwrap();
/// ```
pub fn parse_and_render_cached(input: &str, options: HtmlOptions) -> Result<String, String> {
    // TODO: Phase 2.5 - Integrate with cache.rs
    parse_and_render(input, options)
}
