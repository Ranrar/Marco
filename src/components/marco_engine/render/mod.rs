//! Marco rendering system - transforms AST into various output formats
//!
//! This module provides a flexible rendering system that can transform Marco AST
//! into multiple output formats while maintaining consistency and extensibility.

pub mod html;
pub mod json;

// Re-export all renderer types
pub use html::{HtmlOptions, HtmlRenderer};
pub use json::JsonRenderer;

use crate::components::marco_engine::ast::Node;
use crate::components::marco_engine::errors::MarcoError;

/// Supported output formats for rendering
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Html,
    Json,
    JsonPretty,
}

/// Main rendering interface that provides a unified API for all output formats
pub struct MarcoRenderer;

impl MarcoRenderer {
    /// Render an AST to the specified format with default options
    pub fn render(ast: &Node, format: OutputFormat) -> Result<String, MarcoError> {
        match format {
            OutputFormat::Html => {
                let renderer = HtmlRenderer::new(HtmlOptions::default());
                Ok(renderer.render(ast))
            }
            OutputFormat::Json => {
                let renderer = JsonRenderer::new(false);
                renderer.render(ast).map_err(MarcoError::from)
            }
            OutputFormat::JsonPretty => {
                let renderer = JsonRenderer::new(true);
                renderer.render(ast).map_err(MarcoError::from)
            }
        }
    }

    /// Render to HTML with custom options
    pub fn render_html(ast: &Node, options: HtmlOptions) -> String {
        let renderer = HtmlRenderer::new(options);
        renderer.render(ast)
    }

    /// Render to JSON with formatting control
    pub fn render_json(ast: &Node, pretty: bool) -> Result<String, MarcoError> {
        let renderer = JsonRenderer::new(pretty);
        renderer.render(ast).map_err(MarcoError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::marco_engine::ast::{Node, Span};

    #[test]
    fn test_unified_renderer() {
        let ast = Node::Document {
            children: vec![
                Node::heading(1, vec![Node::text("Test", Span::empty())], Span::empty()),
                Node::paragraph(vec![Node::text("Content", Span::empty())], Span::empty()),
            ],
            span: Span::empty(),
        };

        // Test all formats
        let html = MarcoRenderer::render(&ast, OutputFormat::Html).unwrap();
        let json = MarcoRenderer::render(&ast, OutputFormat::Json).unwrap();
        let json_pretty = MarcoRenderer::render(&ast, OutputFormat::JsonPretty).unwrap();

        assert!(html.contains("<h1"));
        assert!(json.contains("\"type\":\"document\""));
        assert!(json_pretty.contains("\"type\": \"document\""));
    }

    #[test]
    fn test_custom_options() {
        let ast = Node::paragraph(
            vec![Node::text("Test paragraph", Span::empty())],
            Span::empty(),
        );

        // HTML with custom class prefix
        let html_options = HtmlOptions {
            class_prefix: "custom-".to_string(),
            ..Default::default()
        };
        let html = MarcoRenderer::render_html(&ast, html_options);
        assert!(html.contains("custom-paragraph"));
    }
}

// Legacy compatibility
#[derive(Debug, Clone, Default)]
pub struct MarkdownOptions {
    pub html_options: HtmlOptions,
    pub extension: MarkdownExtensions,
}

#[derive(Debug, Clone)]
pub struct MarkdownExtensions {
    pub table: bool,
    pub autolink: bool,
    pub strikethrough: bool,
    pub tasklist: bool,
    pub footnotes: bool,
    pub tagfilter: bool,
}

impl Default for MarkdownExtensions {
    fn default() -> Self {
        Self {
            table: true,
            autolink: true,
            strikethrough: true,
            tasklist: true,
            footnotes: true,
            tagfilter: true,
        }
    }
}

/// Legacy markdown_to_html function
pub fn markdown_to_html(input: &str, options: &MarkdownOptions) -> Result<String, MarcoError> {
    use crate::components::marco_engine::ast::AstBuilder;
    use crate::components::marco_engine::grammar::{MarcoParser, Rule};
    use pest::Parser;

    let pairs = MarcoParser::parse(Rule::document, input)
        .map_err(|e| MarcoError::parse_error(e.to_string()))?;
    let ast = AstBuilder::build(pairs)?;
    let renderer = HtmlRenderer::new(options.html_options.clone());
    Ok(renderer.render(&ast))
}
