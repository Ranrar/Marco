// Marco Core Library - nom-based Markdown parser with intelligence support

// Core modules: grammar → parser → AST → renderer → intelligence
pub mod grammar;
pub mod intelligence;
pub mod logic;
pub mod parser;
pub mod render;

// Re-export main API
pub use intelligence::MarkdownIntelligenceProvider;
pub use parser::parse;
pub use parser::{Document, Node, NodeKind};
pub use render::{render, RenderOptions};

// Re-export commonly used types
pub use logic::cache::{parse_to_html, parse_to_html_cached, ParserCache};
pub use logic::utf8::{sanitize_input, sanitize_input_with_stats, InputSource, SanitizeStats};
