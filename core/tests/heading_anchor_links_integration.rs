use core::parser::parse;
use core::render::RenderOptions;

#[test]
fn integration_test_heading_with_id_renders_anchor_link_with_svg() {
    let input = "## Title {#custom-id}\n";
    let doc = parse(input).expect("parse failed");

    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    // The heading text itself is now wrapped in the anchor (no trailing icon).
    assert!(html.contains("<h2 id=\"custom-id\">"));
    assert!(html.contains("class=\"marco-heading-anchor\""));
    assert!(html.contains("href=\"#custom-id\""));
    // The anchor wraps the text directly — no SVG icon.
    assert!(!html.contains("icon-tabler-anchor"));
}

#[test]
fn integration_test_heading_without_id_renders_auto_slug_anchor() {
    let input = "## Title\n";
    let doc = parse(input).expect("parse failed");

    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    // Headings without explicit {#id} now get an auto-generated slug for TOC navigation.
    assert!(html.contains("<h2 id=\"title\">"));
    assert!(html.contains("class=\"marco-heading-anchor\""));
    assert!(html.contains("href=\"#title\""));
}
