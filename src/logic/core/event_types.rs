//! # Event Stream Types for Markdown Parsing
//!
//! ## Safety and Threading Guidelines (GTK4 + Rust)
//!
//! - **GTK4 is NOT thread-safe:** All widget creation, updates, and signal handling must occur on the main thread.
//! - **Profiling/Diagnostics:** Emit profiling events (timing, memory) only from the main thread, or use message passing (e.g., `glib::Sender`, channels) to communicate results from worker threads.
//! - **Memory Safety:** Use Rust's ownership/borrowing model. For shared state, prefer `Rc<RefCell<T>>` (single-threaded) or `Arc<Mutex<T>>` (multi-threaded). Avoid `.clone()` unless necessary; use `Weak` to break cycles.
//! - **Thread Safety:** Never update GTK widgets from non-main threads. Use `glib::MainContext` or `glib::idle_add_local()` to schedule UI updates.
//! - **Anti-patterns:** Avoid `.unwrap()`/`.expect()` in event emission, global mutable state, and nested locks. Release locks promptly.
//!
//! ## Profiling Event Usage
//!
//! - Use `Event::Profile(ProfileType, value, timestamp)` to emit timing/memory events.
//! - For plugin authors: Always emit events in a thread-safe manner. If profiling in a worker thread, send results to the main thread before emitting.
//!
//! ## Example: Safe Profiling Event Emission
//! ```rust ignore
//! use glib::MainContext;
//! use crate::logic::parser::event::{Event, ProfileType};
//! let sender = MainContext::channel(glib::PRIORITY_DEFAULT);
//! std::thread::spawn(move || {
//!     let value = 12345;
//!     let timestamp = 1620000000;
//!     sender.send(Event::Profile(ProfileType::ParseEnd, value, timestamp)).unwrap();
//! });
//! // In main thread: receive and emit event
//! ```
//!
//! For more, see [GTK4 Rust Book](https://gtk-rs.org/gtk4-rs/git/book/) and [Rust Concurrency Guide](https://doc.rust-lang.org/book/ch20-00-concurrency.html).
//!
//! IMPORTANT: All GTK/UI code must run on the main thread.
//! Never call GTK functions from Rayon worker threads or any thread pool.
//! Use message passing (e.g., glib::Sender) to communicate results to the main thread.
/// Profiling event types for performance hooks
///
/// # Safety
/// - Only emit profiling events from the main thread, or use message passing to forward results.
/// - Extend with more profiling types as needed for analytics/plugins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileType {
    /// Marks the start of parsing (timing)
    ParseStart,
    /// Marks the end of parsing (timing)
    ParseEnd,
    /// Marks a block render event (timing)
    BlockRender,
    /// Memory usage snapshot (bytes)
    MemoryUsage,
    // Extend with more profiling types as needed
}
/// Logical group types for event grouping
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupType {
    List,
    TableRow,
    BlockGroup,
    // Extend with more group types as needed
}
impl Tag {
    /// Helper to create a custom tag
    pub fn custom<S: Into<String>>(name: S, data: Option<String>, attributes: Option<Attributes>) -> Self {
        Tag::CustomTag {
            name: name.into(),
            data,
            attributes,
        }
    }
}
impl TagEnd {
    /// Helper to create a custom tag end
    pub fn custom<S: Into<String>>(name: S, attributes: Option<Attributes>) -> Self {
        TagEnd::CustomTagEnd {
            name: name.into(),
            attributes,
        }
    }
}
// Core event types for Markdown event stream
use crate::logic::core::attr_parser::Attributes;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Profiling/timing/memory usage event
    Profile(ProfileType, u64, u64), // (type, value, timestamp)
    /// Arbitrary metadata event (spread, association, meta, etc.)
    Meta { key: &'static str, value: Option<String>, attributes: Option<Attributes> },
    /// Marks the start of a logical group (e.g., list, table row)
    GroupStart(GroupType, Option<SourcePos>, Option<Attributes>),
    /// Marks the end of a logical group
    GroupEnd(GroupType, Option<SourcePos>, Option<Attributes>),
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
    MathBlock { content: String, math_type: Option<crate::logic::ast::math::MathType>, pos: Option<SourcePos>, attributes: Option<Attributes> },
    Emoji(String, String, Option<SourcePos>),
    Mention(String, Option<SourcePos>),
    Error(String, Option<SourcePos>),
    Warning(String, Option<SourcePos>),
    Unsupported(String, Option<SourcePos>),
    /// Inline content (used for table cells and other contexts)
    Inline(crate::logic::ast::inlines::Inline, Option<SourcePos>, Option<Attributes>),
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
    MathBlock(String, Option<crate::logic::ast::math::MathType>, Option<Attributes>),
    TableCaption(String, Option<Attributes>),
    TaskListMeta(Option<String>, Option<Attributes>),
    /// Custom tag for user-defined extensions, blocks, or inlines
    CustomTag {
        name: String,
        data: Option<String>,
        attributes: Option<Attributes>,
    },
    Table(Option<Attributes>),
    TableRow,
    TableCell,
    /// Footnote definition (GFM)
    FootnoteDefinition(String, Option<String>, Option<Attributes>),
    /// Footnote reference (GFM)
    FootnoteReference(String, Option<String>, Option<Attributes>),
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
    /// Custom tag end for user-defined extensions, blocks, or inlines
    CustomTagEnd {
        name: String,
        attributes: Option<Attributes>,
    },
    Table(Option<Attributes>),
    TableRow,
    TableCell,
    TableCaption,
    /// Footnote definition end (GFM)
    FootnoteDefinition(String),
    /// Footnote reference end (GFM)
    FootnoteReference(String),
}

