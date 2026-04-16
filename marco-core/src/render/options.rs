// Rendering options and configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderOptions {
    pub syntax_highlighting: bool,
    pub line_numbers: bool,
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
