//! Simplified parser module for Marco engine (Two-Stage Parser)
//!
//! Contains parsing functionality for the new two-stage architecture:
//! - Block parser (BlockParser) for document structure
//! - Orchestrator for full document parsing
//! - Basic error handling

// Re-export the block parser and rules
pub use crate::components::engine::grammar::{BlockParser, BlockRule};
pub use crate::components::engine::parsers::orchestrator;

// Re-export basic Pest types that might be needed
pub use pest::iterators::Pairs;
pub use pest::Parser;

/// Type alias for consistent error handling across Marco parser operations.
///
/// Uses String for errors following Marco's simplified architecture pattern.
/// Primarily used by test utilities and external API consumers.
pub type ParseResult<T> = Result<T, String>;

/// Convenience function for parsing complete markdown documents.
///
/// Uses the new two-stage parser orchestrator to build a complete AST.
/// This is the recommended way to parse markdown documents.
///
/// # Arguments
/// * `input` - The markdown text to parse
///
/// # Returns
/// * `Ok(Node)` - Successfully parsed AST
/// * `Err(String)` - Parse error with descriptive message
pub fn parse_document(input: &str) -> Result<crate::components::engine::ast_node::Node, String> {
    orchestrator::parse_document(input)
}

/// Convenience function for testing specific block grammar rules.
///
/// Allows parsing input with any specific BlockRule, useful for unit testing
/// individual grammar rules and debugging parser behavior.
///
/// # Arguments
/// * `input` - The text to parse
/// * `rule` - The specific block grammar rule to test against
///
/// # Returns
/// * `Ok(Pairs)` - Successfully parsed pest pairs
/// * `Err(String)` - Parse error with descriptive message
pub fn parse_with_rule(input: &str, rule: BlockRule) -> ParseResult<Pairs<'_, BlockRule>> {
    BlockParser::parse(rule, input).map_err(|e| e.to_string())
}
