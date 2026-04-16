//! Inline-level markdown grammar/parsing namespace for intelligence.
//!
//! Current implementation reuses parser/grammar inlines from `crate::parser`.

use super::ast::{is_inline_kind, Node, NodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InlineCategory {
    Text,
    TaskCheckboxInline,
    Emphasis,
    Strong,
    StrongEmphasis,
    Strikethrough,
    Mark,
    Superscript,
    Subscript,
    Link,
    LinkReference,
    FootnoteReference,
    Image,
    CodeSpan,
    InlineHtml,
    HardBreak,
    SoftBreak,
    PlatformMention,
    InlineMath,
    DisplayMath,
}

pub fn classify_inline_kind(kind: &NodeKind) -> Option<InlineCategory> {
    match kind {
        NodeKind::Text(_) => Some(InlineCategory::Text),
        NodeKind::TaskCheckboxInline { .. } => Some(InlineCategory::TaskCheckboxInline),
        NodeKind::Emphasis => Some(InlineCategory::Emphasis),
        NodeKind::Strong => Some(InlineCategory::Strong),
        NodeKind::StrongEmphasis => Some(InlineCategory::StrongEmphasis),
        NodeKind::Strikethrough => Some(InlineCategory::Strikethrough),
        NodeKind::Mark => Some(InlineCategory::Mark),
        NodeKind::Superscript => Some(InlineCategory::Superscript),
        NodeKind::Subscript => Some(InlineCategory::Subscript),
        NodeKind::Link { .. } => Some(InlineCategory::Link),
        NodeKind::LinkReference { .. } => Some(InlineCategory::LinkReference),
        NodeKind::FootnoteReference { .. } => Some(InlineCategory::FootnoteReference),
        NodeKind::Image { .. } => Some(InlineCategory::Image),
        NodeKind::CodeSpan(_) => Some(InlineCategory::CodeSpan),
        NodeKind::InlineHtml(_) => Some(InlineCategory::InlineHtml),
        NodeKind::HardBreak => Some(InlineCategory::HardBreak),
        NodeKind::SoftBreak => Some(InlineCategory::SoftBreak),
        NodeKind::PlatformMention { .. } => Some(InlineCategory::PlatformMention),
        NodeKind::InlineMath { .. } => Some(InlineCategory::InlineMath),
        NodeKind::DisplayMath { .. } => Some(InlineCategory::DisplayMath),
        _ => None,
    }
}

pub fn is_inline_node(node: &Node) -> bool {
    is_inline_kind(&node.kind)
}

/// Collect inline nodes recursively in pre-order.
pub fn collect_inline_nodes<'a>(nodes: &'a [Node], out: &mut Vec<&'a Node>) {
    for node in nodes {
        if is_inline_node(node) {
            out.push(node);
        }
        if !node.children.is_empty() {
            collect_inline_nodes(&node.children, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_classify_inline_kind_basic() {
        assert_eq!(
            classify_inline_kind(&NodeKind::Text("x".to_string())),
            Some(InlineCategory::Text)
        );
        assert_eq!(classify_inline_kind(&NodeKind::Paragraph), None);
    }

    #[test]
    fn smoke_test_collect_inline_nodes_recursive() {
        let nodes = vec![Node {
            kind: NodeKind::Paragraph,
            span: None,
            children: vec![Node {
                kind: NodeKind::Link {
                    url: "https://example.com".to_string(),
                    title: None,
                },
                span: None,
                children: vec![Node {
                    kind: NodeKind::Text("example".to_string()),
                    span: None,
                    children: vec![],
                }],
            }],
        }];

        let mut inlines = Vec::new();
        collect_inline_nodes(&nodes, &mut inlines);

        assert_eq!(inlines.len(), 2);
        assert!(matches!(inlines[0].kind, NodeKind::Link { .. }));
        assert!(matches!(inlines[1].kind, NodeKind::Text(_)));
    }
}
