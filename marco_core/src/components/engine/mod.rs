//! Marco Engine - Modular grammar-centered parsing and rendering system
//!
//! This module provides a streamlined markdown processing engine with:
//! - Pest-based parsing with Marco grammar
//! - Modular AST building (builders/)
//! - Modular HTML rendering (renderers/)
//! - Clean API: api::parse_markdown(), api::render_to_html(), api::parse_and_render()
//! - Block-level caching for performance optimization
//!
// ============================================================================
// MODULAR ARCHITECTURE (PRIMARY API)
// ============================================================================

pub mod api;         // Public API functions
pub mod builders;    // AST builders (block + inline)
pub mod renderers;   // HTML renderers (block + inline)
pub mod span;        // Span utilities

// ============================================================================
// CORE COMPONENTS
// ============================================================================

pub mod ast_node;         // AST node definitions (CommonMark only)
pub mod entity_table;     // HTML5 entity decoding (Phase 4)
pub mod grammar;          // Pest grammar
pub mod parser;           // Parser utilities
pub mod parser_cache;     // Caching layer
pub mod parsers;          // Two-stage parser orchestrator
pub mod reference_resolver; // Reference link/image resolution

// ============================================================================
// PUBLIC RE-EXPORTS
// ============================================================================

// Core types
pub use ast_node::Node;
pub use grammar::{Rule};

// API
pub use api::{parse_markdown as parse_to_ast, render_to_html, parse_and_render};

// Caching
pub use parser_cache::global_parser_cache;

// Renderers
pub use renderers::HtmlOptions;