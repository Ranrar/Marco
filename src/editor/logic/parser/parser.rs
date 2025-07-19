// Main parser: Token stream → Event stream or AST (supports streaming)
use super::event::Event;
use crate::editor::logic::ast::blocks_and_inlines::Block;
use crate::editor::logic::transform::EventPipeline;

pub struct EventIter<'a> {
    stack: Vec<&'a Block>,
    state: Vec<Event>,
    pipeline: Option<EventPipeline>,
}

impl<'a> EventIter<'a> {
    pub fn new(root: &'a Block) -> Self {
        Self {
            stack: vec![root],
            state: vec![],
            pipeline: None,
        }
    }
    pub fn with_pipeline(root: &'a Block, pipeline: EventPipeline) -> Self {
        Self {
            stack: vec![root],
            state: vec![],
            pipeline: Some(pipeline),
        }
    }
}
impl<'a> Iterator for EventIter<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut event) = self.state.pop() {
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
