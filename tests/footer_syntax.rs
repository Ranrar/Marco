use std::fs;
use regex::Regex;
use marco::logic::parser::{parse_document_blocks, MarkdownSyntaxMap, SyntaxRule};

#[test]
fn test_footer_syntax_tree_from_file() {
    let path = "dev/test_footer_preview.md";
    let text = fs::read_to_string(path).expect("failed to read test markdown file");

    let mut rules = std::collections::HashMap::new();

    // Video pattern
    let video_re = Regex::new(r"^(?:\[!\[.*?\]\(https?://img\.youtube\.com/vi/(?P<id1>[A-Za-z0-9_-]+)/0\.jpg\)\]\(https?://(?:www\.)?youtube\.com/watch\?v=(?P<id2>[A-Za-z0-9_-]+)\))").unwrap();
    rules.insert("re:youtube-video".to_string(), SyntaxRule { node_type: "video".to_string(), depth: None, ordered: None, markdown_syntax: "re:youtube-video".to_string(), is_regex: true, regex: Some(video_re) });

    // Image width pattern
    let img_re = Regex::new(r#"^(?:<img\s+[^>]*width=['"]?(?P<w>\d+)['"]?[^>]*>)"#).unwrap();
    rules.insert("re:img-width".to_string(), SyntaxRule { node_type: "image-size".to_string(), depth: None, ordered: None, markdown_syntax: "re:img-width".to_string(), is_regex: true, regex: Some(img_re) });

    // Anchor with target
    let link_re = Regex::new(r#"^(?:<a\s+[^>]*href="(?P<h>[^"]+)"[^>]*target="(?P<t>[^"]+)"[^>]*>)"#).unwrap();
    rules.insert("re:link-target".to_string(), SyntaxRule { node_type: "link-target".to_string(), depth: None, ordered: None, markdown_syntax: "re:link-target".to_string(), is_regex: true, regex: Some(link_re) });

    let map = MarkdownSyntaxMap { rules, display_hints: None };

    let (tokens, link_defs) = parse_document_blocks(&text, &map);

    assert!(tokens.iter().any(|t| t.node_type == "frontmatter" && t.captures.as_ref().and_then(|c| c.get("value")).is_some()), "Expected frontmatter token");
    assert!(tokens.iter().any(|t| t.node_type == "heading" && t.depth == Some(1)), "Expected Setext heading token");
    assert!(link_defs.len() >= 1, "Expected at least one link definition");
    assert!(tokens.iter().any(|t| t.node_type == "video"), "Expected video token");
    assert!(tokens.iter().any(|t| t.node_type == "image-size" && t.captures.as_ref().and_then(|c| c.get("w")).map(|s| s == "300").unwrap_or(false)), "Expected image-size w=300");
    assert!(tokens.iter().any(|t| t.node_type == "link-target" && t.captures.as_ref().and_then(|c| c.get("t")).map(|s| s == "_blank").unwrap_or(false)), "Expected link-target _blank");
}
