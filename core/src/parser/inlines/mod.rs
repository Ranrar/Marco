//! Inline parser modules - convert grammar output to AST nodes
//!
//! This module contains specialized parsers that convert inline grammar elements
//! (from grammar/inlines) into AST nodes with proper position tracking.
//!
//! Phase 5: Inline parser module extraction

// Shared utilities for all inline parsers
pub mod shared;

// Individual inline parser modules
pub mod cm_autolink_parser;
pub mod cm_backslash_escape_parser;
pub mod cm_code_span_parser;
pub mod cm_emphasis_parser;
pub mod cm_entity_reference_parser;
pub mod cm_image_parser;
pub mod cm_inline_html_parser;
pub mod cm_line_breaks_parser;
pub mod cm_link_parser;
pub mod cm_reference_link_parser;
pub mod cm_strong_emphasis_parser;
pub mod cm_strong_parser;
pub mod gfm_autolink_literal_parser;
pub mod gfm_footnote_reference_parser;
pub mod gfm_strikethrough_parser;
pub mod marco_dash_strikethrough_parser;
pub mod marco_emoji_shortcode_parser;
pub mod marco_inline_footnote_parser;
pub mod marco_mark_parser;
pub mod marco_platform_mentions_parser;
pub mod marco_subscript_arrow_parser;
pub mod marco_subscript_parser;
pub mod marco_superscript_parser;
pub mod marco_task_checkbox_inline_parser;
pub mod text_parser;

// Re-export parser functions for convenience
pub use cm_autolink_parser::parse_autolink;
pub use cm_backslash_escape_parser::parse_backslash_escape;
pub use cm_code_span_parser::parse_code_span;
pub use cm_emphasis_parser::parse_emphasis;
pub use cm_entity_reference_parser::parse_entity_reference;
pub use cm_image_parser::parse_image;
pub use cm_inline_html_parser::parse_inline_html;
pub use cm_line_breaks_parser::{parse_hard_line_break, parse_soft_line_break};
pub use cm_link_parser::parse_link;
pub use cm_reference_link_parser::parse_reference_link;
pub use cm_strong_emphasis_parser::parse_strong_emphasis;
pub use cm_strong_parser::parse_strong;
pub use gfm_autolink_literal_parser::parse_gfm_autolink_literal;
pub use gfm_footnote_reference_parser::parse_footnote_reference;
pub use gfm_strikethrough_parser::parse_strikethrough;
pub use marco_dash_strikethrough_parser::parse_dash_strikethrough;
pub use marco_emoji_shortcode_parser::parse_emoji_shortcode;
pub use marco_inline_footnote_parser::parse_inline_footnote;
pub use marco_mark_parser::parse_mark;
pub use marco_platform_mentions_parser::parse_platform_mention;
pub use marco_subscript_arrow_parser::parse_subscript_arrow;
pub use marco_subscript_parser::parse_subscript;
pub use marco_superscript_parser::parse_superscript;
pub use marco_task_checkbox_inline_parser::parse_task_checkbox_inline;
pub use text_parser::{parse_special_as_text, parse_text};

use super::ast::{Node, NodeKind};
use anyhow::Result;
use nom::bytes::complete::take;
use shared::{to_parser_span, GrammarSpan};

