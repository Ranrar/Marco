// Grammar definitions for Markdown syntax

pub mod block;
pub mod inline;

// Re-export specific items to avoid ambiguous Span
pub use block::{heading, setext_heading, paragraph, list, blockquote, thematic_break, fenced_code_block, indented_code_block};
pub use inline::{code_span, emphasis, strong, link, image, inline_html};

// Use inline::Span as the default Span type (same as block::Span anyway)
pub use inline::Span;
