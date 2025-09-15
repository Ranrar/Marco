//! # Simplified Marco AST Node Definitions
//!
//! This module contains the simplified Abstract Syntax Tree (AST) node definitions for the Marco
//! markup language parser. Following the grammar-centered design principle from the documentation.
//!
//! ## Design Principles
//!
//! - **Grammar-Centered**: Direct 1:1 mapping with marco.pest rules
//! - **Simplicity**: Single Node enum without enterprise abstractions
//! - **Essential Fields Only**: No over-engineering with metadata variants
//! - **Span-Aware**: Every node tracks source location for error reporting
//!
//! ## Node Categories
//!
//! ### Document Structure
//! - `Document`: Root container for all content
//!
//! ### Block Elements
//! - `Heading`: All heading types (ATX and Setext unified)
//! - `Paragraph`: Regular text content
//! - `CodeBlock`: All code block types (fenced and indented unified)
//! - `MathBlock`: Math display blocks
//! - `List`: Ordered and unordered lists
//! - `ListItem`: Individual list items with optional task checkbox
//! - `Table`: Table structure with headers and rows
//! - `BlockQuote`: Quoted content
//! - `HorizontalRule`: Thematic breaks
//! - `DefinitionList`: Definition list container
//! - `DefinitionTerm`: Definition terms
//! - `DefinitionDescription`: Definition descriptions
//!
//! ### Inline Elements
//! - `Text`: Plain text content
//! - `Strong`: Bold text
//! - `Emphasis`: Italic text
//! - `Strikethrough`: Struck through text
//! - `Highlight`: Highlighted text
//! - `Superscript`: Superscript text
//! - `Subscript`: Subscript text
//! - `Code`: Inline code
//! - `MathInline`: Inline math
//! - `Link`: Links with text and URL
//! - `Image`: Images with alt text and URL
//! - `LineBreak`: Line breaks (all types unified)
//! - `EscapedChar`: Escaped characters
//!
//! ### Marco Extensions
//! - `UserMention`: User mentions with platform
//! - `Bookmark`: Bookmarks with path and line
//! - `TabBlock`: Tab containers
//! - `Tab`: Individual tabs
//! - `Admonition`: Note/warning/tip blocks
//! - `TableOfContents`: Auto-generated TOCs
//! - `RunInline`: Inline executable code
//! - `RunBlock`: Block executable code
//! - `DiagramBlock`: Mermaid/GraphViz diagrams
//!
//! ### Error Recovery
//! - `Unknown`: For unrecognized content during parsing

use serde::{Deserialize, Serialize};

/// Type of line break
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineBreakType {
    /// Hard line break (2+ spaces or backslash + newline)
    Hard,
    /// Soft line break (just newline)
    Soft,
}

/// Source position information for any node in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

