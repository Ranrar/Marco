// Canonical span conversion helpers for the parser layer
// Centralized to ensure blocks and inlines use the same logic.

use crate::parser::position::{Position, Span as ParserSpan};
use nom_locate::LocatedSpan;

/// Grammar span type (nom_locate::LocatedSpan)
pub type GrammarSpan<'a> = LocatedSpan<&'a str>;

/// Convert grammar span (LocatedSpan) to parser span (line/column)
///
/// This handles multi-line fragments by computing end line/column
/// based on newline count and last-line length. Columns are byte-based
/// 1-based offsets to match `Position` semantics.
pub fn to_parser_span(span: GrammarSpan) -> ParserSpan {
    let start_line = span.location_line() as usize; // 1-based
    let newline_count = span.fragment().matches('\n').count();
    let end_line = start_line + newline_count;

    let end_column = if span.fragment().ends_with('\n') {
        // If the fragment ends with newline, end column is column 1 of next line
        1
    } else if let Some(last_newline_pos) = span.fragment().rfind('\n') {
        // Multi-line: bytes after last newline + 1 for 1-based
        span.fragment()[last_newline_pos + 1..].len() + 1
    } else {
        // Single-line: start column (byte-based) + fragment byte length
        span.get_column() + span.fragment().len()
    };

    let start = Position::new(
        start_line,
        span.get_column(),
        span.location_offset(),
    );
    let end = Position::new(
        end_line,
        end_column,
        span.location_offset() + span.fragment().len(),
    );
    ParserSpan::new(start, end)
}

/// Convert grammar span range (start, end) to parser span
/// Convert a grammar span range where `end` is the remainder span
/// (i.e. the nom `rest` after a match). This sets the end position to the
/// `end.location_offset()` (start of the remainder), matching inline parser
/// usage patterns like `to_parser_span_range(start, rest)`.
pub fn to_parser_span_range(start: GrammarSpan, end: GrammarSpan) -> ParserSpan {
    let start_pos = Position::new(
        start.location_line() as usize,
        start.get_column(),
        start.location_offset(),
    );
    let end_pos = Position::new(
        end.location_line() as usize,
        end.get_column(),
        end.location_offset(),
    );
    ParserSpan::new(start_pos, end_pos)
}

/// Convert a grammar span range where `end` is the final fragment of the
/// matched range (i.e. inclusive). This preserves the previous block-level
/// semantics where callers pass the last fragment and expect the end to be
/// at `end.location_offset() + end.fragment().len()`.
pub fn to_parser_span_range_inclusive(start: GrammarSpan, end: GrammarSpan) -> ParserSpan {
    let start_pos = Position::new(
        start.location_line() as usize,
        start.get_column(),
        start.location_offset(),
    );
    let end_pos = Position::new(
        end.location_line() as usize,
        end.get_column() + end.fragment().len(),
        end.location_offset() + end.fragment().len(),
    );
    ParserSpan::new(start_pos, end_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_parser_span_ascii() {
        let input = GrammarSpan::new("hello");
        let span = to_parser_span(input);
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        assert_eq!(span.end.column, 6); // 5 bytes + 1-based
    }

    #[test]
    fn test_to_parser_span_utf8_and_emoji() {
        let input = GrammarSpan::new("Tëst");
        let span = to_parser_span(input);
        assert_eq!(span.start.column, 1);
        // 'Tëst' is 5 bytes; end.column should be 6
        assert_eq!(span.end.column, 6);

        let input2 = GrammarSpan::new("🎨");
        let span2 = to_parser_span(input2);
        assert_eq!(span2.start.column, 1);
        // emoji 4 bytes -> end column 5
        assert_eq!(span2.end.column, 5);
    }
}
