use crate::editor::logic::ast::inlines::Inline;
// ============================================================================
// CommonMark Spec Version 0.31.2
// Section 3: Blocks and inlines
//
// This module defines the Abstract Syntax Tree (AST) nodes for CommonMark
// section 3, covering the high-level block and inline structure of a document.
//
// Reference: https://spec.commonmark.org/0.31.2/#blocks-and-inlines
//
// Chapters covered:
//   3.1 Precedence
//   3.2 Container blocks and leaf blocks
// ============================================================================

/// --------------------------------------------------------------------------
/// 3.1 Precedence
///
/// Block structure indicators always take precedence over inline structure indicators.
/// This means the parser first determines the block structure, then parses inline
/// content within block containers. This AST node is a marker for this precedence rule.
/// --------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockOrInline {
    /// A block-level element (see `Block` enum).
    Block(Block),
    /// An inline-level element (see `Inline` enum).
    Inline(Inline),
}

/// --------------------------------------------------------------------------
/// 3.2 Container blocks and leaf blocks
///
/// Blocks are divided into two types:
///   - Container blocks: can contain other blocks (e.g., block quotes, list items)
///   - Leaf blocks: cannot contain other blocks (e.g., paragraphs, headings, code blocks)
///
/// This section defines the core enums for block structure.
/// --------------------------------------------------------------------------

/// Represents any block-level element in a CommonMark document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A container block (can contain other blocks).
    Container(ContainerBlock),
    /// A leaf block (cannot contain other blocks).
    Leaf(LeafBlock),
}

/// Container blocks: blocks that can contain other blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerBlock {
    /// Block quote (can contain blocks).
    BlockQuote(Vec<Block>),
    /// List item (can contain blocks).
    ListItem(Vec<Block>),
    /// List (meta-container for list items).
    List(Vec<ContainerBlock>),
}

/// Leaf blocks: blocks that cannot contain other blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafBlock {
    /// Paragraph (contains inlines).
    Paragraph(Vec<Inline>),
    /// Heading (contains inlines, with level).
    Heading { level: u8, content: Vec<Inline> },
    /// Code block (literal text).
    CodeBlock(String),
    /// Thematic break (horizontal rule).
    ThematicBreak,
    /// HTML block (raw HTML).
    HtmlBlock(String),
    // ... other leaf block types as needed
}

