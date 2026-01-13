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

/// Table column alignment (GFM tables extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

// All node types
#[derive(Debug, Clone)]
pub enum NodeKind {
    // Block-level
    Heading {
        level: u8,
        text: String,
    },
    Paragraph,
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    ThematicBreak, // Horizontal rule (---, ***, ___)
    List {
        ordered: bool,
        start: Option<u32>, // Starting number for ordered lists
        tight: bool,        // No blank lines between items
    },
    ListItem,
    /// GFM task list checkbox marker for a list item.
    ///
    /// This is emitted by the list parser when a list item begins with
    /// `[ ]` or `[x]` / `[X]`.
    ///
    /// Rendering convention:
    /// - This node is expected to appear as the first child inside a `ListItem`.
    /// - The HTML renderer will convert it into a themed checkbox icon.
    TaskCheckbox {
        checked: bool,
    },
    Blockquote,
    /// GFM table (pipe table extension).
    ///
    /// Children convention:
    /// - Each child is a `TableRow`.
    /// - Each `TableRow` contains `TableCell` children.
    Table {
        alignments: Vec<TableAlignment>,
    },
    TableRow {
        header: bool,
    },
    TableCell {
        header: bool,
        alignment: TableAlignment,
    },
    HtmlBlock {
        html: String,
    }, // Block-level HTML (comments, tags, etc.)

    // Inline-level
    Text(String),
    /// Inline task checkbox marker (extension).
    ///
    /// This is emitted when a paragraph begins with a task marker like
    /// `[ ] ` / `[x] ` / `[X] `.
    ///
    /// Rendering convention:
    /// - The HTML renderer converts it into the same themed SVG checkbox icon
    ///   used for task list items.
    TaskCheckboxInline {
        checked: bool,
    },
    Emphasis,
    Strong,
    /// Combined strong+emphasis, e.g. `***text***` or `___text___`.
    ///
    /// This is parsed as a single inline node to avoid leaving dangling
    /// delimiters that would otherwise be treated as plain text.
    StrongEmphasis,
    /// Strikethrough (extension), e.g. `~~text~~`.
    Strikethrough,
    /// Highlight/mark (extension), e.g. `==text==`.
    Mark,
    /// Superscript (extension), e.g. `^text^`.
    Superscript,
    /// Subscript (extension), e.g. `~text~`.
    Subscript,
    Link {
        url: String,
        title: Option<String>,
    },
    /// Reference-style link placeholder (CommonMark): `[text][label]`, `[label][]`, `[label]`.
    ///
    /// These cannot be fully resolved during inline parsing because reference
    /// definitions may appear later in the document. The top-level `parse()`
    /// performs a post-processing pass that converts this into a `Link` when a
    /// matching definition exists in `Document.references`.
    ///
    /// If no matching definition is found, this should be rendered as literal
    /// bracketed text (preserving the already-parsed `children` for the first
    /// bracketed segment).
    LinkReference {
        /// Label used for reference resolution (will be normalized when looked up).
        label: String,
        /// Extra literal suffix after the first `]` (e.g. "[]" or "[label]").
        /// Empty for shortcut reference links.
        suffix: String,
    },
    Image {
        url: String,
        alt: String,
    },
    CodeSpan(String),
    InlineHtml(String),
    HardBreak, // Two spaces + newline, or backslash + newline
    SoftBreak, // Regular newline (rendered as space in HTML)
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
