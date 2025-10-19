// Inline-level parser: parse emphasis, strong, links, code spans within text

use super::ast::{Node, NodeKind};
use crate::grammar::inline;
use crate::parser::{Position, Span as ParserSpan};
use nom_locate::LocatedSpan;
use nom::bytes::complete::take;
use anyhow::Result;

type GrammarSpan<'a> = LocatedSpan<&'a str>;

// Parse inline elements within text content
// Returns a vector of inline nodes (Text, Emphasis, Strong, Link, CodeSpan)
pub fn parse_inlines(text: &str) -> Result<Vec<Node>> {
    log::debug!("Parsing inline elements in text: {:?}", text);
    
    let mut nodes = Vec::new();
    let mut remaining = GrammarSpan::new(text);
    
    // Safety: prevent infinite loops
    const MAX_ITERATIONS: usize = 1000;
    let mut iteration_count = 0;
    let mut last_offset = 0;
    
    while !remaining.fragment().is_empty() {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            log::error!("Inline parser exceeded MAX_ITERATIONS ({})", MAX_ITERATIONS);
            break;
        }
        
        let start_pos = remaining.location_offset();
        
        // Safety: ensure we're making progress
        if start_pos == last_offset && iteration_count > 1 {
            log::error!("Inline parser not making progress at offset {}, forcing skip", start_pos);
            // Force skip one character
            let skip = remaining.fragment().chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            if let Ok((rest, _)) = take::<_, _, nom::error::Error<_>>(skip)(remaining) {
                remaining = rest;
                last_offset = remaining.location_offset();
                continue;
            } else {
                break;
            }
        }
        last_offset = start_pos;
        
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
        
        // Try parsing backslash escape (before other inline elements)
        if let Ok((rest, escaped_char)) = inline::backslash_escape(remaining) {
            let start = remaining.location_offset();
            let end = rest.location_offset();
            
            let span = ParserSpan::new(
                Position::new(remaining.location_line() as usize, remaining.get_column(), start),
                Position::new(rest.location_line() as usize, rest.get_column(), end),
            );
            
            // Create a text node with just the escaped character (without the backslash)
            let node = Node {
                kind: NodeKind::Text(escaped_char.to_string()),
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
        
        // Try parsing autolink (must come before link and inline HTML since syntax starts with <)
        if let Ok((rest, (uri, is_email))) = inline::autolink(remaining) {
            let span = to_parser_span(uri);
            
            let node = if is_email {
                Node {
                    kind: NodeKind::Link {
                        url: format!("mailto:{}", uri.fragment()),
                        title: None,
                    },
                    span: Some(span),
                    children: vec![Node {
                        kind: NodeKind::Text(uri.fragment().to_string()),
                        span: Some(span),
                        children: Vec::new(),
                    }],
                }
            } else {
                Node {
                    kind: NodeKind::Link {
                        url: uri.fragment().to_string(),
                        title: None,
                    },
                    span: Some(span),
                    children: vec![Node {
                        kind: NodeKind::Text(uri.fragment().to_string()),
                        span: Some(span),
                        children: Vec::new(),
                    }],
                }
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing image (must come before link since syntax is similar but starts with !)
        if let Ok((rest, (alt_text, url, _title))) = inline::image(remaining) {
            let span = to_parser_span(alt_text);
            
            let node = Node {
                kind: NodeKind::Image {
                    url: url.fragment().to_string(),
                    alt: alt_text.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
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
        
        // Try parsing inline HTML
        if let Ok((rest, content)) = inline::inline_html(remaining) {
            let span = to_parser_span(content);
            let html = content.fragment().to_string();
            
            let node = Node {
                kind: NodeKind::InlineHtml(html),
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing hard line break (two spaces + newline, or backslash + newline)
        if let Ok((rest, _)) = inline::hard_line_break(remaining) {
            log::debug!("Parsed hard line break at offset {}", remaining.location_offset());
            let start_offset = remaining.location_offset();
            let end_offset = rest.location_offset();
            
            let node = Node {
                kind: NodeKind::HardBreak,
                span: Some(ParserSpan {
                    start: Position { line: 0, column: start_offset, offset: start_offset },
                    end: Position { line: 0, column: end_offset, offset: end_offset },
                }),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing soft line break (regular newline)
        if let Ok((rest, _)) = inline::soft_line_break(remaining) {
            let start_offset = remaining.location_offset();
            let end_offset = rest.location_offset();
            
            let node = Node {
                kind: NodeKind::SoftBreak,
                span: Some(ParserSpan {
                    start: Position { line: 0, column: start_offset, offset: start_offset },
                    end: Position { line: 0, column: end_offset, offset: end_offset },
                }),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // No inline element matched - accumulate plain text
        // Find the next potential inline element start
        // Include '\n' and '\\' for line breaks, '<' for autolinks/inline HTML, '!' for images
        let text_fragment = remaining.fragment();
        let next_special = text_fragment
            .find(['*', '_', '`', '[', '<', '!', '\n', '\\'])
            .unwrap_or(text_fragment.len());
        
        if next_special > 0 {
            // Check if the upcoming character is a newline and the text ends with spaces
            // If so, don't consume trailing spaces (they might be part of a hard line break)
            let mut text_len = next_special;
            if text_fragment.chars().nth(next_special) == Some('\n') {
                // Check for trailing spaces
                let mut trailing_spaces = 0;
                for ch in text_fragment[..next_special].chars().rev() {
                    if ch == ' ' {
                        trailing_spaces += 1;
                    } else {
                        break;
                    }
                }
                
                // If we have 2+ trailing spaces, don't consume them
                // (they might be part of a hard line break pattern)
                if trailing_spaces >= 2 {
                    text_len = next_special - trailing_spaces;
                }
            }
            
            if text_len > 0 {
                // Use nom's take to properly advance the span
                if let Ok((rest, text_content)) = take::<_, _, nom::error::Error<_>>(text_len)(remaining) {
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
            // If text_len == 0, skip to the next iteration to try parsing the line break
        } else {
            // Special character that didn't parse - include it as text
            // Special case: if it's a backtick, consume all consecutive backticks
            // This prevents ```foo`` from being parsed as ` + ``foo``
            let char_len = if text_fragment.starts_with('`') {
                // Count all consecutive backticks
                text_fragment.chars().take_while(|&c| c == '`').count()
            } else {
                text_fragment.chars().next().map(|c| c.len_utf8()).unwrap_or(1)
            };
            
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
