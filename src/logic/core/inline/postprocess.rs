//! postprocess.rs - Resolves delimiter runs into AST structure

/// Takes raw inline node sequences and delimiter stack, constructs the final AST structure.
/// Collapses adjacent text nodes, converts delimiters into AST nodes, and cleans up structure.


use super::types::InlineNode;

/// Normalize a sequence of raw inline nodes and delimiter stack into a final AST.
/// Steps:
/// 1. Collapse adjacent text nodes
/// 2. Resolve delimiter runs into Emphasis/Strong nodes
/// 3. Convert tokens to AST nodes
/// 4. Remove unmatched delimiters
/// 5. Merge/clean up tree structure
///
/// Returns a normalized Vec<InlineNode> suitable for rendering or event emission.
pub fn normalize_inlines(raw: Vec<InlineNode>) -> Vec<InlineNode> {
    // Step 1: Collapse adjacent text nodes
    let collapsed = collapse_text_nodes(raw);
    // Step 2: Build delimiter stack from collapsed nodes
    let (preprocessed, delim_stack) = preprocess_delimiters(collapsed);
    // Step 3: Resolve delimiter runs (emphasis/strong)
    let resolved = resolve_delimiters_with_stack(preprocessed, delim_stack);
    // Step 4: Remove unmatched delimiters and clean up
    let cleaned = remove_unmatched_delimiters(resolved);
    // Step 5: Merge/clean up tree structure
    let final_ast = merge_tree(cleaned);
    final_ast
}

/// Pre-process raw text nodes to split delimiter runs (*, **, _, __) into separate nodes.
/// Delimiter run metadata for stack-based resolution
#[derive(Debug, Clone)]
struct DelimRun {
    idx: usize, // index in nodes
    delim: char, // '*' or '_'
    count: usize, // number of delimiters
    pos: super::types::SourcePos,
    can_open: bool,
    can_close: bool,
}

/// Pre-process raw text nodes to split delimiter runs (*, **, _, __) into separate nodes and annotate runs for stack resolution.
fn preprocess_delimiters(nodes: Vec<InlineNode>) -> (Vec<InlineNode>, Vec<DelimRun>) {
    use super::types::SourcePos;
    let mut result = Vec::new();
    let mut delim_stack = Vec::new();
    for node in nodes {
        match node {
            InlineNode::Text { text, pos } => {
                let mut i = 0;
                let chars: Vec<char> = text.chars().collect();
                while i < chars.len() {
                    // Detect delimiter run
                    if chars[i] == '*' || chars[i] == '_' {
                        let delim = chars[i];
                        let mut count = 1;
                        while i + count < chars.len() && chars[i + count] == delim {
                            count += 1;
                        }
                        // Determine left/right-flanking for opener/closer
                        let prev = if i == 0 { None } else { Some(chars[i-1]) };
                        let next = if i + count < chars.len() { Some(chars[i+count]) } else { None };
                        let is_whitespace = |c: Option<char>| c.map_or(true, |ch| ch.is_whitespace());
                        let is_punct = |c: Option<char>| c.map_or(false, |ch| ch.is_ascii_punctuation());
                        let left_flanking = !is_whitespace(next) && (!is_punct(next) || is_whitespace(prev) || is_punct(prev));
                        let right_flanking = !is_whitespace(prev) && (!is_punct(prev) || is_whitespace(next) || is_punct(next));
                        let can_open = left_flanking && (delim == '*' || (delim == '_' && (!right_flanking || is_punct(prev))));
                        let can_close = right_flanking && (delim == '*' || (delim == '_' && (!left_flanking || is_punct(next))));
                        let idx = result.len();
                        result.push(InlineNode::Text { text: delim.to_string().repeat(count), pos });
                        delim_stack.push(DelimRun {
                            idx,
                            delim,
                            count,
                            pos,
                            can_open,
                            can_close,
                        });
                        i += count;
                    } else {
                        // Collect text until next delimiter
                        let start = i;
                        while i < chars.len() && chars[i] != '*' && chars[i] != '_' {
                            i += 1;
                        }
                        let segment: String = chars[start..i].iter().collect();
                        result.push(InlineNode::Text { text: segment, pos });
                    }
                }
            }
            other => result.push(other),
        }
    }
    (result, delim_stack)
}

