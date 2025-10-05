// Library entry point for integration tests and consumers.
// Re-export the internal modules needed by tests.
pub mod components;
pub mod footer;
pub mod logic;  // UI-specific logic (menu_items, signal_manager)
pub mod theme;
pub mod ui;

// Re-export commonly used types from marco_core
pub use marco_core::logic::buffer::{DocumentBuffer, RecentFiles};

// Re-export the Marco parser from core
pub use marco_core::{parse_markdown, AstBuilder, MarcoParser, Rule};

// Re-export parser utilities for testing and convenience
pub use marco_core::{parse_document, parse_with_rule, ParseResult};

// Re-export HTML rendering
pub use marco_core::components::marco_engine::render_html::{HtmlOptions, HtmlRenderer};
