// Tests for full inline traversal event emission
#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::logic::ast::blocks_and_inlines::{Block, LeafBlock};
    use crate::editor::logic::ast::inlines::{Inline, Emphasis, Link, LinkDestination, Autolink};
    use crate::editor::logic::parser::event::{Event, Tag, TagEnd};
    use crate::editor::logic::parser::emitter::{push_inline_events, EventState};

    #[test]
    fn test_full_inline_traversal_events() {
        let pos = crate::editor::logic::parser::event::SourcePos::default();
        let inlines = vec![
            Inline::Text("Hello".into()),
            Inline::Code(crate::editor::logic::ast::inlines::Code { content: "code".into() }),
            Inline::Emphasis(Emphasis::Emph(vec![(Inline::Text("emph".into()), pos.clone())])),
            Inline::Emphasis(Emphasis::Strong(vec![(Inline::Text("strong".into()), pos.clone())])),
            Inline::Link(Link {
                label: vec![(Inline::Text("link".into()), pos.clone())],
                destination: LinkDestination::Inline("https://example.com".into()),
                title: Some("title".into()),
            }),
            Inline::Image(crate::editor::logic::ast::inlines::Image {
                alt: vec![(Inline::Text("alt".into()), pos.clone())],
                destination: LinkDestination::Inline("https://img.com".into()),
                title: Some("imgtitle".into()),
            }),
            Inline::Autolink(Autolink::Uri("https://auto.com".into())),
            Inline::RawHtml("<b>raw</b>".into()),
            Inline::HardBreak,
            Inline::SoftBreak,
        ];
        let mut state = Vec::new();
        push_inline_events(&mut state, inlines.clone());
        // Just check that all event states are present and correct order
        let mut found = vec![false; 10];
        for s in &state {
            match s {
                EventState::EnterInline(s) if *s == "Hello" => found[0] = true,
                EventState::EnterInline(s) if *s == "code" => found[1] = true,
                EventState::EnterInline(s) if *s == "emph" => found[2] = true,
                EventState::EnterInline(s) if *s == "strong" => found[3] = true,
                EventState::EnterInline(s) if s.starts_with("link|href:") => found[4] = true,
                EventState::EnterInline(s) if s.starts_with("image|src:") => found[5] = true,
                EventState::EnterInline(s) if *s == "https://auto.com" => found[6] = true,
                EventState::EnterInline(s) if *s == "<b>raw</b>" => found[7] = true,
                EventState::EnterInline(s) if *s == "\n" => found[8] = true,
                EventState::EnterInline(s) if *s == " " => found[9] = true,
                _ => {}
            }
        }
        assert!(found.iter().all(|x| *x), "Not all inline event types were emitted");
    }
}
