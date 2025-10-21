// Marco Parser - 100% CommonMark Compliant (652/652 spec examples passing)
// nom-based parser with full UTF-8 support (em dashes, smart quotes, Japanese, Arabic, emoji)

pub mod ast;
pub mod block_parser;
pub mod inline_parser;
pub mod position;

pub use ast::*;
pub use block_parser::*;
pub use inline_parser::*;
pub use position::*;

use anyhow::Result;

// Parse Markdown text into Document AST
pub fn parse(input: &str) -> Result<Document> {
    log::info!("Starting parse: {} bytes", input.len());
    
    let document = parse_blocks(input)?;
    log::debug!("Parsed {} blocks", document.children.len());
    
    Ok(document)
}
