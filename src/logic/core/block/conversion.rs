use crate::logic::core::block::parser::BlockNode;
use crate::logic::ast::blocks_and_inlines::{Block, ContainerBlock, LeafBlock, ListKind, ListMarker, OrderedDelimiter};
use crate::logic::ast::inlines::Inline;

/// Convert a BlockNode (parser output) to Block (AST for rendering)
pub fn blocknode_to_block(node: &BlockNode) -> Block {
    match node {
        BlockNode::Paragraph { children } => {
            let inlines: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|n| inline_node_to_inline_and_pos(n)).collect();
            Block::Leaf(LeafBlock::Paragraph(inlines, None))
        }
        BlockNode::List { items, ordered, tight, delimiter } => {
            let kind = if *ordered {
                ListKind::Ordered { start: 1, delimiter: if *delimiter == '.' { OrderedDelimiter::Period } else { OrderedDelimiter::Paren } }
            } else {
                ListKind::Bullet { char: *delimiter }
            };
            let converted_items = items.iter().map(|item| blocknode_to_block(item)).collect();
            Block::Container(ContainerBlock::List { kind, tight: *tight, items: converted_items, spread: false, attributes: None })
        }
        BlockNode::Item { children, task } => {
            let marker = ListMarker::Bullet { char: '-' }; // TODO: Detect marker
            let converted = children.iter().map(|c| blocknode_to_block(c)).collect();
            Block::Container(ContainerBlock::ListItem { marker, contents: converted, task_checked: *task, spread: false, association: None, attributes: None })
        }
        BlockNode::BlockQuote { children } => {
            let converted = children.iter().map(|c| blocknode_to_block(c)).collect();
            Block::Container(ContainerBlock::BlockQuote(converted, None))
        }
        BlockNode::Heading { level, children } => {
            let inlines: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|n| inline_node_to_inline_and_pos(n)).collect();
            Block::Leaf(LeafBlock::Heading { level: *level, content: inlines, attributes: None })
        }
        BlockNode::ThematicBreak => {
            Block::Leaf(LeafBlock::ThematicBreak { marker: '-', count: 3, raw: "---".to_string(), attributes: None })
        }
        BlockNode::MathBlock { text } => {
            Block::Leaf(LeafBlock::Math(crate::logic::ast::math::MathBlock { content: text.clone(), display: true, math_type: crate::logic::ast::math::MathType::LaTeX, position: None, attributes: None }))
        }
        BlockNode::CustomTag { name, children } => {
            let converted = children.iter().map(|c| blocknode_to_block(c)).collect();
            Block::Leaf(LeafBlock::CustomTagBlock { name: name.clone(), data: None, content: converted, attributes: None })
        }
        BlockNode::FrontMatter { text } => {
            Block::Leaf(LeafBlock::IndentedCodeBlock { content: text.clone(), meta: None, attributes: None })
        }
        BlockNode::Table { header: _, rows: _ } => {
            // TODO: Map to Table block
            Block::Leaf(LeafBlock::Table { header: crate::logic::ast::gfm::TableRow { cells: vec![] }, alignments: vec![], rows: vec![], caption: None, attributes: None })
        }
        BlockNode::Alert { kind, children } => {
            let converted = children.iter().map(|c| blocknode_to_block(c)).collect();
            // Use CustomTagBlock for alerts for now
            Block::Leaf(LeafBlock::CustomTagBlock { name: kind.clone(), data: None, content: converted, attributes: None })
        }
        BlockNode::DescriptionList { items } => {
            let converted = items.iter().map(|item| blocknode_to_block(item)).collect();
            Block::Container(ContainerBlock::Document(converted, None))
        }
        BlockNode::DescriptionItem { term, details } => {
            let term_block = blocknode_to_block(term);
            let details_block = blocknode_to_block(details);
            Block::Container(ContainerBlock::Document(vec![term_block, details_block], None))
        }
        BlockNode::DescriptionTerm { children } => {
            let inlines: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|n| inline_node_to_inline_and_pos(n)).collect();
            Block::Leaf(LeafBlock::Paragraph(inlines, None))
        }
        BlockNode::DescriptionDetails { children } => {
            let converted = children.iter().map(|c| blocknode_to_block(c)).collect();
            Block::Container(ContainerBlock::Document(converted, None))
        }
    }
}

fn inline_node_to_inline_and_pos(node: &crate::logic::core::inline::types::InlineNode) -> (Inline, crate::logic::core::event_types::SourcePos) {
    use crate::logic::core::inline::types::InlineNode;
    match node {
        InlineNode::Text { text, pos } => (Inline::Text(text.clone()), pos.clone()),
        InlineNode::Emphasis { children, pos } => {
            let children_inlines: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|c| inline_node_to_inline_and_pos(c)).collect();
            (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Emph(children_inlines.clone(), None)), pos.clone())
        }
        InlineNode::Strong { children, pos } => {
            let children_inlines: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|c| inline_node_to_inline_and_pos(c)).collect();
            (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Strong(children_inlines.clone(), None)), pos.clone())
        }
        InlineNode::Code { text, pos } => (Inline::CodeSpan(crate::logic::ast::inlines::CodeSpan { content: text.clone(), meta: None, attributes: None }), pos.clone()),
        InlineNode::Link { href, title, children, pos } => {
            let label: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|c| inline_node_to_inline_and_pos(c)).collect();
            (Inline::Link(crate::logic::ast::inlines::Link {
                label,
                destination: crate::logic::ast::inlines::LinkDestination::Inline(href.clone()),
                title: Some(title.clone()),
                reference_type: None,
                attributes: None,
            }), pos.clone())
        }
        InlineNode::Image { src, alt, title, pos } => {
            let alt_vec: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = alt.iter().map(|c| inline_node_to_inline_and_pos(c)).collect();
            (Inline::Image(crate::logic::ast::inlines::Image {
                alt: alt_vec,
                destination: crate::logic::ast::inlines::LinkDestination::Inline(src.clone()),
                title: Some(title.clone()),
                attributes: None,
                alternative: None,
                resource: None,
            }), pos.clone())
        }
        InlineNode::Math { text, pos } => (Inline::Math(crate::logic::ast::math::MathInline {
            content: text.clone(),
            math_type: crate::logic::ast::math::MathType::LaTeX,
            position: Some(pos.clone()),
            attributes: None,
        }), pos.clone()),
        InlineNode::Html { text, pos } => (Inline::RawHtml(text.clone()), pos.clone()),
        InlineNode::Entity { text, pos } => (Inline::Text(text.clone()), pos.clone()),
        InlineNode::AttributeBlock { text, pos } => (Inline::Text(text.clone()), pos.clone()),
        InlineNode::SoftBreak { pos } => (Inline::SoftBreak, pos.clone()),
        InlineNode::LineBreak { pos } => (Inline::HardBreak, pos.clone()),
        InlineNode::Strikethrough { children, pos } => {
            let children_vec: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|c| inline_node_to_inline_and_pos(c)).collect();
            (Inline::Strikethrough(children_vec.clone(), None), pos.clone())
        }
        InlineNode::TaskListItem { checked: _, children, pos } => {
            let _children_vec: Vec<(Inline, crate::logic::core::event_types::SourcePos)> = children.iter().map(|c| inline_node_to_inline_and_pos(c)).collect();
            (Inline::TaskListMeta(None, None, pos.clone()), pos.clone()) // Map to meta for now
        }
    }
}
