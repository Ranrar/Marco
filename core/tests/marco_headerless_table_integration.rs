use core::parser::{parse, NodeKind};
use core::render::RenderOptions;

#[test]
fn test_marco_headerless_table_parses_as_table_block_without_thead() {
    let input = "|--------|--------|--------|\n| Data 1 | Data 2 | Data 3 |\n| Data 4 | Data 5 | Data 6 |\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    let table = &doc.children[0];
    assert!(matches!(table.kind, NodeKind::Table { .. }));

    // 2 body rows only (no header row)
    assert_eq!(table.children.len(), 2);
    assert!(matches!(
        table.children[0].kind,
        NodeKind::TableRow { header: false }
    ));

    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<table>"));
    assert!(html.contains("<tbody>"));
    assert!(!html.contains("<thead>"));

    // First row cells should render as <td>
    assert!(html.contains("<td>Data 1</td>"));
    assert!(html.contains("<td>Data 2</td>"));
    assert!(html.contains("<td>Data 3</td>"));
}

#[test]
fn test_headerless_table_does_not_break_regular_gfm_table() {
    let input = "| a | b |\n|---|---|\n| 1 | 2 |\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    let table = &doc.children[0];

    // Regular GFM table still has a header row.
    assert!(matches!(table.kind, NodeKind::Table { .. }));
    assert_eq!(table.children.len(), 2);
    assert!(matches!(
        table.children[0].kind,
        NodeKind::TableRow { header: true }
    ));
}
