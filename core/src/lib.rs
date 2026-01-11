// Marco Core Library - nom-based Markdown parser with LSP support

// Core modules: grammar → parser → AST → renderer → LSP
pub mod grammar;
pub mod logic;
pub mod lsp;
pub mod parser;
pub mod paths;
pub mod render;

// Re-export main API
pub use lsp::LspProvider;
pub use parser::parse;
pub use parser::{Document, Node, NodeKind};
pub use render::{render, RenderOptions};

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};
pub use logic::cache::{
    global_parser_cache, parse_to_html, parse_to_html_cached, shutdown_global_parser_cache,
    ParserCache,
};
pub use logic::logger::safe_preview;
pub use logic::utf8::{sanitize_input, sanitize_input_with_stats, InputSource, SanitizeStats};
