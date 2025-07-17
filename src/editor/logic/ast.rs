use std::collections::HashMap;

/// Complete Markdown Abstract Syntax Tree (AST) representation
/// This AST covers all basic and extended Markdown syntax elements
/// as documented in the Markdown Guide (https://www.markdownguide.org/)

/// Root document node containing all block-level elements
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// List of all block-level elements in the document
    pub blocks: Vec<Block>,
    /// Optional metadata for the document
    pub metadata: Option<HashMap<String, String>>,
}

/// Block-level elements that can appear at the top level of a document
/// These elements typically start on a new line and may contain other elements
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// # Heading - ATX-style heading with level 1-6
    /// Example: # Main Title, ## Subtitle, ### Section
    Heading {
        /// Heading level (1-6, corresponding to h1-h6 in HTML)
        level: u8,
        /// Inline content of the heading
        content: Vec<Inline>,
        /// Optional custom ID for the heading (extended syntax)
        /// Example: ### My Heading {#custom-id}
        id: Option<String>,
    },

    /// Alternative heading style using underlines
    /// Example: Title\n===== or Subtitle\n-----
    SetextHeading {
        /// Heading level (1 for =, 2 for -)
        level: u8,
        /// Inline content of the heading
        content: Vec<Inline>,
    },

    /// Regular paragraph containing inline text
    /// Paragraphs are separated by blank lines
    Paragraph {
        /// Inline content of the paragraph
        content: Vec<Inline>,
    },

    /// > Blockquote - Quoted text block
    /// Can contain multiple paragraphs and be nested
    /// Example: > This is a blockquote
    Blockquote {
        /// Block content within the blockquote
        content: Vec<Block>,
    },

    /// Code block - Preformatted code
    /// Can be indented (4 spaces/1 tab) or fenced (```/~~~)
    CodeBlock {
        /// Programming language for syntax highlighting (optional)
        language: Option<String>,
        /// Raw code content
        code: String,
        /// Additional info string (everything after language identifier)
        info: Option<String>,
        /// Whether this is a fenced code block (true) or indented (false)
        fenced: bool,
    },

    /// Horizontal rule/thematic break
    /// Example: ---, ***, ___
    HorizontalRule,

    /// Unordered list (bullet points)
    /// Example: - Item 1\n- Item 2
    UnorderedList {
        /// List items
        items: Vec<ListItem>,
        /// Marker character used (-, *, or +)
        marker: char,
        /// Tightness of the list (tight lists have no blank lines between items)
        tight: bool,
    },

    /// Ordered list (numbered)
    /// Example: 1. First\n2. Second
    OrderedList {
        /// List items
        items: Vec<ListItem>,
        /// Starting number for the list
        start: u32,
        /// Delimiter used after numbers (. or ))
        delimiter: char,
        /// Tightness of the list
        tight: bool,
    },

    /// Task list (checkboxes) - Extended syntax
    /// Example: - [x] Completed task\n- [ ] Incomplete task
    TaskList {
        /// Task list items
        items: Vec<TaskItem>,
        /// Tightness of the list
        tight: bool,
    },

    /// Definition list - Extended syntax
    /// Example: Term\n: Definition
    DefinitionList {
        /// Definition list items
        items: Vec<DefinitionItem>,
    },

    /// Table - Extended syntax
    /// Example: | Header 1 | Header 2 |\n|----------|----------|
    Table {
        /// Table headers
        headers: Vec<TableCell>,
        /// Column alignments (Left, Right, Center, None)
        alignments: Vec<Alignment>,
        /// Table rows
        rows: Vec<Vec<TableCell>>,
    },

    /// Raw HTML block
    /// HTML tags that are treated as block-level elements
    HtmlBlock {
        /// Raw HTML content
        html: String,
    },

    /// Footnote definition - Extended syntax
    /// Example: [^1]: This is a footnote definition
    FootnoteDefinition {
        /// Footnote identifier
        label: String,
        /// Content of the footnote
        content: Vec<Block>,
    },

    /// Math block (LaTeX-style math) - Extended syntax
    /// Example: $$\n x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}\n$$
    MathBlock {
        /// LaTeX math expression
        math: String,
    },
}

/// Individual list item within ordered or unordered lists
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// Block content of the list item
    content: Vec<Block>,
    /// Indentation level for nested lists
    indent: u32,
}

/// Task list item with checkbox state
#[derive(Debug, Clone, PartialEq)]
pub struct TaskItem {
    /// Whether the task is completed (checked)
    checked: bool,
    /// Inline content of the task
    content: Vec<Inline>,
    /// Indentation level for nested tasks
    indent: u32,
}

/// Definition list item with term and definitions
#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionItem {
    /// Term being defined
    term: Vec<Inline>,
    /// One or more definitions for the term
    definitions: Vec<Vec<Block>>,
}

