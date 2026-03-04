//! Reference-style link parser - parse `[text][label]`, `[label][]`, and shortcut `[label]`.
//!
//! Resolution against `[label]: url` definitions happens in a later pass
//! (see `core/src/parser/mod.rs`).

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Input;

fn find_matching_closing_bracket(
    s: &str,
    open_bracket_idx: usize,
    allow_nested_brackets: bool,
) -> Option<usize> {
    if s.as_bytes().get(open_bracket_idx) != Some(&b'[') {
        return None;
    }

    let mut depth = 1usize;
    let mut escaped = false;
    let mut i = open_bracket_idx + 1;

    while i < s.len() {
        let ch = s[i..].chars().next()?;
        let ch_len = ch.len_utf8();

        if escaped {
            escaped = false;
            i += ch_len;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            i += ch_len;
            continue;
        }

        if ch == '[' {
            if allow_nested_brackets {
                depth += 1;
                i += ch_len;
                continue;
            }

            // Link labels (not link text) cannot contain unescaped '['.
            return None;
        }

        if ch == ']' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(i);
            }
        }

        i += ch_len;
    }

    None
}

fn is_valid_reference_label_content(label: &str) -> bool {
    // CommonMark: max 999 characters in label content.
    if label.chars().count() > 999 {
        return false;
    }

    // Must contain at least one non-whitespace character.
    if label.trim().is_empty() {
        return false;
    }

    // Labels cannot contain unescaped '[' or ']'.
    let mut escaped = false;
    for ch in label.chars() {
        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '[' || ch == ']' {
            return false;
        }
    }

    true
}

pub fn parse_reference_link(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start_input = input;
    let content_str = input.fragment();

    if !content_str.starts_with('[') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find closing bracket for first link text, allowing nested brackets and
    // treating escaped brackets as literal text.
    let absolute_bracket_pos =
        find_matching_closing_bracket(content_str, 0, true).ok_or_else(|| {
            nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TakeUntil,
            ))
        })?;

    if absolute_bracket_pos == 1 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        )));
    }

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
            if !is_valid_reference_label_content(&label) {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }

            // Label is the same as the first bracketed text.
            suffix = "[]".to_string();
            consumed_len = after_first_bracket + 2;
        } else {
            // Full reference link: `[label]`
            let close2_abs = find_matching_closing_bracket(content_str, after_first_bracket, false)
                .ok_or_else(|| {
                    nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::TakeUntil,
                    ))
                })?;

            let label_str = &content_str[(after_first_bracket + 1)..close2_abs];
            if !is_valid_reference_label_content(label_str) {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }

            label = label_str.to_string();
            suffix = content_str[after_first_bracket..=close2_abs].to_string();
            consumed_len = close2_abs + 1;
        }
    } else if !is_valid_reference_label_content(&label) {
        // Shortcut label validation.
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
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

    #[test]
    fn smoke_test_parse_reference_link_full_with_escaped_right_bracket_in_label() {
        let input = GrammarSpan::new("[foo][ref\\[]");
        let (rest, node) = parse_reference_link(input).expect("parse failed");
        assert_eq!(rest.fragment(), &"");

        match node.kind {
            NodeKind::LinkReference { label, suffix } => {
                assert_eq!(label, "ref\\[");
                assert_eq!(suffix, "[ref\\[]");
            }
            other => panic!("unexpected node kind: {other:?}"),
        }
    }

    #[test]
    fn smoke_test_parse_reference_link_allows_nested_brackets_in_link_text() {
        let input = GrammarSpan::new("[link [nested]][ref]");
        let (rest, node) = parse_reference_link(input).expect("parse failed");
        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::LinkReference { .. }));
    }

    #[test]
    fn smoke_test_parse_reference_link_rejects_unescaped_bracket_in_label() {
        let input = GrammarSpan::new("[foo][ref[bar]]");
        assert!(parse_reference_link(input).is_err());
    }

    #[test]
    fn smoke_test_parse_reference_link_rejects_blank_label_shortcut() {
        let input = GrammarSpan::new("[  ]");
        assert!(parse_reference_link(input).is_err());
    }
}
