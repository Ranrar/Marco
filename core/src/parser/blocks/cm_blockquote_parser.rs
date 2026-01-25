//! Blockquote parser - converts grammar output to AST nodes
//!
//! Handles conversion of blockquotes (> prefixed lines) from grammar layer to parser AST,
//! including recursive block parsing and lazy continuation line handling.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Document, Node, NodeKind};

/// Parse a blockquote into an AST node with recursive block parsing.
///
/// # Arguments
/// * `content` - The blockquote content from grammar layer (includes > markers)
/// * `depth` - Current recursion depth for safety
/// * `parse_blocks_fn` - Function to recursively parse nested blocks
///
/// # Returns
/// A Node with NodeKind::Blockquote containing parsed block children
///
/// # Processing
/// The function:
/// 1. Extracts content by removing > markers from each line
/// 2. Handles lazy continuation lines (lines without > markers)
/// 3. Prevents setext heading underlines in lazy continuation (CommonMark spec)
/// 4. Recursively parses the cleaned content as block elements
///
/// # Example
/// ```ignore
/// let content = GrammarSpan::new("> Line 1\n> Line 2");
/// let node = parse_blockquote(content, 0, parse_blocks_internal);
/// assert!(matches!(node.kind, NodeKind::Blockquote));
/// ```
pub fn parse_blockquote<F>(content: GrammarSpan, depth: usize, parse_blocks_fn: F) -> Result<Node, Box<dyn std::error::Error>>
where
    F: FnOnce(&str, usize) -> Result<Document, Box<dyn std::error::Error>>,
{
    let span = to_parser_span(content);

    // Extract the block quote content (remove leading > markers)
    // CRITICAL: Per CommonMark spec, "The setext heading underline cannot be a lazy continuation line"
    // So we need to track which lines had > markers and prevent setext matching on lazy lines
    let content_str = content.fragment();
    let mut cleaned_content = String::with_capacity(content_str.len());

    for line in content_str.split_inclusive('\n') {
        let line_trimmed_start = line.trim_start();
        let has_marker = line_trimmed_start.starts_with('>');

        if has_marker {
            // Line has > marker - remove it and optional space
            let after_marker = line_trimmed_start.strip_prefix('>').unwrap();
            let cleaned = after_marker.strip_prefix(' ').unwrap_or(after_marker);
            cleaned_content.push_str(cleaned);
        } else {
            // Lazy continuation line - no > marker
            // Check if this looks like a setext underline (all === or all ---)
            let line_content = line_trimmed_start.trim_end();
            let line_sans_spaces = line_content.replace([' ', '\t'], "");

            let is_underline = !line_sans_spaces.is_empty()
                && (line_sans_spaces.chars().all(|c| c == '=')
                    || line_sans_spaces.chars().all(|c| c == '-'));

            if is_underline {
                // This lazy continuation looks like setext underline
                // Per CommonMark: "underline cannot be lazy continuation"
                // Escape the first character to prevent setext parsing
                if let Some(first_char) = line_content.chars().next() {
                    if first_char == '=' || first_char == '-' {
                        // Add backslash escape before first underline character
                        cleaned_content.push('\\');
                    }
                }
            }

            // Add the line as-is (or with escape prepended)
            cleaned_content.push_str(line);
        }
    }

    // Recursively parse the block quote content
    let inner_doc = parse_blocks_fn(&cleaned_content, depth + 1)?;

    Ok(Node {
        kind: NodeKind::Blockquote,
        span: Some(span),
        children: inner_doc.children, // Use parsed children
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::NodeKind;

    // Mock parse function for testing
    fn mock_parse_blocks(input: &str, _depth: usize) -> Result<Document, Box<dyn std::error::Error>> {
        let mut doc = Document::new();
        if !input.is_empty() {
            doc.children.push(Node {
                kind: NodeKind::Text(input.to_string()),
                span: None,
                children: Vec::new(),
            });
        }
        Ok(doc)
    }

    #[test]
    fn smoke_test_parse_blockquote_basic() {
        let content = GrammarSpan::new("> Line 1\n> Line 2");
        let node = parse_blockquote(content, 0, mock_parse_blocks).unwrap();

        assert!(matches!(node.kind, NodeKind::Blockquote));
        assert!(!node.children.is_empty());
    }

    #[test]
    fn smoke_test_blockquote_lazy_continuation() {
        let content = GrammarSpan::new("> Line 1\nLine 2 (lazy)");
        let node = parse_blockquote(content, 0, mock_parse_blocks).unwrap();

        assert!(matches!(node.kind, NodeKind::Blockquote));
    }

    #[test]
    fn smoke_test_blockquote_span() {
        let content = GrammarSpan::new("> Test");
        let node = parse_blockquote(content, 0, mock_parse_blocks).unwrap();

        assert!(node.span.is_some());
    }

    #[test]
    fn smoke_test_blockquote_empty() {
        let content = GrammarSpan::new(">");
        let node = parse_blockquote(content, 0, mock_parse_blocks).unwrap();

        assert!(matches!(node.kind, NodeKind::Blockquote));
    }

    #[test]
    fn smoke_test_blockquote_nested_content() {
        let content = GrammarSpan::new("> # Heading\n> Paragraph");
        let node = parse_blockquote(content, 0, mock_parse_blocks).unwrap();

        assert!(matches!(node.kind, NodeKind::Blockquote));
        assert!(!node.children.is_empty());
    }
}
