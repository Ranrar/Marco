// HTML renderer: AST → HTML for WebKit6 preview

pub mod code_languages;
pub mod html;
pub mod options;
pub mod plarform_mentions;
pub mod preview_document;
pub mod syntect_highlighter;

pub use code_languages::*;
pub use html::*;
pub use options::*;
pub use preview_document::*;
pub use syntect_highlighter::*;

use crate::parser::Document;
use anyhow::Result;

// Main render entry point
pub fn render(document: &Document, options: &RenderOptions) -> Result<String> {
    log::info!("Starting HTML render");
    let html = render_html(document, options)?;
    log::debug!("Generated {} bytes of HTML", html.len());
    Ok(html)
}
