//! Two-Stage Parser Grammar Module
//!
//! - `BlockParser` - Parses document structure (headings, lists, code blocks, etc.)
//! - `InlineParser` - Parses inline content (bold, italic, links, etc.)
//! - `orchestrator` - Coordinates two-stage parsing and AST building
//!
//! two-stage approach with separate block and inline grammars.

// Re-export block parser (primary parser for document structure)
pub use crate::components::engine::parsers::block_parser::BlockParser;
pub use crate::components::engine::parsers::block_parser::Rule as BlockRule;

// Re-export inline parser
pub use crate::components::engine::parsers::inline_parser::InlineParser;
pub use crate::components::engine::parsers::inline_parser::Rule as InlineRule;

// Re-export orchestrator Rule as the primary Rule type
pub use crate::components::engine::parsers::orchestrator::Rule;

pub type MarcoParser = BlockParser;
