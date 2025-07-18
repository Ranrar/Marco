// EventEmitter: walks the AST and emits Event stream
use super::event::{Event, Tag, TagEnd, SourcePos};
use crate::editor::logic::ast::inlines::Inline;
use crate::editor::logic::ast::blocks_and_inlines::{Block, ContainerBlock, LeafBlock};

// Helper to push inline events for all inline types
pub fn push_inline_events<'a>(state: &mut Vec<EventState<'a>>, inlines: Vec<(Inline, SourcePos)>) {
    for (inline, pos) in inlines.into_iter().rev() {
        match inline {
            Inline::Text(s) => state.push(EventState::EnterInline(s.clone(), pos.clone())),
            Inline::Code(code) => {
                state.push(EventState::ExitInline("code".to_string(), pos.clone()));
                state.push(EventState::EnterInline(code.content.clone(), pos.clone()));
                state.push(EventState::EnterInline("code".to_string(), pos.clone()));
            }
            Inline::Emphasis(emph) => match emph {
                crate::editor::logic::ast::inlines::Emphasis::Emph(inner, _) => {
                    state.push(EventState::ExitInline("emph".to_string(), pos.clone()));
                    push_inline_events(state, inner.clone());
                    state.push(EventState::EnterInline("emph".to_string(), pos.clone()));
                }
                crate::editor::logic::ast::inlines::Emphasis::Strong(inner, _) => {
                    state.push(EventState::ExitInline("strong".to_string(), pos.clone()));
                    push_inline_events(state, inner.clone());
                    state.push(EventState::EnterInline("strong".to_string(), pos.clone()));
                }
            },
            Inline::Link(link) => {
                state.push(EventState::ExitInline("link".to_string(), pos.clone()));
                push_inline_events(state, link.label.clone());
                let href = match &link.destination {
                    crate::editor::logic::ast::inlines::LinkDestination::Inline(u) => u.as_str(),
                    crate::editor::logic::ast::inlines::LinkDestination::Reference(r) => r.as_str(),
                };
                let title = link.title.as_deref().unwrap_or("");
                let marker = format!("link|href:{}|title:{}", href, title);
                state.push(EventState::EnterInline(marker, pos.clone()));
            }
            Inline::Image(image) => {
                state.push(EventState::ExitInline("image".to_string(), pos.clone()));
                push_inline_events(state, image.alt.clone());
                let alt_text = image.alt.iter().map(|(inline, _pos)| match inline {
                    Inline::Text(s) => s.as_str(),
                    _ => "",
                }).collect::<Vec<_>>().join(" ");
                let src = match &image.destination {
                    crate::editor::logic::ast::inlines::LinkDestination::Inline(u) => u.as_str(),
                    crate::editor::logic::ast::inlines::LinkDestination::Reference(r) => r.as_str(),
                };
                let title = image.title.as_deref().unwrap_or("");
                let marker = format!("image|src:{}|alt:{}|title:{}", src, alt_text, title);
                state.push(EventState::EnterInline(marker, pos.clone()));
            }
            Inline::Autolink(autolink) => match autolink {
                crate::editor::logic::ast::inlines::Autolink::Uri(uri) => {
                    state.push(EventState::ExitInline("autolink".to_string(), pos.clone()));
                    state.push(EventState::EnterInline(uri.clone(), pos.clone()));
                    state.push(EventState::EnterInline("autolink".to_string(), pos.clone()));
                }
                crate::editor::logic::ast::inlines::Autolink::Email(email) => {
                    state.push(EventState::ExitInline("autolink".to_string(), pos.clone()));
                    state.push(EventState::EnterInline(email.clone(), pos.clone()));
                    state.push(EventState::EnterInline("autolink".to_string(), pos.clone()));
                }
            },
            Inline::RawHtml(html) => {
                state.push(EventState::ExitInline("rawhtml".to_string(), pos.clone()));
                state.push(EventState::EnterInline(html.clone(), pos.clone()));
                state.push(EventState::EnterInline("rawhtml".to_string(), pos.clone()));
            }
            Inline::HardBreak => {
                state.push(EventState::ExitInline("hardbreak".to_string(), pos.clone()));
                state.push(EventState::EnterInline("\n".to_string(), pos.clone()));
                state.push(EventState::EnterInline("hardbreak".to_string(), pos.clone()));
            }
            Inline::SoftBreak => {
                state.push(EventState::ExitInline("softbreak".to_string(), pos.clone()));
                state.push(EventState::EnterInline(" ".to_string(), pos.clone()));
                state.push(EventState::EnterInline("softbreak".to_string(), pos.clone()));
            }
        }
    }
}

// EventState for emitter
#[derive(Debug, Clone)]
pub enum EventState<'a> {
    EnterBlock(&'a Block, Option<SourcePos>),
    ExitBlock(&'a Block, Option<SourcePos>),
    EnterInline(String, SourcePos),
    ExitInline(String, SourcePos),
}
