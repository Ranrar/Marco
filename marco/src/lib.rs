// Library entry point for integration tests and consumers.
// Re-export the internal modules needed by tests.
pub mod components;
pub mod footer;
pub mod logic; // UI-specific logic (menu_items, signal_manager)
pub mod theme;
pub mod ui;

// Re-export commonly used types from core
pub use marco_shared::logic::buffer::{DocumentBuffer, RecentFiles};

// Re-export the parser and renderer API
pub use marco_core::{parse, render, RenderOptions};
pub use marco_core::{Document, Node, NodeKind};

// Re-export parser cache for convenience
pub use marco_core::logic::cache::{
    global_parser_cache, parse_to_html, parse_to_html_cached, ParserCache,
};
