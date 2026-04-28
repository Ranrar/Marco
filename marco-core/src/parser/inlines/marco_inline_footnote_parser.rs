//! Inline footnotes (Marco extension).
//!
//! Syntax: `^[footnote content]`
//!
//! This follows the same general rendering model as GFM-style footnotes:
//! - We emit a `FootnoteReference` inline at the point of use.
//! - We also synthesize a `FootnoteDefinition` node (out-of-band in rendering).
//!
//! Notes / constraints:
//! - Inline footnotes are single-use: each `^[...]` creates a new footnote.
//! - The content is parsed as inline Markdown (no multi-paragraph support).
//! - This parser is intentionally conservative and does not span newlines.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Input;

/// Parse an inline footnote of the form `^[content]`.
///
/// Returns a tuple of nodes:
/// - `(FootnoteReference, FootnoteDefinition)`
pub fn parse_inline_footnote(input: GrammarSpan) -> IResult<GrammarSpan, (Node, Node)> {
    let start = input;
    let frag = input.fragment();

    if !frag.starts_with("^[") {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find the closing `]` on the same line.
    // We skip `]` inside simple backtick regions and ignore escaped `\]`.
    let mut in_code = false;
    let mut pos = 2usize;
    let bytes = frag.as_bytes();

    while pos < bytes.len() {
        let b = bytes[pos];

        if b == b'\n' {
            // Inline footnotes must not span lines.
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        if b == b'`' {
            in_code = !in_code;
            pos += 1;
            continue;
        }

        if b == b']' && !in_code {
            // If escaped, treat as literal.
            if pos > 0 && bytes[pos - 1] == b'\\' {
                pos += 1;
                continue;
            }

            // Reject empty content.
            if pos <= 2 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )));
            }

            let consumed_len = pos + 1;
            let (rest, taken) = start.take_split(consumed_len);

            let content_len = pos - 2;
            let content_span = start.take_from(2).take(content_len);

            // Generate a label that is deterministic and extremely unlikely to
            // collide with user-provided footnote labels.
            let label = format!(
                "marco-inline-{}-{}-{}",
                start.location_line(),
                start.get_column(),
                start.location_offset()
            );

            let reference = Node {
                kind: NodeKind::FootnoteReference {
                    label: label.clone(),
                },
                span: Some(to_parser_span(taken)),
                children: Vec::new(),
            };

            let content_children =
                match crate::parser::inlines::parse_inlines_from_span(content_span) {
                    Ok(children) => children,
                    Err(e) => {
                        log::warn!("Failed to parse inline footnote content: {}", e);
                        vec![]
                    }
                };

            let paragraph = Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: content_children,
            };

            let definition = Node {
                kind: NodeKind::FootnoteDefinition { label },
                // Keep the definition unspanned; it is rendered out-of-band.
                span: None,
                children: vec![paragraph],
            };

            return Ok((rest, (reference, definition)));
        }

        pos += 1;
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::TakeUntil,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_inline_footnote_basic() {
        let input = GrammarSpan::new("^[hi] rest");
        let (rest, (ref_node, def_node)) = parse_inline_footnote(input).expect("should parse");
        assert_eq!(*rest.fragment(), " rest");

        match ref_node.kind {
            NodeKind::FootnoteReference { label } => {
                assert!(label.starts_with("marco-inline-"));
            }
            other => panic!("expected FootnoteReference, got {other:?}"),
        }

        match def_node.kind {
            NodeKind::FootnoteDefinition { label } => {
                assert!(label.starts_with("marco-inline-"));
            }
            other => panic!("expected FootnoteDefinition, got {other:?}"),
        }

        assert_eq!(def_node.children.len(), 1);
        assert!(matches!(def_node.children[0].kind, NodeKind::Paragraph));
    }

    #[test]
    fn smoke_test_parse_inline_footnote_rejects_newline() {
        let input = GrammarSpan::new("^[hi\nthere]");
        assert!(parse_inline_footnote(input).is_err());
    }

    #[test]
    fn smoke_test_parse_inline_footnote_rejects_empty() {
        let input = GrammarSpan::new("^[]");
        assert!(parse_inline_footnote(input).is_err());
    }
}
