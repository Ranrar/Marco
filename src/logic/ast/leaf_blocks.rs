use anyhow::Error;

/// Type alias for AST results with anyhow error handling.
pub type AstResult<T> = Result<T, Error>;

/// Example: minimal error-producing function for demonstration.
pub fn parse_leaf_block_safe(is_valid: bool) -> AstResult<LeafBlock> {
    if !is_valid {
        Err(Error::msg("Invalid leaf block"))
    } else {
        Ok(LeafBlock::BlankLine)
    }
}

/// Trait for visiting AST nodes in leaf_blocks.rs
pub trait AstVisitor {
    fn visit_leaf_block(&mut self, leaf: &LeafBlock) {
        match leaf {
            LeafBlock::ThematicBreak(tb) => self.visit_thematic_break(tb),
            LeafBlock::AtxHeading(h) => self.visit_atx_heading(h),
            LeafBlock::SetextHeading(h) => self.visit_setext_heading(h),
            LeafBlock::IndentedCodeBlock(cb) => self.visit_indented_code_block(cb),
            LeafBlock::FencedCodeBlock(cb) => self.visit_fenced_code_block(cb),
            LeafBlock::HtmlBlock(hb) => self.visit_html_block(hb),
            LeafBlock::LinkReferenceDefinition(lrd) => self.visit_link_reference_definition(lrd),
            LeafBlock::Paragraph(p) => self.visit_paragraph(p),
            LeafBlock::BlankLine => self.visit_blank_line(),
        }
    }

    fn visit_thematic_break(&mut self, _tb: &ThematicBreak) {}
    fn visit_atx_heading(&mut self, _h: &AtxHeading) {}
    fn visit_setext_heading(&mut self, _h: &SetextHeading) {}
    fn visit_indented_code_block(&mut self, _cb: &IndentedCodeBlock) {}
    fn visit_fenced_code_block(&mut self, _cb: &FencedCodeBlock) {}
    fn visit_html_block(&mut self, _hb: &HtmlBlock) {}
    fn visit_link_reference_definition(&mut self, _lrd: &LinkReferenceDefinition) {}
    fn visit_paragraph(&mut self, p: &Paragraph) {
        self.walk_paragraph(p);
    }
    fn walk_paragraph(&mut self, p: &Paragraph) {
        for (_inline, _) in &p.inlines {
            // Traverse inlines (handled in inlines visitor)
        }
    }
    fn visit_blank_line(&mut self) {}
}
// ============================================================================
// CommonMark Spec Version 0.31.2 - Section 4: Leaf blocks
//
// This module defines the Abstract Syntax Tree (AST) node types for all
// "leaf blocks" as described in CommonMark 0.31.2, Section 4 (https://spec.commonmark.org/0.31.2/#leaf-blocks).
//
// Each enum/struct is mapped to a specific subchapter (4.1–4.9) and is fully documented.
// Only AST definitions are provided—no parsing or logic.
// ============================================================================

/// Enum representing all possible leaf block types in CommonMark (Section 4).
#[derive(Debug, Clone, PartialEq)]
pub enum LeafBlock {
    /// 4.1 Thematic breaks: Horizontal rules (***, --- or ___)
    ThematicBreak(ThematicBreak),
    /// 4.2 ATX headings: # Heading
    AtxHeading(AtxHeading),
    /// 4.3 Setext headings: Underline-style headings
    SetextHeading(SetextHeading),
    /// 4.4 Indented code blocks: 4+ space-indented literal code
    IndentedCodeBlock(IndentedCodeBlock),
    /// 4.5 Fenced code blocks: ``` or ~~~ fenced code
    FencedCodeBlock(FencedCodeBlock),
    /// 4.6 HTML blocks: Raw HTML blocks (7 types)
    HtmlBlock(HtmlBlock),
    /// 4.7 Link reference definitions: [label]: destination "title"
    LinkReferenceDefinition(LinkReferenceDefinition),
    /// 4.8 Paragraphs: Regular text blocks
    Paragraph(Paragraph),
    /// 4.9 Blank lines: Ignored, but tracked for block separation
    BlankLine,
}

// --------------------------------------------------------------------------
// 4.1 Thematic breaks
// --------------------------------------------------------------------------

/// 4.1 Thematic breaks: Horizontal rules (***, --- or ___)
#[derive(Debug, Clone, PartialEq)]
pub struct ThematicBreak {
    /// The character used for the break: '-', '*', or '_'
    pub marker: char,
    /// The total number of marker characters (>= 3)
    pub count: usize,
    /// The original line (for round-trip or error reporting)
    pub raw: String,
}

// --------------------------------------------------------------------------
// 4.2 ATX headings
// --------------------------------------------------------------------------

/// 4.2 ATX headings: # Heading
#[derive(Debug, Clone, PartialEq)]
pub struct AtxHeading {
    /// Heading level (1–6)
    pub level: u8,
    /// Raw content (before inline parsing)
    pub raw_content: String,
}

