//! Canonical markdown AST re-exports for intelligence subsystems.

pub use crate::parser::{Document, Node, NodeKind, Position, Span};

/// Alias types used by the intelligence boundary.
pub type MarkdownDocument = Document;
pub type MarkdownNode = Node;
pub type MarkdownNodeKind = NodeKind;

/// Return this node's span if available.
pub fn node_span(node: &Node) -> Option<Span> {
    node.span
}

/// True when the node kind is considered block-level.
pub fn is_block_kind(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Heading { .. }
            | NodeKind::Paragraph
            | NodeKind::CodeBlock { .. }
            | NodeKind::ThematicBreak
            | NodeKind::List { .. }
            | NodeKind::ListItem
            | NodeKind::DefinitionList
            | NodeKind::DefinitionTerm
            | NodeKind::DefinitionDescription
            | NodeKind::TaskCheckbox { .. }
            | NodeKind::Blockquote
            | NodeKind::Admonition { .. }
            | NodeKind::TabGroup
            | NodeKind::TabItem { .. }
            | NodeKind::SliderDeck { .. }
            | NodeKind::Slide { .. }
            | NodeKind::Table { .. }
            | NodeKind::TableRow { .. }
            | NodeKind::TableCell { .. }
            | NodeKind::HtmlBlock { .. }
            | NodeKind::FootnoteDefinition { .. }
            | NodeKind::MermaidDiagram { .. }
    )
}

/// True when the node kind is considered inline-level.
pub fn is_inline_kind(kind: &NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Text(_)
            | NodeKind::TaskCheckboxInline { .. }
            | NodeKind::Emphasis
            | NodeKind::Strong
            | NodeKind::StrongEmphasis
            | NodeKind::Strikethrough
            | NodeKind::Mark
            | NodeKind::Superscript
            | NodeKind::Subscript
            | NodeKind::Link { .. }
            | NodeKind::LinkReference { .. }
            | NodeKind::FootnoteReference { .. }
            | NodeKind::Image { .. }
            | NodeKind::CodeSpan(_)
            | NodeKind::InlineHtml(_)
            | NodeKind::HardBreak
            | NodeKind::SoftBreak
            | NodeKind::PlatformMention { .. }
            | NodeKind::InlineMath { .. }
            | NodeKind::DisplayMath { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_block_and_inline_classification() {
        assert!(is_block_kind(&NodeKind::Paragraph));
        assert!(is_block_kind(&NodeKind::CodeBlock {
            language: None,
            code: "x".to_string()
        }));
        assert!(!is_block_kind(&NodeKind::Text("x".to_string())));

        assert!(is_inline_kind(&NodeKind::Text("x".to_string())));
        assert!(is_inline_kind(&NodeKind::Link {
            url: "https://example.com".to_string(),
            title: None
        }));
        assert!(!is_inline_kind(&NodeKind::Paragraph));
    }
}
