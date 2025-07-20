#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::ast::inlines::Inline;
    use crate::logic::core::event_types::SourcePos;
    // use crate::logic::ast::github::{TableRow, TableAlignment};

    #[test]
    fn test_debug_printer_traversal() {
        // Create a simple AST: Document -> Paragraph -> Inline
        let inline = Inline::Text("Hello, world!".to_string());
        let para = LeafBlock::Paragraph(vec![(inline, SourcePos { line: 1, column: 1 })], None);
        let block = Block::Leaf(para);
        let doc = Block::Container(ContainerBlock::Document(vec![block], None));

        let mut printer = DebugPrinter;
        doc.accept(&mut printer);
        // Output should show traversal of Document, Paragraph, and Inline
        // (Manual verification: check stdout for expected print statements)
    }

    #[test]
    fn test_deeply_nested_blocks() {
        // Document -> BlockQuote -> List -> ListItem -> Paragraph
        let inline = Inline::Text("Nested".to_string());
        let para = Block::Leaf(LeafBlock::Paragraph(vec![(inline, SourcePos { line: 2, column: 2 })], None));
        let list_item = Block::Container(ContainerBlock::ListItem {
            marker: ListMarker::Bullet { char: '-' },
            contents: vec![para.clone()],
            task_checked: None,
            attributes: None,
        });
        let list = Block::Container(ContainerBlock::List {
            kind: ListKind::Bullet { char: '-' },
            tight: false,
            items: vec![list_item.clone()],
            attributes: None,
        });
        let bq = Block::Container(ContainerBlock::BlockQuote(vec![list.clone()], None));
        let doc = Block::Container(ContainerBlock::Document(vec![bq.clone()], None));

        let mut printer = DebugPrinter;
        doc.accept(&mut printer);
        // Should traverse all nested levels
    }

    #[test]
    fn test_empty_blocks() {
        let doc = Block::Container(ContainerBlock::Document(vec![], None));
        let mut printer = DebugPrinter;
        doc.accept(&mut printer);
        // Should handle empty document gracefully
    }

    #[test]
    fn test_table_and_math_blocks() {
        use crate::logic::ast::github::{TableRow, TableAlignment};
        use crate::logic::ast::math::MathBlock;
        let table = Block::Leaf(LeafBlock::Table {
            header: TableRow { cells: vec![] },
            alignments: vec![TableAlignment::None],
            rows: vec![],
            caption: Some("caption".to_string()),
            attributes: None,
        });
        let math = Block::Leaf(LeafBlock::Math(MathBlock {
            content: "x^2".to_string(),
            display: true,
            math_type: crate::logic::ast::math::MathType::LaTeX,
            position: None,
            attributes: None,
        }));
        let doc = Block::Container(ContainerBlock::Document(vec![table, math], None));
        let mut printer = DebugPrinter;
        doc.accept(&mut printer);
        // Should traverse table and math blocks
    }

    #[test]
    fn test_error_handling() {
        let result = super::parse_block_safe(false);
        assert!(result.is_err());
        let result = super::parse_block_safe(true);
        assert!(result.is_ok());
    }
}
// ============================================================================
use anyhow::Error;

/// Type alias for AST results with anyhow error handling.
pub type AstResult<T> = Result<T, Error>;

/// Example: minimal error-producing function for demonstration.
pub fn parse_block_safe(is_valid: bool) -> AstResult<Block> {
    if !is_valid {
        Err(Error::msg("Invalid block"))
    } else {
        Ok(Block::Leaf(LeafBlock::BlankLine))
    }
}
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

impl Block {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_block(self);
    }
}

/// Container blocks: blocks that can contain other blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerBlock {
    /// Document root (contains blocks).
    Document(Vec<Block>, Option<crate::logic::attr_parser::Attributes>),
    /// Block quote (can contain blocks).
    BlockQuote(Vec<Block>, Option<crate::logic::attr_parser::Attributes>),
    /// List item (can contain blocks, with marker and kind).
    /// GFM task list item support: `task_checked` is Some(true) for checked, Some(false) for unchecked, None for regular items.
    ListItem {
        marker: ListMarker,
        contents: Vec<Block>,
        /// If this is a GFM task list item, this is Some(true) for checked, Some(false) for unchecked, None for regular list items.
        task_checked: Option<bool>,
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

impl ContainerBlock {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_container_block(self);
    }
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

impl LeafBlock {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_leaf_block(self);
    }
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

/// Trait for visiting AST nodes in the Markdown document.
pub trait AstVisitor {
    // Visit methods for each node type

    fn visit_block(&mut self, block: &Block) {
        self.walk_block(block);
    }

    fn walk_block(&mut self, block: &Block) {
        match block {
            Block::Container(container) => self.visit_container_block(container),
            Block::Leaf(leaf) => self.visit_leaf_block(leaf),
        }
    }

    fn visit_container_block(&mut self, container: &ContainerBlock) {
        self.walk_container_block(container);
    }

    fn walk_container_block(&mut self, container: &ContainerBlock) {
        match container {
            ContainerBlock::Document(blocks, _) => {
                for block in blocks {
                    self.visit_block(block);
                }
            }
            ContainerBlock::BlockQuote(blocks, _) => {
                for block in blocks {
                    self.visit_block(block);
                }
            }
            ContainerBlock::ListItem { contents, .. } => {
                for block in contents {
                    self.visit_block(block);
                }
            }
            ContainerBlock::List { items, .. } => {
                for block in items {
                    self.visit_block(block);
                }
            }
        }
    }

    fn visit_leaf_block(&mut self, leaf: &LeafBlock) {
        self.walk_leaf_block(leaf);
    }

    fn walk_leaf_block(&mut self, leaf: &LeafBlock) {
        match leaf {
            LeafBlock::Paragraph(inlines, _) => {
                for (inline, _) in inlines {
                    self.visit_inline(inline);
                }
            }
            LeafBlock::Heading { content, .. } => {
                for (inline, _) in content {
                    self.visit_inline(inline);
                }
            }
            LeafBlock::CustomTagBlock { content, .. } => {
                for block in content {
                    self.visit_block(block);
                }
            }
            LeafBlock::Table { header, rows, .. } => {
                self.visit_table_row(header);
                for row in rows {
                    self.visit_table_row(row);
                }
            }
            _ => {}
        }
    }

    // Inline visitor stub (to be implemented in inlines.rs)
    fn visit_inline(&mut self, _inline: &crate::logic::ast::inlines::Inline) {}

    // Table row visitor stub (to be implemented in github.rs)
    fn visit_table_row(&mut self, _row: &crate::logic::ast::github::TableRow) {}
}

/// Sample visitor that prints node types for debugging.
pub struct DebugPrinter;

impl AstVisitor for DebugPrinter {
    fn visit_block(&mut self, block: &Block) {
        println!("Visiting Block: {:?}", block);
        self.walk_block(block);
    }

    fn visit_container_block(&mut self, container: &ContainerBlock) {
        println!("Visiting ContainerBlock: {:?}", container);
        self.walk_container_block(container);
    }

    fn visit_leaf_block(&mut self, leaf: &LeafBlock) {
        println!("Visiting LeafBlock: {:?}", leaf);
        self.walk_leaf_block(leaf);
    }
}

