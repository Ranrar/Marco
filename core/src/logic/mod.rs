pub mod buffer;
pub mod cache;
pub mod crossplatforms;
pub mod layoutstate;
pub mod loaders;
pub mod logger;
pub mod swanson;
pub mod utf8;

// Re-export commonly used types
pub use buffer::{DocumentBuffer, RecentFiles};
pub use cache::{global_parser_cache, parse_to_html, parse_to_html_cached, ParserCache};
pub use utf8::{sanitize_input, sanitize_input_with_stats, InputSource, SanitizeStats};
