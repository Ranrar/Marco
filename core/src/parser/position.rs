// Position tracking for LSP integration (line/column mapping)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
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
