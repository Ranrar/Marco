//! Line breaks parser - convert grammar line breaks to AST nodes
//!
//! Parses hard line breaks (two spaces + newline or backslash + newline) and
//! soft line breaks (regular newline) and converts them to Break nodes.

use super::shared::{GrammarSpan, to_parser_span_range};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse hard line break and convert to AST node
///
/// Tries to parse a hard line break (two spaces + newline, or backslash + newline)
/// from the input. If successful, returns a Node with NodeKind::HardBreak.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed hard break node
/// * `Err(_)` - Not a hard line break at this position
pub fn parse_hard_line_break(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let (rest, _) = grammar::hard_line_break(input)?;
    
    let node = Node {
        kind: NodeKind::HardBreak,
        span: Some(to_parser_span_range(input, rest)),
        children: Vec::new(),
    };
    
    Ok((rest, node))
}

/// Parse soft line break and convert to AST node
///
/// Tries to parse a soft line break (regular newline) from the input.
/// If successful, returns a Node with NodeKind::SoftBreak.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed soft break node
/// * `Err(_)` - Not a soft line break at this position
pub fn parse_soft_line_break(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let (rest, _) = grammar::soft_line_break(input)?;
    
    let node = Node {
        kind: NodeKind::SoftBreak,
        span: Some(to_parser_span_range(input, rest)),
        children: Vec::new(),
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_hard_line_break_spaces() {
        let input = GrammarSpan::new("  \n");
        let result = parse_hard_line_break(input);
        
        assert!(result.is_ok(), "Failed to parse hard line break with spaces");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::HardBreak));
        assert!(node.children.is_empty());
    }

    #[test]
    fn smoke_test_parse_hard_line_break_backslash() {
        let input = GrammarSpan::new("\\\n");
        let result = parse_hard_line_break(input);
        
        assert!(result.is_ok(), "Failed to parse hard line break with backslash");
        let (_, node) = result.unwrap();
        
        assert!(matches!(node.kind, NodeKind::HardBreak));
    }

    #[test]
    fn smoke_test_parse_hard_line_break_not_hard() {
        let input = GrammarSpan::new("\n");
        let result = parse_hard_line_break(input);
        
        // Single newline without spaces/backslash is soft break
        assert!(result.is_err(), "Should not parse soft break as hard break");
    }

    #[test]
    fn smoke_test_parse_soft_line_break() {
        let input = GrammarSpan::new("\n");
        let result = parse_soft_line_break(input);
        
        assert!(result.is_ok(), "Failed to parse soft line break");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::SoftBreak));
        assert!(node.children.is_empty());
    }

    #[test]
    fn smoke_test_parse_soft_line_break_not_newline() {
        let input = GrammarSpan::new("text");
        let result = parse_soft_line_break(input);
        
        assert!(result.is_err(), "Should not parse non-newline as soft break");
    }

    #[test]
    fn smoke_test_parse_hard_line_break_position() {
        let input = GrammarSpan::new("  \nmore text");
        let result = parse_hard_line_break(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"more text");
        assert!(node.span.is_some(), "Hard break should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert_eq!(span.end.offset, 3); // "  \n" is 3 bytes
    }

    #[test]
    fn smoke_test_parse_soft_line_break_position() {
        let input = GrammarSpan::new("\nmore text");
        let result = parse_soft_line_break(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"more text");
        assert!(node.span.is_some(), "Soft break should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert_eq!(span.end.offset, 1); // "\n" is 1 byte
    }
}
