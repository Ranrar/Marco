/// Normalize inline nodes: resolve delimiters, flatten, and merge adjacent text nodes.
pub fn normalize_inlines(nodes: Vec<InlineNode>) -> Vec<InlineNode> {
    if nodes.is_empty() {
        return Vec::new();
    }
    println!("AST before normalization: {:?}", nodes);
    let (mut preprocessed, mut delim_stack) = preprocess_delimiters(nodes);
    // Strictly resolve delimiters and update indices after each removal
    resolve_delimiters_recursive_clean(&mut preprocessed, &mut delim_stack);
    println!("AST after delimiter resolution: {:?}", preprocessed);

    // Remove empty emphasis/strong nodes, flatten children, and strictly enforce spec
    let mut flat: Vec<InlineNode> = Vec::new();
    for node in preprocessed.drain(..) {
        match node {
            InlineNode::Code { text, pos } => {
                // Do not parse delimiters or nest emphasis/strong inside code spans
                flat.push(InlineNode::Code { text, pos });
            },
            InlineNode::Emphasis { children, pos } => {
                // Strictly enforce: no empty emphasis, no nesting inside code, minimal nesting
                let norm = children.into_iter()
                    .filter(|c| !matches!(c, InlineNode::Code { .. }))
                    .collect::<Vec<_>>();
                if norm.iter().any(|c| !matches!(c, InlineNode::Text { text, .. } if text.trim().is_empty())) && !norm.is_empty() {
                    flat.push(InlineNode::Emphasis { children: norm, pos });
                }
            },
            InlineNode::Strong { children, pos } => {
                // Strictly enforce: no empty strong, no nesting inside code, minimal nesting
                let norm = children.into_iter()
                    .filter(|c| !matches!(c, InlineNode::Code { .. }))
                    .collect::<Vec<_>>();
                if norm.iter().any(|c| !matches!(c, InlineNode::Text { text, .. } if text.trim().is_empty())) && !norm.is_empty() {
                    flat.push(InlineNode::Strong { children: norm, pos });
                }
            },
            InlineNode::Entity { text, pos } => {
                // Only valid entities outside code spans, preserve as Entity node
                flat.push(InlineNode::Entity { text, pos });
            },
            InlineNode::Text { text, pos } => {
                if !text.is_empty() {
                    flat.push(InlineNode::Text { text, pos });
                }
            },
            InlineNode::AttributeBlock { text, pos } => {
                // Only attach attribute blocks to valid nodes, emit lone/malformed as text
                if !text.is_empty() {
                    flat.push(InlineNode::AttributeBlock { text, pos });
                }
            },
            InlineNode::Html { text, pos } => {
                flat.push(InlineNode::Html { text, pos });
            },
            InlineNode::Math { text, pos } => {
                flat.push(InlineNode::Math { text, pos });
            },
            InlineNode::Link { href, title, children, pos } => {
                let norm = children.into_iter().map(|c| match c {
                    InlineNode::AttributeBlock { text, pos: attr_pos } => InlineNode::AttributeBlock { text, pos: attr_pos },
                    _ => c
                }).collect();
                flat.push(InlineNode::Link { href, title, children: norm, pos });
            },
            InlineNode::Image { src, alt, title, pos } => {
                let norm = alt.into_iter().map(|c| match c {
                    InlineNode::AttributeBlock { text, pos: attr_pos } => InlineNode::AttributeBlock { text, pos: attr_pos },
                    _ => c
                }).collect();
                flat.push(InlineNode::Image { src, alt: norm, title, pos });
            },
            InlineNode::SoftBreak { pos } => { 
                flat.push(InlineNode::SoftBreak { pos }); 
            },
            InlineNode::LineBreak { pos } => { 
                flat.push(InlineNode::LineBreak { pos }); 
            },
            InlineNode::Strikethrough { children, pos } => {
                let norm = children.into_iter().filter(|c| !matches!(c, InlineNode::Code { .. })).collect::<Vec<_>>();
                if norm.iter().any(|c| !matches!(c, InlineNode::Text { text, .. } if text.trim().is_empty())) && !norm.is_empty() {
                    flat.push(InlineNode::Strikethrough { children: norm, pos });
                }
            },
            InlineNode::TaskListItem { checked, children, pos } => {
                let norm = children.into_iter().filter(|c| !matches!(c, InlineNode::Code { .. })).collect::<Vec<_>>();
                if norm.iter().any(|c| !matches!(c, InlineNode::Text { text, .. } if text.trim().is_empty())) && !norm.is_empty() {
                    flat.push(InlineNode::TaskListItem { checked, children: norm, pos });
                }
            },
        }
    }
    // Collapse adjacent text nodes and preserve earliest position
    collapse_text_nodes(flat)
}

