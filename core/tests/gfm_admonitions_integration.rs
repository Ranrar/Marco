use core::parser::ast::NodeKind;
use core::parser::AdmonitionKind;

fn find_first_kind(
    node: &core::parser::Node,
    kind: fn(&NodeKind) -> bool,
) -> Option<&core::parser::Node> {
    if kind(&node.kind) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first_kind(child, kind) {
            return Some(found);
        }
    }
    None
}

#[test]
fn test_gfm_admonition_transforms_top_level_blockquote_and_strips_marker() {
    let md = "> [!NOTE]\n> Useful information that users should know.\n";
    let doc = core::parser::parse(md).expect("parse failed");

    assert!(matches!(
        doc.children.first().map(|n| &n.kind),
        Some(NodeKind::Admonition {
            kind: AdmonitionKind::Note
        })
    ));

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(html.contains("markdown-alert-note"));
    assert!(html.contains("admonition-note"));
    assert!(
        html.contains("markdown-alert-icon"),
        "icon span should be rendered"
    );
    assert!(html.contains("<svg"), "icon SVG should be in the HTML");
    assert!(
        !html.contains("\\\""),
        "icon SVG should not contain backslash-escaped quotes"
    );
    assert!(!html.contains("[!NOTE]"), "marker should be stripped");
}

#[test]
fn test_gfm_admonition_all_kinds_render_classes() {
    let cases = [
        ("NOTE", AdmonitionKind::Note, "note"),
        ("TIP", AdmonitionKind::Tip, "tip"),
        ("IMPORTANT", AdmonitionKind::Important, "important"),
        ("WARNING", AdmonitionKind::Warning, "warning"),
        ("CAUTION", AdmonitionKind::Caution, "caution"),
    ];

    for (marker, kind, slug) in cases {
        let md = format!("> [!{}]\n> body\n", marker);
        let doc = core::parser::parse(&md).expect("parse failed");

        assert!(matches!(
            doc.children.first().map(|n| &n.kind),
            Some(NodeKind::Admonition { kind: k }) if *k == kind
        ));

        let html = core::render::render(&doc, &core::render::RenderOptions::default())
            .expect("render failed");

        assert!(html.contains(&format!("markdown-alert-{}", slug)));
        assert!(html.contains(&format!("admonition-{}", slug)));
        assert!(!html.contains(&format!("[!{}]", marker)));
    }
}

#[test]
fn test_gfm_admonition_unknown_marker_is_not_transformed() {
    let md = "> [!FOO]\n> bar\n";
    let doc = core::parser::parse(md).expect("parse failed");

    assert!(matches!(
        doc.children.first().map(|n| &n.kind),
        Some(NodeKind::Blockquote)
    ));

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(html.contains("<blockquote>"));
    assert!(html.contains("[!FOO]"));
}

#[test]
fn test_gfm_admonition_is_not_transformed_when_nested() {
    // GitHub docs note alerts cannot be nested within other elements.
    // We enforce that by only transforming top-level blockquotes.
    let md = "- item\n  > [!NOTE]\n  > nested\n";
    let doc = core::parser::parse(md).expect("parse failed");

    // Ensure we still have a nested Blockquote somewhere.
    let list = doc
        .children
        .iter()
        .find(|n| matches!(n.kind, NodeKind::List { .. }))
        .expect("expected a List node");

    let nested = find_first_kind(list, |k| matches!(k, NodeKind::Blockquote));
    assert!(nested.is_some(), "expected nested blockquote to remain");
}
