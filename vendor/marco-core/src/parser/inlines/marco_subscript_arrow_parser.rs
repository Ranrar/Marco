//! Arrow-style subscript parser (extended syntax).

use super::shared::{opt_span_range, GrammarSpan};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse `˅text˅` and convert to AST node.
pub fn parse_subscript_arrow(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, content) = grammar::subscript_arrow(input)?;

    let span = opt_span_range(start, rest);

    let children = match crate::parser::inlines::parse_inlines_from_span(content) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse subscript-arrow children: {}", e);
            vec![]
        }
    };

    Ok((
        rest,
        Node {
            kind: NodeKind::Subscript,
            span,
            children,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_subscript_arrow() {
        let input = GrammarSpan::new("˅hi˅");
        let (rest, node) = parse_subscript_arrow(input).unwrap();
        assert_eq!(*rest.fragment(), "");
        assert!(matches!(node.kind, NodeKind::Subscript));
    }
}
