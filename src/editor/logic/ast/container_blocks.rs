// -----------------------------------------------------------------------------
// CommonMark Spec Version 0.31.2
// Section 5: Container blocks
//
// This file defines the Abstract Syntax Tree (AST) types for CommonMark
// container blocks, as described in Section 5 of the specification:
//   5.1 Block quotes
//   5.2 List items
//   5.3 Lists
//
// Each section is mapped to a Rust enum or struct, with detailed comments
// explaining the mapping and rationale. Only AST types are defined here;
// no parsing or rendering logic is included.
// -----------------------------------------------------------------------------

/// 5.1 Block quotes
///
/// A block quote is a container block that contains other blocks.
/// It is represented by a sequence of blocks, each of which may be any block type.
/// Block quotes are delimited in the source by lines starting with a block quote marker ('>').
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockQuote {
    /// The sequence of blocks contained in this block quote.
    pub contents: Vec<Block>,
}

/// 5.2 List items
///
/// A list item is a container block that contains other blocks.
/// It is delimited in the source by a list marker (bullet or ordered) and indentation.
/// List items may be empty, or may contain any sequence of blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    /// The marker used for this list item (bullet or ordered).
    pub marker: ListMarker,
    /// The sequence of blocks contained in this list item.
    pub contents: Vec<Block>,
}

/// The marker for a list item: bullet or ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListMarker {
    /// A bullet list marker: '-', '+', or '*'.
    Bullet { char: char },
    /// An ordered list marker: 1-9 digits, followed by '.' or ')', with a start number.
    Ordered {
        number: u64,
        delimiter: OrderedDelimiter,
    },
}

/// The delimiter for an ordered list marker: '.' or ')'.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderedDelimiter {
    Period,
    Paren,
}

/// 5.3 Lists
///
/// A list is a container for one or more list items of the same type.
/// Lists may be tight or loose, and may be bullet or ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    /// The type of this list (bullet or ordered).
    pub kind: ListKind,
    /// Whether the list is tight (no blank lines between items) or loose (blank lines present).
    pub tight: bool,
    /// The sequence of list items in this list.
    pub items: Vec<ListItem>,
}

/// The kind of list: bullet or ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListKind {
    /// A bullet list (all items use '-', '+', or '*').
    Bullet { char: char },
    /// An ordered list (all items use numbers and the same delimiter).
    Ordered {
        start: u64,
        delimiter: OrderedDelimiter,
    },
}

/// A block element, which may be a container or leaf block.
/// This is referenced by container blocks (BlockQuote, ListItem, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A block quote (container block).
    BlockQuote(BlockQuote),
    /// A list (container block).
    List(List),
    // Other block types (headings, code blocks, etc.) would be defined elsewhere.
    // ...
}