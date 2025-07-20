#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::core::inline::types::{InlineNode, SourcePos};

    fn pos(line: usize, col: usize) -> SourcePos {
        SourcePos { line, column: col }
    }

    #[test]
    fn test_simple_text() {
        let ast = parse_phrases("Hello World");
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            InlineNode::Text { text, .. } => assert_eq!(text, "Hello World"),
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_emphasis_and_strong() {
        let ast = parse_phrases("*foo* **bar**");
        // Should contain Emphasis and Strong nodes after normalization
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should contain Emphasis node");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Strong { .. })), "Should contain Strong node");
    }

    #[test]
    fn test_code_and_math() {
        let ast = parse_phrases("`code` $math$");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text == "code")), "Should contain Code node with correct content");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Math { text, .. } if text == "math")), "Should contain Math node with correct content");
    }

    #[test]
    fn test_nested_emphasis_strong() {
        let ast = parse_phrases("**_nested_**");
        // Should contain Strong node with Emphasis child
        let mut found_strong = false;
        let mut found_emph = false;
        for n in &ast {
            if let InlineNode::Strong { children, .. } = n {
                found_strong = true;
                for c in children {
                    if let InlineNode::Emphasis { .. } = c {
                        found_emph = true;
                    }
                }
            }
        }
        assert!(found_strong, "Should contain Strong node");
        assert!(found_emph, "Should contain nested Emphasis node");
    }
    }

    #[test]
    fn test_html_and_breaks() {
        let ast = parse_phrases("<b>text</b>\nline");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Html { .. })), "Should contain Html node");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::SoftBreak { .. })), "Should contain SoftBreak node");
    }

    #[test]
    fn test_link_and_image_stub() {
        let ast = parse_phrases("[link] ![img]");
        // For now, links/images are text nodes, but should be present
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("["))), "Should contain link text node");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("!"))), "Should contain image text node");
    }

    #[test]
    fn test_attribute_block_handling() {
        let ast = parse_phrases("foo{.bar}");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("foo{.bar}"))), "Should attach attribute block to next inline node");
        let ast2 = parse_phrases("{.baz}");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("{.baz}"))), "Should treat attribute block as text if no node to attach");
    }

/// parser.rs - Core inline parser: token stream → raw AST
/// Parses a token stream into raw, unprocessed inline nodes.
/// Leaves unresolved things like emphasis and links as placeholder nodes or temporary markers.
/// Constructs a flat or shallow node tree.

use crate::logic::core::inline::types::{InlineNode, Token, SourcePos};
use super::tokenizer::tokenize_inline;
use super::postprocess::normalize_inlines;

