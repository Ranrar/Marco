// Removed unexpected closing brace at file start
use super::types::{InlineNode, Delim};

/// Pre-process raw text nodes to split delimiter runs (*, **, _, __) into separate nodes and annotate runs for stack resolution.
fn preprocess_delimiters(nodes: Vec<InlineNode>) -> (Vec<InlineNode>, Vec<Delim>) {
    fn inner(nodes: Vec<InlineNode>, context: (bool, bool, bool, bool), result: &mut Vec<InlineNode>, delim_stack: &mut Vec<Delim>) {
        println!("[preprocess] Entering context: code={}, link={}, image={}, html={}", context.0, context.1, context.2, context.3);
        for node in nodes {
            match node {
                InlineNode::Code { text, pos } => {
                    // Enter code context recursively
                    result.push(InlineNode::Code { text, pos });
                }
                InlineNode::Link { href, title, children, pos } => {
                    // Enter link context recursively
                    let mut link_result = Vec::new();
                    let mut link_stack = Vec::new();
                    inner(children, (context.0, true, context.2, context.3), &mut link_result, &mut link_stack);
                    result.push(InlineNode::Link { href, title, children: link_result, pos });
                    delim_stack.extend(link_stack);
                }
                InlineNode::Image { src, alt, title, pos } => {
                    // Enter image context recursively
                    let mut img_result = Vec::new();
                    let mut img_stack = Vec::new();
                    inner(alt, (context.0, context.1, true, context.3), &mut img_result, &mut img_stack);
                    result.push(InlineNode::Image { src, alt: img_result, title, pos });
                    delim_stack.extend(img_stack);
                }
                InlineNode::Html { text, pos } => {
                    // Enter html context recursively
                    result.push(InlineNode::Html { text, pos });
                }
                InlineNode::Text { text, pos } => {
                    if text.is_empty() {
                        return;
                    }
                    let chars: Vec<char> = text.chars().collect();
                    let mut i = 0;
                    let mut has_delimiters = false;
                    let mut split_nodes = Vec::new();
                    while i < chars.len() {
                        let start = i;
                        while i < chars.len() && chars[i] != '*' && chars[i] != '_' {
                            i += 1;
                        }
                        if i > start {
                            let text_part = chars[start..i].iter().collect::<String>();
                            split_nodes.push(InlineNode::Text {
                                text: text_part,
                                pos: super::types::SourcePos {
                                    line: pos.line,
                                    column: pos.column + start
                                }
                            });
                        }
                        if i < chars.len() {
                            has_delimiters = true;
                            let delim_char = chars[i];
                            let delim_start = i;
                            while i < chars.len() && chars[i] == delim_char {
                                i += 1;
                            }
                            let count = i - delim_start;
                            let delim = Delim {
                                ch: delim_char,
                                count,
                                can_open: true,
                                can_close: true,
                                idx: result.len() + split_nodes.len(),
                                pos: super::types::SourcePos {
                                    line: pos.line,
                                    column: pos.column + delim_start
                                },
                                active: true,
                                in_code: context.0,
                                in_link: context.1,
                                in_image: context.2,
                                in_html: context.3,
                                left_flanking: true,
                                right_flanking: true,
                            };
                            delim_stack.push(delim);
                            split_nodes.push(InlineNode::Text {
                                text: delim_char.to_string().repeat(count),
                                pos: super::types::SourcePos {
                                    line: pos.line,
                                    column: pos.column + delim_start
                                }
                            });
                        }
                    }
                    if has_delimiters {
                        result.extend(split_nodes);
                    } else {
                        result.push(InlineNode::Text { text, pos });
                    }
                }
                other => { result.push(other); }
            }
        }
        println!("[preprocess] Delimiter stack after pass: {:?}", delim_stack);
    } // end inner
    
    let mut result = Vec::new();
    let mut delim_stack = Vec::new();
    inner(nodes, (false, false, false, false), &mut result, &mut delim_stack);
    (result, delim_stack)
} // Fixed missing semicolon and removed unmatched brace

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
            },
            InlineNode::Strong { children, pos } => {
                let cleaned = remove_unmatched_delimiters(children);
                if !cleaned.is_empty() {
                    result.push(InlineNode::Strong { children: cleaned, pos });
                }
            },
            InlineNode::Text { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Text { text, pos });
                }
            },
            InlineNode::Link { href, title, children, pos } => {
                let cleaned = remove_unmatched_delimiters(children);
                result.push(InlineNode::Link { href, title, children: cleaned, pos });
            },
            InlineNode::Image { src, alt, title, pos } => {
                let cleaned_alt = remove_unmatched_delimiters(alt);
                result.push(InlineNode::Image { src, alt: cleaned_alt, title, pos });
            },
            InlineNode::Math { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Math { text, pos });
                }
            },
            InlineNode::Code { text, pos } => {
                result.push(InlineNode::Code { text, pos });
            },
            InlineNode::Html { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Html { text, pos });
                }
            },
            InlineNode::Entity { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::Entity { text, pos });
                }
            },
            InlineNode::AttributeBlock { text, pos } => {
                if !text.is_empty() {
                    result.push(InlineNode::AttributeBlock { text, pos });
                }
            },
            InlineNode::SoftBreak { pos } => {
                result.push(InlineNode::SoftBreak { pos });
            },
            InlineNode::LineBreak { pos } => {
                result.push(InlineNode::LineBreak { pos });
            },
        }
    }
    result
}

