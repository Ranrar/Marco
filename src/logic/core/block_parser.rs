use crate::logic::ast::blocks_and_inlines::{AstVisitor, ContainerBlock, LeafBlock};

/// Visitor that emits events during AST traversal.
pub struct EventEmitter {
    pub events: Vec<super::event_types::Event>,
}

impl EventEmitter {
    pub fn new() -> Self {
        EventEmitter { events: Vec::new() }
    }
}

impl AstVisitor for EventEmitter {
    fn visit_block(&mut self, block: &Block) {
        // Example: emit a generic event for each block
        // (Replace with real event emission logic)
        // Emit real events for Block node
        match block {
            crate::logic::ast::blocks_and_inlines::Block::Container(_) => {
                // No direct event for generic container, handled in visit_container_block
            }
            crate::logic::ast::blocks_and_inlines::Block::Leaf(leaf) => {
                match leaf {
                    crate::logic::ast::blocks_and_inlines::LeafBlock::Paragraph(_, attrs) => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::Paragraph(attrs.clone()), None, attrs.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::Heading { level, attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::Heading(*level, attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::AtxHeading { level, attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::Heading(*level, attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::SetextHeading { level, attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::Heading(*level, attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::IndentedCodeBlock { attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::CodeBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::FencedCodeBlock { attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::CodeBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::ThematicBreak { attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::HtmlBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::HtmlBlock { attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::HtmlBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::Table { header, rows, attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::Table(attributes.clone()), None, attributes.clone()));
                        // Emit header row
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::TableRow, None, attributes.clone()));
                        for cell in &header.cells {
                            self.events.push(super::event_types::Event::Start(super::event_types::Tag::TableCell, None, attributes.clone()));
                            for (inline, pos) in &cell.content {
                                self.events.push(super::event_types::Event::Inline(inline.clone(), Some(pos.clone()), attributes.clone()));
                            }
                            self.events.push(super::event_types::Event::End(super::event_types::TagEnd::TableCell, None, attributes.clone()));
                        }
                        self.events.push(super::event_types::Event::End(super::event_types::TagEnd::TableRow, None, attributes.clone()));
                        // Emit data rows
                        for row in rows {
                            self.events.push(super::event_types::Event::Start(super::event_types::Tag::TableRow, None, attributes.clone()));
                            for cell in &row.cells {
                                self.events.push(super::event_types::Event::Start(super::event_types::Tag::TableCell, None, attributes.clone()));
                                for (inline, pos) in &cell.content {
                                    self.events.push(super::event_types::Event::Inline(inline.clone(), Some(pos.clone()), attributes.clone()));
                                }
                                self.events.push(super::event_types::Event::End(super::event_types::TagEnd::TableCell, None, attributes.clone()));
                            }
                            self.events.push(super::event_types::Event::End(super::event_types::TagEnd::TableRow, None, attributes.clone()));
                        }
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::LinkReferenceDefinition { attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::HtmlBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::BlankLine => {
                        // No event for blank line
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::Math(math_block) => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::MathBlock(math_block.content.clone(), Some(math_block.math_type.clone()), math_block.attributes.clone()), None, math_block.attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::CustomTagBlock { name, data, attributes, .. } => {
                        self.events.push(super::event_types::Event::Start(super::event_types::Tag::CustomTag {
                            name: name.clone(),
                            data: data.clone(),
                            attributes: attributes.clone(),
                        }, None, attributes.clone()));
                    }
                }
            }
        }
        self.walk_block(block);
    }
    fn visit_container_block(&mut self, container: &ContainerBlock) {
        // Emit real events for ContainerBlock node
        match container {
            crate::logic::ast::blocks_and_inlines::ContainerBlock::Document(_, attrs) => {
                self.events.push(super::event_types::Event::Start(super::event_types::Tag::BlockQuote(attrs.clone()), None, attrs.clone()));
            }
            crate::logic::ast::blocks_and_inlines::ContainerBlock::BlockQuote(_, attrs) => {
                self.events.push(super::event_types::Event::Start(super::event_types::Tag::BlockQuote(attrs.clone()), None, attrs.clone()));
            }
            crate::logic::ast::blocks_and_inlines::ContainerBlock::ListItem { attributes, .. } => {
                self.events.push(super::event_types::Event::Start(super::event_types::Tag::Item(attributes.clone()), None, attributes.clone()));
            }
            crate::logic::ast::blocks_and_inlines::ContainerBlock::List { attributes, .. } => {
                self.events.push(super::event_types::Event::Start(super::event_types::Tag::List(attributes.clone()), None, attributes.clone()));
            }
        }
        self.walk_container_block(container);
    }
    fn visit_leaf_block(&mut self, leaf: &LeafBlock) {
        // Emit real events for LeafBlock node
        // No generic event for LeafBlock, handled in visit_block above
        self.walk_leaf_block(leaf);
    }
    // Add more visit methods for inlines, table rows, etc. as needed
}
#[cfg(test)]
mod tests {
    use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock};
    use crate::logic::core::event_types::Event;
    use crate::logic::core::event_pipeline::EventPipeline;
    use crate::logic::core::diagnostics::Diagnostics;
    use super::*;

    fn dummy_block() -> Block {
        Block::Leaf(LeafBlock::Paragraph(vec![], None))
    }

    #[test]
    fn event_iter_basic() {
        let block = dummy_block();
        let mut iter = EventIter::new(&block, None);
        let mut found = false;
        while let Some(_event) = iter.next() {
            found = true;
            break; // Only need to confirm at least one event is emitted
        }
        assert!(found, "Should emit at least one event");
    }

    #[test]
    fn event_iter_collects_diagnostics() {
        let block = dummy_block();
        let mut diagnostics = Diagnostics::new();
        let mut iter = EventIter::new(&block, Some(&mut diagnostics));
        while let Some(_event) = iter.next() {
            // Diagnostics are collected inside iterator
        }
        assert!(diagnostics.unsupported.is_empty(), "Should not collect unsupported events for dummy block");
    }

    #[test]
    fn event_iter_with_pipeline() {
        let block = dummy_block();
        let mut pipeline = EventPipeline::new();
        pipeline.add_filter(|event: &mut Event| !matches!(event, Event::Unsupported(_, _)));
        let mut iter = EventIter::with_pipeline(&block, pipeline, None);
        let mut found_unsupported = false;
        while let Some(event) = iter.next() {
            if matches!(event, Event::Unsupported(_, _)) {
                found_unsupported = true;
            }
        }
        assert!(!found_unsupported, "Pipeline should filter unsupported events");
    }
}
// Main parser: Token stream → Event stream or AST (supports streaming)
use super::event_types::Event;
use crate::logic::ast::blocks_and_inlines::Block;
use crate::logic::core::event_pipeline::EventPipeline;

pub struct EventIter<'a> {
    emitter: EventEmitter,
    index: usize,
    pipeline: Option<EventPipeline>,
    diagnostics: Option<&'a mut super::diagnostics::Diagnostics>,
}

impl<'a> EventIter<'a> {
    pub fn new(root: &'a Block, diagnostics: Option<&'a mut super::diagnostics::Diagnostics>) -> Self {
        let mut emitter = EventEmitter::new();
        root.accept(&mut emitter);
        Self {
            emitter,
            index: 0,
            pipeline: None,
            diagnostics,
        }
    }
    pub fn with_pipeline(root: &'a Block, pipeline: EventPipeline, diagnostics: Option<&'a mut super::diagnostics::Diagnostics>) -> Self {
        let mut emitter = EventEmitter::new();
        root.accept(&mut emitter);
        Self {
            emitter,
            index: 0,
            pipeline: Some(pipeline),
            diagnostics,
        }
    }
}
impl<'a> Iterator for EventIter<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.emitter.events.len() {
            let mut event = self.emitter.events[self.index].clone();
            self.index += 1;
            if let Some(diag) = &mut self.diagnostics {
                diag.collect(&event);
            }
            if let Some(pipeline) = &mut self.pipeline {
                if pipeline.process(&mut event) {
                    return Some(event);
                } else {
                    continue;
                }
            } else {
                return Some(event);
            }
        }
        None
    }
}
