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

// Re-export shared utilities
pub use shared::{dedent_list_item_content, to_parser_span, to_parser_span_range, GrammarSpan};

use super::ast::Document;
use crate::grammar::blocks as grammar;
use anyhow::Result;

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
        if let Ok((rest, (label, url, title))) = grammar::link_reference_definition(remaining) {
            cm_link_reference_parser::parse_link_reference(&mut document, &label, url, title);
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
