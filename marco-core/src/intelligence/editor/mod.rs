//! Editor-facing intelligence features.

pub mod completion;
pub mod highlight;
pub mod hover;

pub use completion::{get_markdown_completions, CompletionItem};
pub use highlight::{compute_highlights, compute_highlights_with_source, Highlight, HighlightTag};
pub use hover::{get_hover_info, get_position_span, HoverInfo};
