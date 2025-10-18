// Block and inline node types with position information

use crate::parser::Span;

// Block-level nodes
#[derive(Debug, Clone)]
pub struct Block {
    pub kind: BlockKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BlockKind {
    Heading { level: u8 },
    Paragraph,
    CodeBlock { language: Option<String> },
    List { ordered: bool },
    Blockquote,
    Table,
}

// Inline-level nodes
#[derive(Debug, Clone)]
pub struct Inline {
    pub kind: InlineKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum InlineKind {
    Text,
    Emphasis,
    Strong,
    Link { url: String },
    Image { url: String, alt: String },
    CodeSpan,
    InlineHtml,
}
