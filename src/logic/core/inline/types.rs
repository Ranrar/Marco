// types.rs - Token definitions, inline node enums, positions

/// Central location for all token types, node enums, and position tracking structs.

// types.rs - Token definitions, inline node enums, positions

/// Delimiter stack entry for emphasis/strong parsing (CommonMark spec)
/// See: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis
/// and pulldown-cmark/comrak implementations
#[derive(Debug, Clone)]
pub struct Delim {
    /// Delimiter character: '*' or '_'
    pub ch: char,
    /// Number of consecutive delimiters in this run
    pub count: usize,
    /// Source position (line, column) for error reporting and AST
    pub pos: crate::logic::core::event_types::SourcePos,
    /// Can this delimiter open an emphasis/strong span? (left-flanking)
    pub can_open: bool,
    /// Can this delimiter close an emphasis/strong span? (right-flanking)
    pub can_close: bool,
    /// Index in the input string (for stack processing)
    pub idx: usize,
    /// Is this delimiter active? (for future link/image nesting)
    pub active: bool,
}

/// Bracket stack entry for link/image parsing (CommonMark spec)
/// See: https://spec.commonmark.org/0.31.2/#links
#[derive(Debug, Clone)]
pub struct Bracket {
    /// Is this an image opener? ('![')
    pub image: bool,
    /// Source position (line, column) for error reporting and AST
    pub pos: crate::logic::core::event_types::SourcePos,
    /// Index in the input string (for stack processing)
    pub idx: usize,
}

/// Inline token types for tokenizer output
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Star(usize),
    Underscore(usize),
    Backtick(usize),
    Dollar(usize),
    OpenBracket,
    CloseBracket,
    Bang,
    OpenParen,
    CloseParen,
    Backslash(char),
    Ampersand,
    Html(String),
    CodeSpan(String),
    MathSpan(String),
    Entity(String),
    AttributeBlock(String),
    SoftBreak,
    HardBreak,
    // Add more as needed
}
/// Inline AST node definitions for Markdown inlines (CommonMark/GFM)
pub use crate::logic::core::event_types::SourcePos;

#[derive(Debug, Clone, PartialEq)]
pub enum InlineNode {
    Text { text: String, pos: SourcePos },
    Emphasis { children: Vec<InlineNode>, pos: SourcePos },
    Strong { children: Vec<InlineNode>, pos: SourcePos },
    Code { text: String, pos: SourcePos },
    Link { href: String, title: String, children: Vec<InlineNode>, pos: SourcePos },
    Image { src: String, alt: String, title: String, pos: SourcePos },
    Math { text: String, pos: SourcePos },
    Html { text: String, pos: SourcePos },
    SoftBreak { pos: SourcePos },
    LineBreak { pos: SourcePos },
    // Extend with more types as needed (emoji, mention, etc.)
}
