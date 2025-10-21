//! Backslash escape parser - convert grammar escapes to AST nodes
//!
//! Parses backslash escape sequences (\*, \\, etc.) and converts them to Text nodes
//! containing just the escaped character (without the backslash).

use super::shared::GrammarSpan;
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use crate::parser::Position;
use crate::parser::Span as ParserSpan;
use nom::IResult;

/// Parse a backslash escape and convert to AST node
///
/// Tries to parse a backslash escape sequence from the input. If successful,
/// returns a Node with NodeKind::Text containing just the escaped character.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed text node with escaped character
/// * `Err(_)` - Not a backslash escape at this position
pub fn parse_backslash_escape(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let (rest, escaped_char) = grammar::backslash_escape(input)?;
    
    let start = input.location_offset();
    let end = rest.location_offset();
    
    let span = ParserSpan::new(
        Position::new(input.location_line() as usize, input.get_column(), start),
        Position::new(rest.location_line() as usize, rest.get_column(), end),
    );
    
    // Create a text node with just the escaped character (without the backslash)
    let node = Node {
        kind: NodeKind::Text(escaped_char.to_string()),
        span: Some(span),
        children: Vec::new(),
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_backslash_escape_asterisk() {
        let input = GrammarSpan::new(r"\*");
        let result = parse_backslash_escape(input);
        
        assert!(result.is_ok(), "Failed to parse backslash escape");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::Text(_)));
        
        if let NodeKind::Text(text) = node.kind {
            assert_eq!(text, "*");
        }
    }

    #[test]
    fn smoke_test_parse_backslash_escape_backslash() {
        let input = GrammarSpan::new(r"\\");
        let result = parse_backslash_escape(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        if let NodeKind::Text(text) = node.kind {
            assert_eq!(text, "\\");
        }
    }

    #[test]
    fn smoke_test_parse_backslash_escape_bracket() {
        let input = GrammarSpan::new(r"\[");
        let result = parse_backslash_escape(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        if let NodeKind::Text(text) = node.kind {
            assert_eq!(text, "[");
        }
    }

    #[test]
    fn smoke_test_parse_backslash_escape_not_escape() {
        let input = GrammarSpan::new("just text");
        let result = parse_backslash_escape(input);
        
        assert!(result.is_err(), "Should not parse non-escape as backslash escape");
    }

    #[test]
    fn smoke_test_parse_backslash_escape_not_punctuation() {
        let input = GrammarSpan::new(r"\a");
        let result = parse_backslash_escape(input);
        
        // Should fail - 'a' is not ASCII punctuation
        assert!(result.is_err(), "Should not parse backslash before non-punctuation");
    }

    #[test]
    fn smoke_test_parse_backslash_escape_position() {
        let input = GrammarSpan::new(r"\* and text");
        let result = parse_backslash_escape(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &" and text");
        assert!(node.span.is_some(), "Escape should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert_eq!(span.end.offset, 2); // \* is 2 bytes
    }
}
