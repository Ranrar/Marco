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
    Inline(crate::logic::ast::inlines::Inline),
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
    /// Document root (contains blocks).
    Document(Vec<Block>, Option<crate::logic::attr_parser::Attributes>),
    /// Block quote (can contain blocks).
    BlockQuote(Vec<Block>, Option<crate::logic::attr_parser::Attributes>),
    /// List item (can contain blocks, with marker and kind).
    ListItem {
        marker: ListMarker,
        contents: Vec<Block>,
        attributes: Option<crate::logic::attr_parser::Attributes>,
    },
    /// List (container for blocks, with kind, tight/loose, delimiter, start number).
    /// Now uses Vec<Block> for items, allowing safe traversal and transformation.
    List {
        kind: ListKind,
        tight: bool,
        items: Vec<Block>,
        attributes: Option<crate::logic::attr_parser::Attributes>,
    },
}

/// Leaf blocks: blocks that cannot contain other blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafBlock {
    /// Paragraph (contains inlines and source positions).
    Paragraph(Vec<(crate::logic::ast::inlines::Inline, crate::logic::core::event_types::SourcePos)>, Option<crate::logic::attr_parser::Attributes>),
    /// Heading (contains inlines and source positions, with level).
    Heading { level: u8, content: Vec<(crate::logic::ast::inlines::Inline, crate::logic::core::event_types::SourcePos)>, attributes: Option<crate::logic::attr_parser::Attributes> },
    /// ATX Heading (with level and raw content).
    AtxHeading { level: u8, raw_content: String, attributes: Option<crate::logic::attr_parser::Attributes> },
    /// Setext Heading (with level and raw content).
    SetextHeading { level: u8, raw_content: String, attributes: Option<crate::logic::attr_parser::Attributes> },
    /// Indented code block (literal text).
    IndentedCodeBlock { content: String, attributes: Option<crate::logic::attr_parser::Attributes> },
    /// Fenced code block (fence char, count, info string, content).
    FencedCodeBlock {
        fence_char: char,
        fence_count: usize,
        info_string: Option<String>,
        content: String,
        attributes: Option<crate::logic::attr_parser::Attributes>,
    },
    /// Thematic break (horizontal rule, marker and count).
    ThematicBreak { marker: char, count: usize, raw: String, attributes: Option<crate::logic::attr_parser::Attributes> },
    /// HTML block (raw HTML, block type).
    HtmlBlock { block_type: HtmlBlockType, content: String, attributes: Option<crate::logic::attr_parser::Attributes> },
    /// Link reference definition ([label]: destination "title").
    LinkReferenceDefinition {
        label: String,
        destination: String,
        title: Option<String>,
        attributes: Option<crate::logic::attr_parser::Attributes>,
    },
    /// Blank line (for block separation).
    BlankLine,
    /// Math block (GFM/LaTeX, e.g., $$ ... $$ or ```math ... ```)
    Math(crate::logic::ast::math::MathBlock),

    /// Custom tag block (for :::custom ...::: and other extension containers)
    CustomTagBlock {
        name: String,
        data: Option<String>,
        content: Vec<Block>,
        attributes: Option<crate::logic::attr_parser::Attributes>,
    },
    /// Table block (GFM extension)
    Table {
        header: crate::logic::ast::github::TableRow,
        alignments: Vec<crate::logic::ast::github::TableAlignment>,
        rows: Vec<crate::logic::ast::github::TableRow>,
        caption: Option<String>,
        attributes: Option<crate::logic::attr_parser::Attributes>,
    },
}

/// List marker: bullet or ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListMarker {
    Bullet { char: char },
    Ordered { number: u64, delimiter: OrderedDelimiter },
}

/// Ordered list delimiter: '.' or ')'.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderedDelimiter {
    Period,
    Paren,
}

/// List kind: bullet or ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListKind {
    Bullet { char: char },
    Ordered { start: u64, delimiter: OrderedDelimiter },
}

/// HTML block type (1–7, per spec).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlBlockType {
    Type1, // <pre>, <script>, <style>, <textarea>
    Type2, // <!-- ... -->
    Type3, // <? ... ?>
    Type4, // <!A ... >
    Type5, // <![CDATA[ ... ]]>
    Type6, // Block-level open/close tags
    Type7, // Any complete open/close tag on its own line
}

