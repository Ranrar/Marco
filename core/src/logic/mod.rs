pub mod buffer;
pub mod cache;
pub mod layoutstate;
pub mod loaders;
pub mod logger;
pub mod swanson;
pub mod text_completion;
pub mod utf8;

// Re-export commonly used types
pub use buffer::{DocumentBuffer, RecentFiles};
pub use cache::{global_parser_cache, parse_to_html, parse_to_html_cached, ParserCache};
pub use text_completion::{
    emoji_completion_items, emoji_shortcode_matches_query, emoji_shortcodes_for_completion,
    normalize_completion_query,
};
pub use utf8::{sanitize_input, sanitize_input_with_stats, InputSource, SanitizeStats};
