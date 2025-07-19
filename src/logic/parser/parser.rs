// Main parser: Token stream → Event stream or AST (supports streaming)
use super::event::Event;
use crate::logic::ast::blocks_and_inlines::Block;
use crate::logic::parser::transform::EventPipeline;

pub struct EventIter<'a> {
    stack: Vec<&'a Block>,
    state: Vec<Event>,
    pipeline: Option<EventPipeline>,
    diagnostics: Option<&'a mut super::diagnostics::Diagnostics>,
}

impl<'a> EventIter<'a> {
    pub fn new(root: &'a Block, diagnostics: Option<&'a mut super::diagnostics::Diagnostics>) -> Self {
        Self {
            stack: vec![root],
            state: vec![],
            pipeline: None,
            diagnostics,
        }
    }
    pub fn with_pipeline(root: &'a Block, pipeline: EventPipeline, diagnostics: Option<&'a mut super::diagnostics::Diagnostics>) -> Self {
        Self {
            stack: vec![root],
            state: vec![],
            pipeline: Some(pipeline),
            diagnostics,
        }
    }
}
impl<'a> Iterator for EventIter<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        // Example: emit unsupported event for demonstration
        if self.state.is_empty() {
            self.state.push(Event::Unsupported(
                "Footnotes not supported yet".to_string(),
                None,
            ));
        }
        while let Some(mut event) = self.state.pop() {
            // Collect diagnostics for error/warning/unsupported events
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
