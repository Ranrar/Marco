// Core event types for Markdown event stream
use crate::editor::logic::attributes::Attributes;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Start(Tag, Option<SourcePos>, Option<Attributes>),
    End(TagEnd, Option<SourcePos>, Option<Attributes>),
    Text(String, Option<SourcePos>, Option<Attributes>),
    Code(String, Option<SourcePos>, Option<Attributes>),
    Html(String, Option<SourcePos>, Option<Attributes>),
    Autolink(String, Option<SourcePos>, Option<Attributes>),
    RawHtml(String, Option<SourcePos>, Option<Attributes>),
    HardBreak(Option<SourcePos>, Option<Attributes>),
    SoftBreak(Option<SourcePos>, Option<Attributes>),
    EmphasisStart(Option<SourcePos>, Option<Attributes>),
    EmphasisEnd(Option<SourcePos>, Option<Attributes>),
    StrongStart(Option<SourcePos>, Option<Attributes>),
    StrongEnd(Option<SourcePos>, Option<Attributes>),
    LinkStart { href: String, title: Option<String>, pos: Option<SourcePos>, attributes: Option<Attributes> },
    LinkEnd(Option<SourcePos>, Option<Attributes>),
    ImageStart { src: String, alt: String, title: Option<String>, pos: Option<SourcePos>, attributes: Option<Attributes> },
    ImageEnd(Option<SourcePos>, Option<Attributes>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Paragraph(Option<Attributes>),
    Heading(u8, Option<Attributes>),
    BlockQuote(Option<Attributes>),
    List(Option<Attributes>),
    Item(Option<Attributes>),
    CodeBlock(Option<Attributes>),
    HtmlBlock(Option<Attributes>),
    Emphasis(Option<Attributes>),
    Strong(Option<Attributes>),
    Link(Option<Attributes>),
    Image(Option<Attributes>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagEnd {
    Paragraph(Option<Attributes>),
    Heading(Option<Attributes>),
    BlockQuote(Option<Attributes>),
    List(Option<Attributes>),
    Item(Option<Attributes>),
    CodeBlock(Option<Attributes>),
    HtmlBlock(Option<Attributes>),
    Emphasis(Option<Attributes>),
    Strong(Option<Attributes>),
    Link(Option<Attributes>),
    Image(Option<Attributes>),
}

// Source position tracking for advanced features
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
}
