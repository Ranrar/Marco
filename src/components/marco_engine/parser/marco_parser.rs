//! Enhanced Marco parser wrapper
//!
//! This module provides an enhanced wrapper around the basic Pest parser
//! with additional functionality for error handling, caching, and analysis.

use crate::components::marco_engine::{
    errors::{MarcoError, MarcoResult},
    grammar::{MarcoParser, Rule},
    parser::{
        position::{PositionTracker, PositionedError, SourceSpan},
        utils::{categorize_rule, get_rule_by_name, pairs_to_parse_tree, ParseNode},
    },
};
use lru::LruCache;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

/// Enhanced parser with additional functionality
#[derive(Debug)]
pub struct EnhancedMarcoParser {
    /// Position tracker for source mapping
    position_tracker: Option<PositionTracker>,
    /// LRU cache for performance
    cache: LruCache<String, CachedParseResult>,
    /// Parser configuration
    config: ParserConfig,
}

/// Configuration for the enhanced parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Enable position tracking
    pub track_positions: bool,
    /// Enable parse result caching
    pub enable_cache: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable detailed error reporting
    pub detailed_errors: bool,
    /// Collect parsing statistics
    pub collect_stats: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            track_positions: true,
            enable_cache: false, // Disabled by default for memory efficiency
            max_cache_size: 100,
            detailed_errors: true,
            collect_stats: false,
        }
    }
}

/// Cached parse result
#[derive(Debug, Clone)]
struct CachedParseResult {
    result: Result<Vec<ParseNode>, String>,
    timestamp: Instant,
    rule: Rule,
}

/// Detailed parse result with metadata
#[derive(Debug)]
pub struct ParseResult {
    /// The parse tree if successful
    pub nodes: MarcoResult<Vec<ParseNode>>,
    /// Parsing statistics
    pub stats: ParseStats,
    /// Any warnings generated during parsing
    pub warnings: Vec<PositionedError>,
    /// The rule used for parsing
    pub rule: Rule,
}

/// Statistics collected during parsing
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// Time taken to parse
    pub parse_time: Duration,
    /// Number of nodes created
    pub node_count: usize,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Memory estimate (rough)
    pub memory_estimate: usize,
    /// Cache hit/miss info
    pub cache_hit: bool,
}

