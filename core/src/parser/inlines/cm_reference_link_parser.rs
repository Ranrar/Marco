//! Reference-style link parser - parse `[text][label]`, `[label][]`, and shortcut `[label]`.
//!
//! Resolution against `[label]: url` definitions happens in a later pass
//! (see `core/src/parser/mod.rs`).

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Input;

pub fn parse_reference_link(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start_input = input;
    let content_str = input.fragment();

    if !content_str.starts_with('[') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find closing bracket for first label.
    let bracket_pos = content_str[1..].find(']').ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        ))
    })?;
    if bracket_pos == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        )));
    }

    let absolute_bracket_pos = 1 + bracket_pos;
    let link_text_str = &content_str[1..absolute_bracket_pos];

    // Mirror the inline-link parser behavior: avoid treating unmatched backticks
    // inside the label as a link label (helps avoid weird interactions with code spans).
    let backtick_count = link_text_str.chars().filter(|&c| c == '`').count();
    if backtick_count % 2 != 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    // Preserve position information.
    let link_text = start_input
        .take_from(1)
        .take(absolute_bracket_pos.saturating_sub(1));

    let after_first_bracket = absolute_bracket_pos + 1;

    // If this is an inline link `[text](url...)`, let the inline link parser handle it.
    if after_first_bracket < content_str.len()
        && content_str.as_bytes()[after_first_bracket] == b'('
    {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Parse the displayed link text as inlines.
    let children = match crate::parser::inlines::parse_inlines_from_span(link_text) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse reference link text children: {}", e);
            vec![]
        }
    };

    let mut label = link_text_str.to_string();
    let mut suffix = String::new();
    let mut consumed_len = after_first_bracket;

    // Full/collapsed reference link: `[text][label]` or `[label][]`
    if after_first_bracket < content_str.len()
        && content_str.as_bytes()[after_first_bracket] == b'['
    {
        // Collapsed reference link: `[]`
        if after_first_bracket + 1 < content_str.len()
            && content_str.as_bytes()[after_first_bracket + 1] == b']'
        {
            // Label is the same as the first bracketed text.
            suffix = "[]".to_string();
            consumed_len = after_first_bracket + 2;
        } else {
            // Full reference link: `[label]`
            let rest = &content_str[(after_first_bracket + 1)..];
            let close2_rel = rest.find(']').ok_or_else(|| {
                nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TakeUntil,
                ))
            })?;

            let label_str = &rest[..close2_rel];
            label = label_str.to_string();
            suffix = format!("[{label_str}]");
            consumed_len = after_first_bracket + 1 + close2_rel + 1;
        }
    }

    let span = to_parser_span(link_text);
    let rest = start_input.take_from(consumed_len);

    let node = Node {
        kind: NodeKind::LinkReference { label, suffix },
        span: Some(span),
        children,
    };

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_reference_link_shortcut() {
        let input = GrammarSpan::new("[foo] bar");
        let (rest, node) = parse_reference_link(input).expect("parse failed");
        assert_eq!(rest.fragment(), &" bar");
        assert!(matches!(node.kind, NodeKind::LinkReference { .. }));
    }

    #[test]
    fn smoke_test_parse_reference_link_collapsed() {
        let input = GrammarSpan::new("[foo][]");
        let (rest, node) = parse_reference_link(input).expect("parse failed");
        assert_eq!(rest.fragment(), &"");
        match node.kind {
            NodeKind::LinkReference { label, suffix } => {
                assert_eq!(label, "foo");
                assert_eq!(suffix, "[]");
            }
            other => panic!("unexpected node kind: {other:?}"),
        }
    }

    #[test]
    fn smoke_test_parse_reference_link_full() {
        let input = GrammarSpan::new("[foo][bar]");
        let (rest, node) = parse_reference_link(input).expect("parse failed");
        assert_eq!(rest.fragment(), &"");
        match node.kind {
            NodeKind::LinkReference { label, suffix } => {
                assert_eq!(label, "bar");
                assert_eq!(suffix, "[bar]");
            }
            other => panic!("unexpected node kind: {other:?}"),
        }
    }

    #[test]
    fn smoke_test_reference_link_does_not_match_inline_link() {
        let input = GrammarSpan::new("[foo](url)");
        assert!(parse_reference_link(input).is_err());
    }
}
