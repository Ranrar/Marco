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
}

// --------------------------------------------------------------------------
// 4.9 Blank lines
// --------------------------------------------------------------------------

// Blank lines are represented by the LeafBlock::BlankLine variant (no data)