// Marco Core Library
// Core markdown parsing and rendering functionality without UI dependencies

pub mod components;
pub mod logic;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};

// ============================================================================
// RE-EXPORTS: Marco Engine API
// ============================================================================

pub use components::engine::api::{
    parse_markdown,      // Parse markdown to AST
    render_to_html,      // Render AST to HTML
    parse_and_render,    // One-step parse + render
};

// Parser utilities
pub use components::engine::parser::{
    parse_document,      // Two-stage orchestrator parsing
    parse_with_rule,     // Test specific grammar rules
    ParseResult,         // Result type for parsing
};

// Core types
pub use components::engine::{Rule, Node};

// HTML rendering options
pub use components::engine::renderers::HtmlOptions;

