//! Simplified parser module for Marco engine
//!
//! Contains only essential parsing functionality:
//! - Direct Pest parser usage (MarcoParser)
//! - Basic Rule enum re-export
//! - Simple error handling

// Re-export the essential Pest parser
pub use crate::components::marco_engine::grammar::{MarcoParser, Rule};

// Re-export basic Pest types that might be needed
pub use pest::iterators::Pairs;
pub use pest::Parser;

/// Type alias for consistent error handling across Marco parser operations.
///
/// Uses String for errors following Marco's simplified architecture pattern.
/// Primarily used by test utilities and external API consumers.
#[allow(dead_code)]
pub type ParseResult<T> = Result<T, String>;

/// Convenience function for parsing complete markdown documents.
///
/// Parses input using Rule::document and provides consistent error handling.
/// This function is primarily used by the integration test suite and external
/// API consumers who need to parse full markdown documents.
///
/// # Arguments
/// * `input` - The markdown text to parse
///
/// # Returns
/// * `Ok(Pairs)` - Successfully parsed pest pairs
/// * `Err(String)` - Parse error with descriptive message
#[allow(dead_code)]
pub fn parse_document(input: &str) -> ParseResult<Pairs<'_, Rule>> {
    MarcoParser::parse(Rule::document, input).map_err(|e| e.to_string())
}

/// Convenience function for testing specific grammar rules.
///
/// Allows parsing input with any specific Rule, useful for unit testing
/// individual grammar rules and debugging parser behavior. This function
/// is extensively used by the integration test suite for grammar validation.
///
/// # Arguments
/// * `input` - The text to parse
/// * `rule` - The specific grammar rule to test against
///
/// # Returns
/// * `Ok(Pairs)` - Successfully parsed pest pairs
/// * `Err(String)` - Parse error with descriptive message
#[allow(dead_code)]
pub fn parse_with_rule(input: &str, rule: Rule) -> ParseResult<Pairs<'_, Rule>> {
    MarcoParser::parse(rule, input).map_err(|e| e.to_string())
}