// --------------------------------------------------------------------------
// 4.3 Setext headings
// --------------------------------------------------------------------------

/// 4.3 Setext headings: Underline-style headings
#[derive(Debug, Clone, PartialEq)]
pub struct SetextHeading {
    /// Heading level (1 for '=', 2 for '-')
    pub level: u8,
    /// Raw content (before inline parsing)
    pub raw_content: String,
}

// --------------------------------------------------------------------------
// 4.4 Indented code blocks
// --------------------------------------------------------------------------

/// 4.4 Indented code blocks: 4+ space-indented literal code
#[derive(Debug, Clone, PartialEq)]
pub struct IndentedCodeBlock {
    /// The literal code content (including line endings)
    pub content: String,
}

// --------------------------------------------------------------------------
// 4.5 Fenced code blocks
// --------------------------------------------------------------------------

/// 4.5 Fenced code blocks: ``` or ~~~ fenced code
#[derive(Debug, Clone, PartialEq)]
pub struct FencedCodeBlock {
    /// The fence marker: '`' or '~'
    pub fence_char: char,
    /// Number of fence characters (>= 3)
    pub fence_count: usize,
    /// Optional info string (language, etc.)
    pub info_string: Option<String>,
    /// The literal code content (including line endings)
    pub content: String,
}

// --------------------------------------------------------------------------
// 4.6 HTML blocks
// --------------------------------------------------------------------------

/// 4.6 HTML blocks: Raw HTML blocks (7 types)
#[derive(Debug, Clone, PartialEq)]
pub struct HtmlBlock {
    /// HTML block type (1–7, per spec)
    pub block_type: HtmlBlockType,
    /// The raw HTML content (including line endings)
    pub content: String,
}

/// Enum for the 7 HTML block types (see spec for details)
#[derive(Debug, Clone, PartialEq)]
pub enum HtmlBlockType {
    Type1, // <pre>, <script>, <style>, <textarea>
    Type2, // <!-- ... -->
    Type3, // <? ... ?>
    Type4, // <!A ... >
    Type5, // <![CDATA[ ... ]]>
    Type6, // Block-level open/close tags
    Type7, // Any complete open/close tag on its own line
}

// --------------------------------------------------------------------------
// 4.7 Link reference definitions
// --------------------------------------------------------------------------

/// 4.7 Link reference definitions: [label]: destination "title"
#[derive(Debug, Clone, PartialEq)]
pub struct LinkReferenceDefinition {
    /// The normalized label (case-insensitive, collapsed whitespace)
    pub label: String,
    /// The link destination (URL or path)
    pub destination: String,
    /// Optional link title
    pub title: Option<String>,
}

// --------------------------------------------------------------------------
// 4.8 Paragraphs
// --------------------------------------------------------------------------

/// 4.8 Paragraphs: Regular text blocks
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// Raw content (before inline parsing)
    pub raw_content: String,
    /// Parsed inline content (for visitor traversal)
    pub inlines: Vec<(crate::logic::ast::inlines::Inline, crate::logic::core::event_types::SourcePos)>,
}
impl LeafBlock {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_leaf_block(self);
    }
}

impl ThematicBreak {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_thematic_break(self);
    }
}

impl AtxHeading {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_atx_heading(self);
    }
}

impl SetextHeading {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_setext_heading(self);
    }
}

impl IndentedCodeBlock {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_indented_code_block(self);
    }
}

impl FencedCodeBlock {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_fenced_code_block(self);
    }
}

impl HtmlBlock {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_html_block(self);
    }
}

impl LinkReferenceDefinition {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_link_reference_definition(self);
    }
}

impl Paragraph {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_paragraph(self);
    }
}

// --------------------------------------------------------------------------
// 4.9 Blank lines
// --------------------------------------------------------------------------

// Blank lines are represented by the LeafBlock::BlankLine variant (no data)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::core::event_types::SourcePos;
    use crate::logic::ast::inlines::Inline;

    #[test]
    fn test_paragraph_traversal() {
        let para_struct = Paragraph {
            raw_content: "para".to_string(),
            inlines: vec![(Inline::Text("para".to_string()), SourcePos { line: 1, column: 1 })],
        };
        struct Printer;
        impl AstVisitor for Printer {
            fn visit_paragraph(&mut self, p: &Paragraph) {
                self.walk_paragraph(p);
            }
        }
        let mut printer = Printer;
        printer.visit_paragraph(&para_struct);
    }

    #[test]
    fn test_blank_line() {
        struct Printer;
        impl AstVisitor for Printer {
            fn visit_blank_line(&mut self) {
                assert!(true);
            }
        }
        let mut printer = Printer;
        printer.visit_blank_line();
    }

    #[test]
    fn test_error_handling() {
        let result = super::parse_leaf_block_safe(false);
        assert!(result.is_err());
        let result = super::parse_leaf_block_safe(true);
        assert!(result.is_ok());
    }
}