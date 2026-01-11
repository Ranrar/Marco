// HTML renderer: AST → HTML for WebKit6 preview

pub mod html;
pub mod options;
pub mod preview_document;

pub use html::*;
pub use options::*;
pub use preview_document::*;

use crate::parser::Document;
use anyhow::Result;

// Main render entry point
pub fn render(document: &Document, options: &RenderOptions) -> Result<String> {
    log::info!("Starting HTML render");
    let html = render_html(document, options)?;
    log::debug!("Generated {} bytes of HTML", html.len());
    Ok(html)
}
