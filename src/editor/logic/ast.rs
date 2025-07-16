// Abstract Syntax Tree

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownNode {
    // Root node of the document
    Document(Vec<MarkdownNode>),

    // Block-level elements
    Heading {
        level: u8,
        content: Vec<MarkdownNode>, // inline nodes
    },
    Paragraph(Vec<MarkdownNode>),
    BlockQuote(Vec<MarkdownNode>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    List {
        ordered: bool,
        items: Vec<MarkdownNode>, // ListItem nodes
    },
    ListItem(Vec<MarkdownNode>),
    ThematicBreak, // Represents '---' or '***'

    // Inline elements
    Text(String),
    Emphasis(Vec<MarkdownNode>),      // *italic*
    Strong(Vec<MarkdownNode>),        // **bold**
    Strikethrough(Vec<MarkdownNode>), // ~~text~~
    Code(String),                     // `inline code`
    Link {
        label: Vec<MarkdownNode>,     // Can include formatting
        destination: String,
        title: Option<String>,
    },
    Image {
        alt: Vec<MarkdownNode>,
        src: String,
        title: Option<String>,
    },
    LineBreak, // Hard line break (two spaces at end of line)
}
