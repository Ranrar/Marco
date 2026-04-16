pub mod cache;
pub mod logger;
pub mod text_completion;
pub mod utf8;

// Re-export commonly used types
pub use cache::{global_parser_cache, parse_to_html, parse_to_html_cached, ParserCache};
pub use utf8::{sanitize_input, sanitize_input_with_stats, InputSource, SanitizeStats};