impl EnhancedMarcoParser {
    /// Create a new enhanced parser
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        let cache_capacity =
            NonZeroUsize::new(config.max_cache_size).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            position_tracker: None,
            cache: LruCache::new(cache_capacity),
            config,
        }
    }

    /// Parse input with a specific rule name
    pub fn parse_with_rule(&mut self, rule_name: &str, input: &str) -> ParseResult {
        let rule = match get_rule_by_name(rule_name) {
            Some(r) => r,
            None => {
                return ParseResult {
                    nodes: Err(MarcoError::parse_error(format!(
                        "Unknown rule: {}",
                        rule_name
                    ))),
                    stats: ParseStats::default(),
                    warnings: vec![],
                    rule: Rule::file, // Default fallback
                };
            }
        };

        self.parse_with_rule_enum(rule, input)
    }

    /// Parse input with a Rule enum value
    pub fn parse_with_rule_enum(&mut self, rule: Rule, input: &str) -> ParseResult {
        let start_time = Instant::now();
        let mut stats = ParseStats::default();
        let mut warnings = Vec::new();

        // Setup position tracking if enabled
        if self.config.track_positions {
            self.position_tracker = Some(PositionTracker::new(input.to_string()));
        }

        // Check cache if enabled
        if self.config.enable_cache {
            let cache_key = format!("{:?}:{}", rule, input);
            if let Some(cached) = self.cache.get(&cache_key) {
                stats.cache_hit = true;
                stats.parse_time = start_time.elapsed();

                return ParseResult {
                    nodes: cached
                        .result
                        .clone()
                        .map_err(|e| MarcoError::parse_error(e)),
                    stats,
                    warnings,
                    rule,
                };
            }
        }

        // Perform the actual parsing with error recovery
        let parse_result = MarcoParser::parse(rule, input);
        let parse_time = start_time.elapsed();

        let nodes_result = match parse_result {
            Ok(pairs) => {
                let nodes = pairs_to_parse_tree(pairs);

                // Collect statistics if enabled
                if self.config.collect_stats {
                    stats.node_count = nodes.iter().map(|n| n.node_count()).sum();
                    stats.max_depth = nodes.iter().map(|n| n.depth()).max().unwrap_or(0);
                    stats.memory_estimate = self.estimate_memory_usage(&nodes);
                }

                // Validate parse tree and collect warnings
                if self.config.detailed_errors {
                    warnings.extend(self.validate_parse_tree(&nodes));
                }

                Ok(nodes)
            }
            Err(original_error) => {
                // Error recovery: try fallback rules based on the failed rule
                if let Some(fallback_rules) = self.get_fallback_rules(rule) {
                    for fallback_rule in fallback_rules {
                        if let Ok(pairs) = MarcoParser::parse(fallback_rule, input) {
                            let nodes = pairs_to_parse_tree(pairs);

                            // Add warning about fallback recovery
                            let warning_msg = format!(
                                "Recovered using rule {:?} after {:?} failed: {}",
                                fallback_rule, rule, original_error
                            );

                            // Create positioned warning (using the whole input as span for simplicity)
                            use crate::components::marco_engine::parser::position::{
                                Position, PositionedError, SourceSpan,
                            };
                            let start_pos = Position::start();
                            let end_pos = Position::new(input.len(), 1, input.len() + 1);
                            let span = SourceSpan::new(start_pos, end_pos);
                            warnings.push(PositionedError::warning(warning_msg, span));

                            // Collect statistics for successful fallback
                            if self.config.collect_stats {
                                stats.node_count = nodes.iter().map(|n| n.node_count()).sum();
                                stats.max_depth =
                                    nodes.iter().map(|n| n.depth()).max().unwrap_or(0);
                                stats.memory_estimate = self.estimate_memory_usage(&nodes);
                            }

                            return ParseResult {
                                nodes: Ok(nodes),
                                stats,
                                warnings,
                                rule: fallback_rule, // Update to show the successful rule
                            };
                        }
                    }
                }

                // No recovery possible, return original error
                Err(MarcoError::parse_error(format!(
                    "Parse error: {}",
                    original_error
                )))
            }
        };

        // Cache result if enabled
        if self.config.enable_cache {
            let cache_key = format!("{:?}:{}", rule, input);
            let cached_result = CachedParseResult {
                result: match &nodes_result {
                    Ok(nodes) => Ok(nodes.clone()),
                    Err(e) => Err(e.to_string()),
                },
                timestamp: Instant::now(),
                rule,
            };

            // LruCache automatically manages size and eviction
            self.cache.put(cache_key, cached_result);
        }

        stats.parse_time = parse_time;

        ParseResult {
            nodes: nodes_result,
            stats,
            warnings,
            rule,
        }
    }

    /// Parse a complete document (uses 'file' rule)
    pub fn parse_document(&mut self, input: &str) -> ParseResult {
        self.parse_with_rule_enum(Rule::file, input)
    }

    /// Parse inline content
    pub fn parse_inline(&mut self, input: &str) -> ParseResult {
        self.parse_with_rule_enum(Rule::inline, input)
    }

    /// Parse block content
    pub fn parse_block(&mut self, input: &str) -> ParseResult {
        self.parse_with_rule_enum(Rule::block, input)
    }

    /// Quick validation - just check if input is valid for a rule
    pub fn validate(&mut self, rule_name: &str, input: &str) -> MarcoResult<bool> {
        let rule = get_rule_by_name(rule_name)
            .ok_or_else(|| MarcoError::parse_error(format!("Unknown rule: {}", rule_name)))?;

        match MarcoParser::parse(rule, input) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Clear the parse cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            max_size: self.config.max_cache_size,
            enabled: self.config.enable_cache,
        }
    }

    /// Get position tracker if available
    pub fn position_tracker(&self) -> Option<&PositionTracker> {
        self.position_tracker.as_ref()
    }

    /// Estimate memory usage of parse tree (rough calculation)
    fn estimate_memory_usage(&self, nodes: &[ParseNode]) -> usize {
        nodes
            .iter()
            .map(|node| self.estimate_node_memory(node))
            .sum()
    }

    fn estimate_node_memory(&self, node: &ParseNode) -> usize {
        // Base size of the node structure
        let base_size = std::mem::size_of::<ParseNode>();

        // Size of the text content
        let text_size = node.text.len();

        // Size of children (recursive)
        let children_size: usize = node
            .children
            .iter()
            .map(|child| self.estimate_node_memory(child))
            .sum();

        base_size + text_size + children_size
    }

    /// Validate parse tree and generate warnings
    fn validate_parse_tree(&self, nodes: &[ParseNode]) -> Vec<PositionedError> {
        let mut warnings = Vec::new();

        for node in nodes {
            self.validate_node_recursive(node, &mut warnings);
        }

        warnings
    }

    fn validate_node_recursive(&self, node: &ParseNode, warnings: &mut Vec<PositionedError>) {
        // Check for empty content in nodes that should have content
        if node.text.trim().is_empty() && !self.should_allow_empty(&node.rule) {
            warnings.push(PositionedError::warning(
                format!("Empty content in {} node", node.rule_name()),
                node.span.clone(),
            ));
        }

        // Check for very deep nesting
        if node.depth() > 20 {
            warnings.push(PositionedError::warning(
                format!(
                    "Deep nesting detected in {} (depth: {})",
                    node.rule_name(),
                    node.depth()
                ),
                node.span.clone(),
            ));
        }

        // Check for single-child nodes that might be unnecessary
        if node.children.len() == 1 && self.is_structural_rule(&node.rule) {
            warnings.push(PositionedError::info(
                format!("Single child in structural node: {}", node.rule_name()),
                node.span.clone(),
            ));
        }

        // Recursively validate children
        for child in &node.children {
            self.validate_node_recursive(child, warnings);
        }
    }

    /// Check if a rule should allow empty content
    fn should_allow_empty(&self, rule: &Rule) -> bool {
        matches!(rule, Rule::line_break | Rule::paragraph_line)
    }

    /// Check if a rule is structural (containers, lists, etc.)
    fn is_structural_rule(&self, rule: &Rule) -> bool {
        matches!(
            rule,
            Rule::document
                | Rule::section
                | Rule::block
                | Rule::paragraph
                | Rule::list
                | Rule::list_item
                | Rule::table
                | Rule::table_row
                | Rule::blockquote
        )
    }

    /// Get fallback rules for error recovery
    fn get_fallback_rules(&self, failed_rule: Rule) -> Option<Vec<Rule>> {
        use crate::components::marco_engine::grammar::Rule;

        match failed_rule {
            // Block-level fallbacks: try more general block types
            Rule::heading => Some(vec![Rule::paragraph, Rule::text]),
            Rule::admonition_block => Some(vec![Rule::blockquote, Rule::paragraph]),
            Rule::code_block => Some(vec![Rule::paragraph, Rule::text]),
            Rule::math_block => Some(vec![Rule::paragraph, Rule::text]),
            Rule::table => Some(vec![Rule::paragraph, Rule::text]),
            Rule::list => Some(vec![Rule::paragraph, Rule::text]),
            Rule::blockquote => Some(vec![Rule::paragraph, Rule::text]),

            // Inline fallbacks: try simpler inline elements
            Rule::bold => Some(vec![Rule::emphasis, Rule::text]),
            Rule::italic => Some(vec![Rule::emphasis, Rule::text]),
            Rule::emphasis => Some(vec![Rule::text]),
            Rule::code_inline => Some(vec![Rule::text]),
            Rule::math_inline => Some(vec![Rule::text]),
            Rule::inline_link => Some(vec![Rule::text]),
            Rule::inline_image => Some(vec![Rule::text]),
            Rule::strikethrough => Some(vec![Rule::text]),
            Rule::highlight => Some(vec![Rule::text]),

            // Structure fallbacks: try more general structures
            Rule::section => Some(vec![Rule::block, Rule::paragraph]),
            Rule::block => Some(vec![Rule::paragraph, Rule::text]),
            Rule::paragraph => Some(vec![Rule::text]),
            Rule::document => Some(vec![Rule::section, Rule::block]),

            // For very basic rules, no fallback needed
            Rule::text | Rule::word | Rule::inner_char | Rule::file => None,

            // Default: try text as last resort for most inline content
            _ => Some(vec![Rule::text]),
        }
    }
}