/// Collapse adjacent InlineNode::Text nodes into a single node.
fn collapse_text_nodes(nodes: Vec<InlineNode>) -> Vec<InlineNode> {
    let mut result = Vec::with_capacity(nodes.len());
    let mut buffer = String::new();
    let mut buffer_pos: Option<super::types::SourcePos> = None;
    for node in nodes {
        match node {
            InlineNode::Text { text, pos } => {
                if buffer.is_empty() {
                    buffer_pos = Some(pos);
                }
                buffer.push_str(&text);
            }
            other => {
                if !buffer.is_empty() {
                    result.push(InlineNode::Text { text: std::mem::take(&mut buffer), pos: buffer_pos.unwrap() });
                    buffer_pos = None;
                }
                result.push(other);
            }
        }
    }
    if !buffer.is_empty() {
        result.push(InlineNode::Text { text: buffer, pos: buffer_pos.unwrap() });
    }
    result
}

/// Resolve delimiter runs into Emphasis/Strong nodes per CommonMark spec.
/// Resolve delimiter runs into Emphasis/Strong nodes using the delimiter stack per CommonMark spec.
fn resolve_delimiters_with_stack(mut nodes: Vec<InlineNode>, mut delim_stack: Vec<DelimRun>) -> Vec<InlineNode> {
    // Track which delimiters have been matched
    let mut used = vec![false; nodes.len()];
    let mut result = Vec::new();
    let mut i = 0;
    while i < delim_stack.len() {
        if !delim_stack[i].can_open || delim_stack[i].count == 0 || used[delim_stack[i].idx] {
            i += 1;
            continue;
        }
        // Find matching closer
        let mut j = i + 1;
        while j < delim_stack.len() {
            if !delim_stack[j].can_close || delim_stack[j].count == 0 || used[delim_stack[j].idx] {
                j += 1;
                continue;
            }
            // Must match type and not be the same run
            if delim_stack[i].delim == delim_stack[j].delim {
                // Multiple of 3 rule for ambiguous delimiters
                let sum = delim_stack[i].count + delim_stack[j].count;
                if delim_stack[i].can_open && delim_stack[j].can_close && sum % 3 == 0 && (delim_stack[i].count % 3 != 0 || delim_stack[j].count % 3 != 0) {
                    j += 1;
                    continue;
                }
                // Determine how many delimiters to use (1 for emphasis, 2 for strong)
                let use_count = if delim_stack[i].count >= 2 && delim_stack[j].count >= 2 { 2 } else { 1 };
                // Only wrap if indices are valid and opener is before closer
                let start = delim_stack[i].idx + 1;
                let end = delim_stack[j].idx;
                let children = if start < end && end <= nodes.len() {
                    nodes[start..end].to_vec()
                } else {
                    Vec::new()
                };
                let node = if use_count == 2 {
                    InlineNode::Strong { children, pos: delim_stack[i].pos }
                } else {
                    InlineNode::Emphasis { children, pos: delim_stack[i].pos }
                };
                // Mark delimiters as used
                used[delim_stack[i].idx] = true;
                used[delim_stack[j].idx] = true;
                // Replace opener with node, closer with empty text
                nodes[delim_stack[i].idx] = node;
                nodes[delim_stack[j].idx] = InlineNode::Text { text: String::new(), pos: delim_stack[j].pos };
                // Remove used delimiters from stack
                delim_stack[i].count -= use_count;
                delim_stack[j].count -= use_count;
                break;
            }
            j += 1;
        }
        i += 1;
    }
    // Remove empty text nodes and collect result
    for node in nodes {
        match &node {
            InlineNode::Text { text, .. } if text.is_empty() => {}
            _ => result.push(node),
        }
    }
    result
}

/// Remove unmatched delimiters and empty nodes.
fn remove_unmatched_delimiters(nodes: Vec<InlineNode>) -> Vec<InlineNode> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            InlineNode::Emphasis { children, pos } => {
                let cleaned = remove_unmatched_delimiters(children);
                if !cleaned.is_empty() {
                    result.push(InlineNode::Emphasis { children: cleaned, pos });
                }
            }
            InlineNode::Strong { children, pos } => {
                let cleaned = remove_unmatched_delimiters(children);
                if !cleaned.is_empty() {
                    result.push(InlineNode::Strong { children: cleaned, pos });
                }
            }
            InlineNode::Text { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Text { text, pos });
                }
            }
            InlineNode::Link { href, title, children, pos } => {
                let cleaned = remove_unmatched_delimiters(children);
                result.push(InlineNode::Link { href, title, children: cleaned, pos });
            }
            InlineNode::Image { src, alt, title, pos } => {
                if !alt.is_empty() {
                    result.push(InlineNode::Image { src, alt, title, pos });
                }
            }
            InlineNode::Math { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Math { text, pos });
                }
            }
            InlineNode::Code { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Code { text, pos });
                }
            }
            InlineNode::Html { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Html { text, pos });
                }
            }
            InlineNode::SoftBreak { pos } => result.push(InlineNode::SoftBreak { pos }),
            InlineNode::LineBreak { pos } => result.push(InlineNode::LineBreak { pos }),
        }
    }
    result
}

