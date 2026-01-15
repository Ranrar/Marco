use core::parser::ast::NodeKind;

#[test]
fn test_gfm_task_list_items_parse_with_task_checkbox_marker() {
    let md = "- [ ] todo\n- [x] done\n";
    let doc = core::parser::parse(md).expect("parse failed");

    // Ensure we got a list and the list items contain TaskCheckbox as first child.
    let list = doc
        .children
        .iter()
        .find(|n| matches!(n.kind, NodeKind::List { .. }))
        .expect("expected a List node");

    assert!(!list.children.is_empty(), "expected list items");
    for (idx, li) in list.children.iter().enumerate() {
        assert!(matches!(li.kind, NodeKind::ListItem));
        let first = li.children.first().expect("expected at least one child");
        assert!(
            matches!(first.kind, NodeKind::TaskCheckbox { .. }),
            "item {} missing TaskCheckbox",
            idx
        );
    }
}

#[test]
fn test_gfm_task_list_renders_svg_checkbox_icons() {
    let md = "- [ ] todo\n- [x] done\n";
    let doc = core::parser::parse(md).expect("parse failed");
    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(html.contains("marco-task-icon"), "expected svg icon class");
    assert!(
        html.contains("task-list-item-checkbox"),
        "expected checkbox span class"
    );
    assert!(
        !html.contains("[ ]"),
        "marker should be stripped from output"
    );
    assert!(
        !html.contains("[x]"),
        "marker should be stripped from output"
    );
}

#[test]
fn test_task_checkbox_marker_without_list_still_renders_svg() {
    // No leading '-', '+', '*': should not become a list, but should render SVG.
    let md = "[ ] plan\n\n[x] shipped\n";
    let doc = core::parser::parse(md).expect("parse failed");

    // Should be paragraphs, not a list.
    assert!(doc
        .children
        .iter()
        .all(|n| !matches!(n.kind, NodeKind::List { .. })));

    // Should contain inline checkbox nodes.
    let mut inline_count = 0usize;
    for p in &doc.children {
        if matches!(p.kind, NodeKind::Paragraph)
            && matches!(
                p.children.first().map(|c| &c.kind),
                Some(NodeKind::TaskCheckboxInline { .. })
            )
        {
            inline_count += 1;
        }
    }
    assert_eq!(inline_count, 2, "expected two inline task checkboxes");

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");
    assert!(html.contains("marco-task-icon"), "expected svg icon class");
    assert!(!html.contains("<ul>"), "should not render a list");
    assert!(
        !html.contains("[ ]"),
        "marker should be stripped from output"
    );
    assert!(
        !html.contains("[x]"),
        "marker should be stripped from output"
    );
}

#[test]
fn test_task_checkbox_marker_mid_paragraph_renders_svg_and_strips_marker() {
    let md = "This is [ ] an inline task, and this is [x].\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let paragraph = doc
        .children
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Paragraph))
        .expect("expected a Paragraph node");

    let inline_count = paragraph
        .children
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::TaskCheckboxInline { .. }))
        .count();
    assert_eq!(inline_count, 2, "expected two inline checkboxes");

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");
    assert!(html.contains("marco-task-icon"), "expected svg icon class");
    assert!(
        !html.contains("[ ]"),
        "marker should be stripped from output"
    );
    assert!(
        !html.contains("[x]"),
        "marker should be stripped from output"
    );
}

#[test]
fn test_inline_task_checkbox_does_not_break_link_syntax() {
    // This should parse as a link, not a checkbox.
    let md = "[x](https://example.com)\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let paragraph = doc
        .children
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Paragraph))
        .expect("expected a Paragraph node");

    assert!(paragraph
        .children
        .iter()
        .all(|n| !matches!(n.kind, NodeKind::TaskCheckboxInline { .. })));
    assert!(paragraph
        .children
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Link { .. })));
}

#[test]
fn test_task_checkbox_markers_after_hardbreak_lines_render_svg_for_each_line() {
    // When multiple lines are combined into a single paragraph via hard breaks
    // (two spaces + newline), each line-start marker should still render as an
    // inline checkbox.
    let md = "### Core Parser & LSP (Current Focus)\n\
[ ] Complete LSP integration with SourceView5 (syntax highlighting, diagnostics, completion, hover)  \n\
[ ] Enhanced AST validation and error reporting  \n\
[x] Advanced syntax features with linting support  \n\
[X] Optimize parser performance and caching\n";

    let doc = core::parser::parse(md).expect("parse failed");

    // Should not become a list.
    assert!(doc
        .children
        .iter()
        .all(|n| !matches!(n.kind, NodeKind::List { .. })));

    // Find the paragraph after the heading.
    let paragraph = doc
        .children
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Paragraph))
        .expect("expected a Paragraph node");

    let inline_count = paragraph
        .children
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::TaskCheckboxInline { .. }))
        .count();
    assert_eq!(inline_count, 4, "expected one inline checkbox per line");

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");
    assert!(html.contains("marco-task-icon"), "expected svg icon class");
    assert!(
        !html.contains("[ ]"),
        "marker should be stripped from output"
    );
    assert!(
        !html.contains("[x]"),
        "marker should be stripped from output"
    );
    assert!(
        !html.contains("[X]"),
        "marker should be stripped from output"
    );
}
