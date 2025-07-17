
//! # 6 Inlines (CommonMark 0.31.2)
//
//! This module defines the Abstract Syntax Tree (AST) node types for CommonMark section 6 (Inlines),
//! covering all inline elements as described in the specification:
//! # 6 inlines (CommonMark 0.31.2)
//!   6.1 Code spans
//!   6.2 Emphasis and strong emphasis
//!   6.3 Links
//!   6.4 Images
//!   6.5 Autolinks
//!   6.6 Raw HTML
//!   6.7 Hard line breaks
//!   6.8 Soft line breaks
//!   6.9 Textual content
//
//! Each enum/struct is documented with its corresponding section and purpose.

/// The main enum representing any inline element (section 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// 6.1 Code spans: Inline code delimited by backticks.
    Code(CodeSpan),
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
}

// === 6.1 Code spans ===

/// 6.1 Code spans: Inline code delimited by backticks (`...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSpan {
    /// The literal code content (spaces and line endings normalized as per spec).
    pub content: String,
}

// === 6.2 Emphasis and strong emphasis ===

/// 6.2 Emphasis and strong emphasis: *em*, **strong**, possibly nested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Emphasis {
    /// Emphasized text (single * or _).
    Emph(Vec<Inline>),
    /// Strongly emphasized text (double ** or __).
    Strong(Vec<Inline>),
}

// === 6.3 Links ===

/// 6.3 Links: Inline, reference, shortcut, and collapsed links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The link text (inline content inside [ ]).
    pub label: Vec<Inline>,
    /// The link destination (URL or reference label).
    pub destination: LinkDestination,
    /// Optional link title (from title attribute or reference definition).
    pub title: Option<String>,
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
    pub alt: Vec<Inline>,
    /// The image source (URL or reference label).
    pub destination: LinkDestination,
    /// Optional image title (from title attribute or reference definition).
    pub title: Option<String>,
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
