// Position tracking for LSP integration (line/column mapping)

use serde::{Deserialize, Serialize};

/// Position in a source document using multiple coordinate systems.
///
/// This struct tracks a position using three different representations:
/// - **Line/Column**: CommonMark-style 1-based coordinates
/// - **Absolute Offset**: Byte offset from document start
///
/// # Coordinate Systems
///
/// ## Line/Column (Primary for GTK Integration)
/// - `line`: 1-based line number (CommonMark convention)
/// - `column`: 1-based byte offset from the start of the line
///
/// **Important**: `column` is a BYTE offset, not a character offset!
/// - For ASCII: byte offset == character offset
/// - For UTF-8: Multi-byte characters cause divergence
///   - Example: "Tëst" has 'ë' at byte columns 3-4, but char column 2
///   - Example: "🎨" (emoji) occupies 4 bytes but is 1 character
///
/// ## Absolute Offset (For Debugging Only)
/// - `offset`: Absolute byte offset from document start
/// - **Do NOT use** for GTK TextIter positioning!
/// - Use `line` and `column` instead for robust conversion
///
/// # Usage with GTK
///
/// When converting to GTK TextIter:
/// 1. Convert line: `parser_line (1-based)` → `gtk_line (0-based)`
/// 2. Get line text from GTK buffer
/// 3. Convert column: `byte_offset → char_offset` using `char_indices()`
/// 4. Set position: `iter_at_line(gtk_line).set_line_offset(char_offset)`
///
/// See `marco/src/components/editor/lsp_integration.rs::position_to_iter()`
/// for the reference implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-based, CommonMark convention)
    pub line: usize,
    
    /// Column as byte offset from line start (1-based, CommonMark convention)
    /// 
    /// **Note**: This is NOT a character offset! 
    /// Multi-byte UTF-8 characters cause byte offsets to differ from character positions.
    pub column: usize,
    
    /// Absolute byte offset from document start
    /// 
    /// **For debugging/logging only** - do not use for GTK positioning!
    pub offset: usize,
}

/// A span representing a range in the source document.
///
/// Spans are inclusive of the start position and exclusive of the end position.
/// This matches CommonMark and most parser conventions.
///
/// # Example
///
/// For the text "**bold**":
/// - `start`: Position at the first '*'
/// - `end`: Position after the last '*' (one past the last character)
///
/// # Multi-line Spans
///
/// For multi-line content like code blocks, for example a fenced Rust code block,
/// the inner code might look like:
///
///     fn main() {}
///
/// - `start.line`: Line of opening backticks
/// - `end.line`: Line after closing backticks
/// - Columns are byte offsets within their respective lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Start position (inclusive)
    pub start: Position,
    
    /// End position (exclusive)
    pub end: Position,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    /// Compute the absolute byte offset of the start of this position's line.
    ///
    /// Uses the invariant that `column` is a 1-based byte offset from the
    /// start of the line, and `offset` is the absolute byte offset from the
    /// start of the document. The formula is:
    ///
    /// line_start_offset = offset - (column - 1)
    ///
    /// This function uses saturating math to avoid underflow in case of
    /// malformed positions.
    pub fn line_start_offset(&self) -> usize {
        self.offset.saturating_sub(self.column.saturating_sub(1))
    }
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Return the absolute byte offset of the start of the span's first line.
    ///
    /// This is a convenience wrapper around `Position::line_start_offset` for
    /// the span's `start` position.
    pub fn start_line_offset(&self) -> usize {
        self.start.line_start_offset()
    }

    /// Return the absolute byte offset of the start of the span's end line.
    /// Useful when expanding a span to include the whole end line.
    pub fn end_line_offset(&self) -> usize {
        self.end.line_start_offset()
    }
}

/// Convenience helper: compute the absolute byte offset of the start of the
/// given span's starting line.
///
/// This is exposed as a free function to simplify callers that don't have a
/// `Span` method in scope or prefer a function name matching the refactor plan.
pub fn compute_line_start_offset(span: &Span) -> usize {
    span.start_line_offset()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_start_offset_simple() {
        let pos = Position::new(1, 1, 0);
        assert_eq!(pos.line_start_offset(), 0);

        let pos2 = Position::new(1, 5, 4);
        // offset 4, column 5 -> line start = 4 - (5 - 1) = 0
        assert_eq!(pos2.line_start_offset(), 0);

        let pos3 = Position::new(3, 4, 25);
        // offset 25, column 4 -> line start = 25 - (4 - 1) = 22
        assert_eq!(pos3.line_start_offset(), 22);
    }

    #[test]
    fn test_span_line_offsets() {
        let start = Position::new(2, 3, 10);
        let end = Position::new(4, 1, 40);
        let span = Span::new(start, end);

        assert_eq!(span.start_line_offset(), 10 - (3 - 1));
        assert_eq!(span.end_line_offset(), 40 - (1 - 1));
    }
}