/// Simplified Marco AST Node with direct grammar mapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    // ===========================================
    // DOCUMENT STRUCTURE
    // ===========================================
    /// Root document node containing all content
    Document { children: Vec<Node>, span: Span },

    // ===========================================
    // BLOCK ELEMENTS
    // ===========================================
    /// Heading (ATX ## or Setext underlined) - unified
    Heading {
        level: u8,          // 1-6
        content: Vec<Node>, // Inline content
        span: Span,
    },

    /// Paragraph containing inline content
    Paragraph { content: Vec<Node>, span: Span },

    /// Code block (fenced ``` or indented) - unified
    CodeBlock {
        language: Option<String>, // Programming language if specified
        content: String,          // Raw code content
        span: Span,
    },

    /// Math block $$formula$$
    MathBlock { content: String, span: Span },

    /// List (ordered or unordered)
    List {
        ordered: bool,    // true for numbered lists
        items: Vec<Node>, // ListItem nodes
        span: Span,
    },

    /// List item with optional task checkbox
    ListItem {
        content: Vec<Node>,    // Item content
        checked: Option<bool>, // For task lists: None, Some(false), Some(true)
        span: Span,
    },

    /// Table with headers and rows
    Table {
        headers: Vec<Node>,   // Header cells (TableCell nodes)
        rows: Vec<Vec<Node>>, // Data rows (each row is Vec<TableCell>)
        span: Span,
    },

    /// Table cell
    TableCell {
        content: Vec<Node>,        // Cell content
        alignment: Option<String>, // "left", "center", "right"
        span: Span,
    },

    /// Block quote
    BlockQuote { content: Vec<Node>, span: Span },

    /// Horizontal rule (---, ***, ___)
    HorizontalRule { span: Span },

    /// Definition list container
    DefinitionList {
        items: Vec<Node>, // DefinitionTerm and DefinitionDescription nodes
        span: Span,
    },

    /// Definition term
    DefinitionTerm { content: Vec<Node>, span: Span },

    /// Definition description
    DefinitionDescription { content: Vec<Node>, span: Span },

    // ===========================================
    // INLINE ELEMENTS
    // ===========================================
    /// Plain text content
    Text { content: String, span: Span },

    /// Strong emphasis (bold) **text**
    Strong { content: Vec<Node>, span: Span },

    /// Emphasis (italic) *text*
    Emphasis { content: Vec<Node>, span: Span },

    /// Strikethrough ~~text~~
    Strikethrough { content: Vec<Node>, span: Span },

    /// Highlight ==text==
    Highlight { content: Vec<Node>, span: Span },

    /// Superscript ^text^
    Superscript { content: Vec<Node>, span: Span },

    /// Subscript ~text~
    Subscript { content: Vec<Node>, span: Span },

    /// Inline code `code`
    Code { content: String, span: Span },

    /// Inline math $formula$
    MathInline { content: String, span: Span },

    /// Links [text](url "title")
    Link {
        text: Vec<Node>,       // Link text content
        url: String,           // Link URL
        title: Option<String>, // Optional title
        span: Span,
    },

    /// Images ![alt](url "title")
    Image {
        alt: String,           // Alt text
        url: String,           // Image URL
        title: Option<String>, // Optional title
        span: Span,
    },

    /// Line break (hard, soft, or regular) - unified
    LineBreak {
        break_type: LineBreakType, // Type of line break
        span: Span,
    },

    /// Escaped character \x
    EscapedChar { character: char, span: Span },

    // ===========================================
    // MARCO EXTENSIONS
    // ===========================================
    /// User mention @username[platform]
    UserMention {
        username: String,
        platform: Option<String>, // github, twitter, slack, etc.
        display_name: Option<String>,
        span: Span,
    },

    /// Bookmark [bookmark:label](path=line)
    Bookmark {
        label: String,
        path: String,
        line: Option<u32>,
        span: Span,
    },

    /// Tab container :::tab
    TabBlock {
        title: Option<String>,
        tabs: Vec<Node>, // Tab nodes
        span: Span,
    },

    /// Individual tab @tab
    Tab {
        name: Option<String>,
        content: Vec<Node>,
        span: Span,
    },

    /// Admonition block :::note
    Admonition {
        kind: String, // "note", "warning", "tip", "danger", "info"
        content: Vec<Node>,
        span: Span,
    },

    /// Table of contents [toc=3]
    TableOfContents {
        depth: Option<u8>,        // Heading depth limit
        document: Option<String>, // Optional document reference
        span: Span,
    },

    /// Inline executable code run@bash(command)
    RunInline {
        script_type: String, // bash, python, etc.
        command: String,
        span: Span,
    },

    /// Block executable code ```run@python
    RunBlock {
        script_type: String, // bash, python, etc.
        content: String,
        span: Span,
    },

    /// Diagram block ```mermaid
    DiagramBlock {
        diagram_type: String, // "mermaid", "graphviz"
        content: String,
        span: Span,
    },

    // ===========================================
    // FOOTNOTES AND REFERENCES
    // ===========================================
    /// Footnote definition [^label]: content
    FootnoteDef {
        label: String,
        content: Vec<Node>,
        span: Span,
    },

    /// Footnote reference [^label]
    FootnoteRef { label: String, span: Span },

    /// Inline footnote ^[content]
    InlineFootnoteRef { content: Vec<Node>, span: Span },

    /// Reference definition [label]: url "title"
    ReferenceDefinition {
        label: String,
        url: String,
        title: Option<String>,
        span: Span,
    },

    /// Reference link [text][label]
    ReferenceLink {
        text: Vec<Node>,
        label: String,
        span: Span,
    },

    /// Reference image ![alt][label]
    ReferenceImage {
        alt: String,
        label: String,
        span: Span,
    },

    // ===========================================
    // HTML ELEMENTS
    // ===========================================
    /// Block HTML <div>...</div>
    HtmlBlock { content: String, span: Span },

    /// Inline HTML <span>text</span>
    InlineHtml { content: String, span: Span },

    // ===========================================
    // ERROR RECOVERY
    // ===========================================
    /// Unknown/unhandled content for error recovery
    Unknown {
        content: String,
        rule: String, // The grammar rule that failed
        span: Span,
    },
}

