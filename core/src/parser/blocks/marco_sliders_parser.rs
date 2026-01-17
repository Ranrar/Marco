//! Marco sliders parser - converts grammar output to AST nodes
//!
//! Converts `grammar::blocks::marco_sliders::MarcoSlideDeck` into:
//! - `NodeKind::SliderDeck { timer_seconds }`
//! - `NodeKind::Slide { vertical }`
//!
//! Each slide's raw markdown is recursively parsed as block nodes.

use super::shared::{to_parser_span, to_parser_span_range, GrammarSpan};
use crate::grammar::blocks::marco_sliders::MarcoSlideDeck;
use crate::parser::ast::{Document, Node, NodeKind};
use anyhow::Result;

/// Parse a Marco slide deck into an AST node.
///
/// # Arguments
/// * `deck` - Grammar output for the full slide deck
/// * `full_start` / `full_end` - Spans covering the entire matched container
/// * `depth` - Recursion depth
/// * `parse_blocks_fn` - Parser callback for parsing each slide body
pub fn parse_marco_slide_deck<F>(
    deck: MarcoSlideDeck<'_>,
    full_start: GrammarSpan<'_>,
    full_end: GrammarSpan<'_>,
    depth: usize,
    mut parse_blocks_fn: F,
) -> Result<Node>
where
    F: FnMut(&str, usize) -> Result<Document>,
{
    let deck_span = to_parser_span_range(full_start, full_end);

    let mut root = Node {
        kind: NodeKind::SliderDeck {
            timer_seconds: deck.timer_seconds,
        },
        span: Some(deck_span),
        children: Vec::new(),
    };

    for slide in deck.slides {
        let body = slide.content.fragment();

        let slide_doc = match parse_blocks_fn(body, depth + 1) {
            Ok(doc) => doc,
            Err(e) => {
                log::warn!("Failed to parse slide content: {}", e);
                Document::new()
            }
        };

        root.children.push(Node {
            kind: NodeKind::Slide {
                vertical: slide.vertical,
            },
            // The grammar currently only provides a precise span for the slide body.
            span: Some(to_parser_span(slide.content)),
            children: slide_doc.children,
        });
    }

    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::shared::Span;

    fn mock_parse_blocks(input: &str, _depth: usize) -> Result<Document> {
        let mut doc = Document::new();
        if !input.trim().is_empty() {
            doc.children.push(Node {
                kind: NodeKind::Text(input.to_string()),
                span: None,
                children: Vec::new(),
            });
        }
        Ok(doc)
    }

    #[test]
    fn smoke_test_parse_marco_slide_deck_builds_ast() {
        let raw = "@slidestart\nA\n---\nB\n@slideend\n";
        let span = Span::new(raw);
        let (rest, deck) = crate::grammar::blocks::marco_sliders::marco_slide_deck(span)
            .expect("grammar parse failed");

        let node = parse_marco_slide_deck(deck, Span::new(raw), rest, 0, mock_parse_blocks)
            .expect("parser failed");

        assert!(matches!(node.kind, NodeKind::SliderDeck { .. }));
        assert_eq!(node.children.len(), 2);
        assert!(matches!(node.children[0].kind, NodeKind::Slide { .. }));
    }
}
