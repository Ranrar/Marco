// Inline-level grammar modules
//
// This module contains individual CommonMark inline element parsers.
// Each parser extracts a specific inline construct and returns nom IResult.
//
// Phase 4: Inline grammar module extraction - IN PROGRESS

// Re-export the Span type for use by all inline modules
pub use nom_locate::LocatedSpan;
pub type Span<'a> = LocatedSpan<&'a str>;

// Individual inline grammar modules
pub mod cm_autolink;
pub mod cm_backslash_escape;
pub mod cm_code_span;
pub mod cm_dash_strikethrough;
pub mod cm_emphasis;
pub mod cm_image;
pub mod cm_inline_html;
pub mod cm_line_breaks;
pub mod cm_link;
pub mod cm_mark;
pub mod cm_strikethrough;
pub mod cm_strong;
pub mod cm_strong_emphasis;
pub mod cm_subscript;
pub mod cm_subscript_arrow;
pub mod cm_superscript;

// Re-export all parser functions for convenience
pub use cm_autolink::autolink;
pub use cm_backslash_escape::backslash_escape;
pub use cm_code_span::code_span;
pub use cm_dash_strikethrough::dash_strikethrough;
pub use cm_emphasis::emphasis;
pub use cm_image::image;
pub use cm_inline_html::inline_html;
pub use cm_line_breaks::{hard_line_break, soft_line_break};
pub use cm_link::link;
pub use cm_mark::mark;
pub use cm_strikethrough::strikethrough;
pub use cm_strong::strong;
pub use cm_strong_emphasis::strong_emphasis;
pub use cm_subscript::subscript;
pub use cm_subscript_arrow::subscript_arrow;
pub use cm_superscript::superscript;