impl Node {
    /// Get the span of any node
    pub fn span(&self) -> &Span {
        match self {
            Node::Document { span, .. } => span,
            Node::Heading { span, .. } => span,
            Node::Paragraph { span, .. } => span,
            Node::CodeBlock { span, .. } => span,
            Node::MathBlock { span, .. } => span,
            Node::List { span, .. } => span,
            Node::ListItem { span, .. } => span,
            Node::Table { span, .. } => span,
            Node::TableCell { span, .. } => span,
            Node::BlockQuote { span, .. } => span,
            Node::HorizontalRule { span, .. } => span,
            Node::DefinitionList { span, .. } => span,
            Node::DefinitionTerm { span, .. } => span,
            Node::DefinitionDescription { span, .. } => span,
            Node::Text { span, .. } => span,
            Node::Strong { span, .. } => span,
            Node::Emphasis { span, .. } => span,
            Node::Strikethrough { span, .. } => span,
            Node::Highlight { span, .. } => span,
            Node::Superscript { span, .. } => span,
            Node::Subscript { span, .. } => span,
            Node::Code { span, .. } => span,
            Node::MathInline { span, .. } => span,
            Node::Link { span, .. } => span,
            Node::Image { span, .. } => span,
            Node::LineBreak { span, .. } => span,
            Node::EscapedChar { span, .. } => span,
            Node::UserMention { span, .. } => span,
            Node::Bookmark { span, .. } => span,
            Node::TabBlock { span, .. } => span,
            Node::Tab { span, .. } => span,
            Node::Admonition { span, .. } => span,
            Node::TableOfContents { span, .. } => span,
            Node::RunInline { span, .. } => span,
            Node::RunBlock { span, .. } => span,
            Node::DiagramBlock { span, .. } => span,
            Node::FootnoteDef { span, .. } => span,
            Node::FootnoteRef { span, .. } => span,
            Node::InlineFootnoteRef { span, .. } => span,
            Node::ReferenceDefinition { span, .. } => span,
            Node::ReferenceLink { span, .. } => span,
            Node::ReferenceImage { span, .. } => span,
            Node::HtmlBlock { span, .. } => span,
            Node::InlineHtml { span, .. } => span,
            Node::Unknown { span, .. } => span,
        }
    }

    /// Check if this is a block-level element
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            Node::Document { .. }
                | Node::Heading { .. }
                | Node::Paragraph { .. }
                | Node::CodeBlock { .. }
                | Node::MathBlock { .. }
                | Node::List { .. }
                | Node::ListItem { .. }
                | Node::Table { .. }
                | Node::TableCell { .. }
                | Node::BlockQuote { .. }
                | Node::HorizontalRule { .. }
                | Node::DefinitionList { .. }
                | Node::DefinitionTerm { .. }
                | Node::DefinitionDescription { .. }
                | Node::TabBlock { .. }
                | Node::Tab { .. }
                | Node::Admonition { .. }
                | Node::TableOfContents { .. }
                | Node::RunBlock { .. }
                | Node::DiagramBlock { .. }
                | Node::FootnoteDef { .. }
                | Node::HtmlBlock { .. }
        )
    }

    /// Check if this is an inline element
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            Node::Text { .. }
                | Node::Strong { .. }
                | Node::Emphasis { .. }
                | Node::Strikethrough { .. }
                | Node::Highlight { .. }
                | Node::Superscript { .. }
                | Node::Subscript { .. }
                | Node::Code { .. }
                | Node::MathInline { .. }
                | Node::Link { .. }
                | Node::Image { .. }
                | Node::LineBreak { .. }
                | Node::EscapedChar { .. }
                | Node::UserMention { .. }
                | Node::Bookmark { .. }
                | Node::RunInline { .. }
                | Node::FootnoteRef { .. }
                | Node::InlineFootnoteRef { .. }
                | Node::ReferenceLink { .. }
                | Node::ReferenceImage { .. }
                | Node::InlineHtml { .. }
        )
    }

    /// Get child nodes if this node can contain children
    pub fn children(&self) -> Option<&[Node]> {
        match self {
            Node::Document { children, .. } => Some(children),
            Node::Heading { content, .. } => Some(content),
            Node::Paragraph { content, .. } => Some(content),
            Node::List { items, .. } => Some(items),
            Node::ListItem { content, .. } => Some(content),
            Node::BlockQuote { content, .. } => Some(content),
            Node::DefinitionList { items, .. } => Some(items),
            Node::DefinitionTerm { content, .. } => Some(content),
            Node::DefinitionDescription { content, .. } => Some(content),
            Node::Strong { content, .. } => Some(content),
            Node::Emphasis { content, .. } => Some(content),
            Node::Strikethrough { content, .. } => Some(content),
            Node::Highlight { content, .. } => Some(content),
            Node::Superscript { content, .. } => Some(content),
            Node::Subscript { content, .. } => Some(content),
            Node::Link { text, .. } => Some(text),
            Node::TabBlock { tabs, .. } => Some(tabs),
            Node::Tab { content, .. } => Some(content),
            Node::Admonition { content, .. } => Some(content),
            Node::FootnoteDef { content, .. } => Some(content),
            Node::InlineFootnoteRef { content, .. } => Some(content),
            Node::ReferenceLink { text, .. } => Some(text),
            _ => None,
        }
    }

    /// Get mutable child nodes if this node can contain children
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Node::Document { children, .. } => Some(children),
            Node::Heading { content, .. } => Some(content),
            Node::Paragraph { content, .. } => Some(content),
            Node::List { items, .. } => Some(items),
            Node::ListItem { content, .. } => Some(content),
            Node::BlockQuote { content, .. } => Some(content),
            Node::DefinitionList { items, .. } => Some(items),
            Node::DefinitionTerm { content, .. } => Some(content),
            Node::DefinitionDescription { content, .. } => Some(content),
            Node::Strong { content, .. } => Some(content),
            Node::Emphasis { content, .. } => Some(content),
            Node::Strikethrough { content, .. } => Some(content),
            Node::Highlight { content, .. } => Some(content),
            Node::Superscript { content, .. } => Some(content),
            Node::Subscript { content, .. } => Some(content),
            Node::Link { text, .. } => Some(text),
            Node::TabBlock { tabs, .. } => Some(tabs),
            Node::Tab { content, .. } => Some(content),
            Node::Admonition { content, .. } => Some(content),
            Node::FootnoteDef { content, .. } => Some(content),
            Node::InlineFootnoteRef { content, .. } => Some(content),
            Node::ReferenceLink { text, .. } => Some(text),
            _ => None,
        }
    }
}

