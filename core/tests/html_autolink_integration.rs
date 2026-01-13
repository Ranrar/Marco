use core::lsp::{compute_highlights, HighlightTag};
/// Integration test to verify HTML tags are not parsed as autolinks.
/// This test validates the fix for the issue where HTML tags like <img> and <span>
/// were incorrectly being parsed as autolinks/links instead of inline HTML.
use core::parser::{parse, NodeKind};

#[test]
fn test_html_img_not_autolink() {
    let input = r#"Text with <img src="test.png" alt="image" /> inline."#;
    let doc = parse(input).expect("Parse failed");

    // Should have a paragraph with Text and InlineHtml nodes
    assert_eq!(doc.children.len(), 1);
    let para = &doc.children[0];
    assert!(matches!(para.kind, NodeKind::Paragraph));

    // Check inline content
    if let NodeKind::Paragraph = &para.kind {
        let inlines = &para.children;

        // Should have: Text, InlineHtml, Text
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[0].kind, NodeKind::Text(_)));
        assert!(matches!(inlines[1].kind, NodeKind::InlineHtml(_)));
        assert!(matches!(inlines[2].kind, NodeKind::Text(_)));

        // Verify it's NOT a Link node
        for inline in inlines {
            assert!(!matches!(inline.kind, NodeKind::Link { .. }));
        }
    }
}

#[test]
fn test_html_span_not_autolink() {
    let input = "Text with <span>content</span> inline.";
    let doc = parse(input).expect("Parse failed");

    let para = &doc.children[0];
    if let NodeKind::Paragraph = &para.kind {
        let inlines = &para.children;

        // Should have: Text, InlineHtml (<span>), Text (content), InlineHtml (</span>), Text
        assert!(inlines.len() >= 3);

        // Verify no Link nodes
        for inline in inlines {
            assert!(!matches!(inline.kind, NodeKind::Link { .. }));
        }

        // Verify InlineHtml nodes exist
        let has_inline_html = inlines
            .iter()
            .any(|n| matches!(n.kind, NodeKind::InlineHtml(_)));
        assert!(has_inline_html, "Should have InlineHtml nodes");
    }
}

#[test]
fn test_valid_autolink_url() {
    let input = "Visit <http://example.com> for info.";
    let doc = parse(input).expect("Parse failed");

    let para = &doc.children[0];
    if let NodeKind::Paragraph = &para.kind {
        let inlines = &para.children;

        // Should have: Text, Link, Text
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[0].kind, NodeKind::Text(_)));
        assert!(matches!(inlines[1].kind, NodeKind::Link { .. }));
        assert!(matches!(inlines[2].kind, NodeKind::Text(_)));

        // Verify it's a Link with correct URL
        if let NodeKind::Link { url, .. } = &inlines[1].kind {
            assert_eq!(url, "http://example.com");
        }
    }
}

#[test]
fn test_valid_autolink_email() {
    let input = "Contact <user@example.com> today.";
    let doc = parse(input).expect("Parse failed");

    let para = &doc.children[0];
    if let NodeKind::Paragraph = &para.kind {
        let inlines = &para.children;

        // Should have: Text, Link, Text
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[1].kind, NodeKind::Link { .. }));

        // Verify it's a Link with mailto: URL
        if let NodeKind::Link { url, .. } = &inlines[1].kind {
            assert_eq!(url, "mailto:user@example.com");
        }
    }
}

#[test]
fn test_html_div_not_autolink() {
    let input = "<div>Content</div>";
    let doc = parse(input).expect("Parse failed");

    let para = &doc.children[0];
    if let NodeKind::Paragraph = &para.kind {
        let inlines = &para.children;

        // Verify no Link nodes
        for inline in inlines {
            assert!(!matches!(inline.kind, NodeKind::Link { .. }));
        }
    }
}

#[test]
fn test_invalid_autolink_no_colon() {
    let input = "Not an autolink: <notaurl>";
    let doc = parse(input).expect("Parse failed");

    let para = &doc.children[0];
    if let NodeKind::Paragraph = &para.kind {
        let inlines = &para.children;

        // Should not have any Link nodes
        for inline in inlines {
            assert!(!matches!(inline.kind, NodeKind::Link { .. }));
        }
    }
}

#[test]
fn test_highlighting_html_vs_autolink() {
    // HTML content INLINE (within text) to get InlineHtml highlight
    let html_input = r#"Text with <img src="test.png" /> inline."#;
    let html_doc = parse(html_input).expect("Parse failed");
    let html_highlights = compute_highlights(&html_doc);

    // Should have InlineHtml highlight, NOT Link highlight
    let has_html_highlight = html_highlights
        .iter()
        .any(|h| matches!(h.tag, HighlightTag::InlineHtml));
    let has_link_highlight = html_highlights
        .iter()
        .any(|h| matches!(h.tag, HighlightTag::Link));

    assert!(
        has_html_highlight,
        "HTML tag should have InlineHtml highlight"
    );
    assert!(
        !has_link_highlight,
        "HTML tag should NOT have Link highlight"
    );

    // Autolink content
    let link_input = "<http://example.com>";
    let link_doc = parse(link_input).expect("Parse failed");
    let link_highlights = compute_highlights(&link_doc);

    // Should have Link highlight, NOT InlineHtml highlight
    let has_html_highlight = link_highlights
        .iter()
        .any(|h| matches!(h.tag, HighlightTag::InlineHtml));
    let has_link_highlight = link_highlights
        .iter()
        .any(|h| matches!(h.tag, HighlightTag::Link));

    assert!(
        !has_html_highlight,
        "Autolink should NOT have InlineHtml highlight"
    );
    assert!(has_link_highlight, "Autolink should have Link highlight");
}

#[test]
fn test_mixed_html_and_autolinks() {
    let input = r#"Text <img src="icon.png" /> and link <https://example.com> end."#;
    let doc = parse(input).expect("Parse failed");

    let para = &doc.children[0];
    if let NodeKind::Paragraph = &para.kind {
        let inlines = &para.children;

        // Should have both InlineHtml and Link nodes
        let has_inline_html = inlines
            .iter()
            .any(|n| matches!(n.kind, NodeKind::InlineHtml(_)));
        let has_link = inlines
            .iter()
            .any(|n| matches!(n.kind, NodeKind::Link { .. }));

        assert!(has_inline_html, "Should have InlineHtml node for img tag");
        assert!(has_link, "Should have Link node for URL");
    }
}