impl Default for EnhancedMarcoParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub enabled: bool,
}

/// Convenience functions for common parsing operations
impl EnhancedMarcoParser {
    /// Parse and analyze rule usage in input
    pub fn analyze_rule_usage(&mut self, input: &str) -> MarcoResult<RuleAnalysis> {
        let result = self.parse_document(input);

        match result.nodes {
            Ok(nodes) => {
                let mut rule_counts = HashMap::new();
                let mut categories = HashMap::new();

                for node in &nodes {
                    self.count_rules_recursive(node, &mut rule_counts, &mut categories);
                }

                Ok(RuleAnalysis {
                    rule_counts,
                    categories,
                    total_nodes: result.stats.node_count,
                    max_depth: result.stats.max_depth,
                    parse_time: result.stats.parse_time,
                })
            }
            Err(e) => Err(e),
        }
    }

    fn count_rules_recursive(
        &self,
        node: &ParseNode,
        rule_counts: &mut HashMap<String, usize>,
        categories: &mut HashMap<String, usize>,
    ) {
        let rule_name = node.rule_name();
        let category = categorize_rule(&rule_name);

        *rule_counts.entry(rule_name).or_insert(0) += 1;
        *categories.entry(category.to_string()).or_insert(0) += 1;

        for child in &node.children {
            self.count_rules_recursive(child, rule_counts, categories);
        }
    }
}