/// Merge/clean up tree structure, ensure correct nesting.
fn merge_tree(nodes: Vec<InlineNode>) -> Vec<InlineNode> {
    use super::types::InlineNode;
    let mut merged: Vec<InlineNode> = Vec::new();
    let mut last_text: Option<(String, super::types::SourcePos)> = None;
    for node in nodes {
        match node {
            InlineNode::Text { text: ref s, pos } => {
                if let Some((ref mut acc, ref acc_pos)) = last_text {
                    (*acc).push_str(s);
                } else {
                    last_text = Some((s.clone(), pos));
                }
            }
            InlineNode::Emphasis { children, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::Emphasis { children: merge_tree(children), pos });
            }
            InlineNode::Strong { children, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::Strong { children: merge_tree(children), pos });
            }
            InlineNode::Link { href, title, children, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::Link {
                    href,
                    title,
                    children: merge_tree(children),
                    pos,
                });
            }
            InlineNode::Image { src, alt, title, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::Image { src, alt, title, pos });
            }
            InlineNode::Math { text, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::Math { text, pos });
            }
            InlineNode::Code { text, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::Code { text, pos });
            }
            InlineNode::Html { text, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::Html { text, pos });
            }
            InlineNode::SoftBreak { pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::SoftBreak { pos });
            }
            InlineNode::LineBreak { pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: acc, pos: acc_pos });
                }
                merged.push(InlineNode::LineBreak { pos });
            }
        }
    }
    if let Some((acc, acc_pos)) = last_text.take() {
        merged.push(InlineNode::Text { text: acc, pos: acc_pos });
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::core::inline::types::{InlineNode, SourcePos};

    fn pos(line: usize, col: usize) -> SourcePos {
        SourcePos { line, column: col }
    }

    #[test]
    fn test_merge_adjacent_text_nodes() {
        let nodes = vec![
            InlineNode::Text { text: "Hello ".into(), pos: pos(1,1) },
            InlineNode::Text { text: "World".into(), pos: pos(1,7) },
        ];
        let merged = collapse_text_nodes(nodes);
        assert_eq!(merged.len(), 1);
        match &merged[0] {
            InlineNode::Text { text, pos: _ } => assert_eq!(text, "Hello World"),
            _ => panic!("Expected merged text node"),
        }
    }

    #[test]
    fn test_emphasis_and_strong_resolution() {
        let nodes = vec![
            InlineNode::Text { text: "*foo* ".into(), pos: pos(1,1) },
            InlineNode::Text { text: "**bar**".into(), pos: pos(1,7) },
        ];
        let norm = normalize_inlines(nodes);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should contain Emphasis node");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Strong { .. })), "Should contain Strong node");
    }

    #[test]
    fn test_link_and_image_nodes() {
        let nodes = vec![
            InlineNode::Link {
                href: "https://example.com".into(),
                title: "Example".into(),
                children: vec![InlineNode::Text { text: "link".into(), pos: pos(2,1) }],
                pos: pos(2,1)
            },
            InlineNode::Image {
                src: "img.png".into(),
                alt: "alt text".into(),
                title: "Image".into(),
                pos: pos(3,1)
            },
        ];
        let norm = normalize_inlines(nodes);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Link { .. })), "Should contain Link node");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Image { .. })), "Should contain Image node");
    }

    #[test]
    fn test_position_preservation() {
        let nodes = vec![
            InlineNode::Text { text: "foo".into(), pos: pos(1,1) },
            InlineNode::Text { text: "bar".into(), pos: pos(1,4) },
        ];
        let norm = normalize_inlines(nodes);
        match &norm[0] {
            InlineNode::Text { text, pos } => {
                assert_eq!(text, "foobar");
                assert_eq!(pos.line, 1);
                assert_eq!(pos.column, 1);
            }
            _ => panic!("Expected merged text node"),
        }
    }

    #[test]
    fn test_empty_and_nested_nodes() {
        let nodes = vec![
            InlineNode::Emphasis { children: vec![], pos: pos(1,1) },
            InlineNode::Strong { children: vec![InlineNode::Text { text: "".into(), pos: pos(1,2) }], pos: pos(1,2) },
        ];
        let norm = normalize_inlines(nodes);
        assert!(norm.is_empty(), "Empty and nested nodes should be removed");
    }
}
