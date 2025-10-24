//! Code span parser - convert grammar code spans to AST nodes
//!
//! Parses inline code spans (`` `code` ``) and converts them to CodeSpan nodes.
//! This is the highest priority inline element to avoid conflicts with other syntax.

use super::shared::{GrammarSpan, to_parser_span};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse a code span and convert to AST node
///
/// Tries to parse an inline code span from the input. If successful, returns
/// a Node with NodeKind::CodeSpan containing the code content.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed code span node
/// * `Err(_)` - Not a code span at this position
pub fn parse_code_span(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let (rest, content) = grammar::code_span(input)?;
    
    let span = to_parser_span(content);
    let code = content.fragment().to_string();
    
    let node = Node {
        kind: NodeKind::CodeSpan(code),
        span: Some(span),
        children: Vec::new(),
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_code_span() {
        let input = GrammarSpan::new("`hello`");
        let result = parse_code_span(input);
        
        assert!(result.is_ok(), "Failed to parse code span");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::CodeSpan(_)));
        
        if let NodeKind::CodeSpan(code) = node.kind {
            assert_eq!(code, "hello");
        }
    }

    #[test]
    fn smoke_test_parse_code_span_with_double_backticks() {
        let input = GrammarSpan::new("``code with `backtick` ``");
        let result = parse_code_span(input);
        
        assert!(result.is_ok(), "Failed to parse double-backtick code span");
        let (_, node) = result.unwrap();
        
        if let NodeKind::CodeSpan(code) = node.kind {
            assert!(code.contains("`backtick`"));
        }
    }

    #[test]
    fn smoke_test_parse_code_span_not_code() {
        let input = GrammarSpan::new("just text");
        let result = parse_code_span(input);
        
        assert!(result.is_err(), "Should not parse non-code as code span");
    }

    #[test]
    fn smoke_test_parse_code_span_unclosed() {
        let input = GrammarSpan::new("`unclosed");
        let result = parse_code_span(input);
        
        assert!(result.is_err(), "Should not parse unclosed code span");
    }

    #[test]
    fn smoke_test_parse_code_span_empty() {
        let input = GrammarSpan::new("` `");
        let result = parse_code_span(input);
        
        assert!(result.is_ok(), "Failed to parse empty code span");
        let (_, node) = result.unwrap();
        
        if let NodeKind::CodeSpan(code) = node.kind {
            assert!(code.is_empty() || code == " ");
        }
    }

    #[test]
    fn smoke_test_parse_code_span_position() {
        let input = GrammarSpan::new("`code` and text");
        let result = parse_code_span(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &" and text");
        assert!(node.span.is_some(), "Code span should have position info");
        
        let span = node.span.unwrap();
        // Code span content starts after opening backtick at position 1
        assert_eq!(span.start.offset, 1);
        assert!(span.end.offset > span.start.offset);
    }
}
