// Marco Parser - 100% CommonMark Compliant (652/652 spec examples passing)
// nom-based parser with full UTF-8 support (em dashes, smart quotes, Japanese, Arabic, emoji)

pub mod ast;
pub mod position;

// Modular parser structure
pub mod blocks;
pub mod inlines;

// Re-export public API
pub use ast::*;
pub use position::*;
pub use blocks::parse_blocks;
pub use inlines::parse_inlines;

use anyhow::Result;

/// Parse Markdown text into Document AST
pub fn parse(input: &str) -> Result<Document> {
    log::info!("Starting parse: {} bytes", input.len());
    
    let document = parse_blocks(input)?;
    log::debug!("Parsed {} blocks", document.children.len());
    
    Ok(document)
}
