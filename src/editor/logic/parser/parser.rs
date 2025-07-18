// Main parser: Token stream → Event stream or AST (supports streaming)
use super::event::{Event, Tag, TagEnd};
use super::event::{SourcePos};
use super::emitter::push_inline_events;
use crate::editor::logic::ast::blocks_and_inlines::{Block, ContainerBlock, LeafBlock};

pub struct EventIter<'a> {
    stack: Vec<&'a Block>,
    state: Vec<Event>,
}

impl<'a> EventIter<'a> {
    pub fn new(root: &'a Block) -> Self {
        Self {
            stack: vec![root],
            state: vec![], // Initial state now empty; will be filled by push_inline_events
        }
    }
}

impl<'a> Iterator for EventIter<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state) = self.state.pop() {
            match state {
                Event::Start(_, _, _) => {
                    // ...existing code...
                }
                Event::End(_, _, _) => {
                    // ...existing code...
                }
                _ => {}
            }
        }
        None
    }
}
