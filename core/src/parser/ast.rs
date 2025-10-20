// AST node definitions: central representation consumed by renderer and LSP

use crate::parser::Span;
use std::collections::HashMap;

// Link reference map: stores [label]: url definitions for later resolution
#[derive(Debug, Clone, Default)]
pub struct ReferenceMap {
    // Key: normalized label (lowercase, whitespace collapsed), Value: (url, optional title)
    defs: HashMap<String, (String, Option<String>)>,
}

impl ReferenceMap {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a link reference definition
    pub fn insert(&mut self, label: &str, url: String, title: Option<String>) {
        let normalized = normalize_label(label);
        self.defs.insert(normalized, (url, title));
    }
    
    /// Lookup a link reference by label
    pub fn get(&self, label: &str) -> Option<&(String, Option<String>)> {
        let normalized = normalize_label(label);
        self.defs.get(&normalized)
    }
    
    /// Check if a label exists
    pub fn contains(&self, label: &str) -> bool {
        let normalized = normalize_label(label);
        self.defs.contains_key(&normalized)
    }
}

/// Normalize label according to CommonMark spec:
/// - Convert to lowercase
/// - Collapse consecutive whitespace to single space
/// - Trim leading/trailing whitespace
fn normalize_label(label: &str) -> String {
    label
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// Root document node
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub children: Vec<Node>,
    pub references: ReferenceMap,
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
