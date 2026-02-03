//! Marco extended tab blocks parser - converts grammar output to AST nodes
//!
//! Converts `grammar::blocks::marco_tab_blocks::MarcoTabBlock` into:
//! - `NodeKind::TabGroup`
//! - `NodeKind::TabItem { title }`
//!
//! Each tab panel's raw markdown is recursively parsed as block nodes.

use super::shared::{to_parser_span, to_parser_span_range, GrammarSpan};
use crate::grammar::blocks::marco_tab_blocks::MarcoTabBlock;
use crate::parser::ast::{Document, Node, NodeKind};

/// Parse a Marco tab block into an AST node.
///
/// # Arguments
/// * `block` - Grammar output for the full tab container
/// * `full_start` / `full_end` - Spans covering the entire matched container
/// * `depth` - Recursion depth
/// * `parse_blocks_fn` - Parser callback for parsing each tab panel body
pub fn parse_marco_tab_block<F>(
    block: MarcoTabBlock<'_>,
    full_start: GrammarSpan<'_>,
    full_end: GrammarSpan<'_>,
    depth: usize,
    mut parse_blocks_fn: F,
) -> Result<Node, Box<dyn std::error::Error>>
where
    F: FnMut(&str, usize) -> Result<Document, Box<dyn std::error::Error>>,
{
    let group_span = to_parser_span_range(full_start, full_end);

    let mut group = Node {
        kind: NodeKind::TabGroup,
        span: Some(group_span),
        children: Vec::new(),
    };

    for item in block.items {
        let body = item.content.fragment();

        let panel_doc = match parse_blocks_fn(body, depth + 1) {
            Ok(doc) => doc,
            Err(e) => {
                log::warn!("Failed to parse tab panel content: {}", e);
                Document::new()
            }
        };

        group.children.push(Node {
            kind: NodeKind::TabItem { title: item.title },
            // The grammar currently only provides a precise span for the panel body.
            span: Some(to_parser_span(item.content)),
            children: panel_doc.children,
        });
    }

    Ok(group)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::shared::Span;

    fn mock_parse_blocks(
        input: &str,
        _depth: usize,
    ) -> Result<Document, Box<dyn std::error::Error>> {
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
    fn smoke_test_parse_marco_tab_block_builds_ast() {
        let raw = ":::tab\n@tab One\nHello\n@tab Two\nWorld\n:::\n";
        let span = Span::new(raw);
        let (rest, block) = crate::grammar::blocks::marco_tab_blocks::marco_tab_block(span)
            .expect("grammar parse failed");

        let node = parse_marco_tab_block(block, Span::new(raw), rest, 0, mock_parse_blocks)
            .expect("parser failed");

        assert!(matches!(node.kind, NodeKind::TabGroup));
        assert_eq!(node.children.len(), 2);
        assert!(matches!(node.children[0].kind, NodeKind::TabItem { .. }));
    }
}
