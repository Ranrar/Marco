//! Strong+emphasis (triple delimiter) parser
//!
//! Converts grammar output for `***text***` / `___text___` into an AST node.

use super::shared::{to_parser_span_range, GrammarSpan};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse combined strong+emphasis and convert to AST node.
pub fn parse_strong_emphasis(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, content) = grammar::strong_emphasis(input)?;

    let span = to_parser_span_range(start, rest);

    let children = match crate::parser::inlines::parse_inlines_from_span(content) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse strong_emphasis children: {}", e);
            vec![]
        }
    };

    let node = Node {
        kind: NodeKind::StrongEmphasis,
        span: Some(span),
        children,
    };

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_strong_emphasis_asterisk() {
        let input = GrammarSpan::new("***hello***");
        let result = parse_strong_emphasis(input);
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        assert_eq!(*rest.fragment(), "");
        assert!(matches!(node.kind, NodeKind::StrongEmphasis));
    }

    #[test]
    fn smoke_test_parse_strong_emphasis_underscore() {
        let input = GrammarSpan::new("___hello___");
        let result = parse_strong_emphasis(input);
        assert!(result.is_ok());
    }
}
