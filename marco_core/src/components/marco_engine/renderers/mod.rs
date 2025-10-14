//! HTML rendering from AST nodes
//!
//! Provides modular renderers for converting AST to HTML.
//! Split into block-level and inline-level renderers.
//!
//! # Architecture
//!
//! - `block_renderer` - Renders block-level nodes to HTML
//! - `inline_renderer` - Renders inline-level nodes to HTML
//! - `helpers` - Shared rendering utilities

pub mod block_renderer;
pub mod helpers;
pub mod inline_renderer;

pub use block_renderer::BlockRenderer;
pub use inline_renderer::InlineRenderer;

/// HTML rendering options
#[derive(Debug, Clone)]
pub struct HtmlOptions {
    /// Enable syntax highlighting for code blocks
    pub syntax_highlighting: bool,
    /// Enable smart punctuation conversion
    pub smart_punctuation: bool,
    /// Base URL for relative links
    pub base_url: Option<String>,
    /// Enable HTML sanitization
    pub sanitize_html: bool,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            smart_punctuation: false,
            base_url: None,
            sanitize_html: true,
        }
    }
}
