// Marco Core Library - nom-based Markdown parser with LSP support

// Core modules: grammar → parser → AST → renderer → LSP
pub mod grammar;
pub mod parser;
pub mod render;
pub mod lsp;
pub mod logic;
pub mod paths;

// Re-export main API
pub use parser::parse;
pub use render::{render, RenderOptions};
pub use parser::{Document, Node, NodeKind};
pub use lsp::LspProvider;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};
pub use logic::cache::{global_parser_cache, parse_to_html, parse_to_html_cached, ParserCache, shutdown_global_parser_cache};
