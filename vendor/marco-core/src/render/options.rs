//! Rendering options and configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
/// Configuration options for HTML rendering.
pub struct RenderOptions {
    /// Enable syntax highlighting for fenced code blocks when possible.
    pub syntax_highlighting: bool,
    /// Show line numbers in rendered code blocks.
    pub line_numbers: bool,
    /// Theme name used by renderer and highlighter integrations.
    pub theme: String,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            line_numbers: false,
            theme: "github".to_string(),
        }
    }
}
