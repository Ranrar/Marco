/// Comprehensive Markdown Abstract Syntax Tree (AST) Implementation
/// 
/// This AST covers all standard Markdown elements including:
/// - Basic syntax elements (headings, paragraphs, lists, etc.)
/// - Extended syntax elements (tables, strikethrough, task lists, etc.)
/// - Hack elements (underline, color, comments, etc.)
/// - HTML elements that can be embedded in Markdown
/// 
/// The AST is designed to be extensible and comprehensive, capturing
/// the full range of Markdown functionality across different flavors.

use std::collections::HashMap;

/// Root document node containing all block-level elements
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// List of all block-level elements in the document
    pub blocks: Vec<Block>,
    /// Optional document metadata (front matter, etc.)
    pub metadata: Option<HashMap<String, String>>,
}

/// Block-level elements that can appear at the top level of a document
/// These elements typically start on a new line and create structural divisions
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Headings from level 1 to 6 (#, ##, ###, ####, #####, ######)
    /// Example: # This is a heading
    Heading {
        level: u8,        // 1-6
        content: Vec<Inline>,
        id: Option<String>, // Optional heading ID for linking
    },
    
    /// Regular paragraph containing inline elements
    /// Example: This is a paragraph with *emphasis*.
    Paragraph {
        content: Vec<Inline>,
    },
    
    /// Blockquote for quoted text (> syntax)
    /// Example: > This is a quote
    /// Can contain nested blocks
    Blockquote {
        content: Vec<Block>,
    },
    
    /// Unordered list (-, *, + syntax)
    /// Example: - Item 1
    ///          - Item 2
    UnorderedList {
        items: Vec<ListItem>,
        tight: bool, // Whether list items are tight (no blank lines)
    },
    
    /// Ordered list (1. syntax)
    /// Example: 1. First item
    ///          2. Second item
    OrderedList {
        items: Vec<ListItem>,
        start: u32,  // Starting number
        tight: bool, // Whether list items are tight
    },
    
    /// Task list (- [ ] and - [x] syntax)
    /// Example: - [x] Completed task
    ///          - [ ] Incomplete task
    TaskList {
        items: Vec<TaskItem>,
    },
    
    /// Code block with optional language specification
    /// Example: ```rust
    ///          let x = 5;
    ///          ```
    CodeBlock {
        language: Option<String>,
        code: String,
        info: Option<String>, // Additional info after language
    },
    
    /// Fenced code block (``` or ~~~ syntax)
    /// Similar to CodeBlock but explicitly fenced
    FencedCodeBlock {
        fence_char: char,    // '`' or '~'
        fence_length: usize, // Number of fence characters
        language: Option<String>,
        code: String,
        info: Option<String>,
    },
    
    /// HTML block that can contain raw HTML
    /// Example: <div>Raw HTML content</div>
    HtmlBlock {
        content: String,
    },
    
    /// Table with headers and rows
    /// Example: | Header 1 | Header 2 |
    ///          |----------|----------|
    ///          | Cell 1   | Cell 2   |
    Table {
        headers: Vec<TableCell>,
        alignments: Vec<TableAlignment>,
        rows: Vec<Vec<TableCell>>,
    },
    
    /// Thematic break (horizontal rule)
    /// Example: --- or *** or ___
    ThematicBreak,
    
    /// Line break (two spaces at end of line or backslash)
    /// Example: Line 1  
    ///          Line 2
    LineBreak,
    
    /// Definition list (not standard Markdown but supported by some parsers)
    /// Example: Term
    ///          : Definition
    DefinitionList {
        items: Vec<DefinitionItem>,
    },
    
    /// Footnote definition
    /// Example: [^1]: This is a footnote
    FootnoteDefinition {
        label: String,
        content: Vec<Block>,
    },
    
    /// Math block (LaTeX-style math)
    /// Example: $$
    ///          x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
    ///          $$
    MathBlock {
        content: String,
    },
    
    /// Admonition/callout block (extensions like MkDocs)
    /// Example: !!! warning "Title"
    ///              Content here
    Admonition {
        kind: String,           // warning, note, tip, etc.
        title: Option<String>,
        content: Vec<Block>,
    },
    
    /// Comment block (not rendered in output)
    /// Example: [comment]: # (This is a comment)
    Comment {
        content: String,
    },
    
    /// Generic container for custom block types
    /// Allows for extension with custom block elements
    Custom {
        name: String,
        attributes: HashMap<String, String>,
        content: Vec<Block>,
    },
}

