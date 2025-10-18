// Two-stage parser: block-level → inline-level

pub mod block_parser;
pub mod inline_parser;
pub mod position;

pub use block_parser::*;
pub use inline_parser::*;
pub use position::*;

use crate::ast::Document;
use anyhow::Result;

// Main parser entry point: Markdown text → Document AST
pub fn parse(input: &str) -> Result<Document> {
    log::info!("Starting parse: {} bytes", input.len());
    
    // Stage 1: Parse block-level structure
    let blocks = parse_blocks(input)?;
    log::debug!("Parsed {} blocks", blocks.len());
    
    // Stage 2: Parse inline elements within blocks
    let document = parse_inlines(blocks)?;
    log::debug!("Document AST built with {} nodes", document.len());
    
    Ok(document)
}
