//! Strikethrough parser (GFM extension)

use super::shared::{to_parser_span_range, GrammarSpan};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse `~~text~~` and convert to AST node.
pub fn parse_strikethrough(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, content) = grammar::strikethrough(input)?;

    let span = to_parser_span_range(start, rest);

    let children = match crate::parser::inlines::parse_inlines_from_span(content) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse strikethrough children: {}", e);
            vec![]
        }
    };

    Ok((
        rest,
        Node {
            kind: NodeKind::Strikethrough,
            span: Some(span),
            children,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_strikethrough() {
        let input = GrammarSpan::new("~~hi~~");
        let (rest, node) = parse_strikethrough(input).unwrap();
        assert_eq!(*rest.fragment(), "");
        assert!(matches!(node.kind, NodeKind::Strikethrough));
    }
}
