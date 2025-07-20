use anyhow::Error;

/// Type alias for AST results with anyhow error handling.
pub type AstResult<T> = Result<T, Error>;

/// Example: minimal error-producing function for demonstration.
pub fn parse_inline_safe(is_valid: bool) -> AstResult<Inline> {
    if !is_valid {
        Err(Error::msg("Invalid inline"))
    } else {
        Ok(Inline::Text("dummy".to_string()))
    }
}



impl Inline {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_inline(self);
    }
}

impl Emphasis {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_emphasis(self);
    }
}

impl Link {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_link(self);
    }
}

impl Image {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_image(self);
    }
}

impl CodeSpan {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_code_span(self);
    }
}

impl Autolink {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_autolink(self);
    }
}
/// Trait for visiting AST nodes in inlines.rs
pub trait AstVisitor {
    fn visit_inline(&mut self, inline: &Inline) {
        match inline {
            Inline::CodeSpan(cs) => self.visit_code_span(cs),
            Inline::Emphasis(emph) => self.visit_emphasis(emph),
            Inline::Link(link) => self.visit_link(link),
            Inline::Image(image) => self.visit_image(image),
            Inline::Autolink(autolink) => self.visit_autolink(autolink),
            Inline::RawHtml(_) => self.visit_raw_html(inline),
            Inline::HardBreak => self.visit_hard_break(inline),
            Inline::SoftBreak => self.visit_soft_break(inline),
            Inline::Text(_) => self.visit_text(inline),
            Inline::Math(math) => self.visit_math_inline(math),
            Inline::Emoji(_, _, _) => self.visit_emoji(inline),
            Inline::Mention(_, _) => self.visit_mention(inline),
            Inline::TableCaption(_, _, _) => self.visit_table_caption(inline),
            Inline::TaskListMeta(_, _, _) => self.visit_task_list_meta(inline),
        }
    }

    fn visit_code_span(&mut self, _cs: &CodeSpan) {}
    fn visit_emphasis(&mut self, emph: &Emphasis) {
        self.walk_emphasis(emph);
    }
    fn walk_emphasis(&mut self, emph: &Emphasis) {
        match emph {
            Emphasis::Emph(inlines, _) | Emphasis::Strong(inlines, _) => {
                for (inline, _) in inlines {
                    self.visit_inline(inline);
                }
            }
        }
    }
    fn visit_link(&mut self, link: &Link) {
        self.walk_link(link);
    }
    fn walk_link(&mut self, link: &Link) {
        for (inline, _) in &link.label {
            self.visit_inline(inline);
        }
    }
    fn visit_image(&mut self, image: &Image) {
        self.walk_image(image);
    }
    fn walk_image(&mut self, image: &Image) {
        for (inline, _) in &image.alt {
            self.visit_inline(inline);
        }
    }
    fn visit_autolink(&mut self, _autolink: &Autolink) {}
    fn visit_raw_html(&mut self, _inline: &Inline) {}
    fn visit_hard_break(&mut self, _inline: &Inline) {}
    fn visit_soft_break(&mut self, _inline: &Inline) {}
    fn visit_text(&mut self, _inline: &Inline) {}
    fn visit_math_inline(&mut self, _math: &crate::logic::ast::math::MathInline) {}
    fn visit_emoji(&mut self, _inline: &Inline) {}
    fn visit_mention(&mut self, _inline: &Inline) {}
    fn visit_table_caption(&mut self, _inline: &Inline) {}
    fn visit_task_list_meta(&mut self, _inline: &Inline) {}
}
/// # 6 Inlines (CommonMark 0.31.2)
///
/// This module defines the Abstract Syntax Tree (AST) node types for CommonMark section 6 (Inlines),
/// covering all inline elements as described in the specification:
/// # 6 inlines (CommonMark 0.31.2)
///   6.1 Code spans
///   6.2 Emphasis and strong emphasis
///   6.3 Links
///   6.4 Images
///   6.5 Autolinks
///   6.6 Raw HTML
///   6.7 Hard line breaks
///   6.8 Soft line breaks
///   6.9 Textual content
///
/// Each enum/struct is documented with its corresponding section and purpose.