// Convenient constructor methods
impl Node {
    /// Create a new document node
    pub fn document(children: Vec<Node>, span: Span) -> Self {
        Node::Document { children, span }
    }

    /// Create a new heading node
    pub fn heading(level: u8, content: Vec<Node>, span: Span) -> Self {
        Node::Heading {
            level,
            content,
            span,
        }
    }

    /// Create a new paragraph node
    pub fn paragraph(content: Vec<Node>, span: Span) -> Self {
        Node::Paragraph { content, span }
    }

    /// Create a new text node
    pub fn text(content: String, span: Span) -> Self {
        Node::Text { content, span }
    }

    /// Create a new code block node
    pub fn code_block(language: Option<String>, content: String, span: Span) -> Self {
        Node::CodeBlock {
            language,
            content,
            span,
        }
    }

    /// Create a new inline code node
    pub fn code(content: String, span: Span) -> Self {
        Node::Code { content, span }
    }

    /// Create a new list node
    pub fn list(ordered: bool, items: Vec<Node>, span: Span) -> Self {
        Node::List {
            ordered,
            items,
            span,
        }
    }

    /// Create a new list item node
    pub fn list_item(content: Vec<Node>, checked: Option<bool>, span: Span) -> Self {
        Node::ListItem {
            content,
            checked,
            span,
        }
    }

    /// Create a new link node
    pub fn link(text: Vec<Node>, url: String, title: Option<String>, span: Span) -> Self {
        Node::Link {
            text,
            url,
            title,
            span,
        }
    }

    /// Create a new image node
    pub fn image(alt: String, url: String, title: Option<String>, span: Span) -> Self {
        Node::Image {
            alt,
            url,
            title,
            span,
        }
    }

    /// Create a new strong node
    pub fn strong(content: Vec<Node>, span: Span) -> Self {
        Node::Strong { content, span }
    }

    /// Create a new emphasis node
    pub fn emphasis(content: Vec<Node>, span: Span) -> Self {
        Node::Emphasis { content, span }
    }

    /// Create a new unknown node for error recovery
    pub fn unknown(content: String, rule: String, span: Span) -> Self {
        Node::Unknown {
            content,
            rule,
            span,
        }
    }

    /// Create a new block quote node
    pub fn block_quote(content: Vec<Node>, span: Span) -> Self {
        Node::BlockQuote { content, span }
    }

    /// Create a new horizontal rule node
    pub fn horizontal_rule(span: Span) -> Self {
        Node::HorizontalRule { span }
    }

