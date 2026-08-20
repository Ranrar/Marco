// Canonical span conversion helpers for the parser layer
// Centralized to ensure blocks and inlines use the same logic.

use crate::parser::position::{Position, Span as ParserSpan};
use nom_locate::LocatedSpan;
use std::cell::Cell;

// ---------------------------------------------------------------------------
// Per-parse runtime option flags (thread-local for zero call-site overhead)
// ---------------------------------------------------------------------------

thread_local! {
    static TRACK_POSITIONS: Cell<bool> = const { Cell::new(true) };
    static PARSE_MATH: Cell<bool>      = const { Cell::new(true) };
    static PARSE_DIAGRAMS: Cell<bool>  = const { Cell::new(true) };
}

/// RAII guard that sets per-thread parse options and restores them on drop.
///
/// Created by [`parse_with_options`] before calling the parse pipeline; dropped
/// (and thus restored) when the parse call returns or panics.
pub(crate) struct ParseOptionsGuard {
    prev_track: bool,
    prev_math: bool,
    prev_diagrams: bool,
}

impl ParseOptionsGuard {
    pub(crate) fn new(track: bool, math: bool, diagrams: bool) -> Self {
        let prev_track = TRACK_POSITIONS.with(|c| c.replace(track));
        let prev_math = PARSE_MATH.with(|c| c.replace(math));
        let prev_diagrams = PARSE_DIAGRAMS.with(|c| c.replace(diagrams));
        Self {
            prev_track,
            prev_math,
            prev_diagrams,
        }
    }
}

impl Drop for ParseOptionsGuard {
    fn drop(&mut self) {
        TRACK_POSITIONS.with(|c| c.set(self.prev_track));
        PARSE_MATH.with(|c| c.set(self.prev_math));
        PARSE_DIAGRAMS.with(|c| c.set(self.prev_diagrams));
    }
}

/// Returns whether math parsing is enabled in the current parse context.
#[inline]
pub(crate) fn parse_math_enabled() -> bool {
    PARSE_MATH.with(|c| c.get())
}

/// Returns whether diagram parsing is enabled in the current parse context.
#[inline]
pub(crate) fn parse_diagrams_enabled() -> bool {
    PARSE_DIAGRAMS.with(|c| c.get())
}

// ---------------------------------------------------------------------------
// opt_span helpers — use these in parser code instead of `Some(to_parser_span(...))`
// ---------------------------------------------------------------------------

/// Returns `None` (skipping O(n) string scans) when position tracking is disabled,
/// or `Some(span)` with real line/column data when enabled.
///
/// Replace `span: Some(to_parser_span(x))` with `span: opt_span(x)`.
#[inline]
pub fn opt_span(span: GrammarSpan) -> Option<ParserSpan> {
    if !TRACK_POSITIONS.with(|c| c.get()) {
        return None;
    }
    Some(to_parser_span(span))
}

/// Like [`opt_span`] but takes a start/end range using exclusive end semantics.
///
/// Replace `span: Some(to_parser_span_range(start, end))` with
/// `span: opt_span_range(start, end)`.
#[inline]
pub fn opt_span_range(start: GrammarSpan, end: GrammarSpan) -> Option<ParserSpan> {
    if !TRACK_POSITIONS.with(|c| c.get()) {
        return None;
    }
    Some(to_parser_span_range(start, end))
}

/// Like [`opt_span`] but takes a start/end range using inclusive end semantics.
///
/// Replace `span: Some(to_parser_span_range_inclusive(start, end))` with
/// `span: opt_span_range_inclusive(start, end)`.
#[inline]
pub fn opt_span_range_inclusive(start: GrammarSpan, end: GrammarSpan) -> Option<ParserSpan> {
    if !TRACK_POSITIONS.with(|c| c.get()) {
        return None;
    }
    Some(to_parser_span_range_inclusive(start, end))
}

/// Grammar span type (nom_locate::LocatedSpan)
pub type GrammarSpan<'a> = LocatedSpan<&'a str>;

/// Convert grammar span (LocatedSpan) to parser span (line/column)
///
/// This handles multi-line fragments by computing end line/column
/// based on newline count and last-line length. Columns are byte-based
/// 1-based offsets to match `Position` semantics.
pub fn to_parser_span(span: GrammarSpan) -> ParserSpan {
    let start_line = span.location_line() as usize; // 1-based
    let frag = span.fragment().as_bytes();

    // Single O(n) pass: count newlines and record the last newline byte position.
    let mut newline_count = 0usize;
    let mut last_nl: Option<usize> = None;
    for (i, &b) in frag.iter().enumerate() {
        if b == b'\n' {
            newline_count += 1;
            last_nl = Some(i);
        }
    }
    let end_line = start_line + newline_count;

    let end_column = match last_nl {
        Some(pos) if pos == frag.len() - 1 => {
            // Fragment ends with '\n' — end column is column 1 of the next line.
            1
        }
        Some(pos) => {
            // Multi-line: bytes after last newline + 1 (1-based).
            frag.len() - pos - 1 + 1
        }
        None => {
            // Single-line: start column (byte-based) + fragment byte length.
            span.get_column() + frag.len()
        }
    };

    let start = Position::new(start_line, span.get_column(), span.location_offset());
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
