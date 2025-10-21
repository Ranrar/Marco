//! Shared utilities for inline parsers
//!
//! This module provides helper functions used by all inline parser modules,
//! primarily for converting between grammar spans and parser spans.

use crate::parser::{Position, Span as ParserSpan};
use nom_locate::LocatedSpan;

/// Type alias for grammar-level spans (nom LocatedSpan)
pub type GrammarSpan<'a> = LocatedSpan<&'a str>;

/// Convert a grammar span to a parser span
///
/// Takes a LocatedSpan from the grammar layer and converts it to the parser's
/// Span type with proper Position tracking (line, column, offset).
pub fn to_parser_span(span: GrammarSpan) -> ParserSpan {
    ParserSpan {
        start: Position {
            line: span.location_line() as usize,
            column: span.get_utf8_column(),
            offset: span.location_offset(),
        },
        end: Position {
            line: span.location_line() as usize,
            column: span.get_utf8_column() + span.fragment().len(),
            offset: span.location_offset() + span.fragment().len(),
        },
    }
}

/// Convert a range of grammar spans (start, end) to a parser span
///
/// Used when you need to create a span from two different positions in the input.
/// The start span provides the beginning position, the end span provides the end position.
pub fn to_parser_span_range(start_span: GrammarSpan, end_span: GrammarSpan) -> ParserSpan {
    ParserSpan {
        start: Position {
            line: start_span.location_line() as usize,
            column: start_span.get_utf8_column(),
            offset: start_span.location_offset(),
        },
        end: Position {
            line: end_span.location_line() as usize,
            column: end_span.get_utf8_column(),
            offset: end_span.location_offset(),
        },
    }
}

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
