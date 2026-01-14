use core::parser::ast::NodeKind;

fn collect_links<'a>(node: &'a core::parser::Node, out: &mut Vec<&'a core::parser::Node>) {
    if matches!(node.kind, NodeKind::Link { .. }) {
        out.push(node);
    }
    for child in &node.children {
        collect_links(child, out);
    }
}

fn links_in_doc(doc: &core::parser::Document) -> Vec<&core::parser::Node> {
    let mut out = Vec::new();
    for node in &doc.children {
        collect_links(node, &mut out);
    }
    out
}

fn http_prefixed(url_without_scheme: &str) -> String {
    format!("{}://{}", "http", url_without_scheme)
}

#[test]
fn test_gfm_www_autolink_basic() {
    let md = "www.commonmark.org\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let links = links_in_doc(&doc);
    assert_eq!(links.len(), 1);

    match &links[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, &http_prefixed("www.commonmark.org")),
        other => panic!("expected Link, got {other:?}"),
    }
}

#[test]
fn test_gfm_www_autolink_with_path_and_trailing_period() {
    let md = "Visit www.commonmark.org/help for more information.\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let links = links_in_doc(&doc);
    assert_eq!(links.len(), 1);

    match &links[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, &http_prefixed("www.commonmark.org/help")),
        other => panic!("expected Link, got {other:?}"),
    }

    let input2 = "Visit www.commonmark.org.\n";
    let doc2 = core::parser::parse(input2).expect("parse failed");
    let links2 = links_in_doc(&doc2);
    assert_eq!(links2.len(), 1);

    match &links2[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, &http_prefixed("www.commonmark.org")),
        other => panic!("expected Link, got {other:?}"),
    }
}

#[test]
fn test_gfm_autolink_parentheses_balancing() {
    let md = "www.google.com/search?q=Markup+(business)))\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let links = links_in_doc(&doc);
    assert_eq!(links.len(), 1);

    let link = links[0];
    let label = match link.children.first().map(|n| &n.kind) {
        Some(NodeKind::Text(t)) => t.as_str(),
        other => panic!("expected link label text, got {other:?}"),
    };

    assert_eq!(
        label, "www.google.com/search?q=Markup+(business)",
        "unmatched trailing ')' must be excluded"
    );
}

#[test]
fn test_gfm_autolink_entity_suffix_trimmed() {
    let md = "www.google.com/search?q=commonmark&hl;\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");

    let expected_href = format!("href=\"{}://www.google.com/search?q=commonmark\"", "http");

    assert!(
        html.contains(&expected_href),
        "entity-like suffix must be excluded from the href"
    );
    assert!(
        html.contains("&amp;hl;"),
        "entity-like suffix should remain as literal text"
    );
}

#[test]
fn test_gfm_email_autolink_and_plus_rules() {
    let md = "hello@mail+xyz.example isn't valid, but hello+xyz@mail.example is.\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let links = links_in_doc(&doc);
    assert_eq!(links.len(), 1, "only the valid email should linkify");

    match &links[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, "mailto:hello+xyz@mail.example"),
        other => panic!("expected Link, got {other:?}"),
    }
}

#[test]
fn test_gfm_protocol_mailto_trims_trailing_dot_and_stops_before_slash() {
    // Trailing '.' should not be part of the email autolink.
    let md = "mailto:a.b-c_d@a.b.\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let links = links_in_doc(&doc);
    assert_eq!(links.len(), 1);

    let link = links[0];
    match &link.kind {
        NodeKind::Link { url, .. } => assert_eq!(url, "mailto:a.b-c_d@a.b"),
        other => panic!("expected Link, got {other:?}"),
    }

    // Slash is not part of the email; link ends before '/'.
    let input2 = "mailto:a.b-c_d@a.b/\n";
    let doc2 = core::parser::parse(input2).expect("parse failed");
    let links2 = links_in_doc(&doc2);
    assert_eq!(links2.len(), 1);

    match &links2[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, "mailto:a.b-c_d@a.b"),
        other => panic!("expected Link, got {other:?}"),
    }

    // Invalid endings '-' and '_' should not linkify.
    let input3 = "mailto:a.b-c_d@a.b-\n";
    let doc3 = core::parser::parse(input3).expect("parse failed");
    assert!(links_in_doc(&doc3).is_empty());

    let input4 = "mailto:a.b-c_d@a.b_\n";
    let doc4 = core::parser::parse(input4).expect("parse failed");
    let links4 = links_in_doc(&doc4);
    assert!(
        links4.is_empty(),
        "expected no links, but got: {:?}",
        links4.iter().map(|n| &n.kind).collect::<Vec<_>>()
    );
}

#[test]
fn test_gfm_protocol_xmpp_resource_and_second_slash() {
    let md = "xmpp:foo@bar.baz/txt@bin.com\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let links = links_in_doc(&doc);
    assert_eq!(links.len(), 1);

    match &links[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, "xmpp:foo@bar.baz/txt@bin.com"),
        other => panic!("expected Link, got {other:?}"),
    }

    // Further '/' characters are not part of the domain/resource.
    let input2 = "xmpp:foo@bar.baz/txt/bin\n";
    let doc2 = core::parser::parse(input2).expect("parse failed");

    let links2 = links_in_doc(&doc2);
    assert_eq!(links2.len(), 1);

    match &links2[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, "xmpp:foo@bar.baz/txt"),
        other => panic!("expected Link, got {other:?}"),
    }
}

#[test]
fn test_gfm_autolink_trailing_right_bracket_is_excluded() {
    let md = "https://example.com]\n";
    let doc = core::parser::parse(md).expect("parse failed");

    let links = links_in_doc(&doc);
    assert_eq!(links.len(), 1);

    match &links[0].kind {
        NodeKind::Link { url, .. } => assert_eq!(url, "https://example.com"),
        other => panic!("expected Link, got {other:?}"),
    }

    let html =
        core::render::render(&doc, &core::render::RenderOptions::default()).expect("render failed");
    assert!(html.contains("<a href=\"https://example.com\">https://example.com</a>]"));
}
