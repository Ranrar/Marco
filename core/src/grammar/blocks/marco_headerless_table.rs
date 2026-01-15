// Marco extension: "headerless" pipe tables
//
// Syntax (extension):
// - First line is a valid GFM delimiter row (pipes + '-' + optional ':' for alignment)
// - Followed by 1+ body rows (pipe rows)
// - No header row is produced; all rows are treated as body rows.
//
// This enables tables like:
// |--------|--------|
// | Data 1 | Data 2 |
// | Data 3 | Data 4 |
//
// Parsing is intentionally conservative:
// - Requires `|` on delimiter row so it cannot be confused with thematic breaks.
// - Requires indentation < 4 (avoid indented code blocks).
// - Requires at least one body row.

use crate::grammar::blocks::gfm_table::{
    count_unescaped_pipes, is_valid_delimiter_cell, split_pipe_row_cells,
};
use crate::grammar::shared::{count_indentation, Span};
use nom::character::complete::{line_ending, not_line_ending};
use nom::IResult;

#[derive(Debug, Clone, PartialEq)]
pub struct MarcoHeaderlessTableBlock<'a> {
    pub delimiter_line: Span<'a>,
    pub body_lines: Vec<Span<'a>>,
}

/// Parse a Marco "headerless" pipe table starting at the current position.
///
/// Returns the consumed table (delimiter+rows) as spans that reference the
/// original input.
pub fn marco_headerless_table(input: Span<'_>) -> IResult<Span<'_>, MarcoHeaderlessTableBlock<'_>> {
    // Table blocks can't start with 4+ spaces (would be indented code).
    if count_indentation(input.fragment()) >= 4 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Delimiter line (must be the first line)
    let (after_delimiter_line, delimiter_line) = not_line_ending(input)?;
    if delimiter_line.fragment().trim().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Require at least one unescaped '|' in the delimiter line.
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

    // Validate delimiter row cell syntax.
    let delimiter_cells = split_pipe_row_cells(delimiter_line);
    if delimiter_cells.is_empty()
        || !delimiter_cells
            .iter()
            .all(|cell| is_valid_delimiter_cell(cell.fragment()))
    {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Require a newline after the delimiter line (need at least one body row).
    let (mut remaining, _) = line_ending(after_delimiter_line)?;

    // First body line: must exist and must look like a pipe row (but not a delimiter row).
    let (after_first_body, first_body_line) = not_line_ending(remaining)?;

    if first_body_line.fragment().trim().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    if count_indentation(first_body_line.fragment()) >= 4 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    if count_unescaped_pipes(first_body_line.fragment()) == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Disambiguation: avoid treating two consecutive delimiter rows as a headerless table.
    // (This also avoids mis-parsing a hyphen-only header row).
    let first_body_cells = split_pipe_row_cells(first_body_line);
    let first_body_is_delimiter_row = !first_body_cells.is_empty()
        && first_body_cells
            .iter()
            .all(|cell| is_valid_delimiter_cell(cell.fragment()))
        && first_body_line.fragment().contains('-');

    if first_body_is_delimiter_row {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let mut body_lines: Vec<Span<'_>> = vec![first_body_line];

    // Consume newline after first body line if present.
    if let Ok((rest, _)) = line_ending::<Span, nom::error::Error<Span>>(after_first_body) {
        remaining = rest;
    } else {
        remaining = after_first_body;
        return Ok((
            remaining,
            MarcoHeaderlessTableBlock {
                delimiter_line,
                body_lines,
            },
        ));
    }

    // Additional body rows: consecutive non-blank lines containing at least one unescaped '|'.
    while !remaining.fragment().is_empty() {
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
        MarcoHeaderlessTableBlock {
            delimiter_line,
            body_lines,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_headerless_table_parses_basic() {
        let input = Span::new("|--------|--------|--------|\n| Data 1 | Data 2 | Data 3 |\n| Data 4 | Data 5 | Data 6 |\n");
        let (rest, table) = marco_headerless_table(input).expect("should parse headerless table");
        assert!(rest.fragment().is_empty());
        assert_eq!(split_pipe_row_cells(table.delimiter_line).len(), 3);
        assert_eq!(table.body_lines.len(), 2);
        assert_eq!(split_pipe_row_cells(table.body_lines[0]).len(), 3);
    }

    #[test]
    fn smoke_test_headerless_table_rejects_missing_body_row() {
        let input = Span::new("|---|---|\n");
        assert!(marco_headerless_table(input).is_err());
    }

    #[test]
    fn smoke_test_headerless_table_rejects_regular_gfm_table() {
        // Regular table should be handled by the GFM parser, not this extension.
        let input = Span::new("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(marco_headerless_table(input).is_err());
    }
}
