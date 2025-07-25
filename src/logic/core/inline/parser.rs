#[cfg(test)]
mod tests {
    #[test]
    fn test_entity_html_unicode_and_malformed() {
        let ast = parse_phrases("foo &amp; bar &#x1F600; &notanentity; <b>html</b>");
        println!("AST: {:?}", ast);
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Entity { text, .. } if text == "&amp;")), "Should parse &amp; as Entity");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Entity { text, .. } if text == "&#x1F600;")), "Should parse Unicode entity");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("&notanentity;"))), "Malformed entity should be text");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Html { text, .. } if text == "<b>html</b>")), "Should parse HTML as Html node");
    }

    #[test]
    fn test_code_spans_nested_unclosed_mixed() {
        // Simple code
        let ast = parse_phrases("`code`");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text == "code")), "Should parse code span");
        // Nested backticks
        let ast2 = parse_phrases("``code `inner` code``");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text.contains("code `inner` code"))), "Should parse nested backtick code span");
        // Unclosed code
        let ast3 = parse_phrases("`unclosed");
        assert!(ast3.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("`unclosed"))), "Unclosed code should be text");
        // Mixed code and text
        let ast4 = parse_phrases("foo `bar` baz");
        assert!(ast4.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text == "bar")), "Should parse code span in mixed content");
    }

    #[test]
    fn test_math_inline_block_malformed() {
        // Inline math
        let ast = parse_phrases("$x+1$");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Math { text, .. } if text == "x+1")), "Should parse inline math");
        // Block math
        let ast2 = parse_phrases("$$E=mc^2$$");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::Math { text, .. } if text == "E=mc^2")), "Should parse block math");
        // Unclosed math
        let ast3 = parse_phrases("$unclosed");
        assert!(ast3.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("$unclosed"))), "Unclosed math should be text");
    }

    #[test]
    fn test_attribute_blocks_attached_lone_malformed() {
        // Attached attribute block
        let ast = parse_phrases("foo{.bar}");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == ".bar")), "Should parse attached attribute block");
        // Lone attribute block
        let ast2 = parse_phrases("{.baz}");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == ".baz")), "Should parse lone attribute block");
        // Malformed attribute block
        let ast3 = parse_phrases("foo {.bad");
        assert!(ast3.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("{.bad"))), "Malformed attribute block should be text");
    }

    #[test]
    fn test_emphasis_strong_code_nesting_edge_cases() {
        // Nested emphasis/strong
        let ast = parse_phrases("**_nested_**");
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
        // Overlapping delimiters
        let ast2 = parse_phrases("**bold *italic** text*");
        assert!(!ast2.is_empty(), "AST should not be empty for overlapping delimiters");
        // Emphasis inside code (should not parse as emphasis)
        let ast3 = parse_phrases("`*not emph*`");
        assert!(ast3.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text == "*not emph*")), "Emphasis inside code should be code");
    }
    use super::*;
    use crate::logic::core::inline::types::{InlineNode, SourcePos};

    /// Helper for constructing SourcePos in test assertions and helpers.
    #[allow(dead_code)]
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
        let ast = parse_phrases("[link](url) ![img](src)");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Link { .. })), "Should contain Link node");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Image { .. })), "Should contain Image node");
    }

    #[test]
    fn test_attribute_block_handling() {
        let ast = parse_phrases("foo{.bar}");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == ".bar")), "Should emit AttributeBlock node for {{.bar}}");
        let ast2 = parse_phrases("{.baz}");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == ".baz")), "Should emit AttributeBlock node for {{.baz}}");
    }

/// parser.rs - Core inline parser: token stream → raw AST
/// Parses a token stream into raw, unprocessed inline nodes.
/// Leaves unresolved things like emphasis and links as placeholder nodes or temporary markers.
/// Constructs a flat or shallow node tree.

use crate::logic::core::inline::types::{InlineNode, Token, SourcePos};
use super::tokenizer::tokenize_inline;
use crate::logic::core::inline::postprocess::normalize_inlines;
use crate::logic::core::options::ParserOptions;
use crate::logic::core::extensions::gfm::GithubExtension;
use crate::logic::core::extensions::math::MathExtension;
use crate::logic::core::extensions::MarkdownExtension;

/// Parse a string into a sequence of nested InlineNode AST nodes.

