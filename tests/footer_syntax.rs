use std::fs;
use marco::components::marco_engine::parser::{parse_document_blocks, MarkdownSyntaxMap, SyntaxRule};

#[test]
fn test_footer_syntax_tree_from_file() {
    let path = "dev/test_footer_preview.md";
    let text = fs::read_to_string(path).expect("failed to read test markdown file");

    let mut rules = std::collections::HashMap::new();

    // Video pattern
    rules.insert("re:youtube-video".to_string(), SyntaxRule { 
        name: "video".to_string(), 
        pattern: "re:youtube-video".to_string(), 
        description: "YouTube video".to_string() 
    });

    // Image width pattern
    rules.insert("re:img-width".to_string(), SyntaxRule { 
        name: "image-size".to_string(), 
        pattern: "re:img-width".to_string(), 
        description: "Image width".to_string() 
    });

    // Anchor with target
    rules.insert("re:link-target".to_string(), SyntaxRule { 
        name: "link-target".to_string(), 
        pattern: "re:link-target".to_string(), 
        description: "Link with target".to_string() 
    });

    let map = MarkdownSyntaxMap { rules, display_hints: None };

    let (tokens, link_defs) = parse_document_blocks(&text, &map);

    assert!(tokens.iter().any(|t| t.node_type == "frontmatter" && t.captures.as_ref().and_then(|c| c.get("value")).is_some()), "Expected frontmatter token");
    assert!(tokens.iter().any(|t| t.node_type == "heading" && t.depth == Some(1)), "Expected Setext heading token");
    assert!(link_defs.len() >= 1, "Expected at least one link definition");
    assert!(tokens.iter().any(|t| t.node_type == "video"), "Expected video token");
    assert!(tokens.iter().any(|t| t.node_type == "image-size" && t.captures.as_ref().and_then(|c| c.get("w")).map(|s| s == "300").unwrap_or(false)), "Expected image-size w=300");
    assert!(tokens.iter().any(|t| t.node_type == "link-target" && t.captures.as_ref().and_then(|c| c.get("t")).map(|s| s == "_blank").unwrap_or(false)), "Expected link-target _blank");
}
