// Inline-level parser: parse emphasis, strong, links, code spans within text

use super::ast::{Node, NodeKind};
use crate::grammar::inline;
use crate::parser::{Position, Span as ParserSpan};
use nom_locate::LocatedSpan;
use nom::bytes::complete::take;
use nom::IResult;
use anyhow::Result;

type GrammarSpan<'a> = LocatedSpan<&'a str>;

// Parse inline elements within text content
// Returns a vector of inline nodes (Text, Emphasis, Strong, Link, CodeSpan)
pub fn parse_inlines(text: &str) -> Result<Vec<Node>> {
    log::debug!("Parsing inline elements in text: {:?}", text);
    
    let mut nodes = Vec::new();
    let mut remaining = GrammarSpan::new(text);
    
    while !remaining.fragment().is_empty() {
        let start_pos = remaining.location_offset();
        
        // Try parsing code span first (highest priority to avoid conflicts)
        if let Ok((rest, content)) = inline::code_span(remaining) {
            let span = to_parser_span(content);
            let code = content.fragment().to_string();
            
            let node = Node {
                kind: NodeKind::CodeSpan(code),
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing strong (must come before emphasis to match ** before *)
        if let Ok((rest, content)) = inline::strong(remaining) {
            let span = to_parser_span(content);
            
            // Recursively parse inline elements within strong text
            let children = parse_inlines(content.fragment())?;
            
            let node = Node {
                kind: NodeKind::Strong,
                span: Some(span),
                children,
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing emphasis
        if let Ok((rest, content)) = inline::emphasis(remaining) {
            let span = to_parser_span(content);
            
            // Recursively parse inline elements within emphasis text
            let children = parse_inlines(content.fragment())?;
            
            let node = Node {
                kind: NodeKind::Emphasis,
                span: Some(span),
                children,
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing link
        if let Ok((rest, (link_text, url, title))) = inline::link(remaining) {
            let span = to_parser_span(link_text);
            
            // Recursively parse inline elements within link text
            let children = parse_inlines(link_text.fragment())?;
            
            let node = Node {
                kind: NodeKind::Link {
                    url: url.fragment().to_string(),
                    title: title.map(|s| s.fragment().to_string()),
                },
                span: Some(span),
                children,
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // No inline element matched - accumulate plain text
        // Find the next potential inline element start
        let text_fragment = remaining.fragment();
        let next_special = text_fragment
            .find(['*', '_', '`', '['])
            .unwrap_or(text_fragment.len());
        
        if next_special > 0 {
            // Use nom's take to properly advance the span
            if let Ok((rest, text_content)) = take::<_, _, nom::error::Error<_>>(next_special)(remaining) {
                let span = to_parser_span(text_content);
                
                let node = Node {
                    kind: NodeKind::Text(text_content.fragment().to_string()),
                    span: Some(span),
                    children: Vec::new(),
                };
                
                nodes.push(node);
                remaining = rest;
            } else {
                // Shouldn't happen, but break to avoid infinite loop
                break;
            }
        } else {
            // Special character that didn't parse - include it as text
            let char_len = text_fragment.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            
            if let Ok((rest, text_content)) = take::<_, _, nom::error::Error<_>>(char_len)(remaining) {
                let span = to_parser_span(text_content);
                
                let node = Node {
                    kind: NodeKind::Text(text_content.fragment().to_string()),
                    span: Some(span),
                    children: Vec::new(),
                };
                
                nodes.push(node);
                remaining = rest;
            } else {
                // Shouldn't happen, but break to avoid infinite loop
                break;
            }
        }
        
        // Safety check to prevent infinite loops
        if remaining.location_offset() == start_pos {
            log::warn!("Inline parser stuck at offset {}, breaking", start_pos);
            break;
        }
    }
    
    log::debug!("Parsed {} inline nodes", nodes.len());
    Ok(nodes)
}

// Convert grammar Span to parser Span
fn to_parser_span(span: GrammarSpan) -> ParserSpan {
    ParserSpan {
        start: Position {
            line: span.location_line() as usize,
            column: span.get_utf8_column(),
            offset: span.location_offset(),
        },
        end: Position {
            line: span.location_line() as usize,
            column: span.get_utf8_column() + span.fragment().len(),
            offset: span.location_offset() + span.fragment().len(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_parse_emphasis() {
        let input = "This is *italic* text.";
        let result = parse_inlines(input);
        
        assert!(result.is_ok(), "Failed to parse inline elements");
        let nodes = result.unwrap();
        
        // Should have: Text("This is "), Emphasis, Text(" text.")
        println!("Parsed {} nodes:", nodes.len());
        for (i, node) in nodes.iter().enumerate() {
            println!("  Node {}: {:?}", i, node.kind);
        }
        
        assert!(nodes.len() >= 3, "Expected at least 3 nodes, got {}", nodes.len());
        
        // Check that we have an Emphasis node
        let has_emphasis = nodes.iter().any(|n| matches!(n.kind, NodeKind::Emphasis));
        assert!(has_emphasis, "Expected to find Emphasis node");
    }
}
