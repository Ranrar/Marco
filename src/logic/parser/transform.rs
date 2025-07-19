#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::parser::event::{Event, Tag};

    #[test]
    fn test_event_filtering_pipeline() {
        let mut pipeline = EventPipeline::new();
        // Example: filter out all SoftBreak events
        pipeline.add_filter(|event: &mut Event| !matches!(event, Event::SoftBreak(_, _)));
        // Example: transform all Text events to uppercase
        pipeline.add_mapper(|event: &mut Event| {
            if let Event::Text(ref mut s, _, _) = event {
                *s = s.to_uppercase();
            }
        });

        let mut events = vec![
            Event::Text("hello".into(), None, None),
            Event::SoftBreak(None, None),
            Event::Text("world".into(), None, None),
        ];
        let mut output = Vec::new();
        for mut event in events {
            if pipeline.process(&mut event) {
                output.push(event);
            }
        }
        assert_eq!(output.len(), 2);
        if let Event::Text(ref s, _, _) = output[0] {
            assert_eq!(s, "HELLO");
        }
        if let Event::Text(ref s, _, _) = output[1] {
            assert_eq!(s, "WORLD");
        }
    }
}
use crate::logic::event::Event;

/// Trait for filtering events. Return true to keep, false to discard.
pub trait EventFilter {
    fn filter(&mut self, event: &mut Event) -> bool;
}

/// Trait for mapping (transforming) events in-place.
pub trait EventMapper {
    fn map(&mut self, event: &mut Event);
}

// Blanket impl for closures for EventFilter
impl<F> EventFilter for F
where
    F: FnMut(&mut Event) -> bool,
{
    fn filter(&mut self, event: &mut Event) -> bool {
        self(event)
    }
}

// Blanket impl for closures for EventMapper
impl<F> EventMapper for F
where
    F: FnMut(&mut Event),
{
    fn map(&mut self, event: &mut Event) {
        self(event)
    }
}

/// Pipeline for chaining filters and mappers.
pub struct EventPipeline {
    pub filters: Vec<Box<dyn EventFilter>>,
    pub mappers: Vec<Box<dyn EventMapper>>,
}

impl EventPipeline {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            mappers: Vec::new(),
        }
    }

    /// Add a filter to the pipeline.
    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: EventFilter + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    /// Add a mapper to the pipeline.
    pub fn add_mapper<M>(&mut self, mapper: M)
    where
        M: EventMapper + 'static,
    {
        self.mappers.push(Box::new(mapper));
    }

    /// Process an event through all mappers and filters.
    pub fn process(&mut self, event: &mut Event) -> bool {
        for mapper in &mut self.mappers {
            mapper.map(event);
        }
        for filter in &mut self.filters {
            if !filter.filter(event) {
                return false;
            }
        }
        true
    }
}
