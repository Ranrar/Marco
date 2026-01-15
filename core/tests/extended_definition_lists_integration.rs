use core::parser::{parse, NodeKind};
use core::render::RenderOptions;

#[test]
fn test_extended_definition_lists_parse_basic_structure() {
    let input = "Term 1\n: Definition for term 1\n\nTerm 2\n: Definition for term 2\n: Alternative definition for term 2\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    let dl = &doc.children[0];
    assert!(matches!(dl.kind, NodeKind::DefinitionList));

    // dt, dd, dt, dd, dd
    assert_eq!(dl.children.len(), 5);
    assert!(matches!(dl.children[0].kind, NodeKind::DefinitionTerm));
    assert!(matches!(
        dl.children[1].kind,
        NodeKind::DefinitionDescription
    ));
    assert!(matches!(dl.children[2].kind, NodeKind::DefinitionTerm));
    assert!(matches!(
        dl.children[3].kind,
        NodeKind::DefinitionDescription
    ));
    assert!(matches!(
        dl.children[4].kind,
        NodeKind::DefinitionDescription
    ));
}

#[test]
fn test_extended_definition_lists_render_to_html_dl_dt_dd() {
    let input = "First Term\n: This is the definition of the first term.\n\nSecond Term\n: This is one definition of the second term.\n: This is another definition of the second term.\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<dl>"));
    assert!(html.contains("<dt>First Term</dt>"));
    assert!(html.contains("<dt>Second Term</dt>"));
    assert!(html.contains("<dd>"));
    assert!(html.contains("</dl>"));
}

#[test]
fn test_extended_definition_lists_support_nested_blocks_in_definitions() {
    let input = "Term\n: A list inside a definition\n  - item 1\n  - item 2\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<dl>"));
    assert!(html.contains("<dt>Term</dt>"));
    // Should render a list inside the <dd>.
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>"));
}

#[test]
fn test_extended_definition_lists_do_not_match_lookalikes() {
    // Missing term: should be a paragraph, not a definition list.
    let input = ": Definition without a term\n\nTerm 5\n:: Double colon (should not be a definition list)\n";
    let doc = parse(input).expect("parse failed");

    // Ensure we didn't create a definition list block anywhere.
    let any_dl = doc
        .children
        .iter()
        .any(|n| matches!(n.kind, NodeKind::DefinitionList));
    assert!(!any_dl);
}
