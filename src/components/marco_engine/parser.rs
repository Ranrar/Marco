//! Simplified parser module for Marco engine
//!
//! Contains only essential parsing functionality:
//! - Direct Pest parser usage (MarcoParser)
//! - Basic Rule enum re-export
//! - Simple error handling

// Re-export the essential Pest parser
pub use crate::components::marco_engine::grammar::{MarcoParser, Rule};

// Re-export basic Pest types that might be needed
pub use pest::iterators::{Pair, Pairs};
pub use pest::Parser;

/// Simple parse result type - use String for errors as per simplified architecture
pub type ParseResult<T> = Result<T, String>;

/// Convenience function: Parse document with Rule::document
pub fn parse_document(input: &str) -> ParseResult<Pairs<'_, Rule>> {
    MarcoParser::parse(Rule::document, input).map_err(|e| e.to_string())
}

/// Convenience function: Parse with specific rule
pub fn parse_with_rule(input: &str, rule: Rule) -> ParseResult<Pairs<'_, Rule>> {
    MarcoParser::parse(rule, input).map_err(|e| e.to_string())
}
