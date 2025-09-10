//! # Marco AST Node Definitions
//!
//! This module contains the core Abstract Syntax Tree (AST) node definitions for the Marco
//! markup language parser. It provides a comprehensive set of node types that represent
//! all possible elements in Marco documents.
//!
//! ## Overview
//!
//! The Marco AST is designed to be:
//! - **Complete**: Covers all Marco syntax elements including extensions
//! - **Extensible**: Easy to add new node types for future Marco features
//! - **Serializable**: Full serde support for JSON/binary serialization
//! - **Span-aware**: Every node tracks its source location for error reporting
//! - **Type-safe**: Strong typing prevents invalid AST constructions
//!
//! ## Node Categories
//!
//! The AST nodes are organized into logical categories:
//!
//! ### Document Structure
//! - `Document`: Root container for all content
//!
//! ### Block Elements
//! Block-level elements that typically start on new lines and create structural boundaries:
//! - Headings: `Heading`, `SetextHeading`
//! - Content blocks: `Paragraph`, `BlockQuote`
//! - Code blocks: `FencedCodeBlock`, `IndentedCodeBlock`, `CodeBlock`
//! - Math blocks: `MathBlock`, `MathBlockDisplay`
//! - Lists: `List`, `ListItem`, `TaskItem`
//! - Tables: `Table`, `TableHeader`, `TableRow`, `TableCell`
//! - Definition lists: `DefinitionList`, `DefinitionTerm`, `DefinitionDescription`
//! - Horizontal rules: `HorizontalRule`, `ThematicBreak`
//! - Tabs: `TabBlock`, `Tab`, `TabWithMetadata`
//! - Admonitions: `Admonition`, `AdmonitionWithIcon`
//! - References: `FootnoteDefinition`, `ReferenceDefinition`, `LinkReferenceDefinition`
//! - HTML blocks: `BlockHTML`, `HtmlBlock`
//! - Executable blocks: `RunBlock`, `DiagramBlock`
//! - Marco extensions: `TableOfContents`, `Details`
//!
//! ### Inline Elements
//! Inline elements that appear within block content:
//! - Text: `Text`
//! - Formatting: `Strong`, `Emphasis`, `Strikethrough`, `Highlight`, `Mark`
//! - Scripts: `Superscript`, `Subscript`
//! - Code: `Code`, `CodeSpan`
//! - Math: `MathInline`
//! - Links: `Link`, `Image`, `Autolink`, `AutolinkUrl`, `AutolinkEmail`
//! - References: `ReferenceLink`, `ReferenceImage`, `FootnoteRef`, `InlineFootnote`
//! - Line breaks: `LineBreak`, `HardLineBreak`, `SoftLineBreak`
//! - Special: `EscapedChar`, `Emoji`, `Keyboard`
//! - HTML: `InlineHTML`, `HtmlInlineTag`
//! - Marco extensions: `UserMention`, `UserMentionWithMetadata`, `Bookmark`,
//!   `PageTag`, `DocumentReference`, `RunInline`, `Citation`
//!
//! ### Error Recovery
//! - `Unknown`: For unrecognized content during parsing
//!
//! ## Key Features
//!
//! ### Span Tracking
//! Every node includes a `Span` that tracks its position in the source text:
//! - Start/end byte offsets
//! - Line and column numbers
//! - Enables precise error reporting and source mapping
//!
//! ### Children Access
//! Nodes that can contain other nodes provide:
//! - `children()`: Immutable access to child nodes
//! - `children_mut()`: Mutable access to child nodes
//!
//! ### Type Classification
//! Utility methods for AST analysis:
//! - `is_block()`: Returns true for block-level elements
//! - `is_inline()`: Returns true for inline elements
//! - `span()`: Gets the source span for any node
//!
//! ### Constructor Methods
//! Convenient constructors for all node types:
//! ```rust
//! let text = Node::text("Hello", span);
//! let para = Node::paragraph(vec![text], span);
//! let heading = Node::heading(1, vec![title_text], span);
//! ```
//!
//! ## Marco Extensions
//!
//! This AST supports Marco-specific extensions beyond standard Markdown:
//! - User mentions: `@username` with platform support
//! - Bookmarks: `[bookmark:label](path=line)` for code navigation
//! - Table of contents: `[toc=3]` with depth control
//! - Executable code: `run@bash` and code block execution
//! - Diagrams: Mermaid, GraphViz integration
//! - Tab containers: Organized content presentation
//! - Enhanced admonitions: Note, warning, tip blocks with icons
//! - Citations: Academic reference support
//! - Task items: Enhanced checkbox lists
//!
//! ## Usage Example
//!
//! ```rust
//! use marco_engine::ast::{Node, Span};
//!
//! // Create a simple document
//! let span = Span::simple(0, 10);
//! let text = Node::text("Hello, World!", span.clone());
//! let paragraph = Node::paragraph(vec![text], span.clone());
//! let document = Node::document(vec![paragraph], span);
//!
//! // Check node types
//! assert!(document.is_block());
//! assert!(!document.is_inline());
//!
//! // Access children
//! if let Some(children) = document.children() {
//!     println!("Document has {} child nodes", children.len());
//! }
//! ```
//!
//! ## Serialization
//!
//! All nodes implement `Serialize` and `Deserialize` for persistence:
//! ```rust
//! let json = serde_json::to_string(&document)?;
//! let restored: Node = serde_json::from_str(&json)?;
//! ```

