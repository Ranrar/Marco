use serde::{Deserialize, Serialize};

/// Position information for source mapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Create a simple span from start and end positions (line/column set to 1)
    pub fn simple(start: usize, end: usize) -> Self {
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

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Main AST node enum covering all Marco syntax elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    // Document structure
    Document {
        children: Vec<Node>,
        span: Span,
    },

    // Block elements
    Heading {
        level: u8,
        content: Vec<Node>,
        span: Span,
    },
    Paragraph {
        content: Vec<Node>,
        span: Span,
    },
    CodeBlock {
        language: Option<String>,
        content: String,
        span: Span,
    },
    MathBlock {
        content: String,
        span: Span,
    },

    // Lists
    List {
        ordered: bool,
        items: Vec<Node>,
        span: Span,
    },
    ListItem {
        content: Vec<Node>,
        checked: Option<bool>, // For task lists
        span: Span,
    },

    // Tables
    Table {
        headers: Vec<Node>,
        rows: Vec<Vec<Node>>,
        span: Span,
    },

    // Inline elements
    Text {
        content: String,
        span: Span,
    },
    Emphasis {
        content: Vec<Node>,
        span: Span,
    },
    Strong {
        content: Vec<Node>,
        span: Span,
    },
    Code {
        content: String,
        span: Span,
    },
    Link {
        text: Vec<Node>,
        url: String,
        title: Option<String>,
        span: Span,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        span: Span,
    },

    // Marco-specific elements
    Macro {
        name: String,
        arguments: Vec<String>,
        content: Option<Vec<Node>>,
        span: Span,
    },

    // Additional elements
    HorizontalRule {
        span: Span,
    },
    BlockQuote {
        content: Vec<Node>,
        span: Span,
    },

    // Error recovery
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
            Node::Text { span, .. } => span,
            Node::Emphasis { span, .. } => span,
            Node::Strong { span, .. } => span,
            Node::Code { span, .. } => span,
            Node::Link { span, .. } => span,
            Node::Image { span, .. } => span,
            Node::Macro { span, .. } => span,
            Node::HorizontalRule { span, .. } => span,
            Node::BlockQuote { span, .. } => span,
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
                | Node::Table { .. }
                | Node::HorizontalRule { .. }
                | Node::BlockQuote { .. }
        )
    }

    /// Check if this is an inline element
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            Node::Text { .. }
                | Node::Emphasis { .. }
                | Node::Strong { .. }
                | Node::Code { .. }
                | Node::Link { .. }
                | Node::Image { .. }
        )
    }

    /// Get children nodes if this node has them
    pub fn children(&self) -> Option<&[Node]> {
        match self {
            Node::Document { children, .. } => Some(children),
            Node::Heading { content, .. } => Some(content),
            Node::Paragraph { content, .. } => Some(content),
            Node::List { items, .. } => Some(items),
            Node::ListItem { content, .. } => Some(content),
            Node::Emphasis { content, .. } => Some(content),
            Node::Strong { content, .. } => Some(content),
            Node::Link { text, .. } => Some(text),
            Node::BlockQuote { content, .. } => Some(content),
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
            Node::Paragraph { content, .. } => Some(content),
            Node::List { items, .. } => Some(items),
            Node::ListItem { content, .. } => Some(content),
            Node::Emphasis { content, .. } => Some(content),
            Node::Strong { content, .. } => Some(content),
            Node::Link { text, .. } => Some(text),
            Node::BlockQuote { content, .. } => Some(content),
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
}
