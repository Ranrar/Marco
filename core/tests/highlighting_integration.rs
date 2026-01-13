use core::lsp::{compute_highlights, HighlightTag};
use core::parser::{Document, Node, NodeKind, Position, Span};

#[test]
fn integration_test_compute_highlights_multi_byte() {
    // Build a document with several nodes including multi-byte content
    let doc = Document {
        children: vec![
            Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "Title 🎨".to_string(),
                },
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 1,
                        column: 12,
                        offset: 11,
                    },
                }),
                children: vec![],
            },
            Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Emphasis,
                        span: Some(Span {
                            start: Position {
                                line: 2,
                                column: 1,
                                offset: 12,
                            },
                            end: Position {
                                line: 2,
                                column: 7,
                                offset: 18,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Strong,
                        span: Some(Span {
                            start: Position {
                                line: 2,
                                column: 9,
                                offset: 20,
                            },
                            end: Position {
                                line: 2,
                                column: 15,
                                offset: 26,
                            },
                        }),
                        children: vec![],
                    },
                ],
            },
        ],
        ..Default::default()
    };

    let highlights = compute_highlights(&doc);

    // Expect at least heading, emphasis and strong highlights
    assert!(highlights.iter().any(|h| h.tag == HighlightTag::Heading1));
    assert!(highlights.iter().any(|h| h.tag == HighlightTag::Emphasis));
    assert!(highlights.iter().any(|h| h.tag == HighlightTag::Strong));

    // Ensure heading highlight was expanded to line start (column == 1)
    if let Some(h) = highlights.iter().find(|h| h.tag == HighlightTag::Heading1) {
        assert_eq!(h.span.start.column, 1);
        assert_eq!(h.span.start.line, 1);
    } else {
        panic!("Missing Heading1 highlight");
    }
}
