use core::parser::{parse, Document, Node, NodeKind};
use core::render::RenderOptions;

fn count_kind(node: &Node, kind: fn(&NodeKind) -> bool) -> usize {
    let mut count = if kind(&node.kind) { 1 } else { 0 };
    for child in &node.children {
        count += count_kind(child, kind);
    }
    count
}

fn count_kind_in_document(document: &Document, kind: fn(&NodeKind) -> bool) -> usize {
    document.children.iter().map(|n| count_kind(n, kind)).sum()
}

#[test]
fn test_tab_blocks_parse_to_ast() {
    let input = ":::tab\n@tab First\nHello **world**\n\n- a\n- b\n\n@tab Second\n# Heading\n\n```\n@tab Not a tab\n:::\n```\n\n:::";

    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    let group = &doc.children[0];
    assert!(matches!(group.kind, NodeKind::TabGroup));

    assert_eq!(group.children.len(), 2);
    assert!(matches!(
        group.children[0].kind,
        NodeKind::TabItem { ref title } if title == "First"
    ));
    assert!(matches!(
        group.children[1].kind,
        NodeKind::TabItem { ref title } if title == "Second"
    ));

    // Panels should contain parsed blocks.
    assert!(group.children[0]
        .children
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Paragraph)));
    assert!(group.children[1]
        .children
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Heading { .. })));

    // Exactly one tab group in the whole tree.
    let total_groups = count_kind_in_document(&doc, |k| matches!(k, NodeKind::TabGroup));
    assert_eq!(total_groups, 1);
}

#[test]
fn test_tab_blocks_render_to_expected_html_skeleton() {
    let input = ":::tab\n@tab A\nHello\n\n@tab B\nWorld\n\n:::";

    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<div class=\"marco-tabs\">"));
    assert!(html.contains("class=\"marco-tabs__tablist\""));
    assert!(html.contains("class=\"marco-tabs__panels\""));

    // Two radios and two labels/panels.
    assert!(html.matches("class=\"marco-tabs__radio\"").count() >= 2);
    assert!(html.matches("class=\"marco-tabs__tab\"").count() >= 2);
    assert!(html.matches("class=\"marco-tabs__panel\"").count() >= 2);
}

#[test]
fn test_nested_tab_blocks_do_not_create_nested_groups() {
    let input = ":::tab\n@tab Outer\n\n:::tab\n@tab Inner\nInner\n\n:::\n\n:::";

    let doc = parse(input).expect("parse failed");

    let total_groups = count_kind_in_document(&doc, |k| matches!(k, NodeKind::TabGroup));
    assert_eq!(total_groups, 1);

    // The literal marker should still be present in the rendered HTML (treated as plain text).
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");
    assert!(html.contains(":::tab"));
}
