// Two-stage parser: block-level → inline-level

pub mod ast;
pub mod block_parser;
pub mod inline_parser;
pub mod position;

pub use ast::*;
pub use block_parser::*;
pub use inline_parser::*;
pub use position::*;

use anyhow::Result;

// Main parser entry point: Markdown text → Document AST
pub fn parse(input: &str) -> Result<Document> {
    log::info!("Starting parse: {} bytes", input.len());
    
    // Stage 1: Parse block-level structure (now returns Document directly)
    let document = parse_blocks(input)?;
    log::debug!("Parsed {} blocks", document.children.len());
    
    // Stage 2: Parse inline elements within blocks (TODO: implement inline parsing)
    // For now, we already have basic text nodes in paragraphs
    // let document = parse_inlines(document)?;
    // log::debug!("Document AST built with {} nodes", document.children.len());
    
    Ok(document)
}
