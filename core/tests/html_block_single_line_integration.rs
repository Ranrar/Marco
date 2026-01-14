#[test]
fn integration_test_single_line_html_block_does_not_swallow_following_markdown() {
    // Regression for editor UX: a single-line <div>...</div> should not turn the
    // rest of the document into a raw HTML block.
    let md = "<div>html</div>\n`www.example.com`\nwww.example.com inside code: `www.example.com`\n";

    let doc = core::parser::parse(md).expect("parse failed");

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    assert!(
        html.contains("<div>html</div>"),
        "first line should be preserved as raw HTML"
    );

    // Code spans must be parsed as Markdown and rendered as <code>...
    // If the HTML block swallows the rest of the document, these would remain
    // backticked literals inside the raw HTML string.
    assert!(
        html.contains("<code>www.example.com</code>"),
        "code span should render as <code>, meaning it was parsed outside the HtmlBlock"
    );
    assert!(
        !html.contains("`www.example.com`"),
        "backticks should not survive into final HTML when parsed as code spans"
    );
}
