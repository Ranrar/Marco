//! Block-level markdown grammar/parsing namespace for intelligence.
//!
//! Current implementation reuses parser/grammar blocks from `crate::parser`.

use super::ast::{is_block_kind, Document, Node, NodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCategory {
    Heading,
    Paragraph,
    CodeBlock,
    ThematicBreak,
    List,
    ListItem,
    DefinitionList,
    DefinitionTerm,
    DefinitionDescription,
    TaskCheckbox,
    Blockquote,
    Admonition,
    TabGroup,
    TabItem,
    SliderDeck,
    Slide,
    Table,
    TableRow,
    TableCell,
    HtmlBlock,
    FootnoteDefinition,
    MermaidDiagram,
}

pub fn classify_block_kind(kind: &NodeKind) -> Option<BlockCategory> {
    match kind {
        NodeKind::Heading { .. } => Some(BlockCategory::Heading),
        NodeKind::Paragraph => Some(BlockCategory::Paragraph),
        NodeKind::CodeBlock { .. } => Some(BlockCategory::CodeBlock),
        NodeKind::ThematicBreak => Some(BlockCategory::ThematicBreak),
        NodeKind::List { .. } => Some(BlockCategory::List),
        NodeKind::ListItem => Some(BlockCategory::ListItem),
        NodeKind::DefinitionList => Some(BlockCategory::DefinitionList),
        NodeKind::DefinitionTerm => Some(BlockCategory::DefinitionTerm),
        NodeKind::DefinitionDescription => Some(BlockCategory::DefinitionDescription),
        NodeKind::TaskCheckbox { .. } => Some(BlockCategory::TaskCheckbox),
        NodeKind::Blockquote => Some(BlockCategory::Blockquote),
        NodeKind::Admonition { .. } => Some(BlockCategory::Admonition),
        NodeKind::TabGroup => Some(BlockCategory::TabGroup),
        NodeKind::TabItem { .. } => Some(BlockCategory::TabItem),
        NodeKind::SliderDeck { .. } => Some(BlockCategory::SliderDeck),
        NodeKind::Slide { .. } => Some(BlockCategory::Slide),
        NodeKind::Table { .. } => Some(BlockCategory::Table),
        NodeKind::TableRow { .. } => Some(BlockCategory::TableRow),
        NodeKind::TableCell { .. } => Some(BlockCategory::TableCell),
        NodeKind::HtmlBlock { .. } => Some(BlockCategory::HtmlBlock),
        NodeKind::FootnoteDefinition { .. } => Some(BlockCategory::FootnoteDefinition),
        NodeKind::MermaidDiagram { .. } => Some(BlockCategory::MermaidDiagram),
        _ => None,
    }
}

pub fn is_block_node(node: &Node) -> bool {
    is_block_kind(&node.kind)
}

/// Iterate top-level block nodes in document order.
pub fn top_level_blocks(document: &Document) -> impl Iterator<Item = &Node> {
    document.children.iter().filter(|node| is_block_node(node))
}

/// Collect all block nodes recursively in pre-order.
pub fn collect_block_nodes<'a>(nodes: &'a [Node], out: &mut Vec<&'a Node>) {
    for node in nodes {
        if is_block_node(node) {
            out.push(node);
        }
        if !node.children.is_empty() {
            collect_block_nodes(&node.children, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_classify_block_kind_basic() {
        assert_eq!(
            classify_block_kind(&NodeKind::Paragraph),
            Some(BlockCategory::Paragraph)
        );
        assert_eq!(classify_block_kind(&NodeKind::Text("x".to_string())), None);
    }

    #[test]
    fn smoke_test_collect_block_nodes_recursive() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Link {
                        url: "https://example.com".to_string(),
                        title: None,
                    },
                    span: None,
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let mut blocks = Vec::new();
        collect_block_nodes(&doc.children, &mut blocks);

        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].kind, NodeKind::Paragraph));
    }
}