/// Table cell content
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    /// Inline content of the cell
    content: Vec<Inline>,
}

/// Column alignment in tables
#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {
    /// Left-aligned column
    Left,
    /// Right-aligned column
    Right,
    /// Center-aligned column
    Center,
    /// No specific alignment
    None,
}

/// Inline elements that can appear within block elements
/// These elements are part of the text flow and don't create new lines
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text content
    Text {
        /// The text content
        content: String,
    },

    /// *emphasis* or _emphasis_ - Italic text
    Emphasis {
        /// Inline content to emphasize
        content: Vec<Inline>,
    },

    /// **strong** or __strong__ - Bold text
    Strong {
        /// Inline content to make strong
        content: Vec<Inline>,
    },

    /// ***strong emphasis*** - Bold and italic text
    StrongEmphasis {
        /// Inline content to make strong and emphasized
        content: Vec<Inline>,
    },

    /// ~~strikethrough~~ - Crossed out text (Extended syntax)
    Strikethrough {
        /// Inline content to strike through
        content: Vec<Inline>,
    },

    /// ==highlight== - Highlighted text (Extended syntax)
    Highlight {
        /// Inline content to highlight
        content: Vec<Inline>,
    },

    /// `code` - Inline code span
    Code {
        /// Code content
        code: String,
    },

    /// [text](url "title") - Hyperlink
    Link {
        /// Link text (inline content)
        text: Vec<Inline>,
        /// URL or reference
        url: String,
        /// Optional title for the link
        title: Option<String>,
        /// Link type (inline, reference, autolink, email)
        link_type: LinkType,
    },

    /// ![alt](url "title") - Image
    Image {
        /// Alt text for the image
        alt: String,
        /// Image URL or path
        url: String,
        /// Optional title for the image
        title: Option<String>,
    },

    /// Line break - Hard line break (two spaces + newline or <br>)
    LineBreak,

    /// Soft line break - Normal line break within paragraph
    SoftBreak,

    /// Raw HTML inline element
    /// Example: <em>text</em>, <span>content</span>
    HtmlInline {
        /// Raw HTML content
        html: String,
    },

    /// Footnote reference - Extended syntax
    /// Example: [^1], [^note]
    FootnoteReference {
        /// Footnote identifier
        label: String,
    },

    /// Math inline (LaTeX-style math) - Extended syntax
    /// Example: $x = y + z$
    MathInline {
        /// LaTeX math expression
        math: String,
    },

    /// Subscript text - Extended syntax
    /// Example: H~2~O
    Subscript {
        /// Inline content to subscript
        content: Vec<Inline>,
    },

    /// Superscript text - Extended syntax
    /// Example: X^2^
    Superscript {
        /// Inline content to superscript
        content: Vec<Inline>,
    },

    /// Emoji - Extended syntax
    /// Can be Unicode emoji or shortcode like :joy:
    Emoji {
        /// Emoji content (Unicode character or shortcode)
        content: String,
        /// Whether this is a shortcode (true) or Unicode (false)
        is_shortcode: bool,
    },

    /// Escaped character
    /// Example: \* (escaped asterisk)
    Escaped {
        /// The escaped character
        character: char,
    },

    /// Autolink - URLs that are automatically converted to links
    /// Example: https://example.com becomes a clickable link
    Autolink {
        /// The URL that was auto-linked
        url: String,
        /// Type of autolink (url, email)
        autolink_type: AutolinkType,
    },
}

/// Types of links
#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    /// Inline link: [text](url)
    Inline,
    /// Reference link: [text][ref]
    Reference,
    /// Shortcut reference link: [text]
    Shortcut,
    /// Collapsed reference link: [text][]
    Collapsed,
    /// Autolink: <https://example.com>
    Autolink,
    /// Email autolink: <user@example.com>
    Email,
}

/// Types of autolinks
#[derive(Debug, Clone, PartialEq)]
pub enum AutolinkType {
    /// URL autolink (http, https, ftp, etc.)
    Url,
    /// Email autolink
    Email,
}

/// Reference definition for reference-style links
/// Example: [label]: url "title"
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceDefinition {
    /// Reference label
    pub label: String,
    /// Destination URL
    pub url: String,
    /// Optional title
    pub title: Option<String>,
}

/// Document metadata and reference definitions
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentContext {
    /// Reference definitions for reference-style links
    pub reference_definitions: HashMap<String, ReferenceDefinition>,
    /// Footnote definitions
    pub footnote_definitions: HashMap<String, Vec<Block>>,
    /// Document-level metadata
    pub metadata: HashMap<String, String>,
}

/// Source position information for error reporting and tooling
#[derive(Debug, Clone, PartialEq)]
pub struct SourcePosition {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset in the source
    pub offset: usize,
}

/// Range in source code
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRange {
    /// Start position
    pub start: SourcePosition,
    /// End position
    pub end: SourcePosition,
}

