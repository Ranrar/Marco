//! List parser - converts grammar output to AST nodes
//!
//! Handles conversion of lists (both ordered and unordered) from grammar layer to parser AST,
//! including tight/loose determination, item content dedenting, and recursive block parsing.

use super::shared::{dedent_list_item_content, to_parser_span, to_parser_span_range, GrammarSpan};
use crate::grammar::blocks::cm_list::ListMarker;
use crate::parser::ast::{Document, Node, NodeKind};
use nom::Input;

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
) -> Result<Node, Box<dyn std::error::Error>>
where
    F: Fn(&str, usize, &mut S) -> Result<Document, Box<dyn std::error::Error>>,
    G: FnMut(usize) -> S,
{
    // Determine if tight or loose
    // A list is tight if no item has blank lines AND no blank lines between items
    let mut is_tight = true;
    for item in &items {
        if item.2 || item.3 {
            // has_blank_in_item or has_blank_before_next
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
        kind: NodeKind::List {
            ordered,
            start,
            tight: is_tight,
        },
        span: Some(list_span),
        children: Vec::new(),
    };

    // Parse each item's content recursively
    for (_marker, content, _has_blank_in, _has_blank_before, content_indent) in items {
        let item_span = to_parser_span(content);

        // Dedent the list item content before parsing
        // This allows block structures (blockquotes, code blocks, nested lists) to be recognized
        let dedented_content = dedent_list_item_content(content.fragment(), content_indent);

        // GFM task list item marker detection.
        // If present, strip it from the content before parsing blocks.
        // We also compute an accurate span for the marker itself so the editor
        // can color `[ ]` / `[x]` distinctly.
        let (task, content_to_parse) =
            match detect_task_checkbox_in_list_item(content, content_indent) {
                Some((checked, marker_span, rest)) => (Some((checked, marker_span)), rest),
                None => (None, dedented_content),
            };

        // Parse the item's content as block elements
        // Create a sub-state for list item content to track nested structures
        let mut item_state = create_state_fn(content_indent);

        let item_content = match parse_blocks_fn(&content_to_parse, depth + 1, &mut item_state) {
            Ok(doc) => doc.children,
            Err(e) => {
                log::warn!("Failed to parse list item content: {}", e);
                vec![]
            }
        };

        let mut item_children = Vec::new();

        if let Some((checked, marker_span)) = task {
            item_children.push(Node {
                kind: NodeKind::TaskCheckbox { checked },
                span: Some(marker_span),
                children: Vec::new(),
            });
        }

        item_children.extend(item_content);

        let item_node = Node {
            kind: NodeKind::ListItem,
            span: Some(item_span),
            children: item_children,
        };

        list_node.children.push(item_node);
    }

    Ok(list_node)
}

/// Detect a GFM task checkbox marker in a list item's content and return:
/// - `checked`
/// - a precise `Span` for just the marker (`[ ]` / `[x]`)
/// - the dedented content with the marker removed (what we should parse as blocks)
fn detect_task_checkbox_in_list_item(
    content: GrammarSpan,
    content_indent: usize,
) -> Option<(bool, crate::parser::position::Span, String)> {
    // Fast path: if there's no marker in the dedented string, bail early.
    // This keeps behaviour identical for non-task items.
    let dedented = dedent_list_item_content(content.fragment(), content_indent);
    let (checked, dedented_rest, _dedented_consumed) =
        parse_task_checkbox_prefix_with_consumed(&dedented)?;

    // Compute marker span in the original input using the grammar span.
    // We try to mirror the dedent behaviour for the *first line* only to find
    // where the marker begins.
    let raw = content.fragment();
    let raw_prefix = raw_dedent_prefix_len_first_line(raw, content_indent);
    if raw_prefix > raw.len() {
        return None;
    }

    // Slice to the first line after the dedent prefix.
    let after_prefix = content.take_from(raw_prefix);
    let bytes = after_prefix.fragment().as_bytes();

    // Allow up to 3 spaces before the marker.
    let mut i = 0usize;
    for _ in 0..3 {
        if bytes.get(i) == Some(&b' ') {
            i += 1;
        } else {
            break;
        }
    }

    let rest = &after_prefix.fragment()[i..];
    let (checked_raw, after_marker_raw): (bool, &str) =
        if let Some(after) = rest.strip_prefix("[ ]") {
            (false, after)
        } else if let Some(after) = rest
            .strip_prefix("[x]")
            .or_else(|| rest.strip_prefix("[X]"))
        {
            (true, after)
        } else {
            return None;
        };

    // Must be followed by at least one whitespace character.
    let mut chars = after_marker_raw.chars();
    match chars.next() {
        Some(' ') | Some('\t') => {
            // ok
        }
        _ => return None,
    }

    // Keep the dedented parser behaviour as the source of truth for content.
    if checked_raw != checked {
        log::debug!(
            "Task checkbox mismatch between raw and dedented detection (raw_checked={}, dedented_checked={})",
            checked_raw,
            checked
        );
    }

    // Build the marker span using exclusive end semantics.
    // Note: `blocks::shared::to_parser_span_range` is inclusive; we want the
    // canonical exclusive version from `parser::shared`.
    let marker_start = after_prefix.take_from(i);
    let (after_marker, _marker_taken) = marker_start.take_split(3);
    let marker_span = crate::parser::shared::to_parser_span_range(marker_start, after_marker);

    Some((checked, marker_span, dedented_rest.to_string()))
}

/// Parse a task checkbox prefix and return (checked, rest, consumed_bytes).
///
/// Consumed bytes include:
/// - up to 3 leading spaces
/// - the 3-byte marker (`[ ]` / `[x]` / `[X]`)
/// - exactly one whitespace character after the marker
fn parse_task_checkbox_prefix_with_consumed(input: &str) -> Option<(bool, &str, usize)> {
    let mut i = 0usize;
    for _ in 0..3 {
        if input.as_bytes().get(i) == Some(&b' ') {
            i += 1;
        } else {
            break;
        }
    }

    let rest = &input[i..];

    let (checked, after_marker) = if let Some(after) = rest.strip_prefix("[ ]") {
        (false, after)
    } else if let Some(after) = rest
        .strip_prefix("[x]")
        .or_else(|| rest.strip_prefix("[X]"))
    {
        (true, after)
    } else {
        return None;
    };

    let mut chars = after_marker.chars();
    match chars.next() {
        Some(' ') | Some('\t') => {
            let remaining = chars.as_str();
            Some((checked, remaining, i + 3 + 1))
        }
        _ => None,
    }
}

/// Compute how many raw bytes are removed from the *first line* when
/// `dedent_list_item_content` strips `content_indent` columns.
///
/// This is needed to map marker highlighting back onto original `GrammarSpan`
/// offsets without relying on the dedented string (which expands tabs).
fn raw_dedent_prefix_len_first_line(input: &str, content_indent: usize) -> usize {
    let first_line = input.split_once('\n').map(|(l, _)| l).unwrap_or(input);

    let mut stripped_cols = 0usize;
    let mut column = content_indent;
    let mut bytes = 0usize;

    for (byte_idx, ch) in first_line.char_indices() {
        if stripped_cols >= content_indent {
            break;
        }

        match ch {
            ' ' => {
                stripped_cols += 1;
                column += 1;
                bytes = byte_idx + 1;
            }
            '\t' => {
                // Tab advances to next multiple of 4, starting from `content_indent`.
                let spaces_to_add = 4 - (column % 4);
                stripped_cols = stripped_cols.saturating_add(spaces_to_add);
                column += spaces_to_add;
                bytes = byte_idx + 1;
            }
            _ => break,
        }
    }

    // Clamp to the input line length.
    bytes.min(first_line.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::NodeKind;

    // Mock parser state
    struct MockState;

    // Mock parse function for testing
    fn mock_parse_blocks(input: &str, _depth: usize, _state: &mut MockState) -> Result<Document, Box<dyn std::error::Error>> {
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
    fn smoke_test_parse_list_task_item_checked_strips_marker() {
        let content = GrammarSpan::new("[x] item");
        let items = vec![(ListMarker::Bullet('-'), content, false, false, 2)];

        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();
        assert_eq!(node.children.len(), 1);

        let item = &node.children[0];
        assert!(matches!(item.kind, NodeKind::ListItem));
        assert!(matches!(
            item.children.first().map(|n| &n.kind),
            Some(NodeKind::TaskCheckbox { checked: true })
        ));
        assert!(
            matches!(item.children.get(1).map(|n| &n.kind), Some(NodeKind::Text(t)) if t == "item")
        );
    }

    #[test]
    fn smoke_test_parse_list_task_item_unchecked_strips_marker() {
        let content = GrammarSpan::new("[ ] item");
        let items = vec![(ListMarker::Bullet('-'), content, false, false, 2)];

        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();
        assert_eq!(node.children.len(), 1);

        let item = &node.children[0];
        assert!(matches!(item.kind, NodeKind::ListItem));
        assert!(matches!(
            item.children.first().map(|n| &n.kind),
            Some(NodeKind::TaskCheckbox { checked: false })
        ));
        assert!(
            matches!(item.children.get(1).map(|n| &n.kind), Some(NodeKind::Text(t)) if t == "item")
        );
    }

    #[test]
    fn smoke_test_parse_list_unordered() {
        let content = GrammarSpan::new("item 1");
        let items = vec![(ListMarker::Bullet('-'), content, false, false, 2)];

        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();

        if let NodeKind::List {
            ordered,
            start,
            tight,
        } = node.kind
        {
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
        let items = vec![(
            ListMarker::Ordered {
                number: 1,
                delimiter: '.',
            },
            content,
            false,
            false,
            3,
        )];

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
        let tight_items = vec![(ListMarker::Bullet('-'), content, false, false, 2)];
        let tight_node = parse_list(tight_items, 0, mock_parse_blocks, mock_create_state).unwrap();
        if let NodeKind::List { tight, .. } = tight_node.kind {
            assert!(tight);
        }

        // Loose list (has blank)
        let loose_items = vec![(ListMarker::Bullet('-'), content, true, false, 2)];
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
        let items = vec![(ListMarker::Bullet('-'), content, false, false, 2)];

        let node = parse_list(items, 0, mock_parse_blocks, mock_create_state).unwrap();

        assert!(node.span.is_some());
    }
}
