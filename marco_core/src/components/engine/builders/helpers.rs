//! Shared helper utilities for AST building
//!
//! Provides common functions used by both block and inline builders:
//! - Span creation from Pest pairs
//! - Text content extraction
//! - Label normalization
//! - Child node building

use crate::components::engine::{
    ast_node::{Node, Span},  // Use Span from ast_node module
    grammar::Rule,
};
use pest::iterators::Pair;

/// Create span from pest pair with proper line/column tracking
pub fn create_span(pair: &Pair<Rule>) -> Span {
    Span::from_pest(pair)
}

/// Build all children of a Pest pair into AST nodes
///
/// **TODO**: Phase 2.2 - This is a temporary stub.
/// Will need to properly implement with builder once ast_builder.rs is split
pub fn build_children_stub(_pair: Pair<Rule>) -> Result<Vec<Node>, String> {
    // Temporary stub - will be implemented properly in Phase 2.2
    Ok(Vec::new())
}

/// Extract text content from a pair
pub fn extract_text_content(pair: &Pair<Rule>) -> String {
    pair.as_str().to_string()
}

/// Normalize label for reference definitions and links
pub fn normalize_label(label: &str) -> String {
    label
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace() || *c == ' ')
        .collect::<String>()
        .to_lowercase()
}

/// Count leading indentation from span column information
/// Supports both tabs and spaces: 1 tab = 1 indent level, 4 spaces = 1 indent level
pub fn calculate_indent_from_span(span: &pest::Span) -> Option<u8> {
    let (line_num, column) = span.start_pos().line_col();
    if column > 1 {
        // Get the full input to analyze the actual leading whitespace
        let full_input = span.get_input();

        // Find the line containing this span
        let lines: Vec<&str> = full_input.lines().collect();
        if let Some(current_line) = lines.get(line_num - 1) {
            // Count actual leading whitespace characters
            let mut indent_level = 0u8;
            let mut space_count = 0u8;

            for ch in current_line.chars() {
                match ch {
                    '\t' => {
                        // Each tab counts as 1 indent level
                        indent_level += 1;
                        space_count = 0; // Reset space counting after tab
                    }
                    ' ' => {
                        space_count += 1;
                        // Every 4 spaces = 1 indent level
                        if space_count >= 4 {
                            indent_level += 1;
                            space_count = 0; // Reset for next group of spaces
                        }
                    }
                    _ => break, // Stop at first non-whitespace character
                }
            }

            if indent_level > 0 {
                Some(indent_level)
            } else {
                None
            }
        } else {
            // Fallback to old column-based calculation for spaces-only content
            let spaces = (column - 1) as u8;
            let indent_level = spaces / 4;
            if indent_level > 0 {
                Some(indent_level)
            } else {
                None
            }
        }
    } else {
        None
    }
}