/// Node with source position information
/// Useful for error reporting and IDE features
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedNode<T> {
    /// The actual node content
    pub node: T,
    /// Source position information
    pub position: SourceRange,
}

/// Type aliases for positioned nodes
pub type PositionedBlock = PositionedNode<Block>;
pub type PositionedInline = PositionedNode<Inline>;
pub type PositionedDocument = PositionedNode<Document>;

/// Visitor pattern trait for traversing the AST
/// Implement this trait to create AST walkers for analysis, transformation, etc.
pub trait Visitor<T = ()> {
    /// Visit a document
    fn visit_document(&mut self, doc: &Document) -> T;
    
    /// Visit a block element
    fn visit_block(&mut self, block: &Block) -> T;
    
    /// Visit an inline element
    fn visit_inline(&mut self, inline: &Inline) -> T;
    
    /// Visit a list item
    fn visit_list_item(&mut self, item: &ListItem) -> T;
    
    /// Visit a task item
    fn visit_task_item(&mut self, item: &TaskItem) -> T;
    
    /// Visit a definition item
    fn visit_definition_item(&mut self, item: &DefinitionItem) -> T;
    
    /// Visit a table cell
    fn visit_table_cell(&mut self, cell: &TableCell) -> T;
}

/// Mutable visitor pattern trait for modifying the AST
pub trait MutVisitor<T = ()> {
    /// Visit and potentially modify a document
    fn visit_document(&mut self, doc: &mut Document) -> T;
    
    /// Visit and potentially modify a block element
    fn visit_block(&mut self, block: &mut Block) -> T;
    
    /// Visit and potentially modify an inline element
    fn visit_inline(&mut self, inline: &mut Inline) -> T;
    
    /// Visit and potentially modify a list item
    fn visit_list_item(&mut self, item: &mut ListItem) -> T;
    
    /// Visit and potentially modify a task item
    fn visit_task_item(&mut self, item: &mut TaskItem) -> T;
    
    /// Visit and potentially modify a definition item
    fn visit_definition_item(&mut self, item: &mut DefinitionItem) -> T;
    
    /// Visit and potentially modify a table cell
    fn visit_table_cell(&mut self, cell: &mut TableCell) -> T;
}

/// Convenience methods for creating AST nodes
impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Document {
            blocks: Vec::new(),
            metadata: None,
        }
    }
    
    /// Create a document with blocks
    pub fn with_blocks(blocks: Vec<Block>) -> Self {
        Document {
            blocks,
            metadata: None,
        }
    }
}

impl Block {
    /// Create a heading block
    pub fn heading(level: u8, content: Vec<Inline>) -> Self {
        Block::Heading {
            level,
            content,
            id: None,
        }
    }
    
    /// Create a paragraph block
    pub fn paragraph(content: Vec<Inline>) -> Self {
        Block::Paragraph { content }
    }
    
    /// Create a code block
    pub fn code_block(code: String, language: Option<String>) -> Self {
        Block::CodeBlock {
            language,
            code,
            info: None,
            fenced: true,
        }
    }
}

impl Inline {
    /// Create a text inline element
    pub fn text(content: String) -> Self {
        Inline::Text { content }
    }
    
    /// Create an emphasis inline element
    pub fn emphasis(content: Vec<Inline>) -> Self {
        Inline::Emphasis { content }
    }
    
    /// Create a strong inline element
    pub fn strong(content: Vec<Inline>) -> Self {
        Inline::Strong { content }
    }
    
    /// Create a code inline element
    pub fn code(code: String) -> Self {
        Inline::Code { code }
    }
    
    /// Create a link inline element
    pub fn link(text: Vec<Inline>, url: String, title: Option<String>) -> Self {
        Inline::Link {
            text,
            url,
            title,
            link_type: LinkType::Inline,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert_eq!(doc.blocks.len(), 0);
        assert!(doc.metadata.is_none());
    }
    
    #[test]
    fn test_heading_creation() {
        let heading = Block::heading(1, vec![Inline::text("Hello World".to_string())]);
        
        if let Block::Heading { level, content, .. } = heading {
            assert_eq!(level, 1);
            assert_eq!(content.len(), 1);
            if let Inline::Text { content: text } = &content[0] {
                assert_eq!(text, "Hello World");
            }
        } else {
            panic!("Expected heading block");
        }
    }
    
    #[test]
    fn test_paragraph_with_emphasis() {
        let paragraph = Block::paragraph(vec![
            Inline::text("This is ".to_string()),
            Inline::emphasis(vec![Inline::text("emphasized".to_string())]),
            Inline::text(" text.".to_string()),
        ]);
        
        if let Block::Paragraph { content } = paragraph {
            assert_eq!(content.len(), 3);
        } else {
            panic!("Expected paragraph block");
        }
    }
}
