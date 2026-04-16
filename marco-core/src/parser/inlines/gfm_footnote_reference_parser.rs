//! GFM-style footnote references (inline extension).
//!
//! Syntax: `[^label]`

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Input;

/// Parse an inline footnote reference of the form `[^label]`.
pub fn parse_footnote_reference(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let frag = input.fragment();
    if !frag.starts_with("[^") {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find the closing bracket on the same line.
    let close = match frag.find(']') {
        Some(idx) => idx,
        None => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    };

    if let Some(nl) = frag.find('\n') {
        if nl < close {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    if close < 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let label = &frag[2..close];
    if label.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let consumed_len = close + 1;
    let (rest, taken) = input.take_split(consumed_len);

    Ok((
        rest,
        Node {
            kind: NodeKind::FootnoteReference {
                label: label.to_string(),
            },
            span: Some(to_parser_span(taken)),
            children: Vec::new(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_footnote_reference_basic() {
        let input = GrammarSpan::new("[^a] test");
        let (rest, node) = parse_footnote_reference(input).expect("should parse");
        assert_eq!(*rest.fragment(), " test");
        match node.kind {
            NodeKind::FootnoteReference { label } => assert_eq!(label, "a"),
            other => panic!("expected FootnoteReference, got {other:?}"),
        }
    }

    #[test]
    fn smoke_test_parse_footnote_reference_unicode_label() {
        let input = GrammarSpan::new("[^参考]。");
        let (rest, node) = parse_footnote_reference(input).expect("should parse");
        assert_eq!(*rest.fragment(), "。");
        match node.kind {
            NodeKind::FootnoteReference { label } => assert_eq!(label, "参考"),
            other => panic!("expected FootnoteReference, got {other:?}"),
        }
    }
}
