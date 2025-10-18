// AST node definitions: central representation consumed by renderer and LSP

pub mod nodes;
pub mod traversal;

pub use nodes::*;
pub use traversal::*;

use crate::parser::Span;

// Root document node
#[derive(Debug, Clone)]
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
    List { ordered: bool },
    ListItem,
    Blockquote,
    Table,
    
    // Inline-level
    Text(String),
    Emphasis,
    Strong,
    Link { url: String, title: Option<String> },
    Image { url: String, alt: String },
    CodeSpan(String),
    InlineHtml(String),
}

impl Document {
    pub fn new() -> Self {
        Self { children: Vec::new() }
    }
    
    pub fn len(&self) -> usize {
        self.children.len()
    }
}
