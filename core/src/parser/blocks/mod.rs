// Block-level parser modules
//
// This module contains individual block parser functions that convert
// grammar output into AST nodes with proper positioning.
//
// Phase 3: Parser module extraction - COMPLETE

// Shared utilities
pub mod shared;

// Individual block parsers
pub mod cm_blockquote_parser;
pub mod cm_fenced_code_block_parser;
pub mod cm_heading_parser;
pub mod cm_html_blocks_parser;
pub mod cm_indented_code_block_parser;
pub mod cm_link_reference_parser;
pub mod cm_list_parser;
pub mod cm_paragraph_parser;
pub mod cm_thematic_break_parser;
pub mod gfm_footnote_definition_parser;
pub mod gfm_table_parser;
pub mod marco_headerless_table_parser;
pub mod gfm_admonitions;

// Re-export shared utilities
pub use shared::{dedent_list_item_content, to_parser_span, to_parser_span_range, GrammarSpan};

use super::ast::Document;
use crate::grammar::blocks as grammar;
use crate::parser::ast::{Node, NodeKind};
use anyhow::Result;
use nom::Input;

// ============================================================================
// BlockContext: Track open blocks for continuation across blank lines
// ============================================================================

/// Type of block that's currently open
#[derive(Debug, Clone, PartialEq)]
enum BlockContextKind {
    /// Individual list item within a list
    /// content_indent: minimum spaces required for content continuation
    ListItem { content_indent: usize },
}

/// Represents an open block that can accept continuation content
#[derive(Debug, Clone)]
struct BlockContext {
    kind: BlockContextKind,
}

impl BlockContext {
    /// Create a new list item context with the given content indent
    pub fn new_list_item(content_indent: usize) -> Self {
        Self {
            kind: BlockContextKind::ListItem { content_indent },
        }
    }

    /// Check if this block can continue at the given indent level
    fn can_continue_at(&self, indent: usize) -> bool {
        match self.kind {
            BlockContextKind::ListItem { content_indent } => {
                // List item content must be indented at least to content_indent
                indent >= content_indent
            }
        }
    }
}

// ============================================================================
// ParserState: Stack of open blocks for context-aware parsing
// ============================================================================

/// Track all currently open block contexts
struct ParserState {
    blocks: Vec<BlockContext>,
}

impl ParserState {
    fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// Add a new block context to the stack
    pub fn push_block(&mut self, context: BlockContext) {
        self.blocks.push(context);
    }

    /// Remove and return the most recent block context
    fn pop_block(&mut self) -> Option<BlockContext> {
        self.blocks.pop()
    }

    /// Check if the current context can continue at the given indent
    fn can_continue_at(&self, indent: usize) -> bool {
        if let Some(context) = self.blocks.last() {
            context.can_continue_at(indent)
        } else {
            // No context, can't continue
            false
        }
    }

    /// Close blocks that can't continue at the given indent
    /// Returns the number of blocks closed
    fn close_blocks_until_indent(&mut self, indent: usize) -> usize {
        let mut closed = 0;

        // Close blocks from innermost to outermost
        while let Some(context) = self.blocks.last() {
            if context.can_continue_at(indent) {
                // This block can continue, stop closing
                break;
            } else {
                // This block can't continue, close it
                self.blocks.pop();
                closed += 1;
            }
        }

        closed
    }
}

// ============================================================================
// Main block parser entry point
// ============================================================================

/// Parse document into block-level structure, returning a Document
pub fn parse_blocks(input: &str) -> Result<Document> {
    let mut state = ParserState::new();
    parse_blocks_internal(input, 0, &mut state)
}

