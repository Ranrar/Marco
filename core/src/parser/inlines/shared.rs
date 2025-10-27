//! Shared utilities for inline parsers
//!
//! This module provides helper functions used by all inline parser modules,
//! primarily for converting between grammar spans and parser spans.

pub use crate::parser::shared::{to_parser_span, to_parser_span_range, GrammarSpan};

// Re-export the canonical helpers from `crate::parser::shared` so inline parsers
// can call `super::shared::to_parser_span(...)` as before.


#[cfg(test)]
mod tests {
    use super::*;
    use nom::Slice;

    #[test]
    fn smoke_test_to_parser_span() {
        let input = "Hello, World!";
        let span = GrammarSpan::new(input);
        
        let parser_span = to_parser_span(span);
        
        assert_eq!(parser_span.start.line, 1);
        assert_eq!(parser_span.start.column, 1);
        assert_eq!(parser_span.start.offset, 0);
        assert_eq!(parser_span.end.offset, 13); // Length of input
    }

    #[test]
    fn smoke_test_to_parser_span_range() {
        let input = "Hello, World!";
        let full_span = GrammarSpan::new(input);
        
        // Simulate taking a slice from offset 0 to 5 ("Hello")
        let start_span = full_span;
        let end_span = full_span.slice(5..);
        
        let parser_span = to_parser_span_range(start_span, end_span);
        
        assert_eq!(parser_span.start.line, 1);
        assert_eq!(parser_span.start.offset, 0);
        assert_eq!(parser_span.end.offset, 5);
    }
}
