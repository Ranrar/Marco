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
    Math { content: String, pos: Option<SourcePos>, attributes: Option<Attributes> },
    MathBlock { content: String, math_type: Option<crate::editor::logic::ast::math::MathType>, pos: Option<SourcePos>, attributes: Option<Attributes> },
    Emoji(String, String, Option<SourcePos>),
    Mention(String, Option<SourcePos>),
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
    MathBlock(String, Option<crate::editor::logic::ast::math::MathType>, Option<Attributes>),
    TableCaption(String, Option<Attributes>),
    TaskListMeta(Option<String>, Option<Attributes>),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::logic::ast::math::MathType;
    #[test]
    fn event_and_tag_variants_work() {
        let math_block = Event::MathBlock {
            content: "x^2".to_string(),
            math_type: Some(MathType::LaTeX),
            pos: None,
            attributes: None,
        };
        let emoji = Event::Emoji("smile".to_string(), "😄".to_string(), None);
        let mention = Event::Mention("user".to_string(), None);
        let tag_math_block = Tag::MathBlock("x^2".to_string(), Some(MathType::LaTeX), None);
        let tag_table_caption = Tag::TableCaption("caption".to_string(), None);
        let tag_task_list_meta = Tag::TaskListMeta(Some("group1".to_string()), None);
        match math_block {
            Event::MathBlock { .. } => {},
            _ => panic!("Expected MathBlock"),
        }
        match emoji {
            Event::Emoji(_, _, _) => {},
            _ => panic!("Expected Emoji"),
        }
        match mention {
            Event::Mention(_, _) => {},
            _ => panic!("Expected Mention"),
        }
        match tag_math_block {
            Tag::MathBlock(_, _, _) => {},
            _ => panic!("Expected Tag::MathBlock"),
        }
        match tag_table_caption {
            Tag::TableCaption(_, _) => {},
            _ => panic!("Expected Tag::TableCaption"),
        }
        match tag_task_list_meta {
            Tag::TaskListMeta(_, _) => {},
            _ => panic!("Expected Tag::TaskListMeta"),
        }
    }
}