// Internal parser with recursion depth limit and state tracking
fn parse_blocks_internal(input: &str, depth: usize, state: &mut ParserState) -> Result<Document> {
    // Prevent infinite recursion
    const MAX_DEPTH: usize = 100;
    if depth > MAX_DEPTH {
        log::warn!("Maximum recursion depth reached in block parser");
        return Ok(Document::new());
    }

    log::debug!(
        "Block parser input: {} bytes at depth {}, state depth: {}",
        input.len(),
        depth,
        state.blocks.len()
    );

    let mut nodes = Vec::new();
    let mut document = Document::new(); // Create document early to collect references
    let mut remaining = GrammarSpan::new(input);

    // Safety: prevent infinite loops.
    // This must be high enough for real documents; the progress-check below is the
    // primary safety mechanism.
    let max_iterations = input.lines().count().saturating_mul(8).max(1_000);
    let mut iteration_count = 0;
    let mut last_offset = 0;

    while !remaining.fragment().is_empty() {
        iteration_count += 1;
        if iteration_count > max_iterations {
            log::error!(
                "Block parser exceeded iteration limit ({}) at depth {}",
                max_iterations,
                depth
            );
            break;
        }

        // Safety: ensure we're making progress
        let current_offset = remaining.location_offset();
        if current_offset == last_offset && iteration_count > 1 {
            log::error!(
                "Block parser not making progress at offset {}, depth {}",
                current_offset,
                depth
            );
            // Force skip one character, while preserving span offsets.
            use nom::bytes::complete::take;
            let skip_len = remaining
                .fragment()
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            if let Ok((rest, _)) =
                take::<_, _, nom::error::Error<GrammarSpan>>(skip_len as u32)(remaining)
            {
                remaining = rest;
                last_offset = remaining.location_offset();
                continue;
            }
            break;
        }
        last_offset = current_offset;

        // ========================================================================
        // BLANK LINE HANDLING WITH CONTEXT AWARENESS (Example 307 fix)
        // ========================================================================
        // Extract the first line to check if it's blank
        let first_line_end = remaining
            .fragment()
            .find('\n')
            .unwrap_or(remaining.fragment().len());
        let first_line = &remaining.fragment()[..first_line_end];

        // A line is blank if it contains only whitespace (spaces, tabs)
        if first_line.trim().is_empty() {
            // Peek at the next non-blank line to determine continuation
            let peek_offset = if first_line_end < remaining.fragment().len() {
                first_line_end + 1
            } else {
                first_line_end
            };

            // Find the next non-blank line
            let mut next_nonblank_indent: Option<usize> = None;
            let rest_of_input = &remaining.fragment()[peek_offset..];

            for peek_line in rest_of_input.lines() {
                if !peek_line.trim().is_empty() {
                    // Count leading spaces (expand tabs)
                    let mut indent = 0;
                    for ch in peek_line.chars() {
                        if ch == ' ' {
                            indent += 1;
                        } else if ch == '\t' {
                            indent += 4 - (indent % 4); // Tab to next multiple of 4
                        } else {
                            break;
                        }
                    }
                    next_nonblank_indent = Some(indent);
                    break;
                }
            }

            // Determine if we should preserve context or close blocks
            let should_continue = if let Some(next_indent) = next_nonblank_indent {
                // Check if the next content can continue the current context
                state.can_continue_at(next_indent)
            } else {
                // No more content, close all contexts
                false
            };

            if should_continue {
                // Blank line continues the current block
                // Skip the blank but preserve block context
                log::debug!(
                    "Blank line: continuing context at indent {:?}",
                    next_nonblank_indent
                );

                use nom::bytes::complete::take;
                let skip_len = if first_line_end < remaining.fragment().len() {
                    first_line_end + 1 // Include newline
                } else {
                    first_line_end
                };

                if let Ok((new_remaining, _)) =
                    take::<_, _, nom::error::Error<GrammarSpan>>(skip_len as u32)(remaining)
                {
                    remaining = new_remaining;
                    continue;
                } else {
                    break;
                }
            } else {
                // Blank line ends the current context(s)
                // Close blocks that can't continue at the next indent
                if let Some(next_indent) = next_nonblank_indent {
                    let closed = state.close_blocks_until_indent(next_indent);
                    log::debug!(
                        "Blank line: closed {} blocks due to indent {}",
                        closed,
                        next_indent
                    );
                } else {
                    // No more content, close everything
                    log::debug!("Blank line: end of input, closing all blocks");
                    while state.pop_block().is_some() {}
                }

                // Skip the blank line and continue parsing
                use nom::bytes::complete::take;
                let skip_len = if first_line_end < remaining.fragment().len() {
                    first_line_end + 1
                } else {
                    first_line_end
                };

                if let Ok((new_remaining, _)) =
                    take::<_, _, nom::error::Error<GrammarSpan>>(skip_len as u32)(remaining)
                {
                    remaining = new_remaining;
                    continue;
                } else {
                    break;
                }
            }
        }

        // Try parsing HTML blocks (types 1-7, in order)
        // Type 1: Special raw content tags (script, pre, style, textarea)
        if let Ok((rest, content)) = grammar::html_special_tag(remaining) {
            nodes.push(cm_html_blocks_parser::parse_html_block(content));
            remaining = rest;
            continue;
        }

        // Type 2: HTML comments
        if let Ok((rest, content)) = grammar::html_comment(remaining) {
            nodes.push(cm_html_blocks_parser::parse_html_block(content));
            remaining = rest;
            continue;
        }

        // Type 3: Processing instructions
        if let Ok((rest, content)) = grammar::html_processing_instruction(remaining) {
            nodes.push(cm_html_blocks_parser::parse_html_block(content));
            remaining = rest;
            continue;
        }

        // Type 4: Declarations
        if let Ok((rest, content)) = grammar::html_declaration(remaining) {
            nodes.push(cm_html_blocks_parser::parse_html_block(content));
            remaining = rest;
            continue;
        }

        // Type 5: CDATA sections
        if let Ok((rest, content)) = grammar::html_cdata(remaining) {
            nodes.push(cm_html_blocks_parser::parse_html_block(content));
            remaining = rest;
            continue;
        }

        // Type 6: Standard block tags (div, table, etc.)
        if let Ok((rest, content)) = grammar::html_block_tag(remaining) {
            nodes.push(cm_html_blocks_parser::parse_html_block(content));
            remaining = rest;
            continue;
        }

        // Type 7: Complete tags (CANNOT interrupt paragraphs)
        // Try this but it will fail if we're in the middle of paragraph text
        if let Ok((rest, content)) = grammar::html_complete_tag(remaining) {
            nodes.push(cm_html_blocks_parser::parse_html_block(content));
            remaining = rest;
            continue;
        } // Try parsing heading
        if let Ok((rest, (level, content))) = grammar::heading(remaining) {
            nodes.push(cm_heading_parser::parse_atx_heading(level, content));
            remaining = rest;
            continue;
        }

        // Try parsing fenced code block
        if let Ok((rest, (language, content))) = grammar::fenced_code_block(remaining) {
            nodes.push(cm_fenced_code_block_parser::parse_fenced_code_block(
                language, content,
            ));
            remaining = rest;
            continue;
        }

        // Try parsing thematic break (---, ***, ___)
        if let Ok((rest, content)) = grammar::thematic_break(remaining) {
            nodes.push(cm_thematic_break_parser::parse_thematic_break(content));
            remaining = rest;
            continue;
        }

        // Try parsing block quote (lines starting with >)
        if let Ok((rest, content)) = grammar::blockquote(remaining) {
            let node =
                cm_blockquote_parser::parse_blockquote(content, depth, |cleaned, new_depth| {
                    parse_blocks_internal(cleaned, new_depth, state)
                })?;

            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing indented code block (4 spaces or 1 tab)
        // NOTE: Must come BEFORE lists to avoid indented code being consumed as list content
        if let Ok((rest, content)) = grammar::indented_code_block(remaining) {
            nodes.push(cm_indented_code_block_parser::parse_indented_code_block(
                content,
            ));
            remaining = rest;
            continue;
        }

        // Try parsing list
        // NOTE: Must come BEFORE setext heading to avoid "---" being parsed as underline
        if let Ok((rest, items)) = grammar::list(remaining) {
            let node = cm_list_parser::parse_list(
                items,
                depth,
                parse_blocks_internal,
                |content_indent| {
                    let mut item_state = ParserState::new();
                    item_state.push_block(BlockContext::new_list_item(content_indent));
                    item_state
                },
            )?;

            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing Setext heading (underline style: === or ---)
        // NOTE: Must come AFTER lists to avoid eating list marker patterns like "- foo\n---"
        let full_start = remaining;
        if let Ok((rest, (level, content))) = grammar::setext_heading(remaining) {
            let full_end = rest;
            nodes.push(cm_heading_parser::parse_setext_heading(
                level, content, full_start, full_end,
            ));
            remaining = rest;
            continue;
        }

        // Try parsing link reference definition
        // Must come BEFORE paragraph to avoid treating definitions as paragraphs
        if let Some((rest, node)) =
            gfm_footnote_definition_parser::parse_footnote_definition(remaining)
        {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        if let Ok((rest, (label, url, title))) = grammar::link_reference_definition(remaining) {
            cm_link_reference_parser::parse_link_reference(&mut document, &label, url, title);
            remaining = rest;
            continue;
        }

        // Try parsing GFM pipe table (extension)
        // Must come BEFORE paragraph so tables aren't consumed as plain text.
        //
        // Also try parsing Marco "headerless" pipe tables (delimiter-first).
        // Must come BEFORE paragraph for the same reason.
        let headerless_table_start = remaining;
        if let Ok((rest, table)) = grammar::marco_headerless_table(remaining) {
            nodes.push(marco_headerless_table_parser::parse_marco_headerless_table(
                table,
                headerless_table_start,
                rest,
            ));
            remaining = rest;
            continue;
        }

        let table_start = remaining;
        if let Ok((rest, table)) = grammar::gfm_table(remaining) {
            nodes.push(gfm_table_parser::parse_gfm_table(table, table_start, rest));
            remaining = rest;
            continue;
        }

        // Try parsing extended definition lists (Markdown Guide / Markdown Extra-style)
        // Must come BEFORE paragraph so definition lists aren't consumed as plain text.
        if let Some((rest, node)) = parse_extended_definition_list(remaining, depth) {
            nodes.push(node);
            remaining = rest;
            continue;
        }

        // Try parsing paragraph
        if let Ok((rest, content)) = grammar::paragraph(remaining) {
            nodes.push(cm_paragraph_parser::parse_paragraph(content));
            remaining = rest;
            continue;
        }

        // If nothing matched, skip one character to avoid infinite loop.
        // Use `take` so we preserve nom_locate offsets (important for spans/highlights).
        log::warn!(
            "Could not parse block at offset {}, skipping character",
            remaining.location_offset()
        );
        use nom::bytes::complete::take;
        let skip_len = remaining
            .fragment()
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        if let Ok((rest, _)) =
            take::<_, _, nom::error::Error<GrammarSpan>>(skip_len as u32)(remaining)
        {
            remaining = rest;
        } else {
            break;
        }
    }

    log::info!("Parsed {} blocks", nodes.len());

    // Add parsed nodes to document
    document.children = nodes;
    Ok(document)
}

/// Attempt to parse a Markdown Guide extended definition list at the current input.
///
/// Syntax (canonical):
/// ```text
/// Term
/// : definition
///
/// Another term
/// : first definition
/// : second definition
/// ```
///
/// Supported extensions:
/// - Multiple `: ...` definition lines per term
/// - Multiple term groups in a single list, with optional blank lines between items
/// - Multi-line definition bodies via indented continuation lines (>= 2 spaces)
/// - Nested blocks inside a definition (via recursive block parsing after dedent)
///
/// Explicit non-goals / disambiguation:
/// - Lines starting with `::` are *not* treated as definition markers.
fn parse_extended_definition_list<'a>(
    input: GrammarSpan<'a>,
    depth: usize,
) -> Option<(GrammarSpan<'a>, Node)> {
    // We only match at a non-blank line; blank lines are already handled by the main loop.
    let text = input.fragment();
    if text.is_empty() {
        return None;
    }

    const CONTINUATION_INDENT: usize = 2;

    fn line_bounds(s: &str, start: usize) -> (usize, usize, usize) {
        // Returns: (line_start, line_end_no_nl, next_start)
        let rel_end = s[start..].find('\n').map(|i| start + i).unwrap_or(s.len());
        let next = if rel_end < s.len() {
            rel_end + 1
        } else {
            rel_end
        };
        (start, rel_end, next)
    }

    fn count_indent_columns(line: &str) -> usize {
        // Count leading indentation, expanding tabs to 4-wide tab stops.
        let mut indent = 0usize;
        for ch in line.chars() {
            if ch == ' ' {
                indent += 1;
            } else if ch == '\t' {
                indent += 4 - (indent % 4);
            } else {
                break;
            }
        }
        indent
    }

    fn def_marker_content_start(line: &str) -> Option<usize> {
        // Optional leading spaces (up to 3) are allowed.
        let bytes = line.as_bytes();
        let mut i = 0usize;
        for _ in 0..3 {
            if bytes.get(i) == Some(&b' ') {
                i += 1;
            } else {
                break;
            }
        }

        if bytes.get(i) != Some(&b':') {
            return None;
        }
        // Disallow "::" (reserved for other extensions / lookalikes).
        if bytes.get(i + 1) == Some(&b':') {
            return None;
        }

        // Require at least one whitespace after ':' (Markdown Guide uses ': ')
        match bytes.get(i + 1) {
            Some(b' ') | Some(b'\t') => {
                // Strip exactly one whitespace after the marker; any extra stays as content.
                Some(i + 2)
            }
            _ => None,
        }
    }

    fn can_start_item_at(text: &str, start: usize) -> bool {
        if start >= text.len() {
            return false;
        }
        let (_t0s, t0e, t1s) = line_bounds(text, start);
        let term_line = &text[start..t0e];
        if term_line.trim().is_empty() {
            return false;
        }
        if t1s >= text.len() {
            return false;
        }
        let (_d0s, d0e, _d1s) = line_bounds(text, t1s);
        let def_line = &text[t1s..d0e];
        def_marker_content_start(def_line).is_some()
    }

    // We build a single <dl> node, potentially containing multiple term groups.
    let mut children: Vec<Node> = Vec::new();
    let mut cursor = 0usize;
    let mut parsed_any = false;

    // Parse one or more items.
    loop {
        if cursor >= text.len() {
            break;
        }

        // Parse term line.
        let (term_start, term_end, after_term) = line_bounds(text, cursor);
        let term_line = &text[term_start..term_end];

        // If we're at a blank line here, it means we consumed optional blanks between items.
        // Stop the list; the main loop will handle blanks.
        if term_line.trim().is_empty() {
            break;
        }

        // Term must be followed immediately by at least one definition marker line.
        if after_term >= text.len() {
            break;
        }

        let (def_line_start, def_line_end, _after_def_line) = line_bounds(text, after_term);
        let first_def_line = &text[def_line_start..def_line_end];
        if def_marker_content_start(first_def_line).is_none() {
            break;
        }

        // Build the <dt> node.
        let term_start_span = input.take_from(term_start);
        let (term_after_span, term_taken_span) = term_start_span.take_split(term_end - term_start);
        let term_children = match crate::parser::inlines::parse_inlines_from_span(term_taken_span) {
            Ok(children) => children,
            Err(e) => {
                log::warn!("Failed to parse inline elements in definition term: {}", e);
                vec![Node {
                    kind: NodeKind::Text(term_taken_span.fragment().to_string()),
                    span: Some(crate::parser::shared::to_parser_span(term_taken_span)),
                    children: Vec::new(),
                }]
            }
        };

        children.push(Node {
            kind: NodeKind::DefinitionTerm,
            span: Some(crate::parser::shared::to_parser_span_range(
                term_start_span,
                term_after_span,
            )),
            children: term_children,
        });

        // Parse one or more definitions for this term.
        cursor = after_term;
        while cursor < text.len() {
            let (line_start, line_end, next_line_start) = line_bounds(text, cursor);
            let line = &text[line_start..line_end];

            let content_start_in_line = match def_marker_content_start(line) {
                Some(i) => i,
                None => break,
            };

            // Definition block span starts at the marker line.
            let def_block_start = line_start;
            let mut def_block_end = next_line_start;

            // Build raw definition body text: first line after ": ", then indented continuations.
            let mut raw_lines: Vec<&str> = Vec::new();
            raw_lines.push(&line[content_start_in_line..]);

            let mut scan = next_line_start;
            while scan < text.len() {
                let (ls, le, ln) = line_bounds(text, scan);
                let l = &text[ls..le];

                // Next definition marker starts a new <dd>.
                if def_marker_content_start(l).is_some() {
                    break;
                }

                if l.trim().is_empty() {
                    // Only treat a blank line as part of this definition if the
                    // next non-blank line is indented enough to continue.
                    let mut look = ln;
                    let mut next_indent: Option<usize> = None;
                    while look < text.len() {
                        let (_pls, ple, pln) = line_bounds(text, look);
                        let pl = &text[look..ple];
                        if !pl.trim().is_empty() {
                            next_indent = Some(count_indent_columns(pl));
                            break;
                        }
                        look = pln;
                    }

                    if next_indent.unwrap_or(0) >= CONTINUATION_INDENT {
                        raw_lines.push("");
                        scan = ln;
                        def_block_end = scan;
                        continue;
                    }

                    break;
                }

                let indent = count_indent_columns(l);
                if indent >= CONTINUATION_INDENT {
                    raw_lines.push(l);
                    scan = ln;
                    def_block_end = scan;
                    continue;
                }

                break;
            }

            let raw_body = raw_lines.join("\n");
            let dedented = dedent_list_item_content(&raw_body, CONTINUATION_INDENT);

            // Parse the definition body as nested blocks.
            let mut def_state = ParserState::new();
            def_state.push_block(BlockContext::new_list_item(CONTINUATION_INDENT));
            let def_children = match parse_blocks_internal(&dedented, depth + 1, &mut def_state) {
                Ok(doc) => doc.children,
                Err(e) => {
                    log::warn!("Failed to parse definition description blocks: {}", e);
                    Vec::new()
                }
            };

            let dd_start_span = input.take_from(def_block_start);
            let dd_end_span = input.take_from(def_block_end);
            children.push(Node {
                kind: NodeKind::DefinitionDescription,
                span: Some(crate::parser::shared::to_parser_span_range(
                    dd_start_span,
                    dd_end_span,
                )),
                children: def_children,
            });

            parsed_any = true;
            cursor = def_block_end;
        }

        // Between items, allow blank lines *only if* another valid item follows.
        let mut scan = cursor;
        while scan < text.len() {
            let (_ls, le, ln) = line_bounds(text, scan);
            let l = &text[scan..le];
            if !l.trim().is_empty() {
                break;
            }
            scan = ln;
        }

        if scan != cursor && can_start_item_at(text, scan) {
            cursor = scan;
            continue;
        }

        break;
    }

    if !parsed_any {
        return None;
    }

    let (rest, _taken) = input.take_split(cursor);
    let span = crate::parser::shared::to_parser_span_range(input, rest);
    Some((
        rest,
        Node {
            kind: NodeKind::DefinitionList,
            span: Some(span),
            children,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse_blocks;
    use crate::parser::ast::NodeKind;

    #[test]
    fn smoke_test_block_parser_handles_large_documents() {
        // Regression test: we previously had an iteration cap (100) that could truncate
        // parsing for realistic documents, which in turn truncated syntax highlighting.
        let count = 250;
        let mut input = String::new();
        for i in 0..count {
            input.push_str(&format!("Paragraph {i}\n\n"));
        }

        let doc = parse_blocks(&input).expect("parse_blocks failed");
        assert_eq!(doc.children.len(), count);
        assert!(matches!(
            doc.children.last().unwrap().kind,
            NodeKind::Paragraph
        ));
    }
}