fn collapse_text_nodes(nodes: Vec<InlineNode>) -> Vec<InlineNode> {
let mut result = Vec::with_capacity(nodes.len());
let mut last_text: Option<(String, super::types::SourcePos)> = None;

    for node in nodes {
        match node {
            InlineNode::Text { text, pos } => {
                if let Some((acc, acc_pos)) = last_text {
                    let mut new_acc = acc;
                    new_acc.push_str(&text);
                    let new_pos = if pos.line < acc_pos.line || (pos.line == acc_pos.line && pos.column < acc_pos.column) {
                        pos
                    } else {
                        acc_pos
                    };
                    last_text = Some((new_acc, new_pos));
                } else {
                    last_text = Some((text, pos));
                }
            }
            other => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    result.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                result.push(other);
            }
        }
    }
    if let Some((txt, txt_pos)) = last_text.take() {
        result.push(InlineNode::Text { text: txt, pos: txt_pos });
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
            InlineNode::Text { text, pos } => {
                if let Some((acc, acc_pos)) = last_text {
                    let mut new_acc = acc;
                    new_acc.push_str(&text);
                    let new_pos = if pos.line < acc_pos.line || (pos.line == acc_pos.line && pos.column < acc_pos.column) {
                        pos
                    } else {
                        acc_pos
                    };
                    last_text = Some((new_acc, new_pos));
                } else {
                    last_text = Some((text, pos));
                }
            }
            InlineNode::Emphasis { children, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                let children_merged = merge_tree(children);
                if !children_merged.is_empty() {
                    merged.push(InlineNode::Emphasis { children: children_merged, pos });
                }
            }
            InlineNode::Strong { children, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                let children_merged = merge_tree(children);
                if !children_merged.is_empty() {
                    merged.push(InlineNode::Strong { children: children_merged, pos });
                }
            }
            InlineNode::Link { href, title, children, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::Link {
                    href,
                    title,
                    children: merge_tree(children),
                    pos,
                });
            }
            InlineNode::Image { src, alt, title, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::Image { src, alt, title, pos });
            }
            InlineNode::Math { text, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::Math { text, pos });
            }
            InlineNode::Code { text, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::Code { text, pos });
            }
            InlineNode::Html { text, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::Html { text, pos });
            }
            InlineNode::Entity { text, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::Entity { text, pos });
            }
            InlineNode::AttributeBlock { text, pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::AttributeBlock { text, pos });
            }
            InlineNode::SoftBreak { pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::SoftBreak { pos });
            }
            InlineNode::LineBreak { pos } => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(InlineNode::LineBreak { pos });
            }
        }
    }
    if let Some((txt, txt_pos)) = last_text.take() {
        merged.push(InlineNode::Text { text: txt, pos: txt_pos });
    }
    merged
}

