//! Indent-aware parsing utilities for nested Markdown structures
//!
//! This module provides a reusable framework for parsing indentation-based nested structures
//! in Markdown, such as nested lists, blockquotes, or code blocks. It follows the hybrid
//! Pest + Rust pattern established in the lookbehind module.
//!
//! ## Architecture
//!
//! **Phase 1 (Pest)**: Grammar captures lines with optional leading spaces/tabs
//! **Phase 2 (Rust)**: Post-processing builds nested tree based on indentation levels
//!
//! ## CommonMark Indentation Rules
//!
//! Per CommonMark spec, indentation is **context-dependent**:
//! - List items: "Let the width and indentation of the list marker determine the indentation necessary"
//! - Block quotes: Each level requires `>` marker, not fixed spaces
//! - Code blocks: 4 spaces OR 1 tab from paragraph indent
//!
//! This module provides generic indentation tracking, allowing calling code to implement
//! specific CommonMark rules on top.
//!
//! ## Usage Example
//!
//! ```rust
//! use marco_core::components::engine::indent_aware::{IndentNode, build_indent_tree};
//!
//! // From Pest parsing, you get (indent, content) pairs
//! let items = vec![
//!     (0, "Item 1".to_string()),
//!     (2, "Nested 1".to_string()),
//!     (4, "Nested 2".to_string()),
//!     (0, "Item 2".to_string()),
//!     (2, "Nested A".to_string()),
//! ];
//!
//! // Build nested tree
//! let tree = build_indent_tree(items);
//!
//! // Tree structure:
//! // - Item 1 (indent: 0)
//! //   - Nested 1 (indent: 2)
//! //     - Nested 2 (indent: 4)
//! // - Item 2 (indent: 0)
//! //   - Nested A (indent: 2)
//! assert_eq!(tree.len(), 2);
//! assert_eq!(tree[0].children.len(), 1);
//! assert_eq!(tree[0].children[0].children.len(), 1);
//! ```
//!
//! ## Pest Grammar Integration
//!
//! In your `.pest` file:
//!
//! ```pest
//! SPACES = _{ " " | "\t" }
//! NEWLINE = _{ "\n" }
//! CONTENT = { (!NEWLINE ~ ANY)* }
//! INDENTED_LINE = { SPACES* ~ "-" ~ " " ~ CONTENT ~ NEWLINE? }
//! ```
//!
//! In your Rust parser:
//!
//! ```rust,ignore
//! for pair in pairs {
//!     let line_str = pair.as_str();
//!     let indent = line_str.chars().take_while(|c| *c == ' ' || *c == '\t').count();
//!     let content = line_str.trim().to_string();
//!     items.push((indent, content));
//! }
//! let tree = build_indent_tree(items);
//! ```

/// A node in an indent-based tree structure
///
/// Represents a single line of Markdown content and its nested children,
/// organized by indentation level.
///
/// # Fields
///
/// - `content`: The text content of this line (trimmed of leading/trailing whitespace)
/// - `children`: Nested items with greater indentation than this node
/// - `indent`: The indentation level (number of leading spaces/tabs)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentNode {
    pub content: String,
    pub children: Vec<IndentNode>,
    pub indent: usize,
}

impl IndentNode {
    /// Create a new indent node
    pub fn new(content: String, indent: usize) -> Self {
        Self {
            content,
            children: Vec::new(),
            indent,
        }
    }

    /// Add a child node to this node
    pub fn add_child(&mut self, child: IndentNode) {
        self.children.push(child);
    }

    /// Get the total number of descendants (including self)
    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
    }

    /// Check if this node is a leaf (has no children)
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Get the maximum depth of the tree rooted at this node
    pub fn max_depth(&self) -> usize {
        if self.is_leaf() {
            1
        } else {
            1 + self.children.iter().map(|c| c.max_depth()).max().unwrap_or(0)
        }
    }
}

