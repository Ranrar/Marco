//! Marco inline task checkbox parser
//!
//! Parses inline task checkbox markers that appear *mid-paragraph*, e.g.:
//! `Do this [ ] today` or `Done? [x], great.`
//!
//! Notes:
//! - We keep this conservative to avoid breaking CommonMark link syntax.
//! - Boundary rules (previous character) are handled by the caller.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::bytes::complete::take;
use nom::IResult;
use nom::Parser;

fn match_task_marker(fragment: &str) -> Option<(bool, usize)> {
    if fragment.starts_with("[ ]") {
        Some((false, 3))
    } else if fragment.starts_with("[x]") || fragment.starts_with("[X]") {
        Some((true, 3))
    } else {
        None
    }
}

fn next_char_is_acceptable_boundary(after_marker: &str) -> bool {
    match after_marker.chars().next() {
        None => true,
        Some(ch) if ch.is_whitespace() => true,
        // Allow common punctuation adjacency: "[x]," or "[x]."
        Some('.') | Some(',') | Some(';') | Some(':') | Some('!') | Some('?') => true,
        Some(')') | Some(']') | Some('}') => true,
        // Explicitly reject CommonMark link/reference-link continuations.
        Some('(') | Some('[') => false,
        // Reject alphanumeric continuations like "[x]done".
        Some(ch) if ch.is_alphanumeric() || ch == '_' => false,
        // Otherwise, be permissive (e.g. dashes).
        Some(_) => true,
    }
}

/// Parse an inline task checkbox marker.
///
/// Recognizes:
/// - `[ ]`
/// - `[x]` / `[X]`
///
/// Boundary rules:
/// - Caller should ensure we are at a reasonable *start boundary* (e.g. preceded by
///   whitespace or punctuation). This avoids matching `word[x]`.
/// - This parser ensures the following character isn't a link continuation.
pub fn parse_task_checkbox_inline(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let fragment = input.fragment();

    let (checked, consumed) = match_task_marker(fragment).ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
    })?;

    let after = &fragment[consumed..];
    if !next_char_is_acceptable_boundary(after) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    let (rest, taken) = take(consumed).parse(input)?;

    Ok((
        rest,
        Node {
            kind: NodeKind::TaskCheckboxInline { checked },
            span: Some(to_parser_span(taken)),
            children: Vec::new(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_inline_task_checkbox_unchecked() {
        let input = GrammarSpan::new("[ ] todo");
        let (rest, node) = parse_task_checkbox_inline(input).expect("should parse");

        assert_eq!(rest.fragment(), &" todo");
        assert!(matches!(
            node.kind,
            NodeKind::TaskCheckboxInline { checked: false }
        ));
    }

    #[test]
    fn smoke_test_parse_inline_task_checkbox_checked_uppercase() {
        let input = GrammarSpan::new("[X], done");
        let (rest, node) = parse_task_checkbox_inline(input).expect("should parse");

        assert_eq!(rest.fragment(), &", done");
        assert!(matches!(
            node.kind,
            NodeKind::TaskCheckboxInline { checked: true }
        ));
    }

    #[test]
    fn smoke_test_inline_task_checkbox_rejects_link_continuation() {
        let input = GrammarSpan::new("[x](https://example.com)");
        assert!(parse_task_checkbox_inline(input).is_err());
    }
}
