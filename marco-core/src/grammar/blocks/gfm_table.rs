// GitHub Flavored Markdown (GFM) pipe tables (extension)
//
// Grammar-level detection/parsing of a table block:
// - Header row (a line containing `|`)
// - Delimiter row (a line containing `-` and `|`, with optional `:` for alignment)
// - 0+ body rows (subsequent lines containing `|`)
//
// This is intentionally conservative:
// - Requires `|` on both header and delimiter rows so it can't be confused with
//   Setext headings (`---`).
// - Requires the header cell count to match delimiter cell count.

use crate::grammar::shared::{count_indentation, Span};
use nom::character::complete::{line_ending, not_line_ending};
use nom::{IResult, Input};

#[derive(Debug, Clone, PartialEq)]
pub struct GfmTableBlock<'a> {
    pub header_line: Span<'a>,
    pub delimiter_line: Span<'a>,
    pub body_lines: Vec<Span<'a>>,
}

/// Parse a GFM pipe table starting at the current position.
///
/// Returns the consumed table (header+delimiter+rows) as spans that reference the
/// original input.
pub fn gfm_table(input: Span<'_>) -> IResult<Span<'_>, GfmTableBlock<'_>> {
    // Table blocks can't start with 4+ spaces (would be indented code).
    if count_indentation(input.fragment()) >= 4 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Header line
    let (after_header_line, header_line) = not_line_ending(input)?;
    if header_line.fragment().trim().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Require at least one unescaped '|' in the header line.
    if count_unescaped_pipes(header_line.fragment()) == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Require a newline after the header line (need a delimiter row).
    let (after_header_newline, _) = line_ending(after_header_line)?;

    // Delimiter line
    let (after_delimiter_line, delimiter_line) = not_line_ending(after_header_newline)?;

    // Require at least one unescaped '|' in the delimiter line to avoid Setext confusion.
    if count_unescaped_pipes(delimiter_line.fragment()) == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Require at least one '-' somewhere on delimiter line.
    if !delimiter_line.fragment().contains('-') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Validate header/delimiter cell counts and delimiter cell syntax.
    let header_cells = split_pipe_row_cells(header_line);
    let delimiter_cells = split_pipe_row_cells(delimiter_line);

    if header_cells.is_empty() || header_cells.len() != delimiter_cells.len() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    if !delimiter_cells
        .iter()
        .all(|cell| is_valid_delimiter_cell(cell.fragment()))
    {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Consume newline after delimiter line if present.
    // (Use an explicit error type to avoid nom 8 type inference issues.)
    let mut remaining = after_delimiter_line;
    if let Ok((rest, _)) = line_ending::<Span, nom::error::Error<Span>>(remaining) {
        remaining = rest;
    }

    // Body rows: consecutive non-blank lines containing at least one unescaped '|'.
    let mut body_lines = Vec::new();
    while !remaining.fragment().is_empty() {
        // Peek line without consuming newline
        let (after_line, line) = match not_line_ending::<Span, nom::error::Error<Span>>(remaining) {
            Ok(v) => v,
            Err(_) => break,
        };

        if line.fragment().trim().is_empty() {
            break;
        }

        // Stop if the line is indented code.
        if count_indentation(line.fragment()) >= 4 {
            break;
        }

        if count_unescaped_pipes(line.fragment()) == 0 {
            break;
        }

        body_lines.push(line);

        // Consume the newline if present.
        match line_ending::<Span, nom::error::Error<Span>>(after_line) {
            Ok((rest, _)) => remaining = rest,
            Err(_) => {
                remaining = after_line;
                break;
            }
        }
    }

    Ok((
        remaining,
        GfmTableBlock {
            header_line,
            delimiter_line,
            body_lines,
        },
    ))
}

/// Split a table row into cell spans.
///
/// Rules (GFM):
/// - Leading/trailing `|` are optional and do not create extra columns.
/// - `\|` does not act as a delimiter.
/// - Leading/trailing spaces and tabs around each cell are trimmed.
///
/// Returned spans reference the original input.
pub(crate) fn split_pipe_row_cells(line: Span<'_>) -> Vec<Span<'_>> {
    let trimmed_line = trim_ws_span(line);

    let fragment = trimmed_line.fragment();
    let has_leading_pipe = fragment.starts_with('|');
    let has_trailing_pipe = fragment.ends_with('|');

    let bytes = fragment.as_bytes();
    let mut ranges: Vec<(usize, usize)> = Vec::new();

    let mut start = 0;
    let mut backslash_run = 0usize;

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'|' {
            if backslash_run.is_multiple_of(2) {
                ranges.push((start, i));
                start = i + 1;
            }
            backslash_run = 0;
            continue;
        }

        if b == b'\\' {
            backslash_run += 1;
        } else {
            backslash_run = 0;
        }
    }

    // Final cell
    ranges.push((start, bytes.len()));

    let mut cells: Vec<Span<'_>> = ranges
        .into_iter()
        .map(|(s, e)| {
            if e <= s {
                trimmed_line.take_from(s).take(0)
            } else {
                trimmed_line.take_from(s).take(e - s)
            }
        })
        .collect();

    // Drop optional outer pipes (they create empty first/last cells).
    if has_leading_pipe {
        if let Some(first) = cells.first() {
            if first.fragment().is_empty() {
                cells.remove(0);
            }
        }
    }
    if has_trailing_pipe {
        if let Some(last) = cells.last() {
            if last.fragment().is_empty() {
                cells.pop();
            }
        }
    }

    cells.into_iter().map(trim_ws_span).collect::<Vec<_>>()
}

fn trim_ws_span(span: Span) -> Span {
    let s = span.fragment();

    let leading = s.bytes().take_while(|b| *b == b' ' || *b == b'\t').count();

    let trailing = s
        .bytes()
        .rev()
        .take_while(|b| *b == b' ' || *b == b'\t')
        .count();

    let len = s.len();
    let start = leading.min(len);
    let end = len.saturating_sub(trailing);

    if end <= start {
        span.take_from(start).take(0)
    } else {
        span.take_from(start).take(end - start)
    }
}

pub(crate) fn count_unescaped_pipes(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut count = 0usize;
    let mut backslash_run = 0usize;

    for &b in bytes {
        if b == b'|' {
            if backslash_run.is_multiple_of(2) {
                count += 1;
            }
            backslash_run = 0;
            continue;
        }

        if b == b'\\' {
            backslash_run += 1;
        } else {
            backslash_run = 0;
        }
    }

    count
}

pub(crate) fn is_valid_delimiter_cell(cell: &str) -> bool {
    // Trim spaces/tabs already applied by caller.
    let cell = cell.trim_matches([' ', '\t']);
    if cell.is_empty() {
        return false;
    }

    let left_colon = cell.starts_with(':');
    let right_colon = cell.ends_with(':');

    // Strip optional edge colons (alignment markers).
    let mut core = cell;
    if left_colon {
        core = &core[1..];
    }
    if right_colon {
        if core.is_empty() {
            return false;
        }
        core = &core[..core.len() - 1];
    }

    // Core must be 1+ hyphens only.
    if core.is_empty() {
        return false;
    }
    if !core.chars().all(|c| c == '-') {
        return false;
    }

    // Any additional colons are invalid.
    if core.contains(':') {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_split_cells_basic_with_outer_pipes() {
        let line = Span::new("| a | b |  c |");
        let cells = split_pipe_row_cells(line);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].fragment(), &"a");
        assert_eq!(cells[1].fragment(), &"b");
        assert_eq!(cells[2].fragment(), &"c");
    }

    #[test]
    fn smoke_test_split_cells_no_outer_pipes() {
        let line = Span::new("a|b|c");
        let cells = split_pipe_row_cells(line);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].fragment(), &"a");
        assert_eq!(cells[1].fragment(), &"b");
        assert_eq!(cells[2].fragment(), &"c");
    }

    #[test]
    fn smoke_test_split_cells_escaped_pipe_not_delimiter() {
        let line = Span::new("a \\| b | c");
        let cells = split_pipe_row_cells(line);
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].fragment(), &"a \\| b");
        assert_eq!(cells[1].fragment(), &"c");
    }

    #[test]
    fn smoke_test_delimiter_validation() {
        assert!(is_valid_delimiter_cell("---"));
        assert!(is_valid_delimiter_cell(":---"));
        assert!(is_valid_delimiter_cell("---:"));
        assert!(is_valid_delimiter_cell(":---:"));
        assert!(is_valid_delimiter_cell("-")); // GFM allows 1 dash

        assert!(!is_valid_delimiter_cell(""));
        assert!(!is_valid_delimiter_cell("--a--"));
        assert!(!is_valid_delimiter_cell(":-:-:"));
    }

    #[test]
    fn smoke_test_table_parses_header_delimiter_only() {
        let input = Span::new("| a | b |\n|---|---|\n");
        let (rest, table) = gfm_table(input).expect("should parse table");
        assert!(rest.fragment().is_empty());
        assert_eq!(split_pipe_row_cells(table.header_line).len(), 2);
        assert_eq!(split_pipe_row_cells(table.delimiter_line).len(), 2);
        assert!(table.body_lines.is_empty());
    }

    #[test]
    fn smoke_test_table_rejects_setext_heading() {
        let input = Span::new("Title\n---\n");
        assert!(gfm_table(input).is_err());
    }

    #[test]
    fn smoke_test_table_rejects_mismatched_cell_counts() {
        let input = Span::new("| a | b |\n|---|\n");
        assert!(gfm_table(input).is_err());
    }
}
