//! Inline parser modules - convert grammar output to AST nodes
//!
//! This module contains specialized parsers that convert inline grammar elements
//! (from grammar/inlines) into AST nodes with proper position tracking.
//!
//! Phase 5: Inline parser module extraction

// Shared utilities for all inline parsers
pub mod shared;

// Individual inline parser modules
pub mod cm_code_span_parser;
pub mod cm_backslash_escape_parser;
pub mod cm_emphasis_parser;
pub mod cm_strong_parser;
pub mod cm_link_parser;
pub mod cm_image_parser;
pub mod cm_inline_html_parser;
pub mod cm_autolink_parser;
pub mod cm_line_breaks_parser;
pub mod text_parser;

// Re-export parser functions for convenience
pub use cm_code_span_parser::parse_code_span;
pub use cm_backslash_escape_parser::parse_backslash_escape;
pub use cm_emphasis_parser::parse_emphasis;
pub use cm_strong_parser::parse_strong;
pub use cm_link_parser::parse_link;
pub use cm_image_parser::parse_image;
pub use cm_inline_html_parser::parse_inline_html;
pub use cm_autolink_parser::parse_autolink;
pub use cm_line_breaks_parser::{parse_hard_line_break, parse_soft_line_break};
pub use text_parser::{parse_text, parse_special_as_text};

use super::ast::Node;
use shared::GrammarSpan;
use nom::bytes::complete::take;
use anyhow::Result;

/// Parse inline elements within text content
/// Takes a GrammarSpan to preserve position information
/// Returns a vector of inline nodes (Text, Emphasis, Strong, Link, CodeSpan)
pub fn parse_inlines_from_span(span: GrammarSpan) -> Result<Vec<Node>> {
    log::debug!("Parsing inline elements in span at line {}: {:?}", span.location_line(), span.fragment());
    
    let mut nodes = Vec::new();
    let mut remaining = span;
    
    // Safety: prevent infinite loops
    const MAX_ITERATIONS: usize = 1000;
    let mut iteration_count = 0;
    let mut last_offset = 0;
    
    while !remaining.fragment().is_empty() {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            log::error!("Inline parser exceeded MAX_ITERATIONS ({})", MAX_ITERATIONS);
            break;
        }
        
        let start_pos = remaining.location_offset();
        
        // Safety: ensure we're making progress
        if start_pos == last_offset && iteration_count > 1 {
            log::error!("Inline parser not making progress at offset {}, forcing skip", start_pos);
            // Force skip one character
            let skip = remaining.fragment().chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            if let Ok((rest, _)) = take::<_, _, nom::error::Error<_>>(skip)(remaining) {
                remaining = rest;
                last_offset = remaining.location_offset();
                continue;
            } else {
                break;
            }
        }
        last_offset = start_pos;
        
        // Try parsing code span first (highest priority to avoid conflicts)
        if let Ok((rest, node)) = parse_code_span(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing backslash escape (before other inline elements)
        if let Ok((rest, node)) = parse_backslash_escape(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing strong (must come before emphasis to match ** before *)
        if let Ok((rest, node)) = parse_strong(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing emphasis
        if let Ok((rest, node)) = parse_emphasis(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing autolink (must come before link and inline HTML since syntax starts with <)
        if let Ok((rest, node)) = parse_autolink(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing image (must come before link since syntax is similar but starts with !)
        if let Ok((rest, node)) = parse_image(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing link
        if let Ok((rest, node)) = parse_link(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing inline HTML
        if let Ok((rest, node)) = parse_inline_html(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing hard line break (two spaces + newline, or backslash + newline)
        if let Ok((rest, node)) = parse_hard_line_break(remaining) {
            log::debug!("Parsed hard line break at offset {}", remaining.location_offset());
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing soft line break (regular newline)
        if let Ok((rest, node)) = parse_soft_line_break(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // No inline element matched - try parsing plain text
        if let Ok((rest, node)) = parse_text(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Special character that didn't parse as any inline element - consume as text
        if let Ok((rest, node)) = parse_special_as_text(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Safety check: if we reach here, we failed to parse anything
        // This should not happen if all parsers are working correctly
        log::error!("Inline parser unable to make progress at offset {}", start_pos);
        break;
    }
    
    log::debug!("Parsed {} inline nodes", nodes.len());
    Ok(nodes)
}

/// Parse inline elements within text content (backward compatibility wrapper)
/// Creates a new span at position 0:0 - USE parse_inlines_from_span() for position-aware parsing
/// Returns a vector of inline nodes (Text, Emphasis, Strong, Link, CodeSpan)
pub fn parse_inlines(text: &str) -> Result<Vec<Node>> {
    parse_inlines_from_span(GrammarSpan::new(text))
}