/// Parse inline elements within text content
/// Takes a GrammarSpan to preserve position information
/// Returns a vector of inline nodes (Text, Emphasis, Strong, Link, CodeSpan)
pub fn parse_inlines_from_span(span: GrammarSpan) -> Result<Vec<Node>> {
    log::debug!(
        "Parsing inline elements in span at line {}: {:?}",
        span.location_line(),
        span.fragment()
    );

    let mut nodes = Vec::new();
    let mut remaining = span;

    // Safety: prevent infinite loops
    const MAX_ITERATIONS: usize = 1000;
    let mut iteration_count = 0;
    let mut last_offset = 0;

    while !remaining.fragment().is_empty() {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            log::error!("Inline parser exceeded MAX_ITERATIONS ({})", MAX_ITERATIONS);
            break;
        }

        let start_pos = remaining.location_offset();

        // Safety: ensure we're making progress
        if start_pos == last_offset && iteration_count > 1 {
            log::error!(
                "Inline parser not making progress at offset {}, forcing skip",
                start_pos
            );
            // Force skip one character
            let skip = remaining
                .fragment()
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            if let Ok((rest, _)) = take::<_, _, nom::error::Error<_>>(skip)(remaining) {
                remaining = rest;
                last_offset = remaining.location_offset();
                continue;
            } else {
                break;
            }
        }
        last_offset = start_pos;

        // Try parsing code span first (highest priority to avoid conflicts)
        if let Ok((rest, node)) = parse_code_span(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing backslash escape (before other inline elements)
        if let Ok((rest, node)) = parse_backslash_escape(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Extension inlines (non-CommonMark): try these early so their delimiter
        // sequences aren't consumed as plain text.
        if let Ok((rest, node)) = parse_strikethrough(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        if let Ok((rest, node)) = parse_dash_strikethrough(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        if let Ok((rest, node)) = parse_mark(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // CommonMark underscore emphasis has special delimiter rules. In
        // particular, intraword underscores (alnum _ alnum) should not open
        // or close emphasis. Because our parser advances left-to-right, the
        // underscore parsers may not be able to see the previous character.
        //
        // Workaround: when we're at an underscore run and the previous emitted
        // character is alphanumeric and the next character after the run is
        // alphanumeric, consume the underscore run as literal text.
        if let Some(run_len) = intraword_underscore_run_len(&nodes, remaining.fragment()) {
            if let Ok((rest, consumed)) = take::<_, _, nom::error::Error<_>>(run_len)(remaining) {
                nodes.push(Node {
                    kind: NodeKind::Text("_".repeat(run_len)),
                    span: Some(to_parser_span(consumed)),
                    children: Vec::new(),
                });
                remaining = rest;
                continue;
            }
        }

        // Try parsing strong+emphasis (***text*** / ___text___) before strong
        // so we don't consume the first two delimiters as strong and leave a
        // dangling delimiter behind.
        if let Ok((rest, node)) = parse_strong_emphasis(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing strong (must come before emphasis to match ** before *)
        if let Ok((rest, node)) = parse_strong(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing emphasis
        if let Ok((rest, node)) = parse_emphasis(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Marco extension: inline footnotes `^[...]`.
        // Try before superscript since both start with '^'.
        if let Ok((rest, (ref_node, def_node))) = parse_inline_footnote(remaining) {
            nodes.push(ref_node);
            nodes.push(def_node);
            remaining = rest;
            continue;
        }

        if let Ok((rest, node)) = parse_superscript(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        if let Ok((rest, node)) = parse_subscript_arrow(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        if let Ok((rest, node)) = parse_subscript(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing GFM autolink literals (www/http(s)/email/protocol forms)
        if let Ok((rest, node)) = parse_gfm_autolink_literal(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing autolink (must come before link and inline HTML since syntax starts with <)
        if let Ok((rest, node)) = parse_autolink(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing GFM-style footnote references `[^label]`.
        // Must come before link parsing since it also starts with '['.
        if let Ok((rest, node)) = parse_footnote_reference(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Marco extension: inline task checkbox markers mid-paragraph.
        // This must come before link parsing since it starts with '['.
        if is_task_checkbox_inline_start_boundary_ok(&nodes, remaining.fragment()) {
            if let Ok((rest, node)) = parse_task_checkbox_inline(remaining) {
                nodes.push(node);
                remaining = rest;
                continue;
            }
        }

        // Try parsing image (must come before link since syntax is similar but starts with !)
        if let Ok((rest, node)) = parse_image(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing link
        if let Ok((rest, node)) = parse_link(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing reference-style links (CommonMark)
        if let Ok((rest, node)) = parse_reference_link(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing inline HTML
        if let Ok((rest, node)) = parse_inline_html(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing hard line break (two spaces + newline, or backslash + newline)
        if let Ok((rest, node)) = parse_hard_line_break(remaining) {
            log::debug!(
                "Parsed hard line break at offset {}",
                remaining.location_offset()
            );
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing soft line break (regular newline)
        if let Ok((rest, node)) = parse_soft_line_break(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing entity references (e.g. &copy;, &#169;)
        if let Ok((rest, node)) = parse_entity_reference(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing emoji shortcodes (Marco extension), e.g. :joy:
        if let Ok((rest, node)) = parse_emoji_shortcode(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing platform mentions (Marco extension), e.g. @user[github](Name)
        if let Ok((rest, node)) = parse_platform_mention(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // No inline element matched - try parsing plain text
        if let Ok((rest, node)) = parse_text(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Special character that didn't parse as any inline element - consume as text
        if let Ok((rest, node)) = parse_special_as_text(remaining) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Safety check: if we reach here, we failed to parse anything
        // This should not happen if all parsers are working correctly
        log::error!(
            "Inline parser unable to make progress at offset {}",
            start_pos
        );
        break;
    }

    log::debug!("Parsed {} inline nodes", nodes.len());
    Ok(nodes)
}

fn intraword_underscore_run_len(nodes: &[Node], fragment: &str) -> Option<usize> {
    if !fragment.starts_with('_') {
        return None;
    }

    let prev = last_emitted_char(nodes)?;
    if !prev.is_alphanumeric() {
        return None;
    }

    let run_len = fragment.chars().take_while(|&c| c == '_').count();
    let after = fragment.chars().nth(run_len)?;
    if !after.is_alphanumeric() {
        return None;
    }

    Some(run_len)
}

fn is_task_checkbox_inline_start_boundary_ok(nodes: &[Node], fragment: &str) -> bool {
    if !fragment.starts_with('[') {
        return false;
    }

    // If the previous emitted character is alphanumeric/underscore, we do not
    // treat `[x]` / `[ ]` as a task marker (avoid matching `word[x]`).
    match last_emitted_char(nodes) {
        None => true,
        Some(prev) => !(prev.is_alphanumeric() || prev == '_'),
    }
}

fn last_emitted_char(nodes: &[Node]) -> Option<char> {
    nodes.iter().rev().find_map(last_char_in_node)
}

fn last_char_in_node(node: &Node) -> Option<char> {
    match &node.kind {
        NodeKind::Text(t) => t.chars().last(),
        // Formatting/container nodes: use their last child.
        _ => node.children.iter().rev().find_map(last_char_in_node),
    }
}

/// Parse inline elements within text content (backward compatibility wrapper)
/// Creates a new span at position 0:0 - USE parse_inlines_from_span() for position-aware parsing
/// Returns a vector of inline nodes (Text, Emphasis, Strong, Link, CodeSpan)
pub fn parse_inlines(text: &str) -> Result<Vec<Node>> {
    parse_inlines_from_span(GrammarSpan::new(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_triple_delimiter_parses_as_single_node() {
        let nodes = parse_inlines("***hi***").expect("inline parse failed");
        assert_eq!(nodes.len(), 1);
        assert!(matches!(
            nodes[0].kind,
            crate::parser::ast::NodeKind::StrongEmphasis
        ));
    }

    #[test]
    fn smoke_test_extension_inlines_parse_mid_line() {
        let nodes = parse_inlines(
            "This is ^sup^ and ~sub~ and ˅sub2˅ and ==mark== and ~~del~~ and --del2--.",
        )
        .expect("inline parse failed");

        use crate::parser::ast::NodeKind;

        assert!(nodes
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Superscript)));
        assert!(nodes.iter().any(|n| matches!(n.kind, NodeKind::Subscript)));
        assert!(nodes.iter().any(|n| matches!(n.kind, NodeKind::Mark)));
        assert!(nodes
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Strikethrough)));
    }
}