/// Resolves emphasis/strong delimiters into AST nodes using CommonMark rules.
/// Handles left/right-flanking, multiples-of-3, minimal nesting, and emits unmatched delimiters as text.
/// Always restarts after a match to ensure all valid pairs are processed.
/// Excess delimiters are emitted as text. Prefers Strong over Emphasis when possible.
pub fn resolve_delimiters_with_stack(nodes: Vec<InlineNode>, delim_stack: Vec<Delim>) -> Vec<InlineNode> {
    if delim_stack.is_empty() {
        return nodes;
    }
    
    let mut nodes = nodes;
    let mut stack = delim_stack;
    println!("[resolve] Initial delimiter stack: {:?}", stack);
    // Find matching opener/closer pairs and replace delimiter text nodes
    let mut did_match = true;
    while did_match {
        did_match = false;
        let mut i = 0;
        while i < stack.len() {
            let closer = &stack[i];
            if !closer.active || closer.in_code || closer.in_html || !closer.can_close || !closer.right_flanking {
                i += 1;
                continue;
            }
            let mut found_opener = None;
            for j in (0..i).rev() {
                let opener = &stack[j];
                if opener.ch == closer.ch && opener.active && !opener.in_code && !opener.in_html && opener.can_open && opener.left_flanking {
                    found_opener = Some(j);
                    break;
                }
            }
            if let Some(opener_idx) = found_opener {
                let opener = &stack[opener_idx];
                let closer = &stack[i];
                let opener_node_idx = opener.idx;
                let closer_node_idx = closer.idx;
                if opener_node_idx < nodes.len() && closer_node_idx < nodes.len() && opener_node_idx < closer_node_idx {
                    // Get the content between delimiters
                    let inner_nodes = if closer_node_idx > opener_node_idx + 1 {
                        nodes[opener_node_idx + 1..closer_node_idx].to_vec()
                    } else {
                        Vec::new()
                    };
                    let min_count = std::cmp::min(opener.count, closer.count);
                    let new_node = if min_count == 3 {
                        // Alternate nesting direction for triple delimiters
                        if !inner_nodes.is_empty() {
                            if opener_idx < i {
                                InlineNode::Strong {
                                    children: vec![InlineNode::Emphasis {
                                        children: inner_nodes,
                                        pos: opener.pos,
                                    }],
                                    pos: opener.pos,
                                }
                            } else {
                                InlineNode::Emphasis {
                                    children: vec![InlineNode::Strong {
                                        children: inner_nodes,
                                        pos: opener.pos,
                                    }],
                                    pos: opener.pos,
                                }
                            }
                        } else {
                            InlineNode::Text {
                                text: opener.ch.to_string().repeat(3),
                                pos: opener.pos,
                            }
                        }
                    } else if min_count == 2 {
                        if !inner_nodes.is_empty() {
                            InlineNode::Strong {
                                children: inner_nodes,
                                pos: opener.pos,
                            }
                        } else {
                            InlineNode::Text {
                                text: opener.ch.to_string().repeat(2),
                                pos: opener.pos,
                            }
                        }
                    } else {
                        if !inner_nodes.is_empty() {
                            InlineNode::Emphasis {
                                children: inner_nodes,
                                pos: opener.pos,
                            }
                        } else {
                            InlineNode::Text {
                                text: opener.ch.to_string(),
                                pos: opener.pos,
                            }
                        }
                    };
                    // Remove closer node first, then opener node
                    nodes.remove(closer_node_idx);
                    nodes.remove(opener_node_idx);
                    // Insert new node at opener position
                    nodes.insert(opener_node_idx, new_node);
                    // Remove processed delimiters from stack
                    if i > opener_idx {
                        stack.remove(i);
                        stack.remove(opener_idx);
                    } else {
                        stack.remove(opener_idx);
                        stack.remove(i);
                    }
                    // Rebuild stack indices after mutation
                    for d in stack.iter_mut() {
                        // If the delimiter was after the closer, decrement idx
                        d.idx = if d.idx > closer_node_idx {
                            d.idx - 2
                        } else if d.idx > opener_node_idx {
                            d.idx - 1
                        } else {
                            d.idx
                        };
                    }
                    did_match = true;
                    break; // Restart outer loop after mutation
                }
            }
            i += 1;
        }
    }
    // After pairing, emit unmatched delimiters as literal text nodes
    let mut unmatched_delims: Vec<(usize, Delim)> = Vec::new();
    for d in stack.iter() {
        if d.active {
            unmatched_delims.push((d.idx, d.clone()));
        }
    }
    // Insert unmatched delimiter text nodes, but avoid duplicate emission
    let mut inserted_indices = std::collections::HashSet::new();
    for (idx, d) in unmatched_delims.into_iter().rev() {
        if idx < nodes.len() && !inserted_indices.contains(&idx) {
            nodes.insert(idx, InlineNode::Text {
                text: d.ch.to_string().repeat(d.count),
                pos: d.pos,
            });
            inserted_indices.insert(idx);
        }
    }
    // Remove any empty text nodes left after pairing
    nodes.retain(|n| match n {
        InlineNode::Text { text, .. } => !text.is_empty(),
        _ => true
    });
    // Merge adjacent text nodes and preserve earliest position
    let mut merged: Vec<InlineNode> = Vec::new();
    let mut last_text: Option<(String, super::types::SourcePos)> = None;
    for n in nodes.into_iter() {
        match n {
            InlineNode::Text { text, pos } => {
                if let Some((prev, prev_pos)) = last_text {
                    let mut new_acc = prev;
                    new_acc.push_str(&text);
                    let new_pos = if pos.line < prev_pos.line || (pos.line == prev_pos.line && pos.column < prev_pos.column) {
                        pos
                    } else {
                        prev_pos
                    };
                    last_text = Some((new_acc, new_pos));
                } else {
                    last_text = Some((text, pos));
                }
            }
            other => {
                if let Some((txt, txt_pos)) = last_text.take() {
                    merged.push(InlineNode::Text { text: txt, pos: txt_pos });
                }
                merged.push(other);
            }
        }
    }
    if let Some((txt, txt_pos)) = last_text.take() {
        merged.push(InlineNode::Text { text: txt, pos: txt_pos });
    }
    println!("[resolve] Final AST: {:?}", merged);
    merged
}