use serde::{Deserialize, Serialize};

/// Boxed structures for large Node variants to reduce enum size
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentNode {
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableNode {
    pub headers: Vec<Node>,
    pub rows: Vec<Vec<Node>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabBlockNode {
    pub title: Option<String>,
    pub tabs: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmonitionWithIconNode {
    pub kind: String,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub content: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMentionWithMetadataNode {
    pub username: String,
    pub platform: Option<String>,
    pub display_name: Option<String>,
    pub user_id: Option<String>,
    pub avatar_url: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailsNode {
    pub summary: Vec<Node>,
    pub content: Vec<Node>,
    pub open: bool,
    pub span: Span,
}

/// Position information for source mapping
/// Optimized to use u32 instead of usize for better memory efficiency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

impl Span {
    pub fn new(start: u32, end: u32, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Create a simple span from start and end positions (line/column set to 1)
    pub fn simple(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            line: 1,
            column: 1,
        }
    }

    pub fn empty() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Main AST node enum covering all Marco syntax elements
///
/// This enum is organized into logical categories:
/// - Document Structure: Document root and container nodes
/// - Block Elements: Block-level content (headings, paragraphs, lists, etc.)
/// - Inline Elements: Inline content (text, formatting, links, etc.)
/// - Table Elements: Table-specific components
/// - Marco Extensions: Marco-specific features and extensions
/// - HTML Elements: Raw HTML content and tags
/// - Error Recovery: Nodes for handling parse errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    // ===========================================
    // DOCUMENT STRUCTURE
    // ===========================================
    /// Root document node containing all content
    /// Document root node  
    Document { children: Vec<Node>, span: Span },

    // ===========================================
    // BLOCK ELEMENTS
    // ===========================================
    /// ATX heading (# ## ### etc.)
    Heading {
        level: u8,
        content: Vec<Node>,
        span: Span,
    },

    /// Setext heading (underlined with = or -)
    SetextHeading {
        level: u8, // 1 or 2
        content: Vec<Node>,
        underline_char: char, // '=' or '-'
        span: Span,
    },

    /// Paragraph containing inline content
    Paragraph { content: Vec<Node>, span: Span },

    /// Fenced code block with language and metadata
    FencedCodeBlock {
        language: Option<String>,
        info_string: Option<String>, // Full info string after language
        content: String,
        fence_char: char, // '`' or '~'
        fence_length: u8,
        span: Span,
    },

    /// Indented code block (4+ spaces)
    IndentedCodeBlock { content: String, span: Span },

    /// Legacy code block (for backward compatibility)
    CodeBlock {
        language: Option<String>,
        content: String,
        span: Span,
    },

    /// Math block with display formatting
    MathBlockDisplay {
        content: String,
        delimiter: String, // "$$" or "\\[...\\]"
        span: Span,
    },

    /// Legacy math block (for backward compatibility)
    MathBlock { content: String, span: Span },

    /// Ordered or unordered list
    List {
        ordered: bool,
        items: Vec<Node>,
        span: Span,
    },

    /// List item with optional task list checkbox
    ListItem {
        content: Vec<Node>,
        checked: Option<bool>, // For task lists
        span: Span,
    },

    /// Block quote
    BlockQuote { content: Vec<Node>, span: Span },

    /// Thematic break (--- *** ___)
    ThematicBreak {
        marker: char, // '*', '-', or '_'
        span: Span,
    },

    /// Legacy horizontal rule (for backward compatibility)
    HorizontalRule { span: Span },

    /// Definition list container
    DefinitionList {
        items: Vec<Node>, // Contains DefinitionTerm and DefinitionDescription nodes
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

    /// Strong emphasis (bold) **text** or __text__
    Strong { content: Vec<Node>, span: Span },

    /// Emphasis (italic) *text* or _text_
    Emphasis { content: Vec<Node>, span: Span },

    /// Strikethrough ~~text~~
    Strikethrough { content: Vec<Node>, span: Span },

    /// Highlight ==text==
    Highlight { content: Vec<Node>, span: Span },

    /// Mark/highlight with reason
    Mark {
        content: Vec<Node>,
        reason: Option<String>, // "search", "highlight", etc.
        span: Span,
    },

    /// Superscript ^text^
    Superscript { content: Vec<Node>, span: Span },

    /// Subscript ~text~
    Subscript { content: Vec<Node>, span: Span },

    /// Inline code `code`
    Code { content: String, span: Span },

    /// Code span with backtick count
    CodeSpan {
        content: String,
        backtick_count: u8,
        span: Span,
    },

    /// Inline math $formula$
    MathInline { content: String, span: Span },

    /// Hard line break (two spaces + newline or backslash + newline)
    HardLineBreak { span: Span },

    /// Soft line break (single newline in paragraph)
    SoftLineBreak { span: Span },

    /// Legacy line break (for backward compatibility)
    LineBreak { span: Span },

    /// Escaped character \x
    EscapedChar { character: char, span: Span },

    /// Emoji :name:
    Emoji { name: String, span: Span },

    /// Keyboard input <kbd>key</kbd>
    Keyboard { keys: Vec<String>, span: Span },

    // ===========================================
    // LINK ELEMENTS
    // ===========================================
    /// Inline link [text](url "title")
    Link {
        text: Vec<Node>,
        url: String,
        title: Option<String>,
        span: Span,
    },

    /// Inline image ![alt](url "title")
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        span: Span,
    },

    /// Autolink <url>
    Autolink { url: String, span: Span },

    /// Autolink URL <http://example.com>
    AutolinkUrl { url: String, span: Span },

    /// Autolink email <email@example.com>
    AutolinkEmail { email: String, span: Span },

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

    /// Link reference definition [label]: url "title"
    ReferenceDefinition {
        label: String,
        url: String,
        title: Option<String>,
        span: Span,
    },

    /// Link reference definition (CommonMark compliant)
    LinkReferenceDefinition {
        label: String,
        destination: String,
        title: Option<String>,
        span: Span,
    },

    /// Footnote reference [^label]
    FootnoteRef { label: String, span: Span },

    /// Inline footnote [^note content]
    InlineFootnote { content: Vec<Node>, span: Span },

    /// Footnote definition [^label]: content
    FootnoteDefinition {
        label: String,
        content: Vec<Node>,
        span: Span,
    },

    // ===========================================
    // TABLE ELEMENTS
    // ===========================================
    /// Complete table
    Table {
        headers: Vec<Node>,
        rows: Vec<Vec<Node>>,
        span: Span,
    },

    /// Table header row
    TableHeader { cells: Vec<Node>, span: Span },

    /// Table data row
    TableRow { cells: Vec<Node>, span: Span },

    /// Table cell
    TableCell {
        content: Vec<Node>,
        alignment: Option<String>, // "left", "center", "right"
        span: Span,
    },

    // ===========================================
    // MARCO EXTENSIONS
    // ===========================================
    /// User mention @username
    UserMention {
        username: String,
        platform: Option<String>,
        display_name: Option<String>,
        span: Span,
    },

    /// Enhanced user mention with metadata
    UserMentionWithMetadata {
        username: String,
        platform: Option<String>,
        display_name: Option<String>,
        user_id: Option<String>,
        avatar_url: Option<String>,
        span: Span,
    },

    /// Bookmark [bookmark:label](path=line)
    Bookmark {
        label: String,
        path: String,
        line: Option<u32>,
        span: Span,
    },

    /// Page formatting tag [page=A4]
    PageTag { format: Option<String>, span: Span },

    /// Document reference [@doc](path)
    DocumentReference { path: String, span: Span },

    /// Table of contents [toc=3]
    TableOfContents {
        depth: Option<u8>,
        document: Option<String>,
        span: Span,
    },

    /// Inline executable code `run@bash`
    RunInline {
        script_type: String,
        command: String,
        span: Span,
    },

    /// Block executable code ```run@python
    RunBlock {
        script_type: String,
        content: String,
        span: Span,
    },

    /// Diagram block ```mermaid
    DiagramBlock {
        diagram_type: String, // "mermaid", "graphviz"
        content: String,
        span: Span,
    },

    /// Tab container
    TabBlock {
        title: Option<String>,
        tabs: Vec<Node>, // Contains Tab nodes
        span: Span,
    },

    /// Individual tab
    Tab {
        name: Option<String>,
        content: Vec<Node>,
        span: Span,
    },

    /// Enhanced tab with metadata
    TabWithMetadata {
        name: Option<String>,
        icon: Option<String>,
        active: bool,
        content: Vec<Node>,
        span: Span,
    },

    /// Admonition block :::note
    Admonition {
        kind: String, // "note", "warning", "tip", "danger", "info"
        content: Vec<Node>,
        span: Span,
    },

    /// Enhanced admonition with icon and title
    AdmonitionWithIcon {
        kind: String,
        icon: Option<String>,
        title: Option<String>,
        content: Vec<Node>,
        span: Span,
    },

    /// Task list item with checkbox
    TaskItem {
        checked: bool,
        content: Vec<Node>,
        span: Span,
    },

    /// Citation [key]
    Citation {
        key: String,
        locator: Option<String>,
        span: Span,
    },

    /// Details/summary disclosure element
    Details {
        summary: Vec<Node>,
        content: Vec<Node>,
        open: bool,
        span: Span,
    },

    /// Generic macro system
    Macro {
        name: String,
        arguments: Vec<String>,
        content: Option<Vec<Node>>,
        span: Span,
    },

    // ===========================================
    // HTML ELEMENTS
    // ===========================================
    /// Raw inline HTML
    InlineHTML { content: String, span: Span },

    /// Raw block HTML
    BlockHTML { content: String, span: Span },

    /// HTML block with type information (CommonMark)
    HtmlBlock {
        html_type: u8, // 1-7 as per CommonMark spec
        content: String,
        span: Span,
    },

    /// Structured HTML inline tag
    HtmlInlineTag {
        tag_name: String,
        attributes: Vec<(String, Option<String>)>, // (name, value)
        content: Option<Vec<Node>>,                // For paired tags
        is_self_closing: bool,
        span: Span,
    },

    // ===========================================
    // ERROR RECOVERY
    // ===========================================
    /// Unknown/unhandled content for error recovery
    Unknown {
        content: String,
        rule: String,
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
            Node::DefinitionList { span, .. } => span,
            Node::DefinitionTerm { span, .. } => span,
            Node::DefinitionDescription { span, .. } => span,
            Node::FootnoteDefinition { span, .. } => span,
            Node::ReferenceDefinition { span, .. } => span,
            Node::InlineHTML { span, .. } => span,
            Node::BlockHTML { span, .. } => span,
            Node::Text { span, .. } => span,
            Node::Emphasis { span, .. } => span,
            Node::Strong { span, .. } => span,
            Node::Strikethrough { span, .. } => span,
            Node::Highlight { span, .. } => span,
            Node::Superscript { span, .. } => span,
            Node::Subscript { span, .. } => span,
            Node::Code { span, .. } => span,
            Node::MathInline { span, .. } => span,
            Node::Link { span, .. } => span,
            Node::Image { span, .. } => span,
            Node::Autolink { span, .. } => span,
            Node::ReferenceLink { span, .. } => span,
            Node::ReferenceImage { span, .. } => span,
            Node::FootnoteRef { span, .. } => span,
            Node::InlineFootnote { span, .. } => span,
            Node::Emoji { span, .. } => span,
            Node::LineBreak { span, .. } => span,
            Node::EscapedChar { span, .. } => span,
            Node::Macro { span, .. } => span,
            Node::UserMention { span, .. } => span,
            Node::Bookmark { span, .. } => span,
            Node::PageTag { span, .. } => span,
            Node::DocumentReference { span, .. } => span,
            Node::TableOfContents { span, .. } => span,
            Node::RunInline { span, .. } => span,
            Node::RunBlock { span, .. } => span,
            Node::DiagramBlock { span, .. } => span,
            Node::TabBlock { span, .. } => span,
            Node::Tab { span, .. } => span,
            Node::HorizontalRule { span, .. } => span,
            Node::BlockQuote { span, .. } => span,
            Node::Admonition { span, .. } => span,
            Node::TaskItem { span, .. } => span,
            Node::SetextHeading { span, .. } => span,
            Node::TableHeader { span, .. } => span,
            Node::TableRow { span, .. } => span,
            Node::TableCell { span, .. } => span,
            Node::ThematicBreak { span, .. } => span,
            Node::SoftLineBreak { span, .. } => span,
            Node::HardLineBreak { span, .. } => span,
            Node::HtmlBlock { span, .. } => span,
            Node::FencedCodeBlock { span, .. } => span,
            Node::IndentedCodeBlock { span, .. } => span,
            Node::LinkReferenceDefinition { span, .. } => span,
            Node::MathBlockDisplay { span, .. } => span,
            Node::CodeSpan { span, .. } => span,
            Node::HtmlInlineTag { span, .. } => span,
            Node::AutolinkUrl { span, .. } => span,
            Node::AutolinkEmail { span, .. } => span,
            Node::AdmonitionWithIcon { span, .. } => span,
            Node::TabWithMetadata { span, .. } => span,
            Node::UserMentionWithMetadata { span, .. } => span,
            Node::Citation { span, .. } => span,
            Node::Keyboard { span, .. } => span,
            Node::Mark { span, .. } => span,
            Node::Details { span, .. } => span,
            Node::Unknown { span, .. } => span,
        }
    }

    /// Check if this is a block-level element
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            Node::Document { .. }
                | Node::Heading { .. }
                | Node::SetextHeading { .. }
                | Node::Paragraph { .. }
                | Node::CodeBlock { .. }
                | Node::FencedCodeBlock { .. }
                | Node::IndentedCodeBlock { .. }
                | Node::MathBlock { .. }
                | Node::MathBlockDisplay { .. }
                | Node::List { .. }
                | Node::ListItem { .. }
                | Node::Table { .. }
                | Node::TableHeader { .. }
                | Node::TableRow { .. }
                | Node::TableCell { .. }
                | Node::DefinitionList { .. }
                | Node::DefinitionTerm { .. }
                | Node::DefinitionDescription { .. }
                | Node::FootnoteDefinition { .. }
                | Node::ReferenceDefinition { .. }
                | Node::LinkReferenceDefinition { .. }
                | Node::BlockHTML { .. }
                | Node::HtmlBlock { .. }
                | Node::RunBlock { .. }
                | Node::DiagramBlock { .. }
                | Node::TabBlock { .. }
                | Node::Tab { .. }
                | Node::TabWithMetadata { .. }
                | Node::HorizontalRule { .. }
                | Node::ThematicBreak { .. }
                | Node::BlockQuote { .. }
                | Node::Admonition { .. }
                | Node::AdmonitionWithIcon { .. }
                | Node::TaskItem { .. }
                | Node::TableOfContents { .. }
                | Node::Details { .. }
        )
    }

    /// Check if this is an inline element
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            Node::Text { .. }
                | Node::Emphasis { .. }
                | Node::Strong { .. }
                | Node::Strikethrough { .. }
                | Node::Highlight { .. }
                | Node::Mark { .. }
                | Node::Superscript { .. }
                | Node::Subscript { .. }
                | Node::Code { .. }
                | Node::CodeSpan { .. }
                | Node::MathInline { .. }
                | Node::Link { .. }
                | Node::Image { .. }
                | Node::Autolink { .. }
                | Node::AutolinkUrl { .. }
                | Node::AutolinkEmail { .. }
                | Node::ReferenceLink { .. }
                | Node::ReferenceImage { .. }
                | Node::FootnoteRef { .. }
                | Node::InlineFootnote { .. }
                | Node::Citation { .. }
                | Node::Emoji { .. }
                | Node::LineBreak { .. }
                | Node::HardLineBreak { .. }
                | Node::SoftLineBreak { .. }
                | Node::EscapedChar { .. }
                | Node::InlineHTML { .. }
                | Node::HtmlInlineTag { .. }
                | Node::UserMention { .. }
                | Node::UserMentionWithMetadata { .. }
                | Node::Bookmark { .. }
                | Node::PageTag { .. }
                | Node::DocumentReference { .. }
                | Node::RunInline { .. }
                | Node::Keyboard { .. }
        )
    }

    /// Get children nodes if this node has them
    pub fn children(&self) -> Option<&[Node]> {
        match self {
            Node::Document { children, .. } => Some(children),
            Node::Heading { content, .. } => Some(content),
            Node::SetextHeading { content, .. } => Some(content),
            Node::Paragraph { content, .. } => Some(content),
            Node::List { items, .. } => Some(items),
            Node::ListItem { content, .. } => Some(content),
            Node::TableHeader { cells, .. } => Some(cells),
            Node::TableRow { cells, .. } => Some(cells),
            Node::TableCell { content, .. } => Some(content),
            Node::DefinitionList { items, .. } => Some(items),
            Node::DefinitionTerm { content, .. } => Some(content),
            Node::DefinitionDescription { content, .. } => Some(content),
            Node::FootnoteDefinition { content, .. } => Some(content),
            Node::Emphasis { content, .. } => Some(content),
            Node::Strong { content, .. } => Some(content),
            Node::Strikethrough { content, .. } => Some(content),
            Node::Highlight { content, .. } => Some(content),
            Node::Mark { content, .. } => Some(content),
            Node::Superscript { content, .. } => Some(content),
            Node::Subscript { content, .. } => Some(content),
            Node::Link { text, .. } => Some(text),
            Node::ReferenceLink { text, .. } => Some(text),
            Node::InlineFootnote { content, .. } => Some(content),
            Node::BlockQuote { content, .. } => Some(content),
            Node::Admonition { content, .. } => Some(content),
            Node::AdmonitionWithIcon { content, .. } => Some(content),
            Node::TaskItem { content, .. } => Some(content),
            Node::TabBlock { tabs, .. } => Some(tabs),
            Node::Tab { content, .. } => Some(content),
            Node::TabWithMetadata { content, .. } => Some(content),
            Node::Details { content, .. } => Some(content),
            Node::HtmlInlineTag {
                content: Some(content),
                ..
            } => Some(content),
            Node::Macro {
                content: Some(content),
                ..
            } => Some(content),
            _ => None,
        }
    }

    /// Get mutable children nodes if this node has them
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Node::Document { children, .. } => Some(children),
            Node::Heading { content, .. } => Some(content),
            Node::SetextHeading { content, .. } => Some(content),
            Node::Paragraph { content, .. } => Some(content),
            Node::List { items, .. } => Some(items),
            Node::ListItem { content, .. } => Some(content),
            Node::TableHeader { cells, .. } => Some(cells),
            Node::TableRow { cells, .. } => Some(cells),
            Node::TableCell { content, .. } => Some(content),
            Node::DefinitionList { items, .. } => Some(items),
            Node::DefinitionTerm { content, .. } => Some(content),
            Node::DefinitionDescription { content, .. } => Some(content),
            Node::FootnoteDefinition { content, .. } => Some(content),
            Node::Emphasis { content, .. } => Some(content),
            Node::Strong { content, .. } => Some(content),
            Node::Strikethrough { content, .. } => Some(content),
            Node::Highlight { content, .. } => Some(content),
            Node::Mark { content, .. } => Some(content),
            Node::Superscript { content, .. } => Some(content),
            Node::Subscript { content, .. } => Some(content),
            Node::Link { text, .. } => Some(text),
            Node::ReferenceLink { text, .. } => Some(text),
            Node::InlineFootnote { content, .. } => Some(content),
            Node::BlockQuote { content, .. } => Some(content),
            Node::Admonition { content, .. } => Some(content),
            Node::AdmonitionWithIcon { content, .. } => Some(content),
            Node::TaskItem { content, .. } => Some(content),
            Node::TabBlock { tabs, .. } => Some(tabs),
            Node::Tab { content, .. } => Some(content),
            Node::TabWithMetadata { content, .. } => Some(content),
            Node::Details { content, .. } => Some(content),
            Node::HtmlInlineTag {
                content: Some(content),
                ..
            } => Some(content),
            Node::Macro {
                content: Some(content),
                ..
            } => Some(content),
            _ => None,
        }
    }

    /// Create a text node
    pub fn text(content: impl Into<String>, span: Span) -> Self {
        Node::Text {
            content: content.into(),
            span,
        }
    }

    /// Create a document node
    pub fn document(children: Vec<Node>, span: Span) -> Self {
        Node::Document { children, span }
    }

    /// Create a paragraph node
    pub fn paragraph(content: Vec<Node>, span: Span) -> Self {
        Node::Paragraph { content, span }
    }

    /// Create a heading node
    pub fn heading(level: u8, content: Vec<Node>, span: Span) -> Self {
        Node::Heading {
            level,
            content,
            span,
        }
    }

    /// Create a code block node
    pub fn code_block(language: Option<String>, content: impl Into<String>, span: Span) -> Self {
        Node::CodeBlock {
            language,
            content: content.into(),
            span,
        }
    }

    /// Create a math block node
    pub fn math_block(content: impl Into<String>, span: Span) -> Self {
        Node::MathBlock {
            content: content.into(),
            span,
        }
    }

    /// Create a list node
    pub fn list(ordered: bool, items: Vec<Node>, span: Span) -> Self {
        Node::List {
            ordered,
            items,
            span,
        }
    }

    /// Create a list item node
    pub fn list_item(content: Vec<Node>, checked: Option<bool>, span: Span) -> Self {
        Node::ListItem {
            content,
            checked,
            span,
        }
    }

    /// Create a task item node
    pub fn task_item(checked: bool, content: Vec<Node>, span: Span) -> Self {
        Node::TaskItem {
            checked,
            content,
            span,
        }
    }

    /// Create a math block display node
    pub fn math_block_display(
        content: impl Into<String>,
        delimiter: impl Into<String>,
        span: Span,
    ) -> Self {
        Node::MathBlockDisplay {
            content: content.into(),
            delimiter: delimiter.into(),
            span,
        }
    }

    /// Create a setext heading node
    pub fn setext_heading(level: u8, content: Vec<Node>, underline_char: char, span: Span) -> Self {
        Node::SetextHeading {
            level,
            content,
            underline_char,
            span,
        }
    }

    /// Create a fenced code block node
    pub fn fenced_code_block(
        language: Option<String>,
        info_string: Option<String>,
        content: impl Into<String>,
        fence_char: char,
        fence_length: u8,
        span: Span,
    ) -> Self {
        Node::FencedCodeBlock {
            language,
            info_string,
            content: content.into(),
            fence_char,
            fence_length,
            span,
        }
    }

    /// Create a table cell node
    pub fn table_cell(content: Vec<Node>, alignment: Option<String>, span: Span) -> Self {
        Node::TableCell {
            content,
            alignment,
            span,
        }
    }

    /// Create a thematic break node
    pub fn thematic_break(marker: char, span: Span) -> Self {
        Node::ThematicBreak { marker, span }
    }

    /// Create a hard line break node
    pub fn hard_line_break(span: Span) -> Self {
        Node::HardLineBreak { span }
    }

    /// Create a soft line break node
    pub fn soft_line_break(span: Span) -> Self {
        Node::SoftLineBreak { span }
    }

    /// Create a code span node
    pub fn code_span(content: impl Into<String>, backtick_count: u8, span: Span) -> Self {
        Node::CodeSpan {
            content: content.into(),
            backtick_count,
            span,
        }
    }

    /// Create an autolink URL node
    pub fn autolink_url(url: impl Into<String>, span: Span) -> Self {
        Node::AutolinkUrl {
            url: url.into(),
            span,
        }
    }

    /// Create an autolink email node
    pub fn autolink_email(email: impl Into<String>, span: Span) -> Self {
        Node::AutolinkEmail {
            email: email.into(),
            span,
        }
    }

    /// Create a mark/highlight node with reason
    pub fn mark(content: Vec<Node>, reason: Option<String>, span: Span) -> Self {
        Node::Mark {
            content,
            reason,
            span,
        }
    }

    /// Create a citation node
    pub fn citation(key: impl Into<String>, locator: Option<String>, span: Span) -> Self {
        Node::Citation {
            key: key.into(),
            locator,
            span,
        }
    }

    /// Create a keyboard input node
    pub fn keyboard(keys: Vec<String>, span: Span) -> Self {
        Node::Keyboard { keys, span }
    }

    /// Create a details/summary node
    pub fn details(summary: Vec<Node>, content: Vec<Node>, open: bool, span: Span) -> Self {
        Node::Details {
            summary,
            content,
            open,
            span,
        }
    }

    /// Create a strong (bold) node
    pub fn strong(content: Vec<Node>, span: Span) -> Self {
        Node::Strong { content, span }
    }

    /// Create an emphasis (italic) node
    pub fn emphasis(content: Vec<Node>, span: Span) -> Self {
        Node::Emphasis { content, span }
    }

    /// Create a strikethrough node
    pub fn strikethrough(content: Vec<Node>, span: Span) -> Self {
        Node::Strikethrough { content, span }
    }

    /// Create a highlight node
    pub fn highlight(content: Vec<Node>, span: Span) -> Self {
        Node::Highlight { content, span }
    }

    /// Create a superscript node
    pub fn superscript(content: Vec<Node>, span: Span) -> Self {
        Node::Superscript { content, span }
    }

    /// Create a subscript node
    pub fn subscript(content: Vec<Node>, span: Span) -> Self {
        Node::Subscript { content, span }
    }

    /// Create a link node
    pub fn link(
        text: Vec<Node>,
        url: impl Into<String>,
        title: Option<String>,
        span: Span,
    ) -> Self {
        Node::Link {
            text,
            url: url.into(),
            title,
            span,
        }
    }

    /// Create an image node
    pub fn image(
        alt: impl Into<String>,
        url: impl Into<String>,
        title: Option<String>,
        span: Span,
    ) -> Self {
        Node::Image {
            alt: alt.into(),
            url: url.into(),
            title,
            span,
        }
    }

    /// Create a table node
    pub fn table(headers: Vec<Node>, rows: Vec<Vec<Node>>, span: Span) -> Self {
        Node::Table {
            headers,
            rows,
            span,
        }
    }

    /// Create a table row node
    pub fn table_row(cells: Vec<Node>, span: Span) -> Self {
        Node::TableRow { cells, span }
    }

    /// Create a table header node
    pub fn table_header(cells: Vec<Node>, span: Span) -> Self {
        Node::TableHeader { cells, span }
    }

    /// Create a blockquote node
    pub fn blockquote(content: Vec<Node>, span: Span) -> Self {
        Node::BlockQuote { content, span }
    }

    /// Create an admonition node
    pub fn admonition(kind: impl Into<String>, content: Vec<Node>, span: Span) -> Self {
        Node::Admonition {
            kind: kind.into(),
            content,
            span,
        }
    }

    /// Create a user mention node
    pub fn user_mention(
        username: impl Into<String>,
        platform: Option<String>,
        display_name: Option<String>,
        span: Span,
    ) -> Self {
        Node::UserMention {
            username: username.into(),
            platform,
            display_name,
            span,
        }
    }

    /// Create an enhanced user mention node with metadata
    pub fn user_mention_with_metadata(
        username: impl Into<String>,
        platform: Option<String>,
        display_name: Option<String>,
        user_id: Option<String>,
        avatar_url: Option<String>,
        span: Span,
    ) -> Self {
        Node::UserMentionWithMetadata {
            username: username.into(),
            platform,
            display_name,
            user_id,
            avatar_url,
            span,
        }
    }

    /// Create a bookmark node
    pub fn bookmark(
        label: impl Into<String>,
        path: impl Into<String>,
        line: Option<u32>,
        span: Span,
    ) -> Self {
        Node::Bookmark {
            label: label.into(),
            path: path.into(),
            line,
            span,
        }
    }

    /// Create a table of contents node
    pub fn table_of_contents(depth: Option<u8>, document: Option<String>, span: Span) -> Self {
        Node::TableOfContents {
            depth,
            document,
            span,
        }
    }

    /// Create a tab block node
    pub fn tab_block(title: Option<String>, tabs: Vec<Node>, span: Span) -> Self {
        Node::TabBlock { title, tabs, span }
    }

    /// Create a tab node
    pub fn tab(name: Option<String>, content: Vec<Node>, span: Span) -> Self {
        Node::Tab {
            name,
            content,
            span,
        }
    }

    /// Create a run inline node
    pub fn run_inline(
        script_type: impl Into<String>,
        command: impl Into<String>,
        span: Span,
    ) -> Self {
        Node::RunInline {
            script_type: script_type.into(),
            command: command.into(),
            span,
        }
    }

    /// Create a run block node
    pub fn run_block(
        script_type: impl Into<String>,
        content: impl Into<String>,
        span: Span,
    ) -> Self {
        Node::RunBlock {
            script_type: script_type.into(),
            content: content.into(),
            span,
        }
    }

    /// Create a diagram block node
    pub fn diagram_block(
        diagram_type: impl Into<String>,
        content: impl Into<String>,
        span: Span,
    ) -> Self {
        Node::DiagramBlock {
            diagram_type: diagram_type.into(),
            content: content.into(),
            span,
        }
    }

    /// Create a horizontal rule node
    pub fn horizontal_rule(span: Span) -> Self {
        Node::HorizontalRule { span }
    }

    /// Create a math inline node
    pub fn math_inline(content: impl Into<String>, span: Span) -> Self {
        Node::MathInline {
            content: content.into(),
            span,
        }
    }

    /// Create a code node
    pub fn code(content: impl Into<String>, span: Span) -> Self {
        Node::Code {
            content: content.into(),
            span,
        }
    }

    /// Create an unknown node for error recovery
    pub fn unknown(content: impl Into<String>, rule: impl Into<String>, span: Span) -> Self {
        Node::Unknown {
            content: content.into(),
            rule: rule.into(),
            span,
        }
    }

    /// Create a definition list node
    pub fn definition_list(items: Vec<Node>, span: Span) -> Self {
        Node::DefinitionList { items, span }
    }

    /// Create a definition term node
    pub fn definition_term(content: Vec<Node>, span: Span) -> Self {
        Node::DefinitionTerm { content, span }
    }

    /// Create a definition description node
    pub fn definition_description(content: Vec<Node>, span: Span) -> Self {
        Node::DefinitionDescription { content, span }
    }

    /// Create a block HTML node
    pub fn block_html(content: impl Into<String>, span: Span) -> Self {
        Node::BlockHTML {
            content: content.into(),
            span,
        }
    }

    /// Create an emoji node
    pub fn emoji(name: impl Into<String>, span: Span) -> Self {
        Node::Emoji {
            name: name.into(),
            span,
        }
    }

    /// Create a line break node
    pub fn line_break(span: Span) -> Self {
        Node::LineBreak { span }
    }

    /// Create an inline HTML node
    pub fn inline_html(content: impl Into<String>, span: Span) -> Self {
        Node::InlineHTML {
            content: content.into(),
            span,
        }
    }

    /// Create an autolink node
    pub fn autolink(url: impl Into<String>, span: Span) -> Self {
        Node::Autolink {
            url: url.into(),
            span,
        }
    }

    /// Create a reference link node
    pub fn reference_link(text: Vec<Node>, label: impl Into<String>, span: Span) -> Self {
        Node::ReferenceLink {
            text,
            label: label.into(),
            span,
        }
    }

    /// Create a reference image node
    pub fn reference_image(alt: impl Into<String>, label: impl Into<String>, span: Span) -> Self {
        Node::ReferenceImage {
            alt: alt.into(),
            label: label.into(),
            span,
        }
    }

    /// Create a reference definition node
    pub fn reference_definition(
        label: impl Into<String>,
        url: impl Into<String>,
        title: Option<String>,
        span: Span,
    ) -> Self {
        Node::ReferenceDefinition {
            label: label.into(),
            url: url.into(),
            title,
            span,
        }
    }

    /// Create a footnote reference node
    pub fn footnote_ref(label: impl Into<String>, span: Span) -> Self {
        Node::FootnoteRef {
            label: label.into(),
            span,
        }
    }

    /// Create an inline footnote node
    pub fn inline_footnote(content: Vec<Node>, span: Span) -> Self {
        Node::InlineFootnote { content, span }
    }

    /// Create a footnote definition node
    pub fn footnote_definition(label: impl Into<String>, content: Vec<Node>, span: Span) -> Self {
        Node::FootnoteDefinition {
            label: label.into(),
            content,
            span,
        }
    }

    /// Create a page tag node
    pub fn page_tag(format: Option<String>, span: Span) -> Self {
        Node::PageTag { format, span }
    }

    /// Create a document reference node
    pub fn document_reference(path: impl Into<String>, span: Span) -> Self {
        Node::DocumentReference {
            path: path.into(),
            span,
        }
    }

    /// Create an escaped character node
    pub fn escaped_char(character: char, span: Span) -> Self {
        Node::EscapedChar { character, span }
    }

    /// Create an indented code block node
    pub fn indented_code_block(content: impl Into<String>, span: Span) -> Self {
        Node::IndentedCodeBlock {
            content: content.into(),
            span,
        }
    }

    /// Create an HTML block node with type
    pub fn html_block(html_type: u8, content: impl Into<String>, span: Span) -> Self {
        Node::HtmlBlock {
            html_type,
            content: content.into(),
            span,
        }
    }

    /// Create an HTML inline tag node
    pub fn html_inline_tag(
        tag_name: impl Into<String>,
        attributes: Vec<(String, Option<String>)>,
        content: Option<Vec<Node>>,
        is_self_closing: bool,
        span: Span,
    ) -> Self {
        Node::HtmlInlineTag {
            tag_name: tag_name.into(),
            attributes,
            content,
            is_self_closing,
            span,
        }
    }

    /// Create a link reference definition node (CommonMark)
    pub fn link_reference_definition(
        label: impl Into<String>,
        destination: impl Into<String>,
        title: Option<String>,
        span: Span,
    ) -> Self {
        Node::LinkReferenceDefinition {
            label: label.into(),
            destination: destination.into(),
            title,
            span,
        }
    }

    /// Create an enhanced admonition with icon node
    pub fn admonition_with_icon(
        kind: impl Into<String>,
        icon: Option<String>,
        title: Option<String>,
        content: Vec<Node>,
        span: Span,
    ) -> Self {
        Node::AdmonitionWithIcon {
            kind: kind.into(),
            icon,
            title,
            content,
            span,
        }
    }

    /// Create a tab with metadata node
    pub fn tab_with_metadata(
        name: Option<String>,
        icon: Option<String>,
        active: bool,
        content: Vec<Node>,
        span: Span,
    ) -> Self {
        Node::TabWithMetadata {
            name,
            icon,
            active,
            content,
            span,
        }
    }

    /// Create a macro node
    pub fn macro_node(
        name: impl Into<String>,
        arguments: Vec<String>,
        content: Option<Vec<Node>>,
        span: Span,
    ) -> Self {
        Node::Macro {
            name: name.into(),
            arguments,
            content,
            span,
        }
    }
}
