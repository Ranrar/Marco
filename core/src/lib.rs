// Marco Core Library - nom-based Markdown parser with LSP support

// Core modules: grammar → parser → AST → renderer → LSP
pub mod grammar;
pub mod parser;
pub mod render;
pub mod lsp;
pub mod logic;

// Re-export main API
pub use parser::parse;
pub use render::{render, RenderOptions};
pub use parser::{Document, Node, NodeKind};
pub use lsp::LspProvider;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};