/// Parse phrases with configurable extensions
pub fn parse_phrases_with_options(input: &str, options: &ParserOptions) -> Vec<InlineNode> {
    let tokens = tokenize_inline(input);
    let line = 1;
    let column = 1;

    // Improved stack for brackets and links/images, with title parsing
    let mut bracket_stack: Vec<(bool, usize)> = Vec::new(); // (is_image, start_idx)
    let mut temp_nodes: Vec<InlineNode> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Bang => { // Handles image marker '!'
                if i + 1 < tokens.len() && matches!(&tokens[i+1], Token::OpenBracket) {
                    bracket_stack.push((true, temp_nodes.len()));
                    i += 1;
                } else {
                    temp_nodes.push(InlineNode::Text { text: "!".to_string(), pos: SourcePos { line, column } });
                }
            }
            Token::OpenBracket => { // Handles link/image open bracket '['
                bracket_stack.push((false, temp_nodes.len()));
            }
            Token::CloseBracket => { // Handles link/image close bracket ']'
                if let Some((is_image, start_idx)) = bracket_stack.pop() {
                    let label_nodes = temp_nodes.drain(start_idx..).collect::<Vec<_>>();
                    // Look ahead for (url "title")
                    if i + 2 < tokens.len() && matches!(&tokens[i+1], Token::OpenParen) {
                        let mut url = String::new();
                        let mut title = String::new();
                        let mut j = i + 2;
                        if j < tokens.len() {
                            if let Token::Text(ref s) = tokens[j] {
                                url = s.clone();
                                j += 1;
                            }
                        }
                        // Optional title in quotes
                        if j < tokens.len() {
                            if let Token::Text(ref s) = tokens[j] {
                                if s.starts_with('"') && s.ends_with('"') && s.len() > 1 {
                                    title = s[1..s.len()-1].to_string();
                                }
                            }
                        }
                        if is_image {
                            temp_nodes.push(InlineNode::Image { src: url, alt: label_nodes, title, pos: SourcePos { line, column } });
                        } else {
                            temp_nodes.push(InlineNode::Link { href: url, title, children: label_nodes, pos: SourcePos { line, column } });
                        }
                        i = j;
                    } else {
                        // Not a valid link/image, treat as text
                        temp_nodes.extend(label_nodes);
                        temp_nodes.push(InlineNode::Text { text: "]".to_string(), pos: SourcePos { line, column } });
                    }
                } else {
                    temp_nodes.push(InlineNode::Text { text: "]".to_string(), pos: SourcePos { line, column } });
                }
            }
            Token::HardBreak => { // Handles hard line break
                temp_nodes.push(InlineNode::LineBreak { pos: SourcePos { line, column } });
            }
            Token::Backtick(count) => { // Handles inline code span
                // Parse code span: collect text until matching backtick run
                let mut code = String::new();
                let mut found = false;
                let mut j = i + 1;
                while j < tokens.len() {
                    match &tokens[j] {
                        Token::Backtick(c2) if c2 == count => {
                            found = true;
                            break;
                        }
                        Token::Text(s) => code.push_str(s),
                        _ => {}
                    }
                    j += 1;
                }
                if found {
                    temp_nodes.push(InlineNode::Code { text: code, pos: SourcePos { line, column } });
                    i = j; // skip to after closing backtick
                } else {
                    // Unclosed, treat as text
                    temp_nodes.push(InlineNode::Text { text: "`".repeat(*count), pos: SourcePos { line, column } });
                }
            }
            Token::Dollar(count) => { // Handles math span (inline/block)
                // Parse math span: collect text until matching dollar run
                let mut math = String::new();
                let mut found = false;
                let mut j = i + 1;
                while j < tokens.len() {
                    match &tokens[j] {
                        Token::Dollar(c2) if c2 == count => {
                            found = true;
                            break;
                        }
                        Token::Text(s) => math.push_str(s),
                        _ => {}
                    }
                    j += 1;
                }
                if found {
                    temp_nodes.push(InlineNode::Math { text: math, pos: SourcePos { line, column } });
                    i = j; // skip to after closing dollar
                } else {
                    // Unclosed, treat as text
                    temp_nodes.push(InlineNode::Text { text: "$".repeat(*count), pos: SourcePos { line, column } });
                }
            }
            Token::AttributeBlock(s) => { // Handles attribute block
                // Remove outer braces if present
                let text = s.trim_matches(|c| c == '{' || c == '}').to_string();
                temp_nodes.push(InlineNode::AttributeBlock { text, pos: SourcePos { line, column } });
            }
            Token::Entity(s) => { // Handles HTML entity
                temp_nodes.push(InlineNode::Entity { text: s.clone(), pos: SourcePos { line, column } });
            }
            Token::Html(s) => { // Handles inline HTML
                temp_nodes.push(InlineNode::Html { text: s.clone(), pos: SourcePos { line, column } });
            }
            Token::Text(s) => { // Handles plain text
                temp_nodes.push(InlineNode::Text { text: s.clone(), pos: SourcePos { line, column } });
            }
            Token::SoftBreak => { // Handles soft line break
                temp_nodes.push(InlineNode::SoftBreak { pos: SourcePos { line, column } });
            }
            _ => {} // Unknown or unsupported token
        }
        i += 1;
    }

    // Normalize to produce nested AST (emphasis, strong, links, etc.)
    let mut ast = normalize_inlines(temp_nodes);

    // Extension activation pattern
    let extensions: Vec<Box<dyn MarkdownExtension>> = vec![
        Box::new(GithubExtension),
        Box::new(MathExtension),
        // Add more extensions here
    ];
    for ext in extensions {
        ext.apply(&mut ast, options);
    }
    ast
}

/// Backwards compatibility: default to CommonMark only
pub fn parse_phrases(input: &str) -> Vec<InlineNode> {
    parse_phrases_with_options(input, &ParserOptions::default())
}