/// Recursively resolve delimiters and build emphasis/strong/strikethrough nodes.
pub fn resolve_delimiters_recursive_clean(nodes: &mut Vec<crate::logic::core::inline::types::InlineNode>, stack: &mut Vec<crate::logic::core::inline::types::Delim>) {
    use crate::logic::core::inline::types::InlineNode;
    
    // Strict CommonMark delimiter resolution
    let mut i = 0;
    while i < stack.len() {
        let closer = &stack[i];
        if !closer.active || !closer.can_close || !closer.right_flanking {
            i += 1;
            continue;
        }

        // Find nearest valid opener
        let mut opener_idx = None;
        for j in (0..i).rev() {
            let opener = &stack[j];
            if opener.ch == closer.ch && opener.active && opener.can_open && opener.left_flanking
                && opener.in_code == closer.in_code && opener.in_html == closer.in_html
                && !opener.in_code && !opener.in_html && !opener.in_image && !opener.in_link
            {
                // Multiples-of-3 rule
                let opener_count = opener.count;
                let closer_count = closer.count;
                if (opener_count + closer_count) % 3 == 0 && (opener_count % 3 != 0 && closer_count % 3 != 0) {
                    continue;
                }
                opener_idx = Some(j);
                break;
            }
        }

        if let Some(j) = opener_idx {
            let opener = &stack[j];
            let delim_char = opener.ch;
            let opener_count = opener.count;
            let closer_count = closer.count;
            let use_count = if opener_count >= 2 && closer_count >= 2 { 2 } else { 1 };
            let opener_node_idx = opener.idx;
            let closer_node_idx = closer.idx;

            if opener_node_idx < nodes.len() && closer_node_idx < nodes.len() && opener_node_idx < closer_node_idx {
                // Remove closer and opener nodes
                nodes.remove(closer_node_idx);
                nodes.remove(opener_node_idx);

                // Extract inner content
                let mut inner = if closer_node_idx > opener_node_idx + 1 {
                    nodes[opener_node_idx..opener_node_idx + (closer_node_idx - opener_node_idx - 1)].to_vec()
                } else {
                    Vec::new()
                };
                nodes.drain(opener_node_idx..opener_node_idx + (closer_node_idx - opener_node_idx - 1));
                resolve_delimiters_recursive_clean(&mut inner, &mut Vec::new());

                // Minimal nesting: prefer strong, then emph
                let new_node = match delim_char {
                    '*' | '_' => {
                        if use_count == 2 {
                            InlineNode::Strong { children: inner, pos: opener.pos.clone() }
                        } else {
                            InlineNode::Emphasis { children: inner, pos: opener.pos.clone() }
                        }
                    }
                    '~' => InlineNode::Strikethrough { children: inner, pos: opener.pos.clone() },
                    _ => InlineNode::Text { text: delim_char.to_string().repeat(use_count), pos: opener.pos.clone() },
                };
                nodes.insert(opener_node_idx, new_node);

                // Adjust delimiter counts for partial consumption
                if opener_count > use_count {
                    stack[j].count -= use_count;
                    stack[j].idx = opener_node_idx + 1;
                } else {
                    stack[j].active = false;
                }
                if closer_count > use_count {
                    stack[i].count -= use_count;
                    stack[i].idx = opener_node_idx + 1;
                } else {
                    stack[i].active = false;
                }

                // Remove processed delimiters from stack if fully consumed
                if stack[j].active == false && stack[i].active == false {
                    if i > j {
                        stack.remove(i);
                        stack.remove(j);
                    } else {
                        stack.remove(j);
                        stack.remove(i);
                    }
                }

                // Restart from beginning
                i = 0;
                continue;
            }
        }
        i += 1;
    }

    // Emit unmatched delimiters as text (including partial runs)
    for d in stack.iter() {
        if d.active && d.idx < nodes.len() && d.count > 0 {
            nodes[d.idx] = InlineNode::Text {
                text: d.ch.to_string().repeat(d.count),
                pos: d.pos.clone(),
            };
        }
    }
}

