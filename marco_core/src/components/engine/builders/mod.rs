//! AST building from Pest pairs
//!
//! Provides modular builders for constructing AST nodes from parsed markdown.
//! Split into block-level and inline-level builders for maintainability.
//!
//! # Architecture
//!
//! - `block_builder` - Builds block-level nodes (Document, Heading, Paragraph, etc.)
//! - `inline_builder` - Builds inline-level nodes (Text, Strong, Emphasis, etc.)
//! - `helpers` - Shared utilities for both builders

pub mod block_builder;
pub mod helpers;
pub mod inline_builder;

pub use block_builder::BlockBuilder;
pub use inline_builder::InlineBuilder;

/// Error type for AST building operations
#[derive(Debug, Clone)]
pub enum AstError {
    /// Parse error with message
    ParseError(String),
    /// Invalid node structure
    InvalidStructure(String),
    /// Missing required content
    MissingContent(String),
}

impl std::fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AstError::InvalidStructure(msg) => write!(f, "Invalid structure: {}", msg),
            AstError::MissingContent(msg) => write!(f, "Missing content: {}", msg),
        }
    }
}

impl std::error::Error for AstError {}
