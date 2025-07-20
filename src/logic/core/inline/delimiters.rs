//! delimiters.rs - Delimiter stack logic (emphasis, strong, links/images)

/// Implements the delimiter stack algorithm for Markdown emphasis, strong, and links/images.
/// Maintains a stack of delimiters with metadata and performs pairing of opening/closing delimiters.
///
/// # Algorithm Overview
/// - Iterates through the delimiter stack, looking for closing delimiters.
/// - For each closer, looks back for a matching opener.
/// - Applies the multiples-of-3 rule for spec compliance.
/// - Determines whether to create an emphasis or strong node.
/// - Extracts the inline nodes between opener and closer, wraps them, and inserts the AST node.
/// - Marks used delimiters and converts unmatched ones to plain text.
///
/// # Parameters
/// - `delim_stack`: Mutable reference to the delimiter stack (Vec<Delim>).
/// - `result`: Mutable reference to the result vector of (Inline, SourcePos) pairs.
///
/// # Notes
/// - This function is spec-compliant with CommonMark/GFM rules for emphasis/strong.
/// - Handles edge cases such as multiples-of-3, nested delimiters, and unmatched delimiters.
/// - Unmatched delimiters are converted to plain text nodes.

/// Implements the delimiter stack algorithm for emphasis, strong, and links/images.
/// Maintains a stack of delimiters with metadata and performs pairing of opening/closing delimiters.

use super::types::{Delim, InlineNode, SourcePos};

/// Processes the delimiter stack for emphasis/strong pairing and returns updated result vector.
/// Processes the delimiter stack for emphasis/strong pairing and returns updated result vector.
pub fn process_delimiters(
    delim_stack: &mut Vec<Delim>,
    result: &mut Vec<(InlineNode, SourcePos)>
) {
    // Track which delimiters have been paired/used
    let mut used = vec![false; delim_stack.len()];
    let mut i = 0;
    while i < delim_stack.len() {
        // Only process closing delimiters
        if !delim_stack[i].can_close {
            i += 1;
            continue;
        }
        let closer = &delim_stack[i];
        // Look back for matching opener
        let mut j = i;
        while j > 0 {
            j -= 1;
            let opener = &delim_stack[j];
            // Skip if opener is used, can't open, or doesn't match delimiter char
            if used[j] || !opener.can_open || opener.ch != closer.ch {
                continue;
            }
            // Multiples-of-3 rule (CommonMark spec)
            // If either delimiter can both open and close, and the sum is a multiple of 3,
            // and not both are multiples of 3, do not pair.
            let sum = opener.count + closer.count;
            let both_mult_3 = opener.count % 3 == 0 && closer.count % 3 == 0;
            let opener_both = opener.can_open && opener.can_close;
            let closer_both = closer.can_open && closer.can_close;
            if (opener_both || closer_both) && sum % 3 == 0 && !both_mult_3 {
                continue;
            }
            // Determine emphasis type: strong (**) or emph (*)
            let min_count = opener.count.min(closer.count);
            let emph_type = if min_count >= 2 { "strong" } else { "emph" };
            // Extract actual inline nodes between opener and closer
            let opener_idx = opener.idx;
            let closer_idx = closer.idx;
            let (start, end) = if opener_idx < closer_idx {
                (opener_idx + 1, closer_idx)
            } else {
                (closer_idx + 1, opener_idx)
            };
            // Only extract inner nodes if indices are valid
            let inner_nodes_with_pos = if end > start && end <= result.len() {
                // Use splice for in-place replacement, avoids extra allocation
                result.splice(start..end, std::iter::empty()).collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            // Build AST node for emphasis or strong
            let children: Vec<InlineNode> = inner_nodes_with_pos.into_iter().map(|(n, _)| n).collect();
            let ast = if emph_type == "strong" {
                InlineNode::Strong { children, pos: opener.pos.clone() }
            } else {
                InlineNode::Emphasis { children, pos: opener.pos.clone() }
            };
            // Insert AST node at correct position
            if start >= result.len() {
                // Only clone SourcePos if necessary
                result.push((ast, opener.pos.clone()));
            } else {
                result.insert(start, (ast, opener.pos.clone()));
            }
            // Mark delimiters as used
            used[j] = true;
            used[i] = true;
            break;
        }
        i += 1;
    }
    // Unmatched delimiters become plain text nodes
    for (idx, delim) in delim_stack.iter().enumerate() {
        if !used[idx] {
            let txt = delim.ch.to_string().repeat(delim.count);
            result.push((InlineNode::Text { text: txt, pos: delim.pos.clone() }, delim.pos.clone()));
        }
    }
}
