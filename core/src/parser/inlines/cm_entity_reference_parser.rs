//! Entity reference parser - decode HTML entities into Unicode text
//!
//! CommonMark treats valid entity references as a single literal character (or
//! sometimes a small character sequence), e.g. `&copy;` -> `©`, `&#169;` -> `©`.
//!
//! We decode the entity here and emit it as `NodeKind::Text`, leaving the HTML
//! renderer to escape it safely.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Input;

/// Parse an entity reference starting at `&` and ending at `;`.
///
/// On success, returns a `Text` node containing the decoded character(s).
pub fn parse_entity_reference(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let fragment = input.fragment();

    if !fragment.starts_with('&') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Entity references must be terminated by ';' to be considered valid.
    // Keep the scan small to avoid accidentally treating long stretches as entities.
    const MAX_ENTITY_LEN: usize = 64;
    let semi_pos = fragment
        .find(';')
        .filter(|&idx| idx > 0 && idx < MAX_ENTITY_LEN)
        .ok_or_else(|| {
            nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TakeUntil,
            ))
        })?;

    let consumed_len = semi_pos + 1;
    let entity_str = &fragment[..consumed_len];

    // htmlescape decodes named and numeric entities; it errors on invalid entities.
    let decoded = match htmlescape::decode_html(entity_str) {
        Ok(s) => s,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }
    };

    // If decoding yields the same text, treat it as not-an-entity so other parsers can handle it.
    if decoded == entity_str {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    let entity_span = input.take(consumed_len);
    let rest = input.take_from(consumed_len);

    let node = Node {
        kind: NodeKind::Text(decoded),
        span: Some(to_parser_span(entity_span)),
        children: Vec::new(),
    };

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_entity_reference_named() {
        let input = GrammarSpan::new("&copy; and more");
        let (rest, node) = parse_entity_reference(input).expect("parse failed");

        assert_eq!(rest.fragment(), &" and more");
        match node.kind {
            NodeKind::Text(s) => assert_eq!(s, "©"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn smoke_test_parse_entity_reference_numeric_decimal() {
        let input = GrammarSpan::new("&#169;");
        let (_, node) = parse_entity_reference(input).expect("parse failed");

        match node.kind {
            NodeKind::Text(s) => assert_eq!(s, "©"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn smoke_test_parse_entity_reference_invalid_entity_fails() {
        let input = GrammarSpan::new("&nosuchentity;");
        assert!(parse_entity_reference(input).is_err());
    }
}
