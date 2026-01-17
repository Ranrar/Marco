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
fn test_sliders_parse_to_ast() {
    let input = "@slidestart:t5\n# One\n---\nTwo\n--\nThree\n@slideend\n";

    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 1);
    let deck = &doc.children[0];
    assert!(matches!(
        deck.kind,
        NodeKind::SliderDeck {
            timer_seconds: Some(5)
        }
    ));

    assert_eq!(deck.children.len(), 3);
    assert!(matches!(
        deck.children[0].kind,
        NodeKind::Slide { vertical: false }
    ));
    assert!(matches!(
        deck.children[1].kind,
        NodeKind::Slide { vertical: false }
    ));
    assert!(matches!(
        deck.children[2].kind,
        NodeKind::Slide { vertical: true }
    ));

    // Slides should contain parsed blocks.
    assert!(deck.children[0]
        .children
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Heading { .. })));
    assert!(deck.children[1]
        .children
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Paragraph)));

    let total_decks = count_kind_in_document(&doc, |k| matches!(k, NodeKind::SliderDeck { .. }));
    assert_eq!(total_decks, 1);
}

#[test]
fn test_sliders_render_to_expected_html_skeleton() {
    let input = "@slidestart:t5\nOne\n---\nTwo\n---\nThree\n@slideend\n";

    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("class=\"marco-sliders\""));
    assert!(html.contains("data-timer-seconds=\"5\""));
    assert!(html.contains("class=\"marco-sliders__viewport\""));
    assert!(html.contains("class=\"marco-sliders__controls\""));
    assert!(html.contains("class=\"marco-sliders__dots\""));

    // Three slides and three dots.
    assert!(html.matches("class=\"marco-sliders__slide").count() >= 3);
    assert!(html.matches("class=\"marco-sliders__dot").count() >= 3);
}

#[test]
fn test_nested_sliders_do_not_create_nested_decks() {
    let input = "@slidestart\nOuter\n\n@slidestart:t1\nInner\n@slideend\n\n@slideend\n";

    let doc = parse(input).expect("parse failed");

    let total_decks = count_kind_in_document(&doc, |k| matches!(k, NodeKind::SliderDeck { .. }));
    assert_eq!(total_decks, 1);

    // The literal marker should still be present in the rendered HTML (treated as plain text).
    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");
    assert!(html.contains("@slidestart"));
}
