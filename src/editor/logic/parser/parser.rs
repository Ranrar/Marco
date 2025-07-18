// Main parser: Token stream → Event stream or AST (supports streaming)
use super::event::{Event, Tag, TagEnd};
use super::event::{SourcePos};
use super::emitter::{push_inline_events, EventState};
use crate::editor::logic::ast::blocks_and_inlines::{Block, ContainerBlock, LeafBlock};

pub struct EventIter<'a> {
    stack: Vec<&'a Block>,
    state: Vec<EventState<'a>>,
}

impl<'a> EventIter<'a> {
    pub fn new(root: &'a Block) -> Self {
        Self {
            stack: vec![root],
            state: vec![EventState::EnterBlock(root, Some(SourcePos::default()))],
        }
    }
}

impl<'a> Iterator for EventIter<'a> {
    type Item = Event<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state) = self.state.pop() {
            match state {
                EventState::EnterBlock(block, _pos) => {
                    match block {
                        Block::Container(ContainerBlock::Document(children, attributes)) => {
                            for child in children.iter().rev() {
                                self.state.push(EventState::EnterBlock(child, Some(SourcePos::default())));
                            }
                        }
                        Block::Container(ContainerBlock::BlockQuote(children, attributes)) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            for child in children.iter().rev() {
                                self.state.push(EventState::EnterBlock(child, Some(SourcePos::default())));
                            }
                            return Some(Event::Start(Tag::BlockQuote(attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        Block::Container(ContainerBlock::List { items, attributes, .. }) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            for item in items.iter().rev() {
                                self.state.push(EventState::EnterBlock(item, Some(SourcePos::default())));
                            }
                            return Some(Event::Start(Tag::List(attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        Block::Container(ContainerBlock::ListItem { contents, attributes, .. }) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            for b in contents.iter().rev() {
                                self.state.push(EventState::EnterBlock(b, Some(SourcePos::default())));
                            }
                            return Some(Event::Start(Tag::Item(attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        Block::Leaf(LeafBlock::Paragraph(inlines, attributes)) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            push_inline_events(&mut self.state, inlines.clone());
                            return Some(Event::Start(Tag::Paragraph(attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        Block::Leaf(LeafBlock::Heading { level, content, attributes }) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            push_inline_events(&mut self.state, content.clone());
                            return Some(Event::Start(Tag::Heading(*level, attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        Block::Leaf(LeafBlock::IndentedCodeBlock { content, attributes }) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            return Some(Event::Start(Tag::CodeBlock(attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        Block::Leaf(LeafBlock::FencedCodeBlock { fence_char, fence_count, info_string, content, attributes }) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            return Some(Event::Start(Tag::CodeBlock(attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        Block::Leaf(LeafBlock::HtmlBlock { block_type, content, attributes }) => {
                            self.state.push(EventState::ExitBlock(block, Some(SourcePos::default())));
                            return Some(Event::Start(Tag::HtmlBlock(attributes.clone()), Some(SourcePos::default()), attributes.clone()));
                        }
                        _ => {}
                    }
                }
                EventState::ExitBlock(block, _pos) => {
                    match block {
                        Block::Container(ContainerBlock::BlockQuote(_, attributes)) => return Some(Event::End(TagEnd::BlockQuote(attributes.clone()), Some(SourcePos::default()), attributes.clone())),
                        Block::Container(ContainerBlock::List { .. }) => return Some(Event::End(TagEnd::List(None), Some(SourcePos::default()), None)),
                        Block::Container(ContainerBlock::ListItem { .. }) => return Some(Event::End(TagEnd::Item(None), Some(SourcePos::default()), None)),
                        Block::Leaf(LeafBlock::Paragraph(_, attributes)) => return Some(Event::End(TagEnd::Paragraph(attributes.clone()), Some(SourcePos::default()), attributes.clone())),
                        Block::Leaf(LeafBlock::Heading { attributes, .. }) => return Some(Event::End(TagEnd::Heading(attributes.clone()), Some(SourcePos::default()), attributes.clone())),
                        Block::Leaf(LeafBlock::IndentedCodeBlock { attributes, .. }) => return Some(Event::End(TagEnd::CodeBlock(attributes.clone()), Some(SourcePos::default()), attributes.clone())),
                        Block::Leaf(LeafBlock::FencedCodeBlock { attributes, .. }) => return Some(Event::End(TagEnd::CodeBlock(attributes.clone()), Some(SourcePos::default()), attributes.clone())),
                        Block::Leaf(LeafBlock::HtmlBlock { attributes, .. }) => return Some(Event::End(TagEnd::HtmlBlock(attributes.clone()), Some(SourcePos::default()), attributes.clone())),
                        _ => {}
                    }
                }
                EventState::EnterInline(text, pos) => {
                    // Emit correct event for inline type
                    match text.as_str() {
                        "emph" => return Some(Event::EmphasisStart(Some(pos.clone()), None)),
                        "strong" => return Some(Event::StrongStart(Some(pos.clone()), None)),
                        "link" => return Some(Event::LinkStart { href: "", title: None, pos: Some(pos.clone()), attributes: None }), // TODO: pass href/title/attributes
                        "image" => return Some(Event::ImageStart { src: "", alt: "", title: None, pos: Some(pos.clone()), attributes: None }), // TODO: pass src/alt/title/attributes
                        "\n" => return Some(Event::HardBreak(Some(pos.clone()), None)),
                        " " => return Some(Event::SoftBreak(Some(pos.clone()), None)),
                        _ => return Some(Event::Text(text.clone(), Some(pos.clone()), None)),
                    }
                }
                EventState::ExitInline(_s, _pos) => {}
            }
        }
        None
    }
}
