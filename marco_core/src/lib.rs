// Marco Core Library
// Core markdown parsing and rendering functionality without UI dependencies

pub mod components;
pub mod logic;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};

// Re-export the Marco parser for external tools
pub use components::marco_engine::{parse_markdown, AstBuilder, MarcoParser, Rule};

// Re-export parser utilities for testing and convenience
pub use components::marco_engine::{parse_document, parse_with_rule, ParseResult};

// Re-export HTML rendering
pub use components::marco_engine::render_html::{HtmlOptions, HtmlRenderer};
