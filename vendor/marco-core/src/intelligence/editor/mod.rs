//! Editor-facing intelligence features.

/// Markdown completion providers and completion item types.
#[cfg(feature = "intelligence-completions")]
pub mod completion;
/// Syntax highlight extraction from parsed Markdown AST.
#[cfg(feature = "intelligence-highlights")]
pub mod highlight;
/// Hover information lookup utilities.
#[cfg(feature = "intelligence-hover")]
pub mod hover;

/// Re-export completion API.
#[cfg(feature = "intelligence-completions")]
pub use completion::{get_markdown_completions, CompletionItem};
/// Re-export highlight API.
#[cfg(feature = "intelligence-highlights")]
pub use highlight::{compute_highlights, compute_highlights_with_source, Highlight, HighlightTag};
/// Re-export hover API.
#[cfg(feature = "intelligence-hover")]
pub use hover::{get_hover_info, get_position_span, HoverInfo};
