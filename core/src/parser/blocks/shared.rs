// Shared utilities for block-level parsers
// Contains span conversion helpers and common types

use crate::parser::position::{Position, Span as ParserSpan};
use nom_locate::LocatedSpan;

/// Grammar span type (nom_locate::LocatedSpan)
pub type GrammarSpan<'a> = LocatedSpan<&'a str>;

/// Convert grammar span (LocatedSpan) to parser span (line/column)
/// 
/// # Arguments
/// * `span` - The grammar span to convert
/// 
/// # Returns
/// * `ParserSpan` with line and column information
pub fn to_parser_span(span: GrammarSpan) -> ParserSpan {
    let start = Position::new(
        span.location_line() as usize,
        span.get_column(),
        span.location_offset(),
    );
    let end = Position::new(
        span.location_line() as usize,
        span.get_column() + span.fragment().len(),
        span.location_offset() + span.fragment().len(),
    );
    ParserSpan::new(start, end)
}

/// Convert grammar span range to parser span (from start to end)
/// 
/// # Arguments
/// * `start` - The starting grammar span
/// * `end` - The ending grammar span
/// 
/// # Returns
/// * `ParserSpan` with full range information
pub fn to_parser_span_range(start: GrammarSpan, end: GrammarSpan) -> ParserSpan {
    let start_pos = Position::new(
        start.location_line() as usize,
        start.get_column(),
        start.location_offset(),
    );
    let end_pos = Position::new(
        end.location_line() as usize,
        end.get_column() + end.fragment().len(),
        end.location_offset() + end.fragment().len(),
    );
    ParserSpan::new(start_pos, end_pos)
}

/// Dedent list item content by removing the specified indent width.
/// This function is used to strip the list item indentation from nested content.
///
/// # Arguments
/// * `content` - The content to dedent
/// * `content_indent` - Number of spaces to remove from each line
///
/// # Returns
/// The dedented content with proper handling of:
/// - Tab expansion to spaces (based on actual column position)
/// - Trailing newline preservation
/// - Leading space removal up to content_indent
///
/// # Tab Expansion
/// Tabs are expanded based on their actual column position in the line.
/// Starting at `content_indent` column, each tab advances to the next multiple of 4.
/// This matches the CommonMark spec for list item indentation handling.
pub fn dedent_list_item_content(content: &str, content_indent: usize) -> String {
    let had_trailing_newline = content.ends_with('\n');
    
    let mut result = content.lines()
        .map(|line| {
            // First, expand tabs to spaces based on ACTUAL column position
            // Tabs must be expanded based on their column position (content_indent + column in line)
            let mut expanded = String::with_capacity(line.len() * 2);
            let mut column = content_indent; // Start at the content_indent column
            
            for ch in line.chars() {
                if ch == '\t' {
                    // Tab advances to next multiple of 4
                    let spaces_to_add = 4 - (column % 4);
                    for _ in 0..spaces_to_add {
                        expanded.push(' ');
                        column += 1;
                    }
                } else {
                    expanded.push(ch);
                    column += 1;
                }
            }
            
            // Now count and strip leading spaces up to content_indent
            let mut spaces_to_strip = 0;
            let mut chars = expanded.chars();
            while spaces_to_strip < content_indent {
                match chars.next() {
                    Some(' ') => spaces_to_strip += 1,
                    _ => break,
                }
            }
            
            // Return the rest of the line after stripping
            expanded[spaces_to_strip..].to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    // Preserve trailing newline if original had one
    if had_trailing_newline {
        result.push('\n');
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_to_parser_span() {
        let input = "line1\nline2\nline3";
        let span = LocatedSpan::new(input);
        let parser_span = to_parser_span(span);
        assert_eq!(parser_span.start.line, 1);
        assert_eq!(parser_span.start.column, 1);
    }

    #[test]
    fn smoke_test_dedent_simple() {
        let content = "  Line 1\n  Line 2\n";
        let result = dedent_list_item_content(content, 2);
        assert_eq!(result, "Line 1\nLine 2\n");
    }

    #[test]
    fn smoke_test_dedent_preserves_extra_indent() {
        let content = "  Line 1\n    Indented\n";
        let result = dedent_list_item_content(content, 2);
        assert_eq!(result, "Line 1\n  Indented\n");
    }

    #[test]
    fn smoke_test_dedent_preserves_blank_lines() {
        let content = "  Line 1\n\n  Line 2\n";
        let result = dedent_list_item_content(content, 2);
        assert_eq!(result, "Line 1\n\nLine 2\n");
    }

    #[test]
    fn smoke_test_dedent_with_tabs() {
        let content = "\tLine 1\n\tLine 2\n";
        let result = dedent_list_item_content(content, 4);
        assert_eq!(result, "Line 1\nLine 2\n");
    }
}
