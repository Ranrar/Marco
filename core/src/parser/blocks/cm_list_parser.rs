//! List parser - converts grammar output to AST nodes
//!
//! Handles conversion of lists (both ordered and unordered) from grammar layer to parser AST,
//! including tight/loose determination, item content dedenting, and recursive block parsing.

use crate::parser::ast::{Node, NodeKind, Document};
use crate::grammar::blocks::cm_list::ListMarker;
use super::shared::{to_parser_span, to_parser_span_range, dedent_list_item_content, GrammarSpan};
use anyhow::Result;

/// Represents the parser state needed for list item parsing.
/// This trait allows the list parser to work with the main parser's state.
pub trait ListParserState {
    /// Create a new state for a list item with the given content indent
    fn new_list_item_state(&self, content_indent: usize) -> Self;
}

/// Parse a list into an AST node with recursive item parsing.
///
/// # Arguments
/// * `items` - List of items from grammar layer (marker, content, blanks, indent)
/// * `depth` - Current recursion depth for safety
/// * `parse_blocks_fn` - Function to recursively parse nested blocks
/// * `create_state_fn` - Function to create parser state for list items
///
/// # Returns
/// A Node with NodeKind::List containing parsed ListItem children
///
/// # Processing
/// The function:
/// 1. Determines if list is tight or loose (based on blank lines)
/// 2. Determines if ordered or unordered (from first marker)
/// 3. Creates list node with appropriate metadata
/// 4. For each item:
///    - Dedents content to remove list marker indentation
///    - Creates sub-state for tracking nested structures
///    - Recursively parses item content as block elements
/// 5. Returns complete list node with all item children
///
/// # Example
/// ```ignore
/// let items = vec![
///     (ListMarker::Bullet('-'), content_span, false, false, 2),
///     (ListMarker::Bullet('-'), content_span2, false, false, 2),
/// ];
/// let node = parse_list(items, 0, parse_fn, state_fn).unwrap();
/// assert!(matches!(node.kind, NodeKind::List { .. }));
/// ```
pub fn parse_list<F, S, G>(
    items: Vec<(ListMarker, GrammarSpan, bool, bool, usize)>,
    depth: usize,
    parse_blocks_fn: F,
    mut create_state_fn: G,
) -> Result<Node>
where
    F: Fn(&str, usize, &mut S) -> Result<Document>,
    G: FnMut(usize) -> S,
{
    // Determine if tight or loose
    // A list is tight if no item has blank lines AND no blank lines between items
    let mut is_tight = true;
    for item in &items {
        if item.2 || item.3 {  // has_blank_in_item or has_blank_before_next
            is_tight = false;
            break;
        }
    }
    
    // Determine list type from first marker
    let (ordered, start) = match items[0].0 {
        ListMarker::Bullet(_) => (false, None),
        ListMarker::Ordered { number, .. } => (true, Some(number)),
    };
    
    // Create list node
    let list_start = items[0].1;
    let list_end = items.last().unwrap().1;
    let list_span = to_parser_span_range(list_start, list_end);
    
    let mut list_node = Node {
        kind: NodeKind::List { ordered, start, tight: is_tight },
        span: Some(list_span),
        children: Vec::new(),
    };
    
    // Parse each item's content recursively
    for (_marker, content, _has_blank_in, _has_blank_before, content_indent) in items {
        let item_span = to_parser_span(content);
        
        // Dedent the list item content before parsing
        // This allows block structures (blockquotes, code blocks, nested lists) to be recognized
        let dedented_content = dedent_list_item_content(content.fragment(), content_indent);
        
        // Parse the item's content as block elements
        // Create a sub-state for list item content to track nested structures
        let mut item_state = create_state_fn(content_indent);
        
        let item_content = match parse_blocks_fn(&dedented_content, depth + 1, &mut item_state) {
            Ok(doc) => doc.children,
            Err(e) => {
                log::warn!("Failed to parse list item content: {}", e);
                vec![]
            }
        };
        
        let item_node = Node {
            kind: NodeKind::ListItem,
            span: Some(item_span),
            children: item_content,
        };
        
        list_node.children.push(item_node);
    }
    
    Ok(list_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::NodeKind;
    
    // Mock parser state
    struct MockState;
    
    // Mock parse function for testing
    fn mock_parse_blocks(input: &str, _depth: usize, _state: &mut MockState) -> Result<Document> {
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
    
    fn mock_create_state(_indent: usize) -> MockState {
        MockState
    }
    
    #[test]
    fn smoke_test_parse_list_unordered() {
        let content = GrammarSpan::new("item 1");
        let items = vec![
            (ListMarker::Bullet('-'), content, false, false, 2),
        ];
        
        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();
        
        if let NodeKind::List { ordered, start, tight } = node.kind {
            assert!(!ordered);
            assert_eq!(start, None);
            assert!(tight);
        } else {
            panic!("Expected List node");
        }
    }
    
    #[test]
    fn smoke_test_parse_list_ordered() {
        let content = GrammarSpan::new("item 1");
        let items = vec![
            (ListMarker::Ordered { number: 1, delimiter: '.' }, content, false, false, 3),
        ];
        
        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();
        
        if let NodeKind::List { ordered, start, .. } = node.kind {
            assert!(ordered);
            assert_eq!(start, Some(1));
        } else {
            panic!("Expected List node");
        }
    }
    
    #[test]
    fn smoke_test_list_tight_vs_loose() {
        let content = GrammarSpan::new("item");
        
        // Tight list (no blanks)
        let tight_items = vec![
            (ListMarker::Bullet('-'), content, false, false, 2),
        ];
        let tight_node = parse_list(tight_items, 0, mock_parse_blocks, mock_create_state).unwrap();
        if let NodeKind::List { tight, .. } = tight_node.kind {
            assert!(tight);
        }
        
        // Loose list (has blank)
        let loose_items = vec![
            (ListMarker::Bullet('-'), content, true, false, 2),
        ];
        let loose_node = parse_list(loose_items, 0, mock_parse_blocks, mock_create_state).unwrap();
        if let NodeKind::List { tight, .. } = loose_node.kind {
            assert!(!tight);
        }
    }
    
    #[test]
    fn smoke_test_list_multiple_items() {
        let content1 = GrammarSpan::new("item 1");
        let content2 = GrammarSpan::new("item 2");
        let content3 = GrammarSpan::new("item 3");
        
        let items = vec![
            (ListMarker::Bullet('-'), content1, false, false, 2),
            (ListMarker::Bullet('-'), content2, false, false, 2),
            (ListMarker::Bullet('-'), content3, false, false, 2),
        ];
        
        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();
        
        assert_eq!(node.children.len(), 3);
    }
    
    #[test]
    fn smoke_test_list_span_tracking() {
        let content = GrammarSpan::new("item");
        let items = vec![
            (ListMarker::Bullet('-'), content, false, false, 2),
        ];
        
        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();
        
        assert!(node.span.is_some());
    }
}
