//! Emphasis parser - convert grammar emphasis to AST nodes
//!
//! Parses emphasis (*text* or _text_) and converts them to Emphasis nodes.
//! Emphasis nodes contain children that are recursively parsed inline elements.

use super::shared::GrammarSpan;
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse emphasis and convert to AST node
///
/// Tries to parse emphasis from the input. If successful, returns a Node with
/// NodeKind::Emphasis containing recursively parsed inline children.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed emphasis node
/// * `Err(_)` - Not emphasis at this position
pub fn parse_emphasis(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, content) = grammar::emphasis(input)?;
    
    // Create span for the full emphasis (including delimiters)
    use crate::parser::{Position, Span as ParserSpan};
    let span = ParserSpan::new(
        Position::new(start.location_line() as usize, start.get_column(), start.location_offset()),
        Position::new(rest.location_line() as usize, rest.get_column(), rest.location_offset()),
    );
    
    // Recursively parse inline elements within emphasis text
    // Note: We need to use the public parse_inlines function from inline_parser
    let children = match crate::parser::inlines::parse_inlines(content.fragment()) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse emphasis children: {}", e);
            vec![]
        }
    };
    
    let node = Node {
        kind: NodeKind::Emphasis,
        span: Some(span),
        children,
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_emphasis_asterisk() {
        let input = GrammarSpan::new("*hello*");
        let result = parse_emphasis(input);
        
        assert!(result.is_ok(), "Failed to parse emphasis");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::Emphasis));
        assert_eq!(node.children.len(), 1); // Should have "hello" text child
    }

    #[test]
    fn smoke_test_parse_emphasis_underscore() {
        let input = GrammarSpan::new("_hello_");
        let result = parse_emphasis(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        assert!(matches!(node.kind, NodeKind::Emphasis));
        assert!(!node.children.is_empty());
    }

    #[test]
    fn smoke_test_parse_emphasis_with_nested_code() {
        let input = GrammarSpan::new("*text with `code`*");
        let result = parse_emphasis(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        assert!(matches!(node.kind, NodeKind::Emphasis));
        // Should have multiple children: text + code span + text
        assert!(node.children.len() >= 2);
    }

    #[test]
    fn smoke_test_parse_emphasis_not_emphasis() {
        let input = GrammarSpan::new("just text");
        let result = parse_emphasis(input);
        
        assert!(result.is_err(), "Should not parse non-emphasis as emphasis");
    }

    #[test]
    fn smoke_test_parse_emphasis_unclosed() {
        let input = GrammarSpan::new("*unclosed");
        let result = parse_emphasis(input);
        
        assert!(result.is_err(), "Should not parse unclosed emphasis");
    }

    #[test]
    fn smoke_test_parse_emphasis_empty() {
        let input = GrammarSpan::new("**");
        let result = parse_emphasis(input);
        
        // This might be parsed as strong, not emphasis
        // Or might fail - either is acceptable
        let _ = result;
    }

    #[test]
    fn smoke_test_parse_emphasis_position() {
        let input = GrammarSpan::new("*hello* world");
        let result = parse_emphasis(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &" world");
        assert!(node.span.is_some(), "Emphasis should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert!(span.end.offset > span.start.offset);
    }
}