/// Inline elements that can appear within block elements
/// These elements are typically part of a paragraph or other block content
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text content
    /// Example: This is plain text
    Text {
        content: String,
    },
    
    /// Emphasis (italic text using * or _)
    /// Example: *italic* or _italic_
    Emphasis {
        content: Vec<Inline>,
    },
    
    /// Strong emphasis (bold text using ** or __)
    /// Example: **bold** or __bold__
    Strong {
        content: Vec<Inline>,
    },
    
    /// Strikethrough text (~~text~~)
    /// Example: ~~strikethrough~~
    Strikethrough {
        content: Vec<Inline>,
    },
    
    /// Underlined text (HTML <ins> tag or <u> tag)
    /// Example: <ins>underlined</ins>
    Underline {
        content: Vec<Inline>,
    },
    
    /// Superscript text (^text^ or <sup>text</sup>)
    /// Example: E=mc^2^
    Superscript {
        content: Vec<Inline>,
    },
    
    /// Subscript text (~text~ or <sub>text</sub>)
    /// Example: H~2~O
    Subscript {
        content: Vec<Inline>,
    },
    
    /// Inline code (backticks)
    /// Example: `code`
    Code {
        content: String,
    },
    
    /// Link with URL and optional title
    /// Example: [text](url "title")
    Link {
        text: Vec<Inline>,
        url: String,
        title: Option<String>,
        target: Option<String>, // _blank, _self, etc.
    },
    
    /// Reference link
    /// Example: [text][ref]
    ReferenceLink {
        text: Vec<Inline>,
        reference: String,
    },
    
    /// Autolink (automatic link detection)
    /// Example: <https://example.com>
    AutoLink {
        url: String,
        link_type: AutoLinkType,
    },
    
    /// Image with alt text, URL, and optional title
    /// Example: ![alt](url "title")
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        width: Option<String>,  // HTML width attribute
        height: Option<String>, // HTML height attribute
    },
    
    /// Reference image
    /// Example: ![alt][ref]
    ReferenceImage {
        alt: String,
        reference: String,
    },
    
    /// Footnote reference
    /// Example: [^1]
    FootnoteReference {
        label: String,
    },
    
    /// Inline HTML
    /// Example: <span>HTML content</span>
    Html {
        content: String,
    },
    
    /// Line break (soft break)
    /// Example: Two spaces at end of line
    SoftBreak,
    
    /// Hard break (forced line break)
    /// Example: Backslash at end of line\
    HardBreak,
    
    /// Inline math (LaTeX-style)
    /// Example: $x^2 + y^2 = z^2$
    InlineMath {
        content: String,
    },
    
    /// Emoji (GitHub-style or Unicode)
    /// Example: :smile: or 😀
    Emoji {
        shortcode: Option<String>, // :smile:
        unicode: String,           // 😀
    },
    
    /// Highlight/mark text (==text==)
    /// Example: ==highlighted==
    Highlight {
        content: Vec<Inline>,
    },
    
    /// Keyboard input (<kbd>text</kbd>)
    /// Example: <kbd>Ctrl</kbd>+<kbd>C</kbd>
    Keyboard {
        content: Vec<Inline>,
    },
    
    /// Colored text (HTML style attribute or font tag)
    /// Example: <font color="red">text</font>
    ColoredText {
        content: Vec<Inline>,
        color: String, // Color name or hex code
    },
    
    /// Text with custom CSS styling
    /// Example: <span style="color: blue;">text</span>
    StyledText {
        content: Vec<Inline>,
        style: String, // CSS style string
    },
    
    /// Custom inline element for extensions
    Custom {
        name: String,
        attributes: HashMap<String, String>,
        content: Vec<Inline>,
    },
}

/// List item that can contain blocks
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// Content of the list item (can be multiple blocks)
    pub content: Vec<Block>,
    /// Whether this item is tight (no blank lines around it)
    pub tight: bool,
}

/// Task list item with checkbox
#[derive(Debug, Clone, PartialEq)]
pub struct TaskItem {
    /// Whether the task is completed
    pub checked: bool,
    /// Content of the task item
    pub content: Vec<Block>,
}

/// Table cell with alignment and content
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    /// Cell content (inline elements)
    pub content: Vec<Inline>,
    /// Cell alignment (inherited from column)
    pub alignment: TableAlignment,
    /// Colspan for merged cells
    pub colspan: Option<u32>,
    /// Rowspan for merged cells
    pub rowspan: Option<u32>,
}

