use core::parser::ast::NodeKind;

#[test]
fn test_emoji_shortcode_parses_mid_text_as_separate_text_node() {
    let md = "Hello :joy: world";
    let doc = core::parser::parse(md).expect("parse failed");

    let para = doc
        .children
        .iter()
        .find(|n| matches!(n.kind, NodeKind::Paragraph))
        .expect("expected a Paragraph");

    // We expect the shortcode to become a dedicated text node containing the emoji.
    let texts: Vec<String> = para
        .children
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect();

    assert!(
        texts.iter().any(|t| t == "😂"),
        "expected parsed emoji text node"
    );
    assert!(
        !texts.iter().any(|t| t.contains(":joy:")),
        "expected shortcode marker to be removed"
    );

    let combined = texts.concat();
    assert_eq!(combined, "Hello 😂 world");
}

#[test]
fn test_emoji_shortcode_renders_as_unicode_emoji() {
    let md = "Hi :rocket:!";
    let doc = core::parser::parse(md).expect("parse failed");
    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(html.contains("🚀"), "expected rocket emoji in HTML");
    assert!(
        !html.contains(":rocket:"),
        "expected shortcode marker to be removed"
    );
}

#[test]
fn test_unknown_shortcode_remains_literal_text() {
    let md = "Hello :unknown: world";
    let doc = core::parser::parse(md).expect("parse failed");
    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(html.contains(":unknown:"), "expected literal shortcode");
    assert!(!html.contains("😂"), "should not convert to emoji");
}

#[test]
fn test_shortcode_not_converted_inside_code_span() {
    let md = "`:joy:`";
    let doc = core::parser::parse(md).expect("parse failed");
    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(
        html.contains(":joy:"),
        "expected literal :joy: inside code span"
    );
    assert!(!html.contains("😂"), "should not convert inside code span");
}

#[test]
fn test_incomplete_shortcode_remains_literal() {
    let md = "Text :joy and more";
    let doc = core::parser::parse(md).expect("parse failed");
    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(html.contains("Text :joy and more"));
    assert!(!html.contains("😂"));
}