/// The main enum representing any inline element (section 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// 6.1 Code spans: Inline code delimited by backticks.
    CodeSpan(CodeSpan),
    /// 6.2 Emphasis and strong emphasis: *em*, **strong**, possibly nested.
    Emphasis(Emphasis),
    /// 6.3 Links: Inline, reference, shortcut, and collapsed links.
    Link(Link),
    /// 6.4 Images: Inline, reference, shortcut, and collapsed images.
    Image(Image),
    /// 6.5 Autolinks: <scheme:...> and <email@...>.
    Autolink(Autolink),
    /// 6.6 Raw HTML: Inline HTML tags, comments, declarations, etc.
    RawHtml(String),
    /// 6.7 Hard line break: Two spaces or backslash at end of line.
    HardBreak,
    /// 6.8 Soft line break: Regular line ending not preceded by two spaces or backslash.
    SoftBreak,
    /// 6.9 Textual content: Any literal text not otherwise interpreted.
    Text(String),
    /// Math inline (GFM/LaTeX, e.g., $ ... $)
    Math(crate::logic::ast::math::MathInline),
    /// Emoji inline (e.g., :smile:)
    Emoji(String, String, crate::logic::core::event_types::SourcePos),
    /// Mention inline (e.g., @username)
    Mention(String, crate::logic::core::event_types::SourcePos),
    /// Table caption inline
    TableCaption(String, Option<crate::logic::attr_parser::Attributes>, crate::logic::core::event_types::SourcePos),
    /// Task list metadata inline
    TaskListMeta(Option<String>, Option<crate::logic::attr_parser::Attributes>, crate::logic::core::event_types::SourcePos),
}


// === 6.1 Code spans ===

/// 6.1 Code spans: Inline code delimited by backticks (`...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSpan {
    /// The literal code content (spaces and line endings normalized as per spec).
    pub content: String,
    pub attributes: Option<crate::logic::attr_parser::Attributes>,
}

// === 6.2 Emphasis and strong emphasis ===

/// 6.2 Emphasis and strong emphasis: *em*, **strong**, possibly nested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Emphasis {
    /// Emphasized text (single * or _).
    Emph(Vec<(Inline, crate::logic::core::event_types::SourcePos)>, Option<crate::logic::attr_parser::Attributes>),
    /// Strongly emphasized text (double ** or __).
    Strong(Vec<(Inline, crate::logic::core::event_types::SourcePos)>, Option<crate::logic::attr_parser::Attributes>),
}

// === 6.3 Links ===

/// 6.3 Links: Inline, reference, shortcut, and collapsed links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The link text (inline content inside [ ]).
    pub label: Vec<(Inline, crate::logic::core::event_types::SourcePos)>,
    /// The link destination (URL or reference label).
    pub destination: LinkDestination,
    /// Optional link title (from title attribute or reference definition).
    pub title: Option<String>,
    pub attributes: Option<crate::logic::attr_parser::Attributes>,
}

/// The destination of a link: either a direct URI or a reference label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkDestination {
    /// Inline link destination (e.g., [text](url)).
    Inline(String),
    /// Reference link (e.g., [text][label], [text][], [label]: url).
    Reference(String),
}

// === 6.4 Images ===

/// 6.4 Images: Inline, reference, shortcut, and collapsed images.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The alt text (inline content inside [ ]).
    pub alt: Vec<(Inline, crate::logic::core::event_types::SourcePos)>,
    /// The image source (URL or reference label).
    pub destination: LinkDestination,
    /// Optional image title (from title attribute or reference definition).
    pub title: Option<String>,
    pub attributes: Option<crate::logic::attr_parser::Attributes>,
}

// === 6.5 Autolinks ===

/// 6.5 Autolinks: <scheme:...> and <email@...>.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Autolink {
    /// URI autolink (e.g., <http://example.com>).
    Uri(String),
    /// Email autolink (e.g., <user@example.com>).
    Email(String),
}

// === 6.6 Raw HTML ===

// See Inline::RawHtml(String) above.

// === 6.7 Hard line breaks ===

// See Inline::HardBreak above.

// === 6.8 Soft line breaks ===

// See Inline::SoftBreak above.

// === 6.9 Textual content ===

// See Inline::Text(String) above.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::core::event_types::SourcePos;

    #[test]
    fn test_text_inline() {
        let inline = Inline::Text("plain text".to_string());
        struct Printer;
        impl AstVisitor for Printer {
            fn visit_text(&mut self, inline: &Inline) {
                if let Inline::Text(s) = inline {
                    assert_eq!(s, "plain text");
                }
            }
        }
        let mut printer = Printer;
        printer.visit_text(&inline);
    }

    #[test]
    fn test_emphasis_traversal() {
        let emph = Emphasis::Emph(vec![(Inline::Text("emph".to_string()), SourcePos { line: 1, column: 1 })], None);
        struct Printer;
        impl AstVisitor for Printer {
            fn visit_emphasis(&mut self, emph: &Emphasis) {
                self.walk_emphasis(emph);
            }
            fn visit_text(&mut self, inline: &Inline) {
                if let Inline::Text(s) = inline {
                    assert_eq!(s, "emph");
                }
            }
        }
        let mut printer = Printer;
        printer.visit_emphasis(&emph);
    }

    #[test]
    fn test_error_handling() {
        let result = super::parse_inline_safe(false);
        assert!(result.is_err());
        let result = super::parse_inline_safe(true);
        assert!(result.is_ok());
    }
}
