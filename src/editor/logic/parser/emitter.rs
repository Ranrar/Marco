// EventEmitter: walks the AST and emits Event stream
use super::event::{Event, Tag, TagEnd, SourcePos};
use crate::editor::logic::ast::inlines::Inline;
use crate::editor::logic::ast::blocks_and_inlines::{Block, ContainerBlock, LeafBlock};

// Helper to push inline events for all inline types
pub fn push_inline_events(state: &mut Vec<Event>, inlines: Vec<(Inline, SourcePos)>) {
    for (inline, pos) in inlines.into_iter().rev() {
        match inline {
            Inline::Text(s) => state.push(Event::Text(s.clone(), Some(pos.clone()), None)),
            Inline::Code(code) => {
                let attrs = code.attributes.clone();
                state.push(Event::Code(code.content.clone(), Some(pos.clone()), attrs.clone()));
            }
            Inline::Emphasis(emph) => match emph {
                crate::editor::logic::ast::inlines::Emphasis::Emph(inner, attrs) => {
                    state.push(Event::EmphasisEnd(Some(pos.clone()), attrs.clone()));
                    push_inline_events(state, inner.clone());
                    state.push(Event::EmphasisStart(Some(pos.clone()), attrs.clone()));
                }
                crate::editor::logic::ast::inlines::Emphasis::Strong(inner, attrs) => {
                    state.push(Event::StrongEnd(Some(pos.clone()), attrs.clone()));
                    push_inline_events(state, inner.clone());
                    state.push(Event::StrongStart(Some(pos.clone()), attrs.clone()));
                }
            },
            Inline::Link(link) => {
                let attrs = link.attributes.clone();
                state.push(Event::LinkEnd(Some(pos.clone()), attrs.clone()));
                push_inline_events(state, link.label.clone());
                let href_owned = match &link.destination {
                    crate::editor::logic::ast::inlines::LinkDestination::Inline(u) => u.clone(),
                    crate::editor::logic::ast::inlines::LinkDestination::Reference(r) => r.clone(),
                };
                let title_owned = link.title.clone();
                state.push(Event::LinkStart { href: href_owned, title: title_owned, pos: Some(pos.clone()), attributes: attrs.clone() });
            }
            Inline::Image(image) => {
                let attrs = image.attributes.clone();
                state.push(Event::ImageEnd(Some(pos.clone()), attrs.clone()));
                push_inline_events(state, image.alt.clone());
                let alt_text_owned = image.alt.iter().map(|(inline, _pos)| match inline {
                    Inline::Text(s) => s.as_str(),
                    _ => "",
                }).collect::<Vec<_>>().join(" ");
                let src_owned = match &image.destination {
                    crate::editor::logic::ast::inlines::LinkDestination::Inline(u) => u.clone(),
                    crate::editor::logic::ast::inlines::LinkDestination::Reference(r) => r.clone(),
                };
                let title_owned = image.title.clone();
                state.push(Event::ImageStart { src: src_owned, alt: alt_text_owned, title: title_owned, pos: Some(pos.clone()), attributes: attrs.clone() });
            }
            Inline::Autolink(autolink) => match autolink {
                crate::editor::logic::ast::inlines::Autolink::Uri(uri) => {
                    state.push(Event::Autolink(uri.clone(), Some(pos.clone()), None));
                }
                crate::editor::logic::ast::inlines::Autolink::Email(email) => {
                    state.push(Event::Autolink(email.clone(), Some(pos.clone()), None));
                }
            },
            Inline::RawHtml(html) => {
                state.push(Event::RawHtml(html.clone(), Some(pos.clone()), None));
            }
            Inline::HardBreak => {
                state.push(Event::HardBreak(Some(pos.clone()), None));
            }
            Inline::SoftBreak => {
                state.push(Event::SoftBreak(Some(pos.clone()), None));
            }
        }
    }
}
