use core::lsp::{compute_highlights, HighlightTag};
use core::parser::{Document, Node, NodeKind, Position, Span};

#[test]
fn integration_test_autolink_highlighting_multi_byte_url() {
    // Link with multi-byte characters (example internationalized domain + emoji path)
    let multi_url = "https://例子.测试/路径/🎨".to_string();

    let doc = Document {
        children: vec![
            Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Link { url: multi_url.clone(), title: None },
                        span: Some(Span {
                            // Simulate the link appearing on line 1 starting at column 1
                            start: Position { line: 1, column: 1, offset: 0 },
                            // End column and offset are approximated; compute_highlights doesn't validate them
                            end: Position { line: 1, column: 30, offset: 29 },
                        }),
                        children: vec![],
                    }
                ],
            }
        ],
    ..Default::default()
    };

    let highlights = compute_highlights(&doc);

    // Expect a Link highlight present
    assert!(highlights.iter().any(|h| h.tag == HighlightTag::Link));

    // Validate that the reported highlight span for the link matches the node span we provided
    let link_hl = highlights.iter().find(|h| h.tag == HighlightTag::Link).expect("Expected Link highlight");
    assert_eq!(link_hl.span.start.line, 1);
    assert_eq!(link_hl.span.start.column, 1);
}
