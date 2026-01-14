//! GitHub-style admonitions / alerts (GFM extension).
//!
//! GitHub implements alerts as a Markdown extension based on blockquotes.
//! Syntax:
//!
//! ```text
//! > [!NOTE]
//! > Body...
//! ```
//!
//! GitHub docs note that alerts cannot be nested within other elements.
//! We implement that by only transforming *top-level* blockquotes.

use crate::parser::ast::{AdmonitionKind, Document, Node, NodeKind};

/// Convert eligible top-level blockquotes into `NodeKind::Admonition`.
///
/// This is a post-parse pass because blockquote content must be parsed into
/// blocks/inlines before we can reliably detect the marker paragraph.
pub fn apply_gfm_admonitions(document: &mut Document) {
    apply_to_nodes(&mut document.children, true);
}

fn apply_to_nodes(nodes: &mut [Node], is_top_level: bool) {
    for node in nodes.iter_mut() {
        if is_top_level {
            try_transform_blockquote(node);
        }

        // Children are never considered "top-level" elements.
        if !node.children.is_empty() {
            apply_to_nodes(&mut node.children, false);
        }
    }
}

fn try_transform_blockquote(node: &mut Node) {
    if !matches!(node.kind, NodeKind::Blockquote) {
        return;
    }

    // Marker must be the very first block inside the blockquote.
    let Some(first_child) = node.children.first_mut() else {
        return;
    };

    let Some((kind, remove_first_paragraph)) =
        strip_admonition_marker_from_first_paragraph(first_child)
    else {
        return;
    };

    // If the marker consumed the full first paragraph, remove it.
    if remove_first_paragraph {
        node.children.remove(0);
    }

    node.kind = NodeKind::Admonition { kind };
}

fn strip_admonition_marker_from_first_paragraph(
    paragraph: &mut Node,
) -> Option<(AdmonitionKind, bool)> {
    if !matches!(paragraph.kind, NodeKind::Paragraph) {
        return None;
    }

    // GitHub alerts are written as two blockquote lines without a blank line.
    // In CommonMark parsing, that typically becomes a *single* paragraph with a
    // soft line break between lines.
    //
    // We therefore treat the marker as the text prefix up to the first break.
    let mut raw = String::new();
    let mut idx = 0usize;

    while idx < paragraph.children.len() {
        match &paragraph.children[idx].kind {
            NodeKind::Text(t) => {
                raw.push_str(t);
                idx += 1;
            }
            NodeKind::SoftBreak | NodeKind::HardBreak => {
                break;
            }
            _ => {
                // Marker must be plain text only.
                return None;
            }
        }
    }

    let kind = admonition_marker_kind_from_raw(&raw)?;

    // If there's a break after the marker, remove the marker and the break,
    // leaving the rest of the paragraph as the first body line.
    if idx < paragraph.children.len()
        && matches!(
            paragraph.children[idx].kind,
            NodeKind::SoftBreak | NodeKind::HardBreak
        )
    {
        paragraph.children.drain(0..=idx);
        return Some((kind, false));
    }

    // Marker consumed the full paragraph.
    Some((kind, true))
}

fn admonition_marker_kind_from_raw(raw: &str) -> Option<AdmonitionKind> {
    let normalized = raw.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "[!NOTE]" => Some(AdmonitionKind::Note),
        "[!TIP]" => Some(AdmonitionKind::Tip),
        "[!IMPORTANT]" => Some(AdmonitionKind::Important),
        "[!WARNING]" => Some(AdmonitionKind::Warning),
        "[!CAUTION]" => Some(AdmonitionKind::Caution),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_detects_marker_case_insensitive() {
        let mut marker = Node {
            kind: NodeKind::Paragraph,
            span: None,
            children: vec![Node {
                kind: NodeKind::Text("[!note]".to_string()),
                span: None,
                children: vec![],
            }],
        };

        let (kind, remove) = strip_admonition_marker_from_first_paragraph(&mut marker).unwrap();
        assert_eq!(kind, AdmonitionKind::Note);
        assert!(remove);
    }

    #[test]
    fn smoke_test_rejects_marker_with_non_text_children() {
        let mut marker = Node {
            kind: NodeKind::Paragraph,
            span: None,
            children: vec![Node {
                kind: NodeKind::Emphasis,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Text("[!NOTE]".to_string()),
                    span: None,
                    children: vec![],
                }],
            }],
        };

        assert!(strip_admonition_marker_from_first_paragraph(&mut marker).is_none());
    }

    #[test]
    fn smoke_test_transforms_top_level_blockquote_only() {
        let mut doc = Document {
            children: vec![Node {
                kind: NodeKind::Blockquote,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Paragraph,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("[!NOTE]".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                    Node {
                        kind: NodeKind::Paragraph,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("Body".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                ],
            }],
            ..Default::default()
        };

        apply_gfm_admonitions(&mut doc);

        assert!(matches!(
            doc.children[0].kind,
            NodeKind::Admonition {
                kind: AdmonitionKind::Note
            }
        ));

        // Marker paragraph should be removed.
        assert_eq!(doc.children[0].children.len(), 1);
    }
}
