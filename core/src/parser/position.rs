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
/// For multi-line content like code blocks:
/// ```markdown
/// ```rust
/// fn main() {}
/// ```
/// ```
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
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}
