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
    /// Enable CSS classes (vs inline styles)
    pub css_classes: bool,
    /// Enable inline styles (vs CSS classes)
    pub inline_styles: bool,
    /// CSS class prefix for generated HTML
    pub class_prefix: String,
    /// Enable HTML sanitization
    pub sanitize_html: bool,
    /// Theme mode for syntax highlighting ("light" or "dark")
    pub theme_mode: String,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            css_classes: true,
            inline_styles: false,
            class_prefix: "marco-".to_string(),
            sanitize_html: true,
            theme_mode: "light".to_string(),
        }
    }
}

impl HtmlOptions {
    /// Create HtmlOptions with a specific theme mode
    pub fn with_theme_mode(theme_mode: &str) -> Self {
        Self {
            theme_mode: theme_mode.to_string(),
            ..Self::default()
        }
    }
}
