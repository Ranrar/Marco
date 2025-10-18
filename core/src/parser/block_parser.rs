// Block-level parser: first stage of two-stage parsing

use super::ast::{Node, NodeKind, Document};
use crate::grammar::block as grammar;
use anyhow::Result;
use nom_locate::LocatedSpan;

type GrammarSpan<'a> = LocatedSpan<&'a str>;

// Convert grammar LocatedSpan to parser Span
fn to_parser_span(input: GrammarSpan) -> crate::parser::position::Span {
    let start = crate::parser::position::Position::new(
        input.location_line() as usize,
        input.get_column(),
        input.location_offset(),
    );
    
    let end = crate::parser::position::Position::new(
        input.location_line() as usize,
        input.get_column() + input.fragment().len(),
        input.location_offset() + input.fragment().len(),
    );
    
    crate::parser::position::Span::new(start, end)
}

// Parse document into block-level structure, returning a Document
pub fn parse_blocks(input: &str) -> Result<Document> {
    log::debug!("Block parser input: {} bytes", input.len());
    
    let mut nodes = Vec::new();
    let mut remaining = GrammarSpan::new(input);
    
    while !remaining.fragment().is_empty() {
        // Skip blank lines
        if remaining.fragment().starts_with('\n') || remaining.fragment().trim().is_empty() {
            let skip_len = remaining.fragment().chars()
                .take_while(|c| c.is_whitespace())
                .map(|c| c.len_utf8())
                .sum();
            
            if skip_len > 0 {
                let new_fragment = &remaining.fragment()[skip_len..];
                remaining = GrammarSpan::new(new_fragment);
                continue;
            } else {
                break;
            }
        }
        
        // Try parsing heading first
        if let Ok((rest, (level, content))) = grammar::heading(remaining) {
            let span = to_parser_span(content);
            let text = content.fragment().to_string();
            
            let node = Node {
                kind: NodeKind::Heading { level, text },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing fenced code block
        if let Ok((rest, (language, content))) = grammar::fenced_code_block(remaining) {
            let span = to_parser_span(content);
            let code = content.fragment().to_string();
            
            let node = Node {
                kind: NodeKind::CodeBlock { language, code },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing paragraph
        if let Ok((rest, content)) = grammar::paragraph(remaining) {
            let span = to_parser_span(content);
            
            // Parse inline elements within paragraph text
            let inline_children = match crate::parser::inline_parser::parse_inlines(content.fragment()) {
                Ok(children) => children,
                Err(e) => {
                    log::warn!("Failed to parse inline elements: {}", e);
                    // Fallback to plain text
                    vec![Node {
                        kind: NodeKind::Text(content.fragment().to_string()),
                        span: Some(span),
                        children: Vec::new(),
                    }]
                }
            };
            
            let node = Node {
                kind: NodeKind::Paragraph,
                span: Some(span),
                children: inline_children,
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // If nothing matched, skip one character to avoid infinite loop
        log::warn!("Could not parse block at offset {}, skipping character", remaining.location_offset());
        let skip = remaining.fragment().chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        let new_fragment = &remaining.fragment()[skip..];
        remaining = GrammarSpan::new(new_fragment);
    }
    
    log::info!("Parsed {} blocks", nodes.len());
    
    let mut document = Document::new();
    document.children = nodes;
    Ok(document)
}
