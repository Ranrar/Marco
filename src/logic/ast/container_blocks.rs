use anyhow::Error;

/// Type alias for AST results with anyhow error handling.
pub type AstResult<T> = Result<T, Error>;

/// Example: minimal error-producing function for demonstration.
pub fn parse_container_block_safe(is_valid: bool) -> AstResult<Block> {
    if !is_valid {
        Err(Error::msg("Invalid container block"))
    } else {
        Ok(Block::BlockQuote(BlockQuote { contents: vec![] }))
    }
}

/// Trait for visiting AST nodes in container_blocks.rs
pub trait AstVisitor {
    fn visit_block(&mut self, block: &Block) {
        match block {
            Block::BlockQuote(bq) => self.visit_block_quote(bq),
            Block::List(list) => self.visit_list(list),
            // ...existing code for other block types...
        }
    }

    fn visit_block_quote(&mut self, block_quote: &BlockQuote) {
        self.walk_block_quote(block_quote);
    }

    fn walk_block_quote(&mut self, block_quote: &BlockQuote) {
        for block in &block_quote.contents {
            self.visit_block(block);
        }
    }

    fn visit_list(&mut self, list: &List) {
        self.walk_list(list);
    }

    fn walk_list(&mut self, list: &List) {
        for block in &list.items {
            self.visit_block(block);
        }
    }

    fn visit_list_item(&mut self, list_item: &ListItem) {
        self.walk_list_item(list_item);
    }

    fn walk_list_item(&mut self, list_item: &ListItem) {
        for block in &list_item.contents {
            self.visit_block(block);
        }
    }
}
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
    /// The sequence of blocks in this list (was Vec<ListItem>, now Vec<Block> for consistency).
    pub items: Vec<Block>,
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