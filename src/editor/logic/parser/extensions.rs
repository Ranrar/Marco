//! GFM and custom extension support for the Markdown parser event stream.
//! Emits specialized events for tables, task lists, strikethrough, autolinks, etc.

use crate::editor::logic::parser::event::{Event, Tag, TagEnd, SourcePos};
use crate::editor::logic::parser::attributes::Attributes;

/// Table alignment for GFM tables.
#[derive(Debug, Clone, PartialEq)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
    None,
}

/// Specialized GFM/custom extension events.
#[derive(Debug, Clone)]
pub enum ExtensionEvent {
    TableStart { alignments: Vec<TableAlignment>, pos: Option<SourcePos>, attr: Option<Attributes> },
    TableEnd { pos: Option<SourcePos> },
    TableRowStart { pos: Option<SourcePos> },
    TableRowEnd { pos: Option<SourcePos> },
    TableCellStart { alignment: TableAlignment, pos: Option<SourcePos> },
    TableCellEnd { pos: Option<SourcePos> },
    TableCaption { content: String, pos: Option<SourcePos>, attr: Option<Attributes> },
    TaskItem { checked: bool, pos: Option<SourcePos>, attr: Option<Attributes> },
    TaskListMeta { group: Option<String>, pos: Option<SourcePos>, attr: Option<Attributes> },
    StrikethroughStart { pos: Option<SourcePos>, attr: Option<Attributes> },
    StrikethroughEnd { pos: Option<SourcePos> },
    Autolink { url: String, pos: Option<SourcePos> },
    FootnoteReference { name: String, pos: Option<SourcePos> },
    FootnoteDefinitionStart { name: String, pos: Option<SourcePos> },
    FootnoteDefinitionEnd { name: String, pos: Option<SourcePos> },
    MathBlock { content: String, math_type: Option<crate::editor::logic::ast::math::MathType>, pos: Option<SourcePos>, attr: Option<Attributes> },
    Emoji { shortcode: String, unicode: String, pos: Option<SourcePos> },
    Mention { username: String, pos: Option<SourcePos> },
    CodeBlockStart { language: Option<String>, info_string: Option<String>, pos: Option<SourcePos>, attr: Option<Attributes> },
    CodeBlockEnd { pos: Option<SourcePos> },
    // Add more GFM/custom extension events as needed
}

/// Emit extension events from the token stream.
pub fn emit_extension_events(tokens: &[Event]) -> Vec<ExtensionEvent> {

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::logic::ast::math::MathType;
    #[test]
    fn extension_event_variants_work() {
        let table_caption = ExtensionEvent::TableCaption {
            content: "Table Caption".to_string(),
            pos: None,
            attr: None,
        };
        let math_block = ExtensionEvent::MathBlock {
            content: "x^2".to_string(),
            math_type: Some(MathType::LaTeX),
            pos: None,
            attr: None,
        };
        let emoji = ExtensionEvent::Emoji {
            shortcode: "smile".to_string(),
            unicode: "😄".to_string(),
            pos: None,
        };
        let mention = ExtensionEvent::Mention {
            username: "user".to_string(),
            pos: None,
        };
        let task_list_meta = ExtensionEvent::TaskListMeta {
            group: Some("group1".to_string()),
            pos: None,
            attr: None,
        };
        let code_block_start = ExtensionEvent::CodeBlockStart {
            language: Some("rust".to_string()),
            info_string: Some("main".to_string()),
            pos: None,
            attr: None,
        };
        let code_block_end = ExtensionEvent::CodeBlockEnd { pos: None };
        // Just check construction and matching
        match table_caption {
            ExtensionEvent::TableCaption { .. } => {},
            _ => panic!("Expected TableCaption"),
        }
        match math_block {
            ExtensionEvent::MathBlock { .. } => {},
            _ => panic!("Expected MathBlock"),
        }
        match emoji {
            ExtensionEvent::Emoji { .. } => {},
            _ => panic!("Expected Emoji"),
        }
        match mention {
            ExtensionEvent::Mention { .. } => {},
            _ => panic!("Expected Mention"),
        }
        match task_list_meta {
            ExtensionEvent::TaskListMeta { .. } => {},
            _ => panic!("Expected TaskListMeta"),
        }
        match code_block_start {
            ExtensionEvent::CodeBlockStart { .. } => {},
            _ => panic!("Expected CodeBlockStart"),
        }
        match code_block_end {
            ExtensionEvent::CodeBlockEnd { .. } => {},
            _ => panic!("Expected CodeBlockEnd"),
        }
    }
}
    let mut ext_events = Vec::new();
    for event in tokens {
        match event {
            Event::Start(Tag::Table(alignments, attr), pos, _) => {
                ext_events.push(ExtensionEvent::TableStart {
                    alignments: alignments.clone(),
                    pos: Some(pos.clone()),
                    attr: Some(attr.clone()),
                });
            }
            Event::End(TagEnd::Table, pos, _) => {
                ext_events.push(ExtensionEvent::TableEnd { pos: Some(pos.clone()) });
            }
            Event::Start(Tag::TableRow, pos, _) => {
                ext_events.push(ExtensionEvent::TableRowStart { pos: Some(pos.clone()) });
            }
            Event::End(TagEnd::TableRow, pos, _) => {
                ext_events.push(ExtensionEvent::TableRowEnd { pos: Some(pos.clone()) });
            }
            Event::Start(Tag::TableCell(alignment), pos, _) => {
                ext_events.push(ExtensionEvent::TableCellStart {
                    alignment: alignment.clone(),
                    pos: Some(pos.clone()),
                });
            }
            Event::End(TagEnd::TableCell, pos, _) => {
                ext_events.push(ExtensionEvent::TableCellEnd { pos: Some(pos.clone()) });
            }
            Event::Start(Tag::TableCaption(content, attr), pos, _) => {
                ext_events.push(ExtensionEvent::TableCaption {
                    content: content.clone(),
                    pos: Some(pos.clone()),
                    attr: Some(attr.clone()),
                });
            }
            Event::Start(Tag::TaskItem(checked, attr), pos, _) => {
                ext_events.push(ExtensionEvent::TaskItem {
                    checked: *checked,
                    pos: Some(pos.clone()),
                    attr: Some(attr.clone()),
                });
            }
            Event::Start(Tag::TaskListMeta(group, attr), pos, _) => {
                ext_events.push(ExtensionEvent::TaskListMeta {
                    group: group.clone(),
                    pos: Some(pos.clone()),
                    attr: Some(attr.clone()),
                });
            }
            Event::Start(Tag::Strikethrough(attr), pos, _) => {
                ext_events.push(ExtensionEvent::StrikethroughStart {
                    pos: Some(pos.clone()),
                    attr: Some(attr.clone()),
                });
            }
            Event::End(TagEnd::Strikethrough, pos, _) => {
                ext_events.push(ExtensionEvent::StrikethroughEnd { pos: Some(pos.clone()) });
            }
            Event::Autolink(url, pos) => {
                ext_events.push(ExtensionEvent::Autolink {
                    url: url.clone(),
                    pos: Some(pos.clone()),
                });
            }
            Event::Emoji(shortcode, unicode, pos) => {
                ext_events.push(ExtensionEvent::Emoji {
                    shortcode: shortcode.clone(),
                    unicode: unicode.clone(),
                    pos: Some(pos.clone()),
                });
            }
            Event::Mention(username, pos) => {
                ext_events.push(ExtensionEvent::Mention {
                    username: username.clone(),
                    pos: Some(pos.clone()),
                });
            }
            Event::FootnoteReference(name, pos) => {
                ext_events.push(ExtensionEvent::FootnoteReference {
                    name: name.clone(),
                    pos: Some(pos.clone()),
                });
            }
            Event::Start(Tag::FootnoteDefinition(name), pos, _) => {
                ext_events.push(ExtensionEvent::FootnoteDefinitionStart {
                    name: name.clone(),
                    pos: Some(pos.clone()),
                });
            }
            Event::End(TagEnd::FootnoteDefinition(name), pos, _) => {
                ext_events.push(ExtensionEvent::FootnoteDefinitionEnd {
                    name: name.clone(),
                    pos: Some(pos.clone()),
                });
            }
            Event::Start(Tag::MathBlock(content, math_type, attr), pos, _) => {
                ext_events.push(ExtensionEvent::MathBlock {
                    content: content.clone(),
                    math_type: math_type.clone(),
                    pos: Some(pos.clone()),
                    attr: Some(attr.clone()),
                });
            }
            Event::Start(Tag::CodeBlock(language, info_string, attr), pos, _) => {
                ext_events.push(ExtensionEvent::CodeBlockStart {
                    language: language.clone(),
                    info_string: info_string.clone(),
                    pos: Some(pos.clone()),
                    attr: Some(attr.clone()),
                });
            }
            Event::End(TagEnd::CodeBlock, pos, _) => {
                ext_events.push(ExtensionEvent::CodeBlockEnd {
                    pos: Some(pos.clone()),
                });
            }
            // ...existing code for other events...
            _ => {}
        }
    }
    ext_events
}
