//! GFM-style footnote definition parser (block-level extension).
//!
//! Syntax:
//! - `[^label]: definition text`
//! - Continuation lines may be indented by 4 spaces or a tab.

use super::shared::{to_parser_span_range, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::Input;

/// Try to parse a footnote definition at the start of `input`.
///
/// Returns `Some((rest, node))` on success, or `None` if the input does not
/// start with a footnote definition.
pub fn parse_footnote_definition(input: GrammarSpan) -> Option<(GrammarSpan, Node)> {
    let frag = input.fragment();

    // Only check the first line quickly.
    let first_line_end = frag.find('\n').unwrap_or(frag.len());
    let first_line = &frag[..first_line_end];

    // Allow up to 3 leading spaces.
    let mut i = 0usize;
    while i < first_line.len() && i < 3 && first_line.as_bytes().get(i) == Some(&b' ') {
        i += 1;
    }
    if i < first_line.len() {
        // If there are 4+ leading spaces, it's an indented code block, not a footnote.
        if first_line.as_bytes().get(3) == Some(&b' ') {
            return None;
        }
    }

    let after_ws = &first_line[i..];
    if !after_ws.starts_with("[^") {
        return None;
    }

    // Find the closing `]:` on the first line.
    let marker_pos = after_ws.find(":").and_then(|colon| {
        // Ensure we have `]:` and that `]` exists before `:`.
        if colon == 0 {
            return None;
        }
        if after_ws.as_bytes().get(colon.wrapping_sub(1)) != Some(&b']') {
            return None;
        }
        Some(colon)
    })?;

    // after_ws looks like: [^label]:...
    // marker_pos points to ':'; label end is at marker_pos-1.
    if marker_pos < 3 {
        return None;
    }
    if !after_ws.starts_with("[^") {
        return None;
    }

    let label = &after_ws[2..marker_pos - 1];
    if label.is_empty() {
        return None;
    }

    // Ensure the exact marker is `]:`.
    if after_ws.as_bytes().get(marker_pos - 1) != Some(&b']')
        || after_ws.as_bytes().get(marker_pos) != Some(&b':')
    {
        return None;
    }

    // Capture content from after the ':' (and optional single space).
    let mut content = String::new();
    let mut after_colon = &after_ws[marker_pos + 1..];
    if after_colon.starts_with(' ') {
        after_colon = &after_colon[1..];
    }
    content.push_str(after_colon);

    // Consume continuation lines.
    let mut consumed_len = first_line_end;
    if first_line_end < frag.len() {
        // include newline
        consumed_len += 1;
    }

    let mut cursor = consumed_len;
    while cursor < frag.len() {
        let next_line_end = frag[cursor..]
            .find('\n')
            .map(|r| cursor + r)
            .unwrap_or(frag.len());
        let next_line = &frag[cursor..next_line_end];

        // Stop at blank line.
        if next_line.trim().is_empty() {
            break;
        }

        let (is_cont, line_content) = if let Some(stripped) = next_line.strip_prefix("    ") {
            (true, stripped)
        } else if let Some(stripped) = next_line.strip_prefix('\t') {
            (true, stripped)
        } else {
            (false, "")
        };

        if !is_cont {
            break;
        }

        content.push('\n');
        content.push_str(line_content);

        cursor = next_line_end;
        if cursor < frag.len() {
            cursor += 1; // newline
        }
        consumed_len = cursor;
    }

    let (rest, _taken) = input.take_split(consumed_len);
    let span = to_parser_span_range(input, rest);

    // Parse the definition content as paragraph-like blocks.
    // NOTE: We keep this conservative for now: a single paragraph with inline parsing.
    let content_children = match crate::parser::inlines::parse_inlines(&content) {
        Ok(nodes) => nodes,
        Err(_) => vec![Node {
            kind: NodeKind::Text(content),
            span: None,
            children: Vec::new(),
        }],
    };

    let paragraph = Node {
        kind: NodeKind::Paragraph,
        span: None,
        children: content_children,
    };

    let node = Node {
        kind: NodeKind::FootnoteDefinition {
            label: label.to_string(),
        },
        span: Some(span),
        children: vec![paragraph],
    };

    Some((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_footnote_definition_single_line() {
        let input = GrammarSpan::new("[^a]: Hello\nNext\n");
        let (rest, node) = parse_footnote_definition(input).expect("should parse");

        assert!(rest.fragment().starts_with("Next"));
        match node.kind {
            NodeKind::FootnoteDefinition { label } => assert_eq!(label, "a"),
            other => panic!("expected FootnoteDefinition, got {other:?}"),
        }

        assert_eq!(node.children.len(), 1);
        assert!(matches!(node.children[0].kind, NodeKind::Paragraph));
    }

    #[test]
    fn smoke_test_parse_footnote_definition_with_continuation_lines() {
        let input = GrammarSpan::new("[^multi]: First\n    second\n    third\nNext\n");
        let (rest, node) = parse_footnote_definition(input).expect("should parse");

        assert!(rest.fragment().starts_with("Next"));
        match node.kind {
            NodeKind::FootnoteDefinition { label } => assert_eq!(label, "multi"),
            other => panic!("expected FootnoteDefinition, got {other:?}"),
        }
    }
}