    /// Create a new math block node
    pub fn math_block(content: String, span: Span) -> Self {
        Node::MathBlock { content, span }
    }

    /// Create a new math inline node
    pub fn math_inline(content: String, span: Span) -> Self {
        Node::MathInline { content, span }
    }

    /// Create a new strikethrough node
    pub fn strikethrough(content: Vec<Node>, span: Span) -> Self {
        Node::Strikethrough { content, span }
    }

    /// Create a new highlight node
    pub fn highlight(content: Vec<Node>, span: Span) -> Self {
        Node::Highlight { content, span }
    }

    /// Create a new superscript node
    pub fn superscript(content: Vec<Node>, span: Span) -> Self {
        Node::Superscript { content, span }
    }

    /// Create a new subscript node
    pub fn subscript(content: Vec<Node>, span: Span) -> Self {
        Node::Subscript { content, span }
    }

    /// Create a new line break node
    pub fn line_break(break_type: LineBreakType, span: Span) -> Self {
        Node::LineBreak { break_type, span }
    }

    /// Create a hard line break node (2+ spaces or backslash + newline)
    pub fn hard_line_break(span: Span) -> Self {
        Self::line_break(LineBreakType::Hard, span)
    }

    /// Create a soft line break node (just newline)
    pub fn soft_line_break(span: Span) -> Self {
        Self::line_break(LineBreakType::Soft, span)
    }

    /// Create a new escaped character node
    pub fn escaped_char(character: char, span: Span) -> Self {
        Node::EscapedChar { character, span }
    }

    /// Create a new table node
    pub fn table(headers: Vec<Node>, rows: Vec<Vec<Node>>, span: Span) -> Self {
        Node::Table {
            headers,
            rows,
            span,
        }
    }

    /// Create a new table cell node
    pub fn table_cell(content: Vec<Node>, alignment: Option<String>, span: Span) -> Self {
        Node::TableCell {
            content,
            alignment,
            span,
        }
    }

    /// Create a new footnote definition node
    pub fn footnote_def(label: String, content: Vec<Node>, span: Span) -> Self {
        Node::FootnoteDef {
            label,
            content,
            span,
        }
    }

    /// Create a new footnote reference node
    pub fn footnote_ref(label: String, span: Span) -> Self {
        Node::FootnoteRef { label, span }
    }

    /// Create a new inline footnote reference node
    pub fn inline_footnote_ref(content: Vec<Node>, span: Span) -> Self {
        Node::InlineFootnoteRef { content, span }
    }

    /// Create a new reference definition node
    pub fn reference_definition(
        label: String,
        url: String,
        title: Option<String>,
        span: Span,
    ) -> Self {
        Node::ReferenceDefinition {
            label,
            url,
            title,
            span,
        }
    }

    /// Create a new reference link node
    pub fn reference_link(text: Vec<Node>, label: String, span: Span) -> Self {
        Node::ReferenceLink { text, label, span }
    }

    /// Create a new reference image node
    pub fn reference_image(alt: String, label: String, span: Span) -> Self {
        Node::ReferenceImage { alt, label, span }
    }

    /// Create a new HTML block node
    pub fn html_block(content: String, span: Span) -> Self {
        Node::HtmlBlock { content, span }
    }

    /// Create a new inline HTML node
    pub fn inline_html(content: String, span: Span) -> Self {
        Node::InlineHtml { content, span }
    }

    /// Create a new definition list node
    pub fn definition_list(items: Vec<Node>, span: Span) -> Self {
        Node::DefinitionList { items, span }
    }

    /// Create a new definition term node
    pub fn definition_term(content: Vec<Node>, span: Span) -> Self {
        Node::DefinitionTerm { content, span }
    }

    /// Create a new definition description node
    pub fn definition_description(content: Vec<Node>, span: Span) -> Self {
        Node::DefinitionDescription { content, span }
    }
}

impl Span {
    /// Create a new span
    pub fn new(start: u32, end: u32, line: u32, column: u32) -> Self {
        Span {
            start,
            end,
            line,
            column,
        }
    }

    /// Create a span from pest Pair
    pub fn from_pest(
        pair: &pest::iterators::Pair<crate::components::marco_engine::grammar::Rule>,
    ) -> Self {
        let span = pair.as_span();
        let (line, column) = span.start_pos().line_col();
        Span {
            start: span.start() as u32,
            end: span.end() as u32,
            line: line as u32,
            column: column as u32,
        }
    }

    /// Check if this span is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the length of this span
    pub fn len(&self) -> u32 {
        self.end - self.start
    }
}