/// Parse a string into a sequence of nested InlineNode AST nodes.
pub fn parse_phrases(input: &str) -> Vec<InlineNode> {
    let tokens = tokenize_inline(input);
    let line = 1;
    let column = 1;

    // Simple stack for brackets and links/images
    let mut bracket_stack: Vec<(bool, usize)> = Vec::new(); // (is_image, start_idx)
    let mut temp_nodes: Vec<InlineNode> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Bang => {
                // Start of image: push to stack, expect OpenBracket next
                if i + 1 < tokens.len() && matches!(&tokens[i+1], Token::OpenBracket) {
                    bracket_stack.push((true, temp_nodes.len()));
                    i += 1; // Skip OpenBracket
                } else {
                    temp_nodes.push(InlineNode::Text { text: "!".to_string(), pos: SourcePos { line, column } });
                }
            }
            Token::OpenBracket => {
                bracket_stack.push((false, temp_nodes.len()));
            }
            Token::CloseBracket => {
                if let Some((is_image, start_idx)) = bracket_stack.pop() {
                    // Collect label nodes
                    let label_nodes = temp_nodes.drain(start_idx..).collect::<Vec<_>>();
                    // Look ahead for (url)
                    if i + 2 < tokens.len() && matches!(&tokens[i+1], Token::OpenParen) && matches!(&tokens[i+2], Token::Text(_)) {
                        let url = if let Token::Text(ref s) = tokens[i+2] { s.clone() } else { String::new() };
                        let title = None; // TODO: parse title if present
                        let pos = SourcePos { line, column };
                        if is_image {
                            temp_nodes.push(InlineNode::Image {
                                src: url,
                                alt: label_nodes.iter().filter_map(|n| if let InlineNode::Text { text, .. } = n { Some(text.clone()) } else { None }).collect::<Vec<_>>().join(" "),
                                title: title.unwrap_or_default(),
                                pos,
                            });
                        } else {
                            temp_nodes.push(InlineNode::Link {
                                href: url,
                                title: title.unwrap_or_default(),
                                children: label_nodes,
                                pos,
                            });
                        }
                        i += 2; // Skip OpenParen and url
                    } else {
                        // Not a valid link/image, treat as text
                        temp_nodes.extend(label_nodes);
                        temp_nodes.push(InlineNode::Text { text: "]".to_string(), pos: SourcePos { line, column } });
                    }
                } else {
                    temp_nodes.push(InlineNode::Text { text: "]".to_string(), pos: SourcePos { line, column } });
                }
            }
            Token::Text(s) => {
                temp_nodes.push(InlineNode::Text { text: s.clone(), pos: SourcePos { line, column } });
            }
            Token::Star(count) | Token::Underscore(count) => {
                let ch = if let Token::Star(_) = &tokens[i] { '*' } else { '_' };
                temp_nodes.push(InlineNode::Text { text: ch.to_string().repeat(*count), pos: SourcePos { line, column } });
            }
            Token::Backtick(count) => {
                // Extract code content between matching backtick runs
                let mut code_content = String::new();
                let mut j = i + 1;
                //
                while j < tokens.len() {
                    match &tokens[j] {
                        Token::Backtick(c) if *c == *count => {
                            i = j; // Move to closing backtick
                            //
                            break;
                        }
                        Token::Text(s) => code_content.push_str(s),
                        _ => {}
                    }
                    j += 1;
                }
                // CommonMark: trim one leading/trailing space if both exist
                let trimmed = if code_content.starts_with(' ') && code_content.ends_with(' ') && code_content.len() > 1 {
                    code_content[1..code_content.len()-1].to_string()
                } else {
                    code_content.clone()
                };
                temp_nodes.push(InlineNode::Code { text: trimmed, pos: SourcePos { line, column } });
            }
            Token::Dollar(count) => {
                // Extract math content between matching dollar runs
                let mut math_content = String::new();
                let mut j = i + 1;
                let is_block = *count > 1;
                while j < tokens.len() {
                    match &tokens[j] {
                        Token::Dollar(c) if *c == *count => {
                            i = j; // Move to closing dollar
                            break;
                        }
                        Token::Text(s) => math_content.push_str(s),
                        _ => {}
                    }
                    j += 1;
                }
                if is_block {
                    // For math blocks ($$...$$), wrap in block node if supported
                    temp_nodes.push(InlineNode::Math { text: math_content, pos: SourcePos { line, column } });
                    // TODO: If block-level AST is available, use MathBlock node
                } else {
                    temp_nodes.push(InlineNode::Math { text: math_content, pos: SourcePos { line, column } });
                }
            }
            Token::OpenParen | Token::CloseParen => {
                let s = match &tokens[i] {
                    Token::OpenParen => "(".to_string(),
                    Token::CloseParen => ")".to_string(),
                    _ => String::new(),
                };
                temp_nodes.push(InlineNode::Text { text: s, pos: SourcePos { line, column } });
            }
            Token::Backslash(ch) => {
                temp_nodes.push(InlineNode::Text { text: ch.to_string(), pos: SourcePos { line, column } });
            }
            Token::Ampersand => {
                // Look ahead for entity name (e.g., &amp;, &#x27;, &#39;)
                let mut entity = String::from("&");
                let mut j = i + 1;
                let mut found_semicolon = false;
                while j < tokens.len() {
                    match &tokens[j] {
                        Token::Text(s) => {
                            entity.push_str(s);
                            if s.contains(';') {
                                found_semicolon = true;
                                i = j; // Move to semicolon
                                break;
                            }
                        }
                        _ => break,
                    }
                    j += 1;
                }
                // Only resolve if we found a semicolon
                let resolved = if found_semicolon {
                    match entity.as_str() {
                        "&amp;" => "&".to_string(),
                        "&lt;" => "<".to_string(),
                        "&gt;" => ">".to_string(),
                        "&quot;" => "\"".to_string(),
                        "&apos;" => "'".to_string(),
                        "&copy;" => "©".to_string(),
                        _ => {
                            // Numeric entities: &#...; or &#x...;
                            if entity.starts_with("&#") && entity.ends_with(";") {
                                let num = &entity[2..entity.len()-1];
                                if num.starts_with('x') || num.starts_with('X') {
                                    // Hex
                                    if let Ok(v) = u32::from_str_radix(&num[1..], 16) {
                                        if let Some(c) = std::char::from_u32(v) {
                                            c.to_string()
                                        } else {
                                            entity.clone()
                                        }
                                    } else {
                                        entity.clone()
                                    }
                                } else {
                                    // Decimal
                                    if let Ok(v) = num.parse::<u32>() {
                                        if let Some(c) = std::char::from_u32(v) {
                                            c.to_string()
                                        } else {
                                            entity.clone()
                                        }
                                    } else {
                                        entity.clone()
                                    }
                                }
                            } else {
                                // Unknown entity, fallback to text
                                entity.clone()
                            }
                        }
                    }
                } else {
                    entity.clone()
                };
                temp_nodes.push(InlineNode::Text { text: resolved, pos: SourcePos { line, column } });
            }
            Token::Html(s) => {
                temp_nodes.push(InlineNode::Html { text: s.clone(), pos: SourcePos { line, column } });
            }
            Token::AttributeBlock(s) => {
                // Buffer attribute block to attach to next node
                let mut attached = false;
                let k_start = i + 1;
                for k in k_start..tokens.len() {
                    match &tokens[k] {
                        Token::Text(t) => {
                            temp_nodes.push(InlineNode::Text { text: t.clone() + &format!("{{{}}}", s), pos: SourcePos { line, column } });
                            attached = true;
                            i = k;
                            break;
                        }
                        Token::Star(count) | Token::Underscore(count) => {
                            let ch = if let Token::Star(_) = &tokens[k] { '*' } else { '_' };
                            temp_nodes.push(InlineNode::Text { text: ch.to_string().repeat(*count) + &format!("{{{}}}", s), pos: SourcePos { line, column } });
                            attached = true;
                            i = k;
                            break;
                        }
                        Token::Backtick(count) => {
                            // Attach to code span
                            let mut code_content = String::new();
                            let mut j = k + 1;
                            while j < tokens.len() {
                                match &tokens[j] {
                                    Token::Backtick(c) if *c == *count => {
                                        //
                                        break;
                                    }
                                    Token::Text(s) => code_content.push_str(s),
                                    _ => {}
                                }
                                j += 1;
                            }
                            let trimmed = if code_content.starts_with(' ') && code_content.ends_with(' ') && code_content.len() > 1 {
                                code_content[1..code_content.len()-1].to_string()
                            } else {
                                code_content.clone()
                            };
                            temp_nodes.push(InlineNode::Code { text: trimmed + &format!("{{{}}}", s), pos: SourcePos { line, column } });
                            attached = true;
                            i = k;
                            break;
                        }
                        Token::Dollar(count) => {
                            // Attach to math span
                            let mut math_content = String::new();
                            let mut j = k + 1;
                            while j < tokens.len() {
                                match &tokens[j] {
                                    Token::Dollar(c) if *c == *count => {
                                        //
                                        break;
                                    }
                                    Token::Text(s) => math_content.push_str(s),
                                    _ => {}
                                }
                                j += 1;
                            }
                            temp_nodes.push(InlineNode::Math { text: math_content + &format!("{{{}}}", s), pos: SourcePos { line, column } });
                            attached = true;
                            i = k;
                            break;
                        }
                        // Extend for other node types as needed (links, images, etc.)
                        _ => break,
                    }
                }
                if !attached {
                    // If no node to attach, treat as text
                    temp_nodes.push(InlineNode::Text { text: s.clone(), pos: SourcePos { line, column } });
                }
            }
            Token::SoftBreak => {
                temp_nodes.push(InlineNode::SoftBreak { pos: SourcePos { line, column } });
            }
            Token::HardBreak => {
                temp_nodes.push(InlineNode::LineBreak { pos: SourcePos { line, column } });
            }
            _ => {}
        }
        i += 1;
    }

    // Normalize to produce nested AST (emphasis, strong, links, etc.)
    let ast = normalize_inlines(temp_nodes);
    ast
}
