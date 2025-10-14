//! Source position tracking for AST nodes
//!
//! Provides the Span type for tracking source locations in the original markdown.

use serde::{Deserialize, Serialize};

/// Source position information for any node in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    /// Starting byte position in source
    pub start: u32,
    /// Ending byte position in source
    pub end: u32,
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
}

impl Span {
    /// Create a new span with the given positions
    pub fn new(start: u32, end: u32, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Create a span covering both spans
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: self.column,
        }
    }

    /// Create a span from pest Pair
    pub fn from_pest(
        pair: &pest::iterators::Pair<crate::components::marco_engine::grammar::Rule>,
    ) -> Self {
        let span = pair.as_span();
        let (line, column) = span.start_pos().line_col();
        Span {
            start: span.start() as u32,
            end: span.end() as u32,
            line: line as u32,
            column: column as u32,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
    }
}