/// Build a nested tree from a flat list of (indent, content) pairs
///
/// This function implements the "indent stack" algorithm used by CommonMark parsers.
/// It processes lines sequentially, maintaining a stack of parent nodes to determine
/// where each new line should be inserted based on its indentation.
///
/// ## Algorithm
///
/// 1. Maintain a stack of (indent_level, node_index) pairs representing active parent nodes
/// 2. For each new line:
///    - Pop stack entries with indent >= current line's indent
///    - If stack is empty, add to root level
///    - Otherwise, add as child of top stack entry
///    - Push current node onto stack
///
/// ## Indentation Rules
///
/// - A line is a **child** of the nearest previous line with **lesser** indentation
/// - Lines with equal indentation are **siblings**
/// - Empty lines or lines with only whitespace are skipped
///
/// # Arguments
///
/// * `items` - Vector of (indentation_level, content) pairs
///
/// # Returns
///
/// A vector of root-level `IndentNode`s representing the forest of indent trees
///
/// # Examples
///
/// ```
/// use marco_core::components::engine::indent_aware::{build_indent_tree, IndentNode};
///
/// let items = vec![
///     (0, "Root 1".to_string()),
///     (2, "Child A".to_string()),
///     (0, "Root 2".to_string()),
/// ];
///
/// let tree = build_indent_tree(items);
/// assert_eq!(tree.len(), 2);
/// assert_eq!(tree[0].content, "Root 1");
/// assert_eq!(tree[0].children.len(), 1);
/// assert_eq!(tree[0].children[0].content, "Child A");
/// ```
pub fn build_indent_tree(items: Vec<(usize, String)>) -> Vec<IndentNode> {
    // Stack of (indent_level, vec_index, tree_path) for tracking parent nodes
    // tree_path is a Vec<usize> representing path from root to this node
    let mut stack: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut result: Vec<IndentNode> = Vec::new();

    for (indent, content) in items {
        // Skip empty content
        if content.is_empty() {
            continue;
        }

        let node = IndentNode::new(content, indent);

        // Pop stack entries with indent >= current indent
        while let Some(&(parent_indent, _)) = stack.last() {
            if indent <= parent_indent {
                stack.pop();
            } else {
                break;
            }
        }

        // Determine where to insert this node
        if let Some((_, path)) = stack.last() {
            // Insert as child of the last stack entry
            let mut current = &mut result;
            for &idx in path.iter() {
                current = &mut current[idx].children;
            }
            current.push(node);

            // Build new path for this node
            let mut new_path = path.clone();
            new_path.push(current.len() - 1);
            stack.push((indent, new_path));
        } else {
            // Insert at root level
            result.push(node);
            stack.push((indent, vec![result.len() - 1]));
        }
    }

    result
}

/// Extract indentation level and content from a line
///
/// Counts leading spaces/tabs and returns (indent_count, trimmed_content).
/// Tabs are counted as single characters (calling code can adjust if needed).
///
/// # Arguments
///
/// * `line` - A line of text with optional leading whitespace
///
/// # Returns
///
/// A tuple of (indentation_count, trimmed_content)
///
/// # Examples
///
/// ```
/// use marco_core::components::engine::indent_aware::extract_indent;
///
/// assert_eq!(extract_indent("  hello"), (2, "hello".to_string()));
/// assert_eq!(extract_indent("\t\tworld"), (2, "world".to_string()));
/// assert_eq!(extract_indent("no indent"), (0, "no indent".to_string()));
/// ```
pub fn extract_indent(line: &str) -> (usize, String) {
    let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
    let content = line.trim().to_string();
    (indent, content)
}

