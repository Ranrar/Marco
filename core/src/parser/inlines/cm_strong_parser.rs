//! Strong emphasis parser - convert grammar strong to AST nodes
//!
//! Parses strong emphasis (**text** or __text__) and converts them to Strong nodes.
//! Strong nodes contain children that are recursively parsed inline elements.

use super::shared::GrammarSpan;
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse strong emphasis and convert to AST node
///
/// Tries to parse strong emphasis from the input. If successful, returns a Node
/// with NodeKind::Strong containing recursively parsed inline children.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed strong node
/// * `Err(_)` - Not strong emphasis at this position
pub fn parse_strong(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, content) = grammar::strong(input)?;
    
    // Create span for the full strong (including delimiters)
    use crate::parser::{Position, Span as ParserSpan};
    let span = ParserSpan::new(
        Position::new(start.location_line() as usize, start.get_column(), start.location_offset()),
        Position::new(rest.location_line() as usize, rest.get_column(), rest.location_offset()),
    );
    
    // Recursively parse inline elements within strong text
    let children = match crate::parser::inlines::parse_inlines(content.fragment()) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse strong children: {}", e);
            vec![]
        }
    };
    
    let node = Node {
        kind: NodeKind::Strong,
        span: Some(span),
        children,
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_strong_asterisk() {
        let input = GrammarSpan::new("**hello**");
        let result = parse_strong(input);
        
        assert!(result.is_ok(), "Failed to parse strong");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::Strong));
        assert_eq!(node.children.len(), 1); // Should have "hello" text child
    }

    #[test]
    fn smoke_test_parse_strong_underscore() {
        let input = GrammarSpan::new("__hello__");
        let result = parse_strong(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        assert!(matches!(node.kind, NodeKind::Strong));
        assert!(!node.children.is_empty());
    }

    #[test]
    fn smoke_test_parse_strong_with_nested_code() {
        let input = GrammarSpan::new("**text with `code`**");
        let result = parse_strong(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        assert!(matches!(node.kind, NodeKind::Strong));
        // Should have multiple children: text + code span + text
        assert!(node.children.len() >= 2);
    }

    #[test]
    fn smoke_test_parse_strong_not_strong() {
        let input = GrammarSpan::new("just text");
        let result = parse_strong(input);
        
        assert!(result.is_err(), "Should not parse non-strong as strong");
    }

    #[test]
    fn smoke_test_parse_strong_unclosed() {
        let input = GrammarSpan::new("**unclosed");
        let result = parse_strong(input);
        
        assert!(result.is_err(), "Should not parse unclosed strong");
    }

    #[test]
    fn smoke_test_parse_strong_empty() {
        let input = GrammarSpan::new("****");
        let result = parse_strong(input);
        
        // Should parse as empty strong or fail - either acceptable
        let _ = result;
    }

    #[test]
    fn smoke_test_parse_strong_position() {
        let input = GrammarSpan::new("**hello** world");
        let result = parse_strong(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &" world");
        assert!(node.span.is_some(), "Strong should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert!(span.end.offset > span.start.offset);
    }
}
