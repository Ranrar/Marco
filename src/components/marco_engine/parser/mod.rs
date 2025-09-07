//! Parser module for Marco engine
//!
//! This module provides comprehensive parsing functionality for Marco markup
//! including position tracking, enhanced error handling, and grammar analysis.
//!
//! # Module Structure
//!
//! - `marco_parser`: Enhanced parser wrapper with caching and analysis
//! - `position`: Position tracking and source span utilities
//! - `utils`: Core parsing utilities and grammar analysis tools
//!
//! # Quick Start
//!
//! ```rust
//! use marco_engine::parser::{EnhancedMarcoParser, ParserConfig};
//!
//! let mut parser = EnhancedMarcoParser::new();
//! let result = parser.parse_document("# Hello World\n\nThis is a test.");
//!
//! match result.nodes {
//!     Ok(nodes) => println!("Parsed {} nodes", nodes.len()),
//!     Err(e) => eprintln!("Parse error: {}", e),
//! }
//! ```

pub mod marco_parser;
pub mod position;
pub mod utils;

// Re-export main types for convenience
pub use marco_parser::{
    CacheStats, EnhancedMarcoParser, ParseResult, ParseStats, ParserConfig, RuleAnalysis,
};

pub use position::{
    ErrorSeverity, Position, PositionTracker, PositionedError, SourceSpan, SpanExt,
};

pub use utils::{
    categorize_rule, extract_rule_dependencies, get_rule_by_name, is_valid_rule_name,
    pairs_to_parse_tree, ParseNode,
};

/// Convenience function to quickly parse a document
pub fn parse_document(input: &str) -> ParseResult {
    let mut parser = EnhancedMarcoParser::new();
    parser.parse_document(input)
}

/// Convenience function to quickly parse inline content
pub fn parse_inline(input: &str) -> ParseResult {
    let mut parser = EnhancedMarcoParser::new();
    parser.parse_inline(input)
}

/// Convenience function to quickly parse block content
pub fn parse_block(input: &str) -> ParseResult {
    let mut parser = EnhancedMarcoParser::new();
    parser.parse_block(input)
}

/// Quick validation without creating a parse tree
pub fn validate_syntax(
    rule_name: &str,
    input: &str,
) -> Result<bool, crate::components::marco_engine::errors::MarcoError> {
    let mut parser = EnhancedMarcoParser::new();
    parser.validate(rule_name, input)
}

/// Parse with specific rule and return only success/failure
pub fn quick_parse_check(rule_name: &str, input: &str) -> bool {
    match validate_syntax(rule_name, input) {
        Ok(valid) => valid,
        Err(_) => false,
    }
}

/// Analyze rule usage in document
pub fn analyze_document(
    input: &str,
) -> Result<RuleAnalysis, crate::components::marco_engine::errors::MarcoError> {
    let mut parser = EnhancedMarcoParser::new();
    parser.analyze_rule_usage(input)
}

/// Create a parser with performance-optimized configuration
pub fn create_performance_parser() -> EnhancedMarcoParser {
    let config = ParserConfig {
        track_positions: false, // Disable for performance
        enable_cache: true,     // Enable caching
        max_cache_size: 200,    // Larger cache
        detailed_errors: false, // Minimal error info
        collect_stats: false,   // No statistics
    };
    EnhancedMarcoParser::with_config(config)
}

/// Create a parser with debugging-optimized configuration
pub fn create_debug_parser() -> EnhancedMarcoParser {
    let config = ParserConfig {
        track_positions: true, // Enable for debugging
        enable_cache: false,   // Disable for consistency
        max_cache_size: 50,
        detailed_errors: true, // Full error reporting
        collect_stats: true,   // Collect statistics
    };
    EnhancedMarcoParser::with_config(config)
}

/// Parse with automatic rule detection based on content
pub fn auto_parse(input: &str) -> ParseResult {
    let mut parser = EnhancedMarcoParser::new();

    // Try to detect the most appropriate rule based on content
    let rule_name = if input.trim().is_empty() {
        "text"
    } else if input.starts_with('#') || input.contains('\n') {
        "file" // Multi-line or heading content
    } else if input.contains('*') || input.contains('_') || input.contains('[') {
        "inline" // Likely has formatting
    } else {
        "text" // Plain text
    };

    parser.parse_with_rule(rule_name, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convenience_functions() {
        let input = "# Test\n\nHello world";

        let result = parse_document(input);
        assert!(result.nodes.is_ok());

        let inline_result = parse_inline("Hello *world*");
        assert!(inline_result.nodes.is_ok());
    }

    #[test]
    fn test_validation() {
        assert!(quick_parse_check("text", "hello world"));
        assert!(!quick_parse_check("nonexistent_rule", "test"));
    }

    #[test]
    fn test_auto_parse() {
        // Test heading detection
        let result = auto_parse("# Heading");
        assert!(result.nodes.is_ok());
        assert_eq!(
            result.rule,
            crate::components::marco_engine::grammar::Rule::file
        );

        // Test inline detection
        let result = auto_parse("Hello *world*");
        assert!(result.nodes.is_ok());
        assert_eq!(
            result.rule,
            crate::components::marco_engine::grammar::Rule::inline
        );

        // Test plain text
        let result = auto_parse("plain text");
        assert!(result.nodes.is_ok());
        assert_eq!(
            result.rule,
            crate::components::marco_engine::grammar::Rule::text
        );
    }

    #[test]
    fn test_parser_configurations() {
        let _perf_parser = create_performance_parser();
        let _debug_parser = create_debug_parser();

        // Test that parsers can be created successfully
        // Config validation would need public getters to test properly
    }

    #[test]
    fn test_document_analysis() {
        let input = "# Title\n\n*Hello* world\n\n- Item 1\n- Item 2";
        let analysis = analyze_document(input);

        assert!(analysis.is_ok());
        let analysis = analysis.unwrap();
        assert!(analysis.total_nodes > 0);
        assert!(!analysis.rule_counts.is_empty());
    }
}
