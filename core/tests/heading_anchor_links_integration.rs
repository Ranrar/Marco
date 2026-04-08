use core::parser::parse;
use core::render::RenderOptions;

#[test]
fn integration_test_heading_with_id_renders_anchor_link_with_svg() {
    let input = "## Title {#custom-id}\n";
    let doc = parse(input).expect("parse failed");

    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<h2 id=\"custom-id\">"));
    assert!(html.contains("class=\"marco-heading-anchor\""));
    assert!(html.contains("href=\"#custom-id\""));
    assert!(html.contains("icon-tabler-anchor"));
    assert!(html.contains("stroke-width=\"2.0\""));
    assert!(html.contains("width=\"1em\""));
    assert!(html.contains("height=\"1em\""));

    // Ensure SVG markup is emitted as valid HTML attributes (no stray backslashes).
    assert!(html.contains(concat!("xmlns=\"", "http", "://www.w3.org/2000/svg\"")));
    assert!(!html.contains("xmlns=\\\""));
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
