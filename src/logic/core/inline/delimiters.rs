use crate::logic::core::inline::types::Delim;
use crate::logic::core::inline::types::SourcePos;
use crate::logic::core::inline::types::InlineNode;
pub fn process_delimiters(
    delim_stack: &mut Vec<Delim>,
    result: &mut Vec<(InlineNode, SourcePos)>
) {
    let mut used = vec![false; delim_stack.len()];
    let mut changed = true;

    // Repeat until no more matches can be made
    while changed {
        changed = false;
        let mut i = 0;
        while i < delim_stack.len() {
            if !delim_stack[i].can_close {
                i += 1;
                continue;
            }

            let closer = &delim_stack[i];
            if closer.in_code || closer.in_link || closer.in_image || closer.in_html {
                i += 1;
                continue;
            }

            let mut j = i;
            while j > 0 {
                j -= 1;
                let opener = &delim_stack[j];

                // Skip unusable openers
                if !opener.can_open || opener.in_code || opener.in_link || opener.in_image || opener.in_html {
                    continue;
                }

                // Skip already used delimiters
                if opener.count == 0 || closer.count == 0 {
                    continue;
                }

                // Only match same delimiter type for strong/emph
                if opener.ch == closer.ch {
                    // Left/right-flanking rule
                    if !opener.left_flanking || !closer.right_flanking {
                        continue;
                    }

                    // Multiples-of-3 rule (CommonMark spec §6.3)
                    let sum = opener.count + closer.count;
                    let both_mult_3 = opener.count % 3 == 0 && closer.count % 3 == 0;
                    let opener_both = opener.can_open && opener.can_close;
                    let closer_both = closer.can_open && closer.can_close;
                    if (opener_both || closer_both) && sum % 3 == 0 && !both_mult_3 {
                        continue;
                    }

                    // Determine if strong or emph based on min delimiter count
                    let delim_len = if opener.count >= 2 && closer.count >= 2 {
                        2 // strong
                    } else {
                        1 // emphasis
                    };

                    // Compute bounds
                    let opener_idx = opener.idx;
                    let closer_idx = closer.idx;
                    let (start, end) = if opener_idx < closer_idx {
                        (opener_idx + 1, closer_idx)
                    } else {
                        (closer_idx + 1, opener_idx)
                    };

                    // Extract children between opener and closer
                    let inner_nodes_with_pos = if end > start && end <= result.len() {
                        result.splice(start..end, std::iter::empty()).collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };

                    let children: Vec<InlineNode> = inner_nodes_with_pos.into_iter().map(|(n, _)| n).collect();

                    if children.is_empty() {
                        break;
                    }

                    // Create AST node: Emphasis or Strong
                    let ast = if delim_len == 2 {
                        InlineNode::Strong { children, pos: opener.pos.clone() }
                    } else {
                        InlineNode::Emphasis { children, pos: opener.pos.clone() }
                    };

                    // Insert new node at match location
                    if start >= result.len() {
                        result.push((ast, opener.pos.clone()));
                    } else {
                        result.insert(start, (ast, opener.pos.clone()));
                    }

                    // Decrement used delimiter count (preserve remaining run)
                    delim_stack[j].count -= delim_len;
                    delim_stack[i].count -= delim_len;

                    // If run is fully used, mark as used
                    if delim_stack[j].count == 0 {
                        used[j] = true;
                    }
                    if delim_stack[i].count == 0 {
                        used[i] = true;
                    }

                    changed = true;
                    break; // continue with current i in next loop
                } else {
                    // Mixed delimiter types: allow nesting (e.g., **_text_**, _**text**_)
                    // Only match if opener and closer are both valid openers/closers
                    if opener.can_open && closer.can_close {
                        // Compute bounds
                        let opener_idx = opener.idx;
                        let closer_idx = closer.idx;
                        let (start, end) = if opener_idx < closer_idx {
                            (opener_idx + 1, closer_idx)
                        } else {
                            (closer_idx + 1, opener_idx)
                        };

                        let inner_nodes_with_pos = if end > start && end <= result.len() {
                            result.splice(start..end, std::iter::empty()).collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        };

                        let children: Vec<InlineNode> = inner_nodes_with_pos.into_iter().map(|(n, _)| n).collect();

                        if children.is_empty() {
                            break;
                        }

                        // Create AST node: Emphasis (outer) with Strong (inner) or vice versa
                        let ast = if opener.ch == '*' && closer.ch == '_' {
                            InlineNode::Emphasis {
                                children: vec![InlineNode::Strong { children, pos: opener.pos.clone() }],
                                pos: opener.pos.clone()
                            }
                        } else if opener.ch == '_' && closer.ch == '*' {
                            InlineNode::Emphasis {
                                children: vec![InlineNode::Strong { children, pos: opener.pos.clone() }],
                                pos: opener.pos.clone()
                            }
                        } else {
                            // Fallback: treat as text if not a valid combination
                            InlineNode::Text { text: opener.ch.to_string().repeat(opener.count) + &closer.ch.to_string().repeat(closer.count), pos: opener.pos.clone() }
                        };

                        if start >= result.len() {
                            result.push((ast, opener.pos.clone()));
                        } else {
                            result.insert(start, (ast, opener.pos.clone()));
                        }

                        delim_stack[j].count = 0;
                        delim_stack[i].count = 0;
                        used[j] = true;
                        used[i] = true;
                        changed = true;
                        break;
                    }
                }
            }
            i += 1;
        }
    }

    // Emit any unmatched delimiters as literal text
    for (idx, delim) in delim_stack.iter().enumerate() {
        if !used[idx] && delim.count > 0 {
            let txt = delim.ch.to_string().repeat(delim.count);
            result.push((
                InlineNode::Text {
                    text: txt,
                    pos: delim.pos.clone()
                },
                delim.pos.clone()
            ));
        }
    }
}
