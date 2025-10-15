// Marco Core Library
// Core markdown parsing and rendering functionality without UI dependencies

pub mod components;
pub mod logic;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};

// ============================================================================
// RE-EXPORTS: Marco Engine API (Phase 2 Modular Architecture)
// ============================================================================

// New modular API (Phase 2.5)
pub use components::engine::api::{
    parse_markdown,      // Parse markdown to AST
    render_to_html,      // Render AST to HTML
    parse_and_render,    // One-step parse + render
};

// Core types
pub use components::engine::{MarcoParser, Rule, Node};

// HTML rendering options
pub use components::engine::renderers::HtmlOptions;

// Parser utilities (deprecated - use api functions instead)
#[deprecated(since = "0.2.0", note = "Use components::engine::parser module directly")]
pub use components::engine::parser::{ParseResult, parse_document, parse_with_rule};

