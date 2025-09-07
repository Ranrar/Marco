//! Position tracking utilities for Marco parser
//!
//! This module provides utilities for tracking positions, spans, and source locations
//! throughout the parsing process, enabling accurate error reporting and source mapping.

use pest::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a position in the source text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Byte offset from the start of the input
    pub offset: usize,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl Position {
    /// Create a new position
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    /// Create position at the start of input
    pub fn start() -> Self {
        Self::new(0, 1, 1)
    }

    /// Advance position by one character
    pub fn advance(mut self, ch: char) -> Self {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self
    }

    /// Advance position by a string
    pub fn advance_str(mut self, s: &str) -> Self {
        for ch in s.chars() {
            self = self.advance(ch);
        }
        self
    }

    /// Check if this position is before another
    pub fn is_before(&self, other: &Position) -> bool {
        self.offset < other.offset
    }

    /// Check if this position is after another
    pub fn is_after(&self, other: &Position) -> bool {
        self.offset > other.offset
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Represents a span of text in the source
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Starting position
    pub start: Position,
    /// Ending position
    pub end: Position,
    /// The actual text content (optional for memory efficiency)
    pub text: Option<String>,
}

impl SourceSpan {
    /// Create a new span
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            text: None,
        }
    }

    /// Create a span with text content
    pub fn with_text(start: Position, end: Position, text: String) -> Self {
        Self {
            start,
            end,
            text: Some(text),
        }
    }

    /// Create a span from a Pest span
    pub fn from_pest_span(span: Span) -> Self {
        let start_pos = span.start_pos().line_col();
        let end_pos = span.end_pos().line_col();

        let start = Position::new(span.start(), start_pos.0, start_pos.1);
        let end = Position::new(span.end(), end_pos.0, end_pos.1);

        Self::with_text(start, end, span.as_str().to_string())
    }

    /// Get the length of the span in bytes
    pub fn len(&self) -> usize {
        self.end.offset - self.start.offset
    }

    /// Check if the span is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this span contains a position
    pub fn contains(&self, pos: &Position) -> bool {
        pos.offset >= self.start.offset && pos.offset <= self.end.offset
    }

    /// Check if this span overlaps with another
    pub fn overlaps(&self, other: &SourceSpan) -> bool {
        !(self.end.offset <= other.start.offset || other.end.offset <= self.start.offset)
    }

    /// Merge this span with another (returns the union)
    pub fn merge(&self, other: &SourceSpan) -> SourceSpan {
        let start = if self.start.is_before(&other.start) {
            self.start
        } else {
            other.start
        };

        let end = if self.end.is_after(&other.end) {
            self.end
        } else {
            other.end
        };

        SourceSpan::new(start, end)
    }

    /// Get the text content if available
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Extract the actual text from source if span doesn't contain it
    pub fn extract_text<'a>(&'a self, source: &'a str) -> Option<&'a str> {
        if let Some(text) = &self.text {
            Some(text)
        } else {
            let bytes = source.as_bytes();
            if self.end.offset <= bytes.len() {
                std::str::from_utf8(&bytes[self.start.offset..self.end.offset]).ok()
            } else {
                None
            }
        }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(text) = &self.text {
            write!(f, "{}-{}: '{}'", self.start, self.end, text)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

/// Tracks positions while parsing
#[derive(Debug, Clone)]
pub struct PositionTracker {
    source: String,
    positions: Vec<Position>,
}

impl PositionTracker {
    /// Create a new position tracker for source text
    pub fn new(source: String) -> Self {
        let mut positions = Vec::new();
        let mut pos = Position::start();
        positions.push(pos);

        for ch in source.chars() {
            pos = pos.advance(ch);
            positions.push(pos);
        }

        Self { source, positions }
    }

    /// Get position at byte offset
    pub fn position_at(&self, offset: usize) -> Option<Position> {
        self.positions.get(offset).copied()
    }

    /// Create a span from byte offsets
    pub fn span_from_offsets(&self, start: usize, end: usize) -> Option<SourceSpan> {
        let start_pos = self.position_at(start)?;
        let end_pos = self.position_at(end)?;

        let text = if end <= self.source.len() {
            Some(self.source[start..end].to_string())
        } else {
            None
        };

        Some(SourceSpan {
            start: start_pos,
            end: end_pos,
            text,
        })
    }

    /// Get the source text
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get line content for a given line number
    pub fn line_content(&self, line: usize) -> Option<&str> {
        self.source.lines().nth(line.saturating_sub(1))
    }

    /// Get context around a position (lines before and after)
    pub fn context_around(&self, pos: &Position, context_lines: usize) -> Vec<(usize, &str)> {
        let start_line = pos.line.saturating_sub(context_lines);
        let end_line = pos.line + context_lines;

        self.source
            .lines()
            .enumerate()
            .skip(start_line.saturating_sub(1))
            .take(end_line - start_line + 1)
            .map(|(i, line)| (i + 1, line))
            .collect()
    }
}

/// Utility for creating spans from pest pairs
pub trait SpanExt {
    /// Convert to a SourceSpan
    fn to_source_span(&self) -> SourceSpan;

    /// Get start position
    fn start_position(&self) -> Position;

    /// Get end position  
    fn end_position(&self) -> Position;
}

impl SpanExt for Span<'_> {
    fn to_source_span(&self) -> SourceSpan {
        SourceSpan::from_pest_span(*self)
    }

    fn start_position(&self) -> Position {
        let pos = self.start_pos().line_col();
        Position::new(self.start(), pos.0, pos.1)
    }

    fn end_position(&self) -> Position {
        let pos = self.end_pos().line_col();
        Position::new(self.end(), pos.0, pos.1)
    }
}

/// Helper for error reporting with position information
#[derive(Debug, Clone)]
pub struct PositionedError {
    pub message: String,
    pub span: SourceSpan,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl PositionedError {
    pub fn error(message: String, span: SourceSpan) -> Self {
        Self {
            message,
            span,
            severity: ErrorSeverity::Error,
        }
    }

    pub fn warning(message: String, span: SourceSpan) -> Self {
        Self {
            message,
            span,
            severity: ErrorSeverity::Warning,
        }
    }

    pub fn info(message: String, span: SourceSpan) -> Self {
        Self {
            message,
            span,
            severity: ErrorSeverity::Info,
        }
    }

    /// Format error message with context
    pub fn format_with_context(&self, tracker: &PositionTracker) -> String {
        let mut output = String::new();

        // Error header
        let severity_str = match self.severity {
            ErrorSeverity::Error => "error",
            ErrorSeverity::Warning => "warning",
            ErrorSeverity::Info => "info",
        };

        output.push_str(&format!(
            "{}: {}\n  --> {}:{}\n",
            severity_str, self.message, self.span.start.line, self.span.start.column
        ));

        // Show context lines
        let context = tracker.context_around(&self.span.start, 2);
        for (line_num, line_content) in context {
            output.push_str(&format!("{:4} | {}\n", line_num, line_content));

            // Add caret pointing to the error position
            if line_num == self.span.start.line {
                output.push_str("     | ");
                output.push_str(&" ".repeat(self.span.start.column.saturating_sub(1)));
                output.push_str(&"^".repeat(self.span.len().max(1)));
                output.push('\n');
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_advance() {
        let mut pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);

        pos = pos.advance('a');
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);
        assert_eq!(pos.offset, 1);

        pos = pos.advance('\n');
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 2);
    }

    #[test]
    fn test_position_advance_str() {
        let pos = Position::start();
        let new_pos = pos.advance_str("hello\nworld");

        assert_eq!(new_pos.line, 2);
        assert_eq!(new_pos.column, 6);
        assert_eq!(new_pos.offset, 11);
    }

    #[test]
    fn test_span_operations() {
        let start = Position::new(0, 1, 1);
        let end = Position::new(5, 1, 6);
        let span = SourceSpan::with_text(start, end, "hello".to_string());

        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
        assert_eq!(span.text(), Some("hello"));

        let pos_inside = Position::new(2, 1, 3);
        let pos_outside = Position::new(10, 1, 11);

        assert!(span.contains(&pos_inside));
        assert!(!span.contains(&pos_outside));
    }

    #[test]
    fn test_position_tracker() {
        let source = "hello\nworld\n!".to_string();
        let tracker = PositionTracker::new(source);

        let pos_0 = tracker.position_at(0).unwrap();
        assert_eq!(pos_0.line, 1);
        assert_eq!(pos_0.column, 1);

        let pos_6 = tracker.position_at(6).unwrap(); // 'w' in "world"
        assert_eq!(pos_6.line, 2);
        assert_eq!(pos_6.column, 1);

        let span = tracker.span_from_offsets(0, 5).unwrap();
        assert_eq!(span.text(), Some("hello"));
        assert_eq!(span.start.line, 1);
        assert_eq!(span.end.line, 1);
    }

    #[test]
    fn test_span_merge() {
        let span1 = SourceSpan::new(Position::new(0, 1, 1), Position::new(5, 1, 6));
        let span2 = SourceSpan::new(Position::new(3, 1, 4), Position::new(8, 1, 9));

        let merged = span1.merge(&span2);
        assert_eq!(merged.start.offset, 0);
        assert_eq!(merged.end.offset, 8);
    }
}