/// Table column alignment
#[derive(Debug, Clone, PartialEq)]
pub enum TableAlignment {
    /// Left-aligned (default)
    Left,
    /// Center-aligned
    Center,
    /// Right-aligned
    Right,
    /// No specific alignment
    None,
}

/// Definition list item
#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionItem {
    /// Term being defined
    pub term: Vec<Inline>,
    /// Definition(s) of the term
    pub definitions: Vec<Vec<Block>>,
}

/// Type of automatic link
#[derive(Debug, Clone, PartialEq)]
pub enum AutoLinkType {
    /// URL link (http://, https://, ftp://, etc.)
    Url,
    /// Email link
    Email,
}

/// Link reference definition
/// Example: [ref]: url "title"
#[derive(Debug, Clone, PartialEq)]
pub struct LinkReference {
    /// Reference label
    pub label: String,
    /// URL destination
    pub url: String,
    /// Optional title
    pub title: Option<String>,
}

/// Document with references
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownDocument {
    /// Main document content
    pub document: Document,
    /// Link reference definitions
    pub references: HashMap<String, LinkReference>,
}

/// Utility trait for AST traversal
pub trait AstVisitor {
    fn visit_document(&mut self, doc: &Document);
    fn visit_block(&mut self, block: &Block);
    fn visit_inline(&mut self, inline: &Inline);
}

/// Utility trait for AST transformation
pub trait AstTransformer {
    fn transform_document(&mut self, doc: Document) -> Document;
    fn transform_block(&mut self, block: Block) -> Block;
    fn transform_inline(&mut self, inline: Inline) -> Inline;
}

/// Helper functions for AST construction
impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Document {
            blocks: Vec::new(),
            metadata: None,
        }
    }
    
    /// Add a block to the document
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

impl Block {
    /// Create a new paragraph block
    pub fn paragraph(content: Vec<Inline>) -> Self {
        Block::Paragraph { content }
    }
    
    /// Create a new heading block
    pub fn heading(level: u8, content: Vec<Inline>) -> Self {
        Block::Heading {
            level,
            content,
            id: None,
        }
    }
    
    /// Create a new code block
    pub fn code_block(code: String, language: Option<String>) -> Self {
        Block::CodeBlock {
            language,
            code,
            info: None,
        }
    }
}

impl Inline {
    /// Create plain text inline element
    pub fn text(content: String) -> Self {
        Inline::Text { content }
    }
    
    /// Create emphasis inline element
    pub fn emphasis(content: Vec<Inline>) -> Self {
        Inline::Emphasis { content }
    }
    
    /// Create strong inline element
    pub fn strong(content: Vec<Inline>) -> Self {
        Inline::Strong { content }
    }
    
    /// Create code inline element
    pub fn code(content: String) -> Self {
        Inline::Code { content }
    }
    
    /// Create link inline element
    pub fn link(text: Vec<Inline>, url: String, title: Option<String>) -> Self {
        Inline::Link {
            text,
            url,
            title,
            target: None,
        }
    }
}

/// Constants for common Markdown elements
pub mod constants {
    /// Maximum heading level in Markdown
    pub const MAX_HEADING_LEVEL: u8 = 6;
    
    /// Default fence length for code blocks
    pub const DEFAULT_FENCE_LENGTH: usize = 3;
    
    /// Common emoji shortcodes
    pub const COMMON_EMOJI: &[(&str, &str)] = &[
        (":smile:", "😀"),
        (":heart:", "❤️"),
        (":thumbsup:", "👍"),
        (":warning:", "⚠️"),
        (":memo:", "📝"),
        (":bulb:", "💡"),
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert!(doc.blocks.is_empty());
        assert!(doc.metadata.is_none());
    }

    #[test]
    fn test_block_creation() {
        let text = vec![Inline::text("Hello world".to_string())];
        let para = Block::paragraph(text);
        
        match para {
            Block::Paragraph { content } => {
                assert_eq!(content.len(), 1);
            }
            _ => panic!("Expected paragraph block"),
        }
    }

    #[test]
    fn test_inline_creation() {
        let text = Inline::text("Hello".to_string());
        let code = Inline::code("println!()".to_string());
        
        match text {
            Inline::Text { content } => assert_eq!(content, "Hello"),
            _ => panic!("Expected text inline"),
        }
        
        match code {
            Inline::Code { content } => assert_eq!(content, "println!()"),
            _ => panic!("Expected code inline"),
        }
    }
}
