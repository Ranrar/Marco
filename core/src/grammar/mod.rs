// Grammar definitions for Markdown syntax

pub mod block;
pub mod inline;

// Re-export specific items to avoid ambiguous Span
pub use block::{heading, paragraph, code_block, list, blockquote, table};
pub use inline::{code_span, emphasis, strong, link, image, inline_html};

// Use inline::Span as the default Span type (same as block::Span anyway)
pub use inline::Span;
