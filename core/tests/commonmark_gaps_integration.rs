use core::{parse, render, RenderOptions};

#[test]
fn entity_references_render_as_decoded_text() {
    let input = "&copy; &amp; &#169; &#xA9;";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p>© &amp; © ©</p>\n");
}

#[test]
fn reference_style_link_full_resolves_forward_definition() {
    let input = "[foo][bar]\n\n[bar]: https://example.com \"Title\"";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(
        html,
        "<p><a href=\"https://example.com\" title=\"Title\">foo</a></p>\n"
    );
}

#[test]
fn reference_style_link_shortcut_resolves() {
    let input = "[bar]\n\n[bar]: https://example.com";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p><a href=\"https://example.com\">bar</a></p>\n");
}

#[test]
fn reference_style_link_collapsed_resolves() {
    let input = "[bar][]\n\n[bar]: https://example.com";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p><a href=\"https://example.com\">bar</a></p>\n");
}

#[test]
fn reference_style_link_unresolved_renders_literally() {
    let input = "See [missing][ref].";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p>See [missing][ref].</p>\n");
}

#[test]
fn reference_style_link_duplicate_definitions_first_wins() {
    let input = "[foo]\n\n[foo]: https://first.example\n[foo]: https://second.example\n";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p><a href=\"https://first.example\">foo</a></p>\n");
}

#[test]
fn reference_style_link_unicode_casefold_sharp_s_resolves() {
    let input = "[ẞ]\n\n[SS]: /url\n";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p><a href=\"/url\">ẞ</a></p>\n");
}

#[test]
fn reference_style_link_escaped_bracket_label_resolves() {
    let input = "[foo][ref\\[]\n\n[ref\\[]: /uri\n";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p><a href=\"/uri\">foo</a></p>\n");
}

#[test]
fn reference_style_link_escaped_label_does_not_match_unescaped_definition() {
    let input = "[bar][foo\\!]\n\n[foo!]: /url\n";
    let doc = parse(input).expect("parse failed");

    let html = render(&doc, &RenderOptions::default()).expect("render failed");
    assert_eq!(html, "<p>[bar][foo!]</p>\n");
}
