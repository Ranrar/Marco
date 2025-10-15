//! Two-Stage Parser Grammar Module
//!
//! This module re-exports the new two-stage parser architecture that replaces
//! the old monolithic MarcoParser.
//!
//! **New Architecture**:
//! - `BlockParser` - Parses document structure (headings, lists, code blocks, etc.)
//! - `InlineParser` - Parses inline content (bold, italic, links, etc.)
//! - `orchestrator` - Coordinates two-stage parsing and AST building
//!
//! The old monolithic grammar has been removed. All parsing now uses the modular
//! two-stage approach with separate block and inline grammars.

// Re-export block parser (primary parser for document structure)
pub use crate::components::engine::parsers::block_parser::BlockParser;
pub use crate::components::engine::parsers::block_parser::Rule as BlockRule;

// Re-export inline parser
pub use crate::components::engine::parsers::inline_parser::InlineParser;
pub use crate::components::engine::parsers::inline_parser::Rule as InlineRule;

// Re-export orchestrator Rule as the primary Rule type
pub use crate::components::engine::parsers::orchestrator::Rule;

// For backward compatibility, alias BlockParser as MarcoParser
// This allows existing code to continue using MarcoParser while we migrate
#[deprecated(
    since = "0.2.0",
    note = "Use BlockParser directly or orchestrator::parse_document() instead"
)]
pub type MarcoParser = BlockParser;
