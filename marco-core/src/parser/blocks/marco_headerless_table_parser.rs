//! Marco headerless table parser - converts grammar output to AST nodes
//!
//! Converts `grammar::blocks::marco_headerless_table::MarcoHeaderlessTableBlock` into the
//! structured table AST representation:
//! - `NodeKind::Table { alignments }`
//! - `NodeKind::TableRow { header: false }`
//! - `NodeKind::TableCell { header: false, alignment }`

use super::gfm_table_parser::{parse_alignment, parse_table_row};
use super::shared::GrammarSpan;
use crate::grammar::blocks::gfm_table::split_pipe_row_cells;
use crate::grammar::blocks::marco_headerless_table::MarcoHeaderlessTableBlock;
use crate::parser::ast::{Node, NodeKind, TableAlignment};

/// Parse a Marco headerless table block into an AST node.
///
/// `full_start..full_end` should cover the entire matched table construct (as
/// returned by the block-level grammar function) so spans/highlighting can
/// reference the full table region.
pub fn parse_marco_headerless_table<'a>(
    table: MarcoHeaderlessTableBlock<'a>,
    full_start: GrammarSpan<'a>,
    full_end: GrammarSpan<'a>,
) -> Node {
    let span = crate::parser::shared::to_parser_span_range(full_start, full_end);

    let delimiter_cells = split_pipe_row_cells(table.delimiter_line);

    let alignments: Vec<TableAlignment> = delimiter_cells
        .iter()
        .map(|cell| parse_alignment(cell.fragment()))
        .collect();

    let column_count = alignments.len();

    let mut rows: Vec<Node> = Vec::new();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::blocks as grammar;

    #[test]
    fn smoke_test_parse_headerless_table_builds_ast_structure() {
        let input = GrammarSpan::new("|:--|--:|:--:|\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |\n");
        let start = input;
        let (rest, table) =
            grammar::marco_headerless_table(input).expect("should parse headerless table");

        let node = parse_marco_headerless_table(table, start, rest);

        assert!(matches!(node.kind, NodeKind::Table { .. }));
        assert_eq!(node.children.len(), 2); // 2 body rows

        // No header rows.
        for row in &node.children {
            assert!(matches!(row.kind, NodeKind::TableRow { header: false }));
        }

        // Alignment is propagated into cells.
        let row0 = &node.children[0];
        assert_eq!(row0.children.len(), 3);

        let c0 = &row0.children[0];
        let c1 = &row0.children[1];
        let c2 = &row0.children[2];

        assert!(matches!(
            c0.kind,
            NodeKind::TableCell {
                header: false,
                alignment: TableAlignment::Left
            }
        ));
        assert!(matches!(
            c1.kind,
            NodeKind::TableCell {
                header: false,
                alignment: TableAlignment::Right
            }
        ));
        assert!(matches!(
            c2.kind,
            NodeKind::TableCell {
                header: false,
                alignment: TableAlignment::Center
            }
        ));
    }
}
