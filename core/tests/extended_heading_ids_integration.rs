use core::parser::{parse, NodeKind};
use core::render::RenderOptions;

#[test]
fn integration_test_extended_heading_ids_render_id_attribute() {
    let input = "### Title {#custom-id}\n\n## Another title {#another-id}\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 2);

    match &doc.children[0].kind {
        NodeKind::Heading {
            level,
            text,
            id: Some(id),
        } => {
            assert_eq!(*level, 3);
            assert_eq!(text, "Title");
            assert_eq!(id, "custom-id");
        }
        other => panic!("expected Heading with id, got {other:?}"),
    }

    let options = RenderOptions::default();
    let html = core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<h3 id=\"custom-id\">Title"));
    assert!(html.contains("href=\"#custom-id\""));
    assert!(html.contains("<h2 id=\"another-id\">Another title"));
    assert!(html.contains("href=\"#another-id\""));
}

#[test]
fn integration_test_extended_heading_ids_invalid_syntax_is_left_in_text() {
    let input = "### Space before id { #bad }\n\n### Title {#id} trailing text\n";
    let doc = parse(input).expect("parse failed");

    assert_eq!(doc.children.len(), 2);

    match &doc.children[0].kind {
        NodeKind::Heading { text, id, .. } => {
            assert_eq!(text, "Space before id { #bad }");
            assert!(id.is_none());
        }
        other => panic!("expected Heading, got {other:?}"),
    }

    match &doc.children[1].kind {
        NodeKind::Heading { text, id, .. } => {
            assert_eq!(text, "Title {#id} trailing text");
            assert!(id.is_none());
        }
        other => panic!("expected Heading, got {other:?}"),
    }
}