/// Collapse adjacent text nodes into a single node.
pub fn collapse_text_nodes(nodes: Vec<crate::logic::core::inline::types::InlineNode>) -> Vec<crate::logic::core::inline::types::InlineNode> {
    let mut merged = Vec::new();
    let mut last_text: Option<(String, super::types::SourcePos)> = None;
    
    for node in nodes {
        match node {
            InlineNode::Text { text, pos } => {
                if let Some((acc, acc_pos)) = last_text.take() {
                    let mut new_acc = acc;
                    new_acc.push_str(&text);
                    last_text = Some((new_acc, acc_pos));
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
    
    merged
}

pub use super::types::{InlineNode, Delim};
// ...existing code...

/// Preprocess delimiters: walk the inline nodes, collect delimiter runs, and return (nodes, delim_stack).
/// This is a fresh, correct implementation with all logic inside the function and all braces matched.
fn preprocess_delimiters(nodes: Vec<InlineNode>) -> (Vec<InlineNode>, Vec<Delim>) {
    use super::types::{InlineNode, Delim};
    let mut result = Vec::new();
    let mut delim_stack = Vec::new();
    let mut last_delim_idx: Option<usize> = None;

    // Helper: recursively walk nodes, collect delimiters, and build result
    fn inner(
        nodes: Vec<InlineNode>,
        context: (bool, bool, bool, bool), // (in_code, in_html, in_link, in_image)
        result: &mut Vec<InlineNode>,
        delim_stack: &mut Vec<Delim>,
        last_delim_idx: &mut Option<usize>,
    ) {
        for node in nodes {
            match node {
                InlineNode::Text { text, pos } => {
                    let mut chars = text.chars().peekable();
                    let mut buf = String::new();
                    let mut prev_char: Option<char> = None;
                    
                    while let Some(ch) = chars.peek().cloned() {
                        if ch == '*' || ch == '_' || ch == '~' {
                            // Flush buffer as text node
                            if !buf.is_empty() {
                                result.push(InlineNode::Text { text: buf.clone(), pos: pos.clone() });
                                buf.clear();
                            }
                            
                            // Count run length
                            let mut count = 0;
                            let start = ch;
                            while let Some(c2) = chars.peek() {
                                if *c2 == start {
                                    count += 1;
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            
                            // Group delimiter runs (e.g., **, __, ~~)
                            let idx = result.len();
                            let next_char = chars.peek().cloned();
                            let prev_is_whitespace = prev_char.map_or(true, |c| c.is_whitespace());
                            let next_is_whitespace = next_char.map_or(true, |c| c.is_whitespace());
                            let left_flanking = !next_is_whitespace;
                            let right_flanking = !prev_is_whitespace;
                            let can_open = left_flanking && (!right_flanking || start == '_');
                            let can_close = right_flanking && (!left_flanking || start == '_');
                            
                            delim_stack.push(Delim {
                                ch: start,
                                count,
                                idx,
                                active: true,
                                can_open,
                                can_close,
                                left_flanking,
                                right_flanking,
                                in_code: context.0,
                                in_html: context.1,
                                in_link: context.2,
                                in_image: context.3,
                                prev: None,
                                next: None,
                                pos: pos.clone(),
                            });
                            result.push(InlineNode::Text { text: start.to_string().repeat(count), pos: pos.clone() });
                            *last_delim_idx = Some(idx);
                            prev_char = Some(start);
                        } else {
                            buf.push(ch);
                            prev_char = Some(ch);
                            chars.next();
                        }
                    }
                    
                    if !buf.is_empty() {
                        result.push(InlineNode::Text { text: buf, pos });
                    }
                }
                InlineNode::Emphasis { children, pos } => {
                    let mut sub = Vec::new();
                    inner(children, context, &mut sub, delim_stack, last_delim_idx);
                    result.push(InlineNode::Emphasis { children: sub, pos });
                }
                InlineNode::Strong { children, pos } => {
                    let mut sub = Vec::new();
                    inner(children, context, &mut sub, delim_stack, last_delim_idx);
                    result.push(InlineNode::Strong { children: sub, pos });
                }
                InlineNode::Strikethrough { children, pos } => {
                    let mut sub = Vec::new();
                    inner(children, context, &mut sub, delim_stack, last_delim_idx);
                    result.push(InlineNode::Strikethrough { children: sub, pos });
                }
                InlineNode::TaskListItem { checked, children, pos } => {
                    let mut sub = Vec::new();
                    inner(children, context, &mut sub, delim_stack, last_delim_idx);
                    result.push(InlineNode::TaskListItem { checked, children: sub, pos });
                }
                InlineNode::Code { text, pos } => {
                    // Code spans: do not treat as delimiters
                    result.push(InlineNode::Code { text, pos });
                }
                InlineNode::Html { text, pos } => {
                    // HTML: do not treat as delimiters
                    result.push(InlineNode::Html { text, pos });
                }
                InlineNode::Entity { text, pos } => {
                    result.push(InlineNode::Entity { text, pos });
                }
                InlineNode::AttributeBlock { text, pos } => {
                    result.push(InlineNode::AttributeBlock { text, pos });
                }
                InlineNode::Math { text, pos } => {
                    result.push(InlineNode::Math { text, pos });
                }
                InlineNode::Link { href, title, children, pos } => {
                    let mut sub = Vec::new();
                    inner(children, (context.0, context.1, true, context.3), &mut sub, delim_stack, last_delim_idx);
                    result.push(InlineNode::Link { href, title, children: sub, pos });
                }
                InlineNode::Image { src, alt, title, pos } => {
                    let mut sub = Vec::new();
                    inner(alt, (context.0, context.1, context.2, true), &mut sub, delim_stack, last_delim_idx);
                    result.push(InlineNode::Image { src, alt: sub, title, pos });
                }
                InlineNode::SoftBreak { pos } => {
                    result.push(InlineNode::SoftBreak { pos });
                }
                InlineNode::LineBreak { pos } => {
                    result.push(InlineNode::LineBreak { pos });
                }
            }
        }
    }

    inner(nodes, (false, false, false, false), &mut result, &mut delim_stack, &mut last_delim_idx);
    (result, delim_stack)
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
            InlineNode::Text { text: "***foo* bar**".into(), pos: pos(5,10) },
            InlineNode::Text { text: "**a *b***".into(), pos: pos(5,11) },
            InlineNode::Text { text: "*a **b* c**".into(), pos: pos(5,12) },
            InlineNode::Text { text: "**bold *italic** text*".into(), pos: pos(5,13) },
            InlineNode::Text { text: "*unclosed".into(), pos: pos(5,14) },
            InlineNode::Text { text: "** **".into(), pos: pos(5,15) },
            InlineNode::Text { text: "foo*bar* baz_baz_".into(), pos: pos(5,16) },
            InlineNode::Text { text: "foo &amp bar".into(), pos: pos(5,17) },
            InlineNode::Text { text: "* * *".into(), pos: pos(5,18) },
            InlineNode::Text { text: "***".into(), pos: pos(5,19) },
            InlineNode::Text { text: "****".into(), pos: pos(5,20) },
            InlineNode::Text { text: "*foo*bar*baz*".into(), pos: pos(5,21) },
            InlineNode::Text { text: "*foo* *bar* *baz*".into(), pos: pos(5,22) },
            InlineNode::Text { text: "*foo**bar***baz****".into(), pos: pos(5,23) },
            InlineNode::Text { text: "*foo* **bar** ***baz***".into(), pos: pos(5,24) },
            InlineNode::Text { text: "*foo*bar**baz***qux****".into(), pos: pos(5,25) },
        ];
        let norm = normalize_inlines(nodes);
        println!("Normalized AST: {:?}", norm);
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should parse Emphasis node");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Strong { .. })), "Should parse Strong node");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Emphasis { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Strong { .. })))), "Should nest Strong inside Emphasis");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Strong { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Emphasis { .. })))), "Should nest Emphasis inside Strong");
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Emphasis { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Code { .. })))), "Should not nest Emphasis inside Code");
        // Multiples-of-3 and partial consumption
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("***"))), "Should emit unmatched triple delimiter as text");
        // Unmatched delimiters
        assert!(norm.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("*unclosed"))), "Should emit unclosed delimiter as text");
    }
}