use core::parser::parse;
use core::render::RenderOptions;

#[test]
fn integration_test_gfm_footnotes_basic_rendering() {
    let input = "Here is a footnote reference[^1].\n\n[^1]: Footnote definition.\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    // Reference becomes a numbered superscript link.
    assert!(html.contains("<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup>"));

    // Definitions should not render in place.
    assert!(!html.contains("[^1]:"));

    // Footnotes section is appended.
    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("<li id=\"fn1\">"));
    assert!(html.contains("Footnote definition."));
}

#[test]
fn integration_test_gfm_footnotes_missing_definition_falls_back_to_literal() {
    let input = "Missing def[^missing].\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    // No footnotes section, and the original marker remains.
    assert!(!html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("[^missing]"));
}

#[test]
fn integration_test_gfm_footnotes_multiline_definition() {
    let input =
        "A multi-line footnote[^multi].\n\n[^multi]: First line\n    second line\n    third line\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("First line"));
    assert!(html.contains("second line"));
    assert!(html.contains("third line"));
}