/// Analysis of rule usage in parsed content
#[derive(Debug, Clone)]
pub struct RuleAnalysis {
    pub rule_counts: HashMap<String, usize>,
    pub categories: HashMap<String, usize>,
    pub total_nodes: usize,
    pub max_depth: usize,
    pub parse_time: Duration,
}

impl RuleAnalysis {
    /// Get the most frequently used rules
    pub fn top_rules(&self, limit: usize) -> Vec<(String, usize)> {
        let mut rules: Vec<_> = self
            .rule_counts
            .iter()
            .map(|(rule, count)| (rule.clone(), *count))
            .collect();
        rules.sort_by(|a, b| b.1.cmp(&a.1));
        rules.into_iter().take(limit).collect()
    }

    /// Get the most frequently used categories
    pub fn top_categories(&self, limit: usize) -> Vec<(String, usize)> {
        let mut categories: Vec<_> = self
            .categories
            .iter()
            .map(|(cat, count)| (cat.clone(), *count))
            .collect();
        categories.sort_by(|a, b| b.1.cmp(&a.1));
        categories.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_parser_creation() {
        let parser = EnhancedMarcoParser::new();
        assert!(!parser.config.enable_cache);
        assert!(parser.config.track_positions);
    }

    #[test]
    fn test_parser_with_config() {
        let config = ParserConfig {
            enable_cache: true,
            max_cache_size: 50,
            ..Default::default()
        };
        let parser = EnhancedMarcoParser::with_config(config);
        assert!(parser.config.enable_cache);
        assert_eq!(parser.config.max_cache_size, 50);
    }

    #[test]
    fn test_cache_stats() {
        let parser = EnhancedMarcoParser::new();
        let stats = parser.cache_stats();
        assert_eq!(stats.size, 0);
        assert!(!stats.enabled);
    }

    #[test]
    fn test_rule_validation() {
        let mut parser = EnhancedMarcoParser::new();

        // Test with a valid rule
        let result = parser.validate("text", "hello world");
        assert!(result.is_ok());

        // Test with invalid rule
        let result = parser.validate("nonexistent", "hello");
        assert!(result.is_err());
    }
}