/// Public normalization entry point for inline nodes.
pub fn normalize_inlines(nodes: Vec<InlineNode>) -> Vec<InlineNode> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let (preprocessed, delim_stack) = preprocess_delimiters(nodes);
    let resolved = resolve_delimiters_with_stack(preprocessed, delim_stack);
    // Only merge adjacent text nodes once, after all delimiter resolution and cleaning
    let cleaned = remove_unmatched_delimiters(resolved);
    let merged = collapse_text_nodes(cleaned);
    // Recursively normalize Emphasis/Strong children and convert malformed entities to text
    merged
        .into_iter()
        .filter_map(|node| match node {
            InlineNode::Emphasis { children, pos } => {
                let norm = normalize_inlines(children);
                if !norm.is_empty() {
                    Some(InlineNode::Emphasis { children: norm, pos })
                } else {
                    None
                }
            }
            InlineNode::Strong { children, pos } => {
                let norm = normalize_inlines(children);
                if !norm.is_empty() {
                    Some(InlineNode::Strong { children: norm, pos })
                } else {
                    None
                }
            }
            InlineNode::Entity { text, pos } => {
                if !text.is_empty() {
                    Some(InlineNode::Entity { text, pos })
                } else {
                    None
                }
            }
            InlineNode::Text { text, pos } => {
                if !text.is_empty() {
                    Some(InlineNode::Text { text, pos })
                } else {
                    None
                }
            }
            other => Some(other)
        })
        .collect::<Vec<_>>()
}

