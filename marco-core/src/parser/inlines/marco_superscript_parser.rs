//! Superscript parser (Marco extension)

use super::shared::{to_parser_span_range, GrammarSpan};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse `^text^` and convert to AST node.
pub fn parse_superscript(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, content) = grammar::superscript(input)?;

    let span = to_parser_span_range(start, rest);

    let children = match crate::parser::inlines::parse_inlines_from_span(content) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse superscript children: {}", e);
            vec![]
        }
    };

    Ok((
        rest,
        Node {
            kind: NodeKind::Superscript,
            span: Some(span),
            children,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_superscript() {
        let input = GrammarSpan::new("^hi^");
        let (rest, node) = parse_superscript(input).unwrap();
        assert_eq!(*rest.fragment(), "");
        assert!(matches!(node.kind, NodeKind::Superscript));
    }
}
