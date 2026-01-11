//! GFM table parser - converts grammar output to AST nodes
//!
//! Converts `grammar::blocks::gfm_table::GfmTableBlock` into the structured table
//! AST representation:
//! - `NodeKind::Table { alignments }`
//! - `NodeKind::TableRow { header }`
//! - `NodeKind::TableCell { header, alignment }`
//!
//! Cell contents are parsed with the inline parser so emphasis/links/etc work
//! inside table cells.

use super::shared::{to_parser_span, to_parser_span_range, GrammarSpan};
use crate::grammar::blocks::gfm_table::{split_pipe_row_cells, GfmTableBlock};
use crate::parser::ast::{Node, NodeKind, TableAlignment};
use nom::Input;

/// Parse a GFM table block into an AST node.
///
/// `full_start..full_end` should cover the entire matched table construct (as
/// returned by the block-level grammar function) so spans/highlighting can
/// reference the full table region.
pub fn parse_gfm_table<'a>(
    table: GfmTableBlock<'a>,
    full_start: GrammarSpan<'a>,
    full_end: GrammarSpan<'a>,
) -> Node {
    let span = to_parser_span_range(full_start, full_end);

    let header_cells = split_pipe_row_cells(table.header_line);
    let delimiter_cells = split_pipe_row_cells(table.delimiter_line);

    // Grammar guarantees: non-empty and same length.
    let alignments: Vec<TableAlignment> = delimiter_cells
        .iter()
        .map(|cell| parse_alignment(cell.fragment()))
        .collect();

    let column_count = alignments.len();

    let mut rows: Vec<Node> = Vec::new();

    // Header row
    rows.push(parse_table_row(
        true,
        table.header_line,
        header_cells,
        &alignments,
        column_count,
    ));

    // Body rows
    for body_line in table.body_lines {
        let body_cells = split_pipe_row_cells(body_line);
        rows.push(parse_table_row(
            false,
            body_line,
            body_cells,
            &alignments,
            column_count,
        ));
    }

    Node {
        kind: NodeKind::Table { alignments },
        span: Some(span),
        children: rows,
    }
}

fn parse_table_row<'a>(
    header: bool,
    row_line: GrammarSpan<'a>,
    mut cells: Vec<GrammarSpan<'a>>,
    alignments: &[TableAlignment],
    column_count: usize,
) -> Node {
    let row_span = to_parser_span(row_line);

    normalize_cells_to_column_count(&mut cells, row_line, column_count);

    let mut children: Vec<Node> = Vec::with_capacity(column_count);
    for (col_idx, cell_span) in cells.into_iter().enumerate().take(column_count) {
        let alignment = alignments
            .get(col_idx)
            .copied()
            .unwrap_or(TableAlignment::None);
        children.push(parse_table_cell(header, alignment, cell_span));
    }

    Node {
        kind: NodeKind::TableRow { header },
        span: Some(row_span),
        children,
    }
}

fn parse_table_cell<'a>(
    header: bool,
    alignment: TableAlignment,
    cell_span: GrammarSpan<'a>,
) -> Node {
    let span = to_parser_span(cell_span);

    let inline_children = match crate::parser::inlines::parse_inlines_from_span(cell_span) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse inline elements in table cell: {}", e);
            vec![Node {
                kind: NodeKind::Text(cell_span.fragment().to_string()),
                span: Some(span),
                children: Vec::new(),
            }]
        }
    };

    Node {
        kind: NodeKind::TableCell { header, alignment },
        span: Some(span),
        children: inline_children,
    }
}

fn normalize_cells_to_column_count<'a>(
    cells: &mut Vec<GrammarSpan<'a>>,
    row_line: GrammarSpan<'a>,
    column_count: usize,
) {
    if cells.len() > column_count {
        cells.truncate(column_count);
    }

    while cells.len() < column_count {
        cells.push(empty_span_at_end_of_line(row_line));
    }
}

fn empty_span_at_end_of_line<'a>(line: GrammarSpan<'a>) -> GrammarSpan<'a> {
    let len = line.fragment().len();
    line.take_from(len).take(0)
}

fn parse_alignment(cell: &str) -> TableAlignment {
    let cell = cell.trim_matches([' ', '\t']);
    let left = cell.starts_with(':');
    let right = cell.ends_with(':');

    match (left, right) {
        (true, true) => TableAlignment::Center,
        (true, false) => TableAlignment::Left,
        (false, true) => TableAlignment::Right,
        (false, false) => TableAlignment::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::blocks as grammar;

    #[test]
    fn smoke_test_parse_gfm_table_builds_ast_structure() {
        let input = GrammarSpan::new("| a | b |\n|---|:--:|\n| 1 | 2 |\n");
        let start = input;
        let (rest, table) = grammar::gfm_table(input).expect("should parse table");

        let node = parse_gfm_table(table, start, rest);

        assert!(matches!(node.kind, NodeKind::Table { .. }));
        assert_eq!(node.children.len(), 2); // header + 1 body row

        assert!(matches!(
            node.children[0].kind,
            NodeKind::TableRow { header: true }
        ));
        assert!(matches!(
            node.children[1].kind,
            NodeKind::TableRow { header: false }
        ));

        assert_eq!(node.children[0].children.len(), 2);
        assert_eq!(node.children[1].children.len(), 2);

        // Alignment is propagated into cells.
        let cell0 = &node.children[0].children[0];
        let cell1 = &node.children[0].children[1];
        assert!(matches!(
            cell0.kind,
            NodeKind::TableCell {
                alignment: TableAlignment::None,
                header: true
            }
        ));
        assert!(matches!(
            cell1.kind,
            NodeKind::TableCell {
                alignment: TableAlignment::Center,
                header: true
            }
        ));
    }

    #[test]
    fn smoke_test_row_padding_and_truncation() {
        let input = GrammarSpan::new("| a | b |\n|---|---|\n| 1 |\n| 2 | 3 | 4 |\n");
        let start = input;
        let (rest, table) = grammar::gfm_table(input).expect("should parse table");

        let node = parse_gfm_table(table, start, rest);

        // header + 2 body rows
        assert_eq!(node.children.len(), 3);

        // Each row should have exactly 2 cells.
        for row in &node.children {
            assert_eq!(row.children.len(), 2);
        }
    }
}
