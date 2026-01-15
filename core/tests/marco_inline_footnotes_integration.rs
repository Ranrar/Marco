use core::parser::parse;
use core::render::RenderOptions;

#[test]
fn integration_test_inline_footnote_basic_rendering() {
    let input = "Inline note.^[This is it]\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    // Reference becomes a numbered superscript link.
    assert!(html.contains("<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup>"));

    // Marker is not preserved.
    assert!(!html.contains("^[This is it]"));

    // Footnotes section includes the content.
    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("This is it"));
}

#[test]
fn integration_test_inline_footnote_supports_inline_markup() {
    let input = "Complex^[*italic* and **bold** and `code`]\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<code>code</code>"));
}

#[test]
fn integration_test_inline_footnote_not_parsed_inside_code_span() {
    let input = "`^[not a footnote]`\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(!html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("<code>^[not a footnote]</code>"));
}

#[test]
fn integration_test_inline_footnote_does_not_break_superscript() {
    let input = "^hi^ and a note^[x]\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    // Superscript is rendered normally.
    assert!(html.contains("<sup>hi</sup>"));

    // Footnote still works.
    assert!(html.contains("<section class=\"footnotes\">"));
}
