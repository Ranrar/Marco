// Library entry point for integration tests and consumers.
// Re-export the internal modules needed by tests.
pub mod components;
pub mod footer;
pub mod logic;
pub mod theme;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};

// Re-export the Marco parser for external tools
pub use components::marco_engine::{
    parse_document, parse_markdown, parse_with_rule, print_pairs, AstBuilder, MarcoParser, Rule,
};

// Re-export HTML rendering
pub use components::marco_engine::render_html::{HtmlOptions, HtmlRenderer};