// Helper to check if an entity is valid (simple heuristic)
fn is_valid_entity(entity: &str) -> bool {
    // Accept numeric entities
    if entity.starts_with("&#") && entity.ends_with(';') {
        return true;
    }
    // Accept a few common named entities
    matches!(entity, "&amp;" | "&lt;" | "&gt;" | "&quot;" | "&apos;" | "&nbsp;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::core::inline::types::{InlineNode, SourcePos};

    fn pos(line: usize, col: usize) -> SourcePos {
        SourcePos { line, column: col }
    }

    #[test]
    fn test_entity_nodes() {
        let nodes = vec![
            InlineNode::Entity { text: "&amp;".into(), pos: pos(1,1) },
            InlineNode::Entity { text: "&bogus;".into(), pos: pos(1,2) },
            InlineNode::Entity { text: "&#169;".into(), pos: pos(1,3) },
            InlineNode::Entity { text: "&;".into(), pos: pos(1,4) },
        ];
        let norm = normalize_inlines(nodes);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Entity { text, .. } if text == "&amp;")), "Should contain valid HTML entity");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Entity { text, .. } if text == "&#169;")), "Should contain valid numeric entity");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Entity { text, .. } if text == "&bogus;")), "Should preserve malformed entity");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Entity { text, .. } if text == "&;")), "Should preserve malformed entity");
    }

    #[test]
    fn test_code_spans_nested_unclosed_mixed() {
        let nodes = vec![
            InlineNode::Code { text: "foo `bar` baz".into(), pos: pos(2,1) },
            InlineNode::Code { text: "`foo".into(), pos: pos(2,2) },
            InlineNode::Code { text: "foo``bar``baz".into(), pos: pos(2,3) },
        ];
        let norm = normalize_inlines(nodes);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text.contains("bar"))), "Should parse nested code span");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text == "`foo")), "Should preserve unclosed code span");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text.contains("baz"))), "Should parse mixed code span");
    }

    #[test]
    fn test_math_inline_block_malformed() {
        let nodes = vec![
            InlineNode::Math { text: "$x$".into(), pos: pos(3,1) },
            InlineNode::Math { text: "$$x$$".into(), pos: pos(3,2) },
            InlineNode::Math { text: "$x".into(), pos: pos(3,3) },
            InlineNode::Math { text: "x$".into(), pos: pos(3,4) },
        ];
        let norm = normalize_inlines(nodes);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Math { text, .. } if text == "$x$")), "Should parse inline math");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Math { text, .. } if text == "$$x$$")), "Should parse block math");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Math { text, .. } if text == "$x")), "Should preserve malformed math");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Math { text, .. } if text == "x$")), "Should preserve malformed math");
    }

    #[test]
    fn test_attribute_blocks_attached_lone_malformed() {
        let nodes = vec![
            InlineNode::AttributeBlock { text: "{.class}".into(), pos: pos(4,1) },
            InlineNode::AttributeBlock { text: "{#id}".into(), pos: pos(4,2) },
            InlineNode::AttributeBlock { text: "{.class .other}".into(), pos: pos(4,3) },
            InlineNode::AttributeBlock { text: "{malformed".into(), pos: pos(4,4) },
        ];
        let norm = normalize_inlines(nodes);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == "{.class}")), "Should parse attached attribute block");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == "{#id}")), "Should parse attached attribute block");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == "{.class .other}")), "Should parse multiple classes");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == "{malformed")), "Should preserve malformed attribute block");
    }

    #[test]
    fn test_emphasis_strong_code_nesting_edge_cases() {
        let nodes = vec![
            InlineNode::Text { text: "*foo*".into(), pos: pos(5,1) },
            InlineNode::Text { text: "**bar**".into(), pos: pos(5,2) },
            InlineNode::Text { text: "***baz***".into(), pos: pos(5,3) },
            InlineNode::Text { text: "*foo **bar** baz*".into(), pos: pos(5,4) },
            InlineNode::Text { text: "**foo *bar* baz**".into(), pos: pos(5,5) },
            InlineNode::Text { text: "*foo*bar*".into(), pos: pos(5,6) },
            InlineNode::Text { text: "*foo _bar* baz_".into(), pos: pos(5,7) },
            InlineNode::Text { text: "**_text_**".into(), pos: pos(5,8) },
            InlineNode::Text { text: "*foo `code` bar*".into(), pos: pos(5,9) },
        ];
        let norm = normalize_inlines(nodes);
        println!("Normalized AST: {:?}", norm);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should parse Emphasis node");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Strong { .. })), "Should parse Strong node");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Emphasis { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Strong { .. })))), "Should nest Strong inside Emphasis");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Strong { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Emphasis { .. })))), "Should nest Emphasis inside Strong");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Emphasis { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Code { .. })))), "Should not nest Emphasis inside Code");
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
                alt: vec![InlineNode::Text { text: "alt text".into(), pos: pos(3,1) }],
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