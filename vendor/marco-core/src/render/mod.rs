//! HTML renderer entry points and render support modules.
//!
//! This module exposes the stable rendering API used by consumers:
//! [`render()`] and [`RenderOptions`].

/// Base stylesheet used by preview rendering.
pub mod base_css;
/// Language normalization and metadata for code blocks.
pub mod code_languages;
/// Diagram rendering helpers (for example Mermaid).
#[cfg(feature = "render-diagrams")]
pub mod diagram;
/// Core Markdown AST to HTML renderer.
pub mod markdown;
/// Math rendering helpers.
#[cfg(feature = "render-math")]
pub mod math;
/// Public render configuration options.
pub mod options;
/// Platform mention rendering helpers.
pub mod plarform_mentions;
/// HTML document wrappers for preview pages.
pub mod preview_document;
/// Syntax highlighting support based on syntect.
#[cfg(feature = "render-syntax-highlighting")]
pub mod syntect_highlighter;
/// Theme metadata: parses `--theme-*` descriptive tokens from theme CSS.
pub mod theme_meta;

/// Re-export code language helpers.
pub use code_languages::*;
/// Re-export core HTML render helpers.
pub use markdown::*;
/// Re-export render options.
pub use options::*;
/// Re-export preview document helpers.
pub use preview_document::*;
/// Re-export syntax highlighter helpers.
#[cfg(feature = "render-syntax-highlighting")]
pub use syntect_highlighter::*;
/// Re-export theme metadata helpers.
pub use theme_meta::*;

use crate::parser::Document;

/// Render a parsed Markdown [`Document`] into HTML.
pub fn render(
    document: &Document,
    options: &RenderOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    log::info!("Starting HTML render");
    let html = render_html(document, options)?;
    log::debug!("Generated {} bytes of HTML", html.len());
    Ok(html)
}
