//! # block_parser.rs
//!
//! This module provides the `EventEmitter` visitor and `EventIter` iterator for traversing a block-level AST and emitting a stream of events.
//!
//! **Role in pipeline:**
//! - Consumes an already-built block-level AST (from the Markdown parser).
//! - Walks the AST using the visitor pattern (`AstVisitor`).
//! - Emits events for each block type (paragraph, heading, code block, table, etc.) for rendering, export, or further processing.
//!
//! **Does NOT parse Markdown.**
//!
//! See `block/parser.rs` for Markdown parsing and AST construction.
// Main parser: Token stream → Event stream or AST (supports streaming)
// use crate::logic::core::event_types::Event; (already imported above)
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
mod tests {
    #[allow(unused_imports)]
    use super::{EventIter, EventEmitter};
    #[allow(unused_imports)]
    use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock, ContainerBlock};
    #[allow(unused_imports)]
    use crate::logic::ast::inlines::Inline;
    #[allow(unused_imports)]
    use crate::logic::ast::github::{TableRow, TableCell, TableAlignment};
    #[allow(unused_imports)]
    use crate::logic::core::event_types::{Event, Tag, TagEnd, SourcePos};
    #[allow(unused_imports)]
    use crate::logic::core::event_pipeline::EventPipeline;
    #[allow(unused_imports)]
    use crate::logic::core::diagnostics::Diagnostics;
    #[test]
    fn test_table_event_emission() {
        // Create a simple table AST
        let cell1 = TableCell { content: vec![(Inline::Text("A".into()), SourcePos { line: 1, column: 1 })] };
        let cell2 = TableCell { content: vec![(Inline::Text("B".into()), SourcePos { line: 1, column: 2 })] };
        let header = TableRow { cells: vec![cell1.clone(), cell2.clone()] };
        let row1 = TableRow { cells: vec![cell1.clone(), cell2.clone()] };
        let table = LeafBlock::Table {
            header,
            alignments: vec![TableAlignment::Left, TableAlignment::Center],
            rows: vec![row1],
            caption: Some("caption".into()),
            attributes: None,
        };
        let block = Block::Leaf(table);
        let doc = Block::Container(ContainerBlock::Document(vec![block], None));

        let mut emitter = EventEmitter::new();
        doc.accept(&mut emitter);
        let events = emitter.events;
        // Check for Table, TableRow, TableCell events
        assert!(events.iter().any(|e| matches!(e, Event::Start(Tag::Table(_), ..))));
        assert!(events.iter().any(|e| matches!(e, Event::Start(Tag::TableRow, ..))));
        assert!(events.iter().any(|e| matches!(e, Event::Start(Tag::TableCell, ..))));
        assert!(events.iter().any(|e| matches!(e, Event::End(TagEnd::TableRow, ..))));
        assert!(events.iter().any(|e| matches!(e, Event::End(TagEnd::TableCell, ..))));
    }

    #[allow(dead_code)]
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

use crate::logic::core::event_types::{Event, Tag, TagEnd};
use crate::logic::ast::blocks_and_inlines::{AstVisitor, ContainerBlock, LeafBlock};

/// Visitor that emits events during AST traversal.
pub struct EventEmitter {
    pub events: Vec<Event>,
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
                        self.events.push(Event::Start(Tag::Paragraph(attrs.clone()), None, attrs.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::Heading { level, attributes, .. } => {
                        self.events.push(Event::Start(Tag::Heading(*level, attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::AtxHeading { level, attributes, .. } => {
                        self.events.push(Event::Start(Tag::Heading(*level, attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::SetextHeading { level, attributes, .. } => {
                        self.events.push(Event::Start(Tag::Heading(*level, attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::IndentedCodeBlock { attributes, .. } => {
                        self.events.push(Event::Start(Tag::CodeBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::FencedCodeBlock { attributes, .. } => {
                        self.events.push(Event::Start(Tag::CodeBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::ThematicBreak { attributes, .. } => {
                        self.events.push(Event::Start(Tag::HtmlBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::HtmlBlock { attributes, .. } => {
                        self.events.push(Event::Start(Tag::HtmlBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::Table { header, rows, attributes, .. } => {
                        self.events.push(Event::Start(Tag::Table(attributes.clone()), None, attributes.clone()));
                        // Emit header row
                        self.events.push(Event::Start(Tag::TableRow, None, attributes.clone()));
                        for cell in &header.cells {
                            self.events.push(Event::Start(Tag::TableCell, None, attributes.clone()));
                            for (inline, pos) in &cell.content {
                                self.events.push(Event::Inline(inline.clone(), Some(pos.clone()), attributes.clone()));
                            }
                            self.events.push(Event::End(TagEnd::TableCell, None, attributes.clone()));
                        }
                        self.events.push(Event::End(TagEnd::TableRow, None, attributes.clone()));
                        // Emit data rows
                        for row in rows {
                            self.events.push(Event::Start(Tag::TableRow, None, attributes.clone()));
                            for cell in &row.cells {
                                self.events.push(Event::Start(Tag::TableCell, None, attributes.clone()));
                                for (inline, pos) in &cell.content {
                                    self.events.push(Event::Inline(inline.clone(), Some(pos.clone()), attributes.clone()));
                                }
                                self.events.push(Event::End(TagEnd::TableCell, None, attributes.clone()));
                            }
                            self.events.push(Event::End(TagEnd::TableRow, None, attributes.clone()));
                        }
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::LinkReferenceDefinition { attributes, .. } => {
                        self.events.push(Event::Start(Tag::HtmlBlock(attributes.clone()), None, attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::BlankLine => {
                        // No event for blank line
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::Math(math_block) => {
                        self.events.push(Event::Start(Tag::MathBlock(math_block.content.clone(), Some(math_block.math_type.clone()), math_block.attributes.clone()), None, math_block.attributes.clone()));
                    }
                    crate::logic::ast::blocks_and_inlines::LeafBlock::CustomTagBlock { name, data, attributes, .. } => {
                        self.events.push(Event::Start(Tag::CustomTag {
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
                self.events.push(Event::Start(Tag::BlockQuote(attrs.clone()), None, attrs.clone()));
            }
            crate::logic::ast::blocks_and_inlines::ContainerBlock::BlockQuote(_, attrs) => {
                self.events.push(Event::Start(Tag::BlockQuote(attrs.clone()), None, attrs.clone()));
            }
            crate::logic::ast::blocks_and_inlines::ContainerBlock::ListItem { attributes, .. } => {
                self.events.push(Event::Start(Tag::Item(attributes.clone()), None, attributes.clone()));
            }
            crate::logic::ast::blocks_and_inlines::ContainerBlock::List { attributes, .. } => {
                self.events.push(Event::Start(Tag::List(attributes.clone()), None, attributes.clone()));
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
