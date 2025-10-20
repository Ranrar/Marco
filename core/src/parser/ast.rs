// AST node definitions: central representation consumed by renderer and LSP

use crate::parser::Span;

// Root document node
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub children: Vec<Node>,
}

// Generic AST node
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub span: Option<Span>,
    pub children: Vec<Node>,
}

// All node types
#[derive(Debug, Clone)]
pub enum NodeKind {
    // Block-level
    Heading { level: u8, text: String },
    Paragraph,
    CodeBlock { language: Option<String>, code: String },
    ThematicBreak, // Horizontal rule (---, ***, ___)
    List { 
        ordered: bool,
        start: Option<u32>,  // Starting number for ordered lists
        tight: bool,         // No blank lines between items
    },
    ListItem,
    Blockquote,
    Table,
    HtmlBlock { html: String }, // Block-level HTML (comments, tags, etc.)
    
    // Inline-level
    Text(String),
    Emphasis,
    Strong,
    Link { url: String, title: Option<String> },
    Image { url: String, alt: String },
    CodeSpan(String),
    InlineHtml(String),
    HardBreak,  // Two spaces + newline, or backslash + newline
    SoftBreak,  // Regular newline (rendered as space in HTML)
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn len(&self) -> usize {
        self.children.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}