/// Convert a list of lines into (indent, content) pairs
///
/// Convenience function that maps `extract_indent` over a collection of lines.
///
/// # Arguments
///
/// * `lines` - Iterator of string slices representing lines
///
/// # Returns
///
/// Vector of (indentation_level, content) pairs
///
/// # Examples
///
/// ```
/// use marco_core::components::engine::indent_aware::lines_to_indent_pairs;
///
/// let lines = vec!["Item 1", "  Nested", "Item 2"];
/// let pairs = lines_to_indent_pairs(lines.into_iter());
///
/// assert_eq!(pairs.len(), 3);
/// assert_eq!(pairs[0], (0, "Item 1".to_string()));
/// assert_eq!(pairs[1], (2, "Nested".to_string()));
/// ```
pub fn lines_to_indent_pairs<'a, I>(lines: I) -> Vec<(usize, String)>
where
    I: Iterator<Item = &'a str>,
{
    lines.map(extract_indent).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Smoke Tests - Core Functionality
    // ============================================================================

    #[test]
    fn smoke_test_indent_node_creation() {
        let node = IndentNode::new("Test".to_string(), 2);
        assert_eq!(node.content, "Test");
        assert_eq!(node.indent, 2);
        assert_eq!(node.children.len(), 0);
        assert!(node.is_leaf());
    }

    #[test]
    fn smoke_test_build_flat_tree() {
        let items = vec![
            (0, "Item 1".to_string()),
            (0, "Item 2".to_string()),
            (0, "Item 3".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 3);
        assert_eq!(tree[0].content, "Item 1");
        assert_eq!(tree[1].content, "Item 2");
        assert_eq!(tree[2].content, "Item 3");
    }

    #[test]
    fn smoke_test_build_nested_tree() {
        let items = vec![
            (0, "Root".to_string()),
            (2, "Child 1".to_string()),
            (2, "Child 2".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].content, "Root");
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].content, "Child 1");
        assert_eq!(tree[0].children[1].content, "Child 2");
    }

    #[test]
    fn smoke_test_build_deep_nested_tree() {
        let items = vec![
            (0, "Level 0".to_string()),
            (2, "Level 1".to_string()),
            (4, "Level 2".to_string()),
            (6, "Level 3".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].children[0].content, "Level 3");
    }

    #[test]
    fn smoke_test_extract_indent() {
        assert_eq!(extract_indent("  hello"), (2, "hello".to_string()));
        assert_eq!(extract_indent("\t\tworld"), (2, "world".to_string()));
        assert_eq!(extract_indent("no spaces"), (0, "no spaces".to_string()));
        assert_eq!(extract_indent("    four spaces"), (4, "four spaces".to_string()));
    }

    // ============================================================================
    // Indentation Logic Tests
    // ============================================================================

    #[test]
    fn test_indent_stack_popping() {
        // Simulates: Root -> Child -> Root again (should pop child from stack)
        let items = vec![
            (0, "Root 1".to_string()),
            (2, "Child".to_string()),
            (0, "Root 2".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].content, "Root 1");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[1].content, "Root 2");
        assert_eq!(tree[1].children.len(), 0);
    }

    #[test]
    fn test_sibling_nodes_same_indent() {
        let items = vec![
            (0, "Parent".to_string()),
            (2, "Sibling 1".to_string()),
            (2, "Sibling 2".to_string()),
            (2, "Sibling 3".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 3);
        assert_eq!(tree[0].children[0].content, "Sibling 1");
        assert_eq!(tree[0].children[1].content, "Sibling 2");
        assert_eq!(tree[0].children[2].content, "Sibling 3");
    }

    #[test]
    fn test_complex_nested_structure() {
        // Example from user's specification:
        // - Item 1
        //   - Nested 1
        //     - Nested 2
        // - Item 2
        //   - Nested A
        let items = vec![
            (0, "Item 1".to_string()),
            (2, "Nested 1".to_string()),
            (4, "Nested 2".to_string()),
            (0, "Item 2".to_string()),
            (2, "Nested A".to_string()),
        ];

        let tree = build_indent_tree(items);

        // Should have 2 root items
        assert_eq!(tree.len(), 2);

        // First root: Item 1
        assert_eq!(tree[0].content, "Item 1");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].content, "Nested 1");
        assert_eq!(tree[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].content, "Nested 2");

        // Second root: Item 2
        assert_eq!(tree[1].content, "Item 2");
        assert_eq!(tree[1].children.len(), 1);
        assert_eq!(tree[1].children[0].content, "Nested A");
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_empty_items() {
        let items: Vec<(usize, String)> = vec![];
        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_empty_content_skipped() {
        let items = vec![
            (0, "Item 1".to_string()),
            (2, "".to_string()), // Empty content
            (0, "Item 2".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].content, "Item 1");
        assert_eq!(tree[1].content, "Item 2");
    }

    #[test]
    fn test_irregular_indentation() {
        // Indentation levels: 0, 3, 1, 5
        // Algorithm behavior:
        // - "Zero" (0): added at root
        // - "Three" (3): 3 > 0, so child of "Zero"
        // - "One" (1): 1 <= 3 (pop "Three"), but 1 > 0 (keep "Zero"), so child of "Zero"
        // - "Five" (5): 5 > 1, so child of "One"
        //
        // Result structure:
        // - Zero (0)
        //   - Three (3)
        //   - One (1)
        //     - Five (5)
        let items = vec![
            (0, "Zero".to_string()),
            (3, "Three".to_string()),
            (1, "One".to_string()),
            (5, "Five".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree.len(), 1); // Only "Zero" at root

        assert_eq!(tree[0].content, "Zero");
        assert_eq!(tree[0].children.len(), 2); // "Three" and "One"
        assert_eq!(tree[0].children[0].content, "Three");
        assert_eq!(tree[0].children[1].content, "One");
        
        // "Five" is child of "One"
        assert_eq!(tree[0].children[1].children.len(), 1);
        assert_eq!(tree[0].children[1].children[0].content, "Five");
    }

    // ============================================================================
    // Node Utility Tests
    // ============================================================================

    #[test]
    fn test_node_count_nodes() {
        let items = vec![
            (0, "Root".to_string()),
            (2, "Child 1".to_string()),
            (2, "Child 2".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree[0].count_nodes(), 3); // Root + 2 children
    }

    #[test]
    fn test_node_max_depth() {
        let items = vec![
            (0, "Level 0".to_string()),
            (2, "Level 1".to_string()),
            (4, "Level 2".to_string()),
        ];

        let tree = build_indent_tree(items);
        assert_eq!(tree[0].max_depth(), 3);
    }

    #[test]
    fn test_node_is_leaf() {
        let mut node = IndentNode::new("Parent".to_string(), 0);
        assert!(node.is_leaf());

        node.add_child(IndentNode::new("Child".to_string(), 2));
        assert!(!node.is_leaf());
    }

    // ============================================================================
    // lines_to_indent_pairs Tests
    // ============================================================================

    #[test]
    fn test_lines_to_indent_pairs() {
        let lines = vec!["Item 1", "  Nested", "    Deep", "Item 2"];
        let pairs = lines_to_indent_pairs(lines.into_iter());

        assert_eq!(pairs.len(), 4);
        assert_eq!(pairs[0], (0, "Item 1".to_string()));
        assert_eq!(pairs[1], (2, "Nested".to_string()));
        assert_eq!(pairs[2], (4, "Deep".to_string()));
        assert_eq!(pairs[3], (0, "Item 2".to_string()));
    }

    #[test]
    fn test_lines_with_tabs() {
        let lines = vec!["\tTabbed", "\t\tDouble tab"];
        let pairs = lines_to_indent_pairs(lines.into_iter());

        assert_eq!(pairs[0], (1, "Tabbed".to_string()));
        assert_eq!(pairs[1], (2, "Double tab".to_string()));
    }

    // ============================================================================
    // Integration Example (Full Workflow)
    // ============================================================================

    #[test]
    fn test_full_workflow_markdown_list() {
        // Simulates parsing a Markdown list from text
        let markdown = "- Item 1\n  - Nested 1\n    - Nested 2\n- Item 2\n  - Nested A";

        let lines: Vec<&str> = markdown.lines().collect();
        let pairs = lines_to_indent_pairs(lines.into_iter());

        // Remove "- " prefix for actual content (parser would do this)
        let cleaned: Vec<(usize, String)> = pairs
            .into_iter()
            .map(|(indent, content)| {
                let clean = content.strip_prefix("- ").unwrap_or(&content).to_string();
                (indent, clean)
            })
            .collect();

        let tree = build_indent_tree(cleaned);

        // Verify structure matches expected nesting
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].content, "Item 1");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].content, "Nested 1");
        assert_eq!(tree[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].content, "Nested 2");

        assert_eq!(tree[1].content, "Item 2");
        assert_eq!(tree[1].children.len(), 1);
        assert_eq!(tree[1].children[0].content, "Nested A");
    }
}
