// Abstract Syntax Tree for Markdown

/// The root node representing the entire Markdown document.
/// Contains a sequence of block-level nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub blocks: Vec<BlockNode>,
}

/// Enum representing all block-level elements in Markdown.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockNode {
    /// Heading element with a level from 1 to 6.
    /// Contains inline content nodes.
    Heading {
        level: u8,               // Heading level (1-6)
        content: Vec<InlineNode>, // Inline nodes inside the heading
    },

    /// Paragraph block containing inline nodes.
    Paragraph(Vec<InlineNode>),

    /// Blockquote block containing nested block nodes.
    BlockQuote(Vec<BlockNode>),

    /// Fenced or indented code block.
    CodeBlock {
        language: Option<String>, // Optional language tag for syntax highlighting
        code: String,             // Raw code text
    },

    /// List block which can be ordered or unordered.
    List {
        ordered: bool,            // True for ordered list, false for unordered
        items: Vec<ListItem>,     // List items (each may contain blocks)
    },

    /// Horizontal rule block, representing a thematic break (e.g. '---' or '***').
    ThematicBreak,
}

/// Represents a single list item containing block-level content.
/// List items can contain multiple block nodes (paragraphs, sublists, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub content: Vec<BlockNode>,
}

/// Enum representing all inline elements inside block nodes like Paragraph or Heading.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineNode {
    /// Plain text content.
    Text(String),

    /// Emphasized text (italic), which can contain nested inline formatting.
    Emphasis(Vec<InlineNode>),

    /// Strongly emphasized text (bold), which can contain nested inline formatting.
    Strong(Vec<InlineNode>),

    /// Strikethrough text.
    Strikethrough(Vec<InlineNode>),

    /// Inline code span.
    Code(String),

    /// Hyperlink with label (which may contain formatting), URL, and optional title attribute.
    Link {
        label: Vec<InlineNode>,
        destination: String,
        title: Option<String>,
    },

    /// Image with alt text (which may contain formatting), source URL, and optional title.
    Image {
        alt: Vec<InlineNode>,
        src: String,
        title: Option<String>,
    },

    /// Hard line break (two or more spaces at end of line).
    LineBreak,
}
