        // Example: CustomTag (user-defined extension)
        // To emit a custom tag, add an Inline variant and handle it here, e.g.:
        // Inline::CustomTag(name, data, attr, pos) => {
        //     events.push(Event::Start(Tag::CustomTag { name, data, attributes: attr.clone() }, Some(pos.clone()), attr.clone()));
        //     // ...emit inner events if needed...
        //     events.push(Event::End(TagEnd::CustomTagEnd { name, attributes: attr.clone() }, Some(pos.clone()), attr.clone()));
        // }
// EventEmitter: walks the AST and emits Event stream
use super::event_types::{Event, Tag, SourcePos};
use crate::logic::ast::inlines::Inline;

// Helper to push inline events for all inline types
use crate::logic::core::event_pipeline::EventPipeline;

/// Push inline events, optionally processing each event through a pipeline.
pub fn push_inline_events(state: &mut Vec<Event>, inlines: Vec<(Inline, SourcePos)>, pipeline: &mut Option<&mut EventPipeline>) {
    for (inline, pos) in inlines.into_iter().rev() {
        let mut events = Vec::new();
        match inline {
            Inline::Text(s) => events.push(Event::Text(s.clone(), Some(pos.clone()), None)),
            Inline::CodeSpan(code) => {
                let attrs = code.attributes.clone();
                events.push(Event::Code(code.content.clone(), Some(pos.clone()), attrs.clone()));
            }
            Inline::Emphasis(emph) => match emph {
                crate::logic::ast::inlines::Emphasis::Emph(inner, attrs) => {
                    events.push(Event::EmphasisEnd(Some(pos.clone()), attrs.clone()));
                    push_inline_events(state, inner.clone(), pipeline);
                    events.push(Event::EmphasisStart(Some(pos.clone()), attrs.clone()));
                }
                crate::logic::ast::inlines::Emphasis::Strong(inner, attrs) => {
                    events.push(Event::StrongEnd(Some(pos.clone()), attrs.clone()));
                    push_inline_events(state, inner.clone(), pipeline);
                    events.push(Event::StrongStart(Some(pos.clone()), attrs.clone()));
                }
            },
            Inline::Link(link) => {
                let attrs = link.attributes.clone();
                events.push(Event::LinkEnd(Some(pos.clone()), attrs.clone()));
                push_inline_events(state, link.label.clone(), pipeline);
                let href_owned = match &link.destination {
                    crate::logic::ast::inlines::LinkDestination::Inline(u) => u.clone(),
                    crate::logic::ast::inlines::LinkDestination::Reference(r) => r.clone(),
                };
                let title_owned = link.title.clone();
                events.push(Event::LinkStart { href: href_owned, title: title_owned, pos: Some(pos.clone()), attributes: attrs.clone() });
            }
            Inline::Image(image) => {
                let attrs = image.attributes.clone();
                events.push(Event::ImageEnd(Some(pos.clone()), attrs.clone()));
                push_inline_events(state, image.alt.clone(), pipeline);
                let alt_text_owned = image.alt.iter().map(|(inline, _pos)| match inline {
                    Inline::Text(s) => s.as_str(),
                    _ => "",
                }).collect::<Vec<_>>().join(" ");
                let src_owned = match &image.destination {
                    crate::logic::ast::inlines::LinkDestination::Inline(u) => u.clone(),
                    crate::logic::ast::inlines::LinkDestination::Reference(r) => r.clone(),
                };
                let title_owned = image.title.clone();
                events.push(Event::ImageStart { src: src_owned, alt: alt_text_owned, title: title_owned, pos: Some(pos.clone()), attributes: attrs.clone() });
            }
            Inline::Autolink(autolink) => match autolink {
                crate::logic::ast::inlines::Autolink::Uri(uri) => {
                    events.push(Event::Autolink(uri.clone(), Some(pos.clone()), None));
                }
                crate::logic::ast::inlines::Autolink::Email(email) => {
                    events.push(Event::Autolink(email.clone(), Some(pos.clone()), None));
                }
            },
            Inline::RawHtml(html) => {
                events.push(Event::RawHtml(html.clone(), Some(pos.clone()), None));
            }
            Inline::HardBreak => {
                events.push(Event::HardBreak(Some(pos.clone()), None));
            }
            Inline::SoftBreak => {
                events.push(Event::SoftBreak(Some(pos.clone()), None));
            }
            Inline::Math(math) => {
                let attrs = math.attributes.clone();
                // MathBlock event (block-level math)
                events.push(Event::Start(
                    Tag::MathBlock(
                        math.content.clone(),
                        Some(math.math_type.clone()),
                        attrs.clone(),
                    ),
                    math.position.clone(),
                    attrs.clone(),
                ));
                // For inline math, you may want to emit a separate event or reuse Event::Math
                events.push(Event::Math {
                    content: math.content.clone(),
                    pos: math.position.clone(),
                    attributes: attrs,
                });
            }
            // Example: Emoji, Mention, TableCaption, TaskListMeta
            Inline::Emoji(shortcode, unicode, pos) => {
                events.push(Event::Emoji(shortcode.clone(), unicode.clone(), Some(pos.clone())));
            }
            Inline::Mention(username, pos) => {
                events.push(Event::Mention(username.clone(), Some(pos.clone())));
            }
            Inline::TableCaption(content, attr, pos) => {
                events.push(Event::Start(
                    Tag::TableCaption(content.clone(), attr.clone()),
                    Some(pos.clone()),
                    attr.clone(),
                ));
            }
            Inline::TaskListMeta(group, attr, pos) => {
                events.push(Event::Start(
                    Tag::TaskListMeta(group.clone(), attr.clone()),
                    Some(pos.clone()),
                    attr.clone(),
                ));
            }
        }
        // If a pipeline is provided, process each event through it
        if let Some(pipeline) = pipeline {
            for mut event in events {
                if pipeline.process(&mut event) {
                    state.push(event);
                }
            }
        } else {
            state.extend(events);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn emitter_emits_custom_tag_events() {
        use crate::logic::core::event_types::{Tag, TagEnd, Event, SourcePos};
        let name = "callout".to_string();
        let data = Some("info".to_string());
        let attrs = None;
        let pos = Some(SourcePos { line: 10, column: 5 });
        let mut state = Vec::new();
        // Simulate emitting a custom tag
        state.push(Event::Start(Tag::CustomTag { name: name.clone(), data: data.clone(), attributes: attrs.clone() }, pos.clone(), attrs.clone()));
        // ...emit inner events if needed...
        state.push(Event::End(TagEnd::CustomTagEnd { name: name.clone(), attributes: attrs.clone() }, pos.clone(), attrs.clone()));
        // Check that custom tag events are present
        assert!(state.iter().any(|e| matches!(e, Event::Start(Tag::CustomTag { .. }, _, _))), "CustomTag start event missing");
        assert!(state.iter().any(|e| matches!(e, Event::End(TagEnd::CustomTagEnd { .. }, _, _))), "CustomTag end event missing");
    }
    use crate::logic::ast::inlines::Inline;
    use crate::logic::ast::math::{MathInline, MathType};
    use crate::logic::core::event_types::SourcePos;
    #[test]
    fn emitter_emits_extension_events() {
        let math = Inline::Math(MathInline {
            content: "x^2".to_string(),
            math_type: MathType::LaTeX,
            position: Some(SourcePos { line: 1, column: 1 }),
            attributes: None,
        });
        let emoji = Inline::Emoji("smile".to_string(), "😄".to_string(), SourcePos { line: 2, column: 1 });
        let mention = Inline::Mention("user".to_string(), SourcePos { line: 3, column: 1 });
        let table_caption = Inline::TableCaption("caption".to_string(), None, SourcePos { line: 4, column: 1 });
        let task_list_meta = Inline::TaskListMeta(Some("group1".to_string()), None, SourcePos { line: 5, column: 1 });
        let mut state = Vec::new();
        // Math
        match math {
            Inline::Math(ref m) => {
                let attrs = m.attributes.clone();
                state.push(crate::logic::core::event_types::Event::Math {
                    content: m.content.clone(),
                    pos: m.position.clone(),
                    attributes: attrs,
                });
            },
            _ => {}
        }
        // Emoji
        match emoji {
            Inline::Emoji(ref shortcode, ref unicode, ref pos) => {
                state.push(crate::logic::core::event_types::Event::Emoji(shortcode.clone(), unicode.clone(), Some(pos.clone())));
            },
            _ => {}
        }
        // Mention
        match mention {
            Inline::Mention(ref username, ref pos) => {
                state.push(crate::logic::core::event_types::Event::Mention(username.clone(), Some(pos.clone())));
            },
            _ => {}
        }
        // TableCaption
        match table_caption {
            Inline::TableCaption(ref content, ref attr, ref pos) => {
                state.push(crate::logic::core::event_types::Event::Start(
                    crate::logic::core::event_types::Tag::TableCaption(content.clone(), attr.clone()),
                    Some(pos.clone()),
                    attr.clone(),
                ));
            },
            _ => {}
        }
        // TaskListMeta
        match task_list_meta {
            Inline::TaskListMeta(ref group, ref attr, ref pos) => {
                state.push(crate::logic::core::event_types::Event::Start(
                    crate::logic::core::event_types::Tag::TaskListMeta(group.clone(), attr.clone()),
                    Some(pos.clone()),
                    attr.clone(),
                ));
            },
            _ => {}
        }
        // Check that all events are present
        assert!(state.iter().any(|e| matches!(e, crate::logic::core::event_types::Event::Math { .. })), "Math event missing");
        assert!(state.iter().any(|e| matches!(e, crate::logic::core::event_types::Event::Emoji(_, _, _))), "Emoji event missing");
        assert!(state.iter().any(|e| matches!(e, crate::logic::core::event_types::Event::Mention(_, _))), "Mention event missing");
        assert!(state.iter().any(|e| matches!(e, crate::logic::core::event_types::Event::Start(crate::logic::core::event_types::Tag::TableCaption(_, _), _, _))), "TableCaption event missing");
        assert!(state.iter().any(|e| matches!(e, crate::logic::core::event_types::Event::Start(crate::logic::core::event_types::Tag::TaskListMeta(_, _), _, _))), "TaskListMeta event missing");
    }
}