// Source position tracking for advanced features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
}

#[cfg(test)]
mod tests {
    #[test]
    fn profile_event_works() {
        use super::{Event, ProfileType};
        let value = 12345;
        let timestamp = 1620000000;
        let start = Event::Profile(ProfileType::ParseStart, value, timestamp);
        let end = Event::Profile(ProfileType::ParseEnd, value + 100, timestamp + 10);
        match start {
            Event::Profile(ProfileType::ParseStart, v, t) => {
                assert_eq!(v, value);
                assert_eq!(t, timestamp);
            }
            _ => panic!("ProfileStart not matched"),
        }
        match end {
            Event::Profile(ProfileType::ParseEnd, v, t) => {
                assert_eq!(v, value + 100);
                assert_eq!(t, timestamp + 10);
            }
            _ => panic!("ProfileEnd not matched"),
        }
    }
    #[test]
    fn group_event_works() {
        use super::{Event, GroupType, SourcePos};
        let attrs = None;
        let pos = Some(SourcePos { line: 1, column: 1 });
        let start = Event::GroupStart(GroupType::List, pos.clone(), attrs.clone());
        let end = Event::GroupEnd(GroupType::List, pos.clone(), attrs.clone());
        match start {
            Event::GroupStart(GroupType::List, p, a) => {
                assert_eq!(p, pos);
                assert_eq!(a, attrs);
            }
            _ => panic!("GroupStart not matched"),
        }
        match end {
            Event::GroupEnd(GroupType::List, p, a) => {
                assert_eq!(p, pos);
                assert_eq!(a, attrs);
            }
            _ => panic!("GroupEnd not matched"),
        }
    }
    #[test]
    fn custom_tag_event_works() {
use crate::logic::core::attr_parser::Attributes;
        let attrs = Some(Attributes::default());
        let tag = Tag::custom("callout", Some("info".to_string()), attrs.clone());
        let tag_end = TagEnd::custom("callout", attrs.clone());
        let start_event = Event::Start(tag.clone(), None, attrs.clone());
        let end_event = Event::End(tag_end.clone(), None, attrs.clone());
        match start_event {
            Event::Start(Tag::CustomTag { name, data, attributes }, _, _) => {
                assert_eq!(name, "callout");
                assert_eq!(data, Some("info".to_string()));
                assert_eq!(attributes, attrs);
            }
            _ => panic!("CustomTag not matched"),
        }
        match end_event {
            Event::End(TagEnd::CustomTagEnd { name, attributes }, _, _) => {
                assert_eq!(name, "callout");
                assert_eq!(attributes, attrs);
            }
            _ => panic!("CustomTagEnd not matched"),
        }
    }
    use super::*;
    use crate::logic::ast::math::MathType;
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
