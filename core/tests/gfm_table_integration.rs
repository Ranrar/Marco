use core::parser::{parse, NodeKind};
use core::render::RenderOptions;

#[test]
fn test_gfm_table_parses_as_table_block() {
    let input = "| a | b |\n|---|---|\n| 1 | 2 |\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    let table = &doc.children[0];
    assert!(matches!(table.kind, NodeKind::Table { .. }));

    // Header + 1 body row
    assert_eq!(table.children.len(), 2);
    assert!(matches!(
        table.children[0].kind,
        NodeKind::TableRow { header: true }
    ));
    assert!(matches!(
        table.children[1].kind,
        NodeKind::TableRow { header: false }
    ));

    // 2 columns
    assert_eq!(table.children[0].children.len(), 2);
    assert_eq!(table.children[1].children.len(), 2);
}

#[test]
fn test_gfm_table_alignment_renders_to_html() {
    let input = "| a | b | c |\n|:--|--:|:--:|\n| 1 | 2 | 3 |\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<table>"));

    // Header cells: left, right, center
    assert!(html.contains("<th style=\"text-align: left;\">a</th>"));
    assert!(html.contains("<th style=\"text-align: right;\">b</th>"));
    assert!(html.contains("<th style=\"text-align: center;\">c</th>"));

    // Body cells should also carry the alignment.
    assert!(html.contains("<td style=\"text-align: left;\">1</td>"));
    assert!(html.contains("<td style=\"text-align: right;\">2</td>"));
    assert!(html.contains("<td style=\"text-align: center;\">3</td>"));
}

#[test]
fn test_gfm_table_row_padding_and_truncation() {
    let input = "| a | b |\n|---|---|\n| 1 |\n| 2 | 3 | 4 |\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    let table = &doc.children[0];

    // header + 2 body rows
    assert_eq!(table.children.len(), 3);

    // Each row should normalize to 2 cells.
    for row in &table.children {
        assert_eq!(row.children.len(), 2);
    }
}

#[test]
fn test_setext_heading_not_misparsed_as_table() {
    let input = "Title\n---\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    match &doc.children[0].kind {
        NodeKind::Heading { level, text } => {
            assert_eq!(*level, 2);
            assert_eq!(text, "Title");
        }
        other => panic!("expected Heading, got {other:?}"),
    }
}
