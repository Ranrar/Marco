enum EventState<'a> {
    EnterBlock(&'a Block),
    ExitBlock(&'a Block),
    EnterInline(&'a str),
    ExitInline(&'a str),
}
// ...existing code...



// # Phraser: Event Stream Markdown Parser
//
// This module provides an idiomatic Rust event stream parser for Markdown ASTs.
// It exposes an iterator (`EventIter`) that yields `Event` values in depth-first order,
// inspired by pulldown-cmark. This is the recommended interface for rendering,
// transformation, or streaming processing of Markdown documents.
//
// Usage:
// ```rust
// use crate::editor::logic::phraser::{EventIter, Event, Tag, TagEnd};
// let ast = ...; // Build or parse your Block AST
// for event in EventIter::new(&ast) {
//     // Handle event
// }
// ```

use super::ast::blocks_and_inlines::{Block, ContainerBlock, LeafBlock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event<'a> {
    Start(Tag),
    End(TagEnd),
    Text(&'a str),
    Code(&'a str),
    Html(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Paragraph,
    Heading(u8),
    BlockQuote,
    List,
    Item,
    CodeBlock,
    HtmlBlock,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagEnd {
    Paragraph,
    Heading,
    BlockQuote,
    List,
    Item,
    CodeBlock,
    HtmlBlock,
}



/// Iterator over a Block AST that yields events in depth-first order.
pub struct EventIter<'a> {
    stack: Vec<&'a Block>,
    state: Vec<EventState<'a>>,
}

// EventState is defined below with EventIter, only keep one definition.

impl<'a> EventIter<'a> {
    pub fn new(root: &'a Block) -> Self {
        Self {
            stack: vec![root],
            state: vec![EventState::EnterBlock(root)],
        }
    }
}

impl<'a> Iterator for EventIter<'a> {
    type Item = Event<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state) = self.state.pop() {
            match state {
                EventState::EnterBlock(block) => {
                    match block {
                        Block::Container(ContainerBlock::Document(children)) => {
                            for child in children.iter().rev() {
                                self.state.push(EventState::EnterBlock(child));
                            }
                        }
                        Block::Container(ContainerBlock::BlockQuote(children)) => {
                            self.state.push(EventState::ExitBlock(block));
                            for child in children.iter().rev() {
                                self.state.push(EventState::EnterBlock(child));
                            }
                            return Some(Event::Start(Tag::BlockQuote));
                        }
                        Block::Container(ContainerBlock::List(_items)) => {
                            self.state.push(EventState::ExitBlock(block));
                            // List item traversal is not supported here due to lifetime issues with local allocations.
                            // To support this, refactor the AST or use an arena allocator for blocks.
                            return Some(Event::Start(Tag::List));
                        }
                        Block::Container(ContainerBlock::ListItem(blocks)) => {
                            self.state.push(EventState::ExitBlock(block));
                            for b in blocks.iter().rev() {
                                self.state.push(EventState::EnterBlock(b));
                            }
                            return Some(Event::Start(Tag::Item));
                        }
                        Block::Leaf(LeafBlock::Paragraph(_inlines)) => {
                            self.state.push(EventState::ExitBlock(block));
                            // Inline events are not emitted due to private Inline type
                            return Some(Event::Start(Tag::Paragraph));
                        }
                        Block::Leaf(LeafBlock::Heading { level, content: _ }) => {
                            self.state.push(EventState::ExitBlock(block));
                            // Inline events are not emitted due to private Inline type
                            return Some(Event::Start(Tag::Heading(*level)));
                        }
                        Block::Leaf(LeafBlock::CodeBlock(_code)) => {
                            self.state.push(EventState::ExitBlock(block));
                            return Some(Event::Start(Tag::CodeBlock));
                        }
                        Block::Leaf(LeafBlock::HtmlBlock(_html)) => {
                            self.state.push(EventState::ExitBlock(block));
                            return Some(Event::Start(Tag::HtmlBlock));
                        }
                        _ => {}
                    }
                }
                EventState::ExitBlock(block) => {
                    match block {
                        Block::Container(ContainerBlock::BlockQuote(_)) => return Some(Event::End(TagEnd::BlockQuote)),
                        Block::Container(ContainerBlock::List(_)) => return Some(Event::End(TagEnd::List)),
                        Block::Container(ContainerBlock::ListItem(_)) => return Some(Event::End(TagEnd::Item)),
                        Block::Leaf(LeafBlock::Paragraph(_)) => return Some(Event::End(TagEnd::Paragraph)),
                        Block::Leaf(LeafBlock::Heading { .. }) => return Some(Event::End(TagEnd::Heading)),
                        Block::Leaf(LeafBlock::CodeBlock(_)) => return Some(Event::End(TagEnd::CodeBlock)),
                        Block::Leaf(LeafBlock::HtmlBlock(_)) => return Some(Event::End(TagEnd::HtmlBlock)),
                        _ => {}
                    }
                }
                EventState::EnterInline(s) => {
                    return Some(Event::Text(s));
                }
                EventState::ExitInline(_s) => {}
            }
        }
        None
    }
}

/// Ergonomic top-level function for streaming events from a Block AST.
pub fn phraser<'a>(ast: &'a Block) -> EventIter<'a> {
    EventIter::new(ast)
}



#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ast::blocks_and_inlines::{Block, ContainerBlock, LeafBlock};

    #[test]
    fn test_event_iter() {
        let ast = Block::Container(ContainerBlock::Document(vec![
            Block::Leaf(LeafBlock::Paragraph(vec![])),
            Block::Leaf(LeafBlock::CodeBlock("let x = 1;".into())),
        ]));
        let events: Vec<_> = phraser(&ast).collect();
        assert_eq!(events[0], Event::Start(Tag::Paragraph));
        assert_eq!(events[1], Event::End(TagEnd::Paragraph));
        assert_eq!(events[2], Event::Start(Tag::CodeBlock));
        assert_eq!(events[3], Event::End(TagEnd::CodeBlock));
    }
}
