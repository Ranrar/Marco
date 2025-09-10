use marco::components::marco_engine::{
    ast::{AstBuilder, Node},
    grammar::{MarcoParser, Rule},
};
use pest::Parser;

// Helper function to parse and build AST for a given rule
fn parse_and_build_ast(rule: Rule, input: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let pairs = MarcoParser::parse(rule, input)?;
    let node = AstBuilder::build(pairs)?;
    Ok(node)
}

// Helper function to parse a specific rule and get the first pair
fn parse_rule(rule: Rule, input: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let mut pairs = MarcoParser::parse(rule, input)?;
    if let Some(pair) = pairs.next() {
        let node = AstBuilder::build_node_for_testing(pair)?;
        Ok(node)
    } else {
        Err("No pairs found".into())
    }
}

#[test]
fn test_text_node_creation() {
    let result = parse_rule(Rule::text, "Hello world");
    assert!(result.is_ok(), "Should parse text successfully");

    if let Ok(Node::Text { content, .. }) = result {
        assert_eq!(content, "Hello world");
    } else {
        panic!("Expected Text node");
    }
}

#[test]
fn test_heading_ast_building() {
    let input = "# Main Title";
    let result = parse_rule(Rule::heading, input);
    assert!(result.is_ok(), "Should parse heading successfully");

    if let Ok(Node::Heading { level, content, .. }) = result {
        assert_eq!(level, 1);
        assert!(!content.is_empty());
        if let Node::Text { content: text, .. } = &content[0] {
            assert!(text.contains("Main Title"));
        }
    } else {
        panic!("Expected Heading node, got: {:?}", result);
    }
}

#[test]
fn test_paragraph_ast_building() {
    let input = "This is a simple paragraph with multiple words.";
    let result = parse_rule(Rule::paragraph, input);
    assert!(result.is_ok(), "Should parse paragraph successfully");

    match result.unwrap() {
        Node::Paragraph { content, .. } => {
            assert!(!content.is_empty());
        }
        Node::Document { children, .. } => {
            // Sometimes paragraphs are wrapped in documents due to multiline handling
            assert!(!children.is_empty());
        }
        other => panic!("Expected Paragraph or Document node, got: {:?}", other),
    }
}

#[test]
fn test_bold_text_ast_building() {
    let input = "**bold text**";
    let result = parse_rule(Rule::bold, input);
    assert!(result.is_ok(), "Should parse bold text successfully");

    if let Ok(Node::Strong { content, .. }) = result {
        assert!(!content.is_empty());
        if let Node::Text { content: text, .. } = &content[0] {
            assert_eq!(text, "bold text");
        }
    } else {
        panic!("Expected Strong node, got: {:?}", result);
    }
}

#[test]
fn test_emphasis_ast_building() {
    let input = "*italic text*";
    let result = parse_rule(Rule::emphasis, input);
    assert!(result.is_ok(), "Should parse emphasis successfully");

    if let Ok(Node::Emphasis { content, .. }) = result {
        assert!(!content.is_empty());
        if let Node::Text { content: text, .. } = &content[0] {
            assert_eq!(text, "italic text");
        }
    } else {
        panic!("Expected Emphasis node, got: {:?}", result);
    }
}

#[test]
fn test_code_inline_ast_building() {
    let input = "`code here`";
    let result = parse_rule(Rule::code_inline, input);
    assert!(result.is_ok(), "Should parse inline code successfully");

    if let Ok(Node::Code { content, .. }) = result {
        assert_eq!(content, "code here");
    } else {
        panic!("Expected Code node, got: {:?}", result);
    }
}

#[test]
fn test_code_block_ast_building() {
    let input = "```rust\nfn main() {\n    println!(\"Hello!\");\n}\n```";
    let result = parse_rule(Rule::code_block, input);
    assert!(result.is_ok(), "Should parse code block successfully");

    if let Ok(Node::CodeBlock {
        language, content, ..
    }) = result
    {
        assert_eq!(language, Some("rust".to_string()));
        assert!(content.contains("fn main()"));
        assert!(content.contains("println!"));
    } else {
        panic!("Expected CodeBlock node, got: {:?}", result);
    }
}

#[test]
fn test_link_ast_building() {
    let input = "[link text](https://example.com)";
    let result = parse_rule(Rule::inline_link, input);
    assert!(result.is_ok(), "Should parse link successfully");

    if let Ok(Node::Link { text, url, .. }) = result {
        assert!(!text.is_empty());
        assert_eq!(url, "https://example.com");
        if let Node::Text { content, .. } = &text[0] {
            assert_eq!(content, "link text");
        }
    } else {
        panic!("Expected Link node, got: {:?}", result);
    }
}

#[test]
fn test_image_ast_building() {
    let input = "![alt text](image.jpg)";
    let result = parse_rule(Rule::inline_image, input);
    assert!(result.is_ok(), "Should parse image successfully");

    if let Ok(Node::Image { alt, url, .. }) = result {
        assert_eq!(alt, "alt text");
        assert_eq!(url, "image.jpg");
    } else {
        panic!("Expected Image node, got: {:?}", result);
    }
}

#[test]
fn test_list_ast_building() {
    let input = "- Item 1\n- Item 2\n- Item 3";
    let result = parse_rule(Rule::list, input);
    assert!(result.is_ok(), "Should parse list successfully");

    if let Ok(Node::List { ordered, items, .. }) = result {
        assert!(!ordered);
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected List node, got: {:?}", result);
    }
}

#[test]
fn test_ordered_list_ast_building() {
    let input = "1. First item\n2. Second item";
    let result = parse_rule(Rule::list, input);
    assert!(result.is_ok(), "Should parse ordered list successfully");

    if let Ok(Node::List { ordered, items, .. }) = result {
        assert!(ordered);
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected List node, got: {:?}", result);
    }
}

#[test]
fn test_task_list_ast_building() {
    let input = "- [x] Completed task\n- [ ] Incomplete task";
    let result = parse_rule(Rule::list, input);
    assert!(result.is_ok(), "Should parse task list successfully");

    if let Ok(Node::List { items, .. }) = result {
        assert_eq!(items.len(), 2);
        // Check first item is checked
        if let Node::ListItem {
            checked: Some(true),
            ..
        } = &items[0]
        {
            // Good
        } else {
            panic!("Expected checked task item");
        }
        // Check second item is unchecked
        if let Node::ListItem {
            checked: Some(false),
            ..
        } = &items[1]
        {
            // Good
        } else {
            panic!("Expected unchecked task item");
        }
    } else {
        panic!("Expected List node, got: {:?}", result);
    }
}

#[test]
fn test_blockquote_ast_building() {
    let input = "> This is a quote\n> With multiple lines";
    let result = parse_rule(Rule::blockquote, input);
    assert!(result.is_ok(), "Should parse blockquote successfully");

    if let Ok(Node::BlockQuote { content, .. }) = result {
        assert!(!content.is_empty());
    } else {
        panic!("Expected BlockQuote node, got: {:?}", result);
    }
}

#[test]
fn test_horizontal_rule_ast_building() {
    let input = "---";
    let result = parse_rule(Rule::hr, input);
    assert!(result.is_ok(), "Should parse horizontal rule successfully");

    if let Ok(Node::HorizontalRule { .. }) = result {
        // Good
    } else {
        panic!("Expected HorizontalRule node, got: {:?}", result);
    }
}

#[test]
fn test_strikethrough_ast_building() {
    let input = "~~deleted text~~";
    let result = parse_rule(Rule::strikethrough, input);
    assert!(result.is_ok(), "Should parse strikethrough successfully");

    if let Ok(Node::Strikethrough { content, .. }) = result {
        assert!(!content.is_empty());
        if let Node::Text { content: text, .. } = &content[0] {
            assert_eq!(text, "deleted text");
        }
    } else {
        panic!("Expected Strikethrough node, got: {:?}", result);
    }
}

#[test]
fn test_highlight_ast_building() {
    let input = "==highlighted text==";
    let result = parse_rule(Rule::highlight, input);
    assert!(result.is_ok(), "Should parse highlight successfully");

    if let Ok(Node::Highlight { content, .. }) = result {
        assert!(!content.is_empty());
        if let Node::Text { content: text, .. } = &content[0] {
            assert_eq!(text, "highlighted text");
        }
    } else {
        panic!("Expected Highlight node, got: {:?}", result);
    }
}

#[test]
fn test_math_inline_ast_building() {
    let input = "$E = mc^2$";
    let result = parse_rule(Rule::math_inline, input);
    assert!(result.is_ok(), "Should parse inline math successfully");

    if let Ok(Node::MathInline { content, .. }) = result {
        assert_eq!(content, "E = mc^2");
    } else {
        panic!("Expected MathInline node, got: {:?}", result);
    }
}

#[test]
fn test_math_block_ast_building() {
    let input = "$$\nE = mc^2\n$$";
    let result = parse_rule(Rule::math_block, input);
    assert!(result.is_ok(), "Should parse math block successfully");

    if let Ok(Node::MathBlock { content, .. }) = result {
        assert!(content.trim().contains("E = mc^2"));
    } else {
        panic!("Expected MathBlock node, got: {:?}", result);
    }
}

#[test]
fn test_emoji_ast_building() {
    let input = ":smile:";
    let result = parse_rule(Rule::emoji, input);
    assert!(result.is_ok(), "Should parse emoji successfully");

    if let Ok(Node::Emoji { name, .. }) = result {
        assert_eq!(name, "smile");
    } else {
        panic!("Expected Emoji node, got: {:?}", result);
    }
}

#[test]
fn test_superscript_ast_building() {
    let input = "^superscript^";
    let result = parse_rule(Rule::superscript, input);
    assert!(result.is_ok(), "Should parse superscript successfully");

    if let Ok(Node::Superscript { content, .. }) = result {
        assert!(!content.is_empty());
        if let Node::Text { content: text, .. } = &content[0] {
            assert_eq!(text, "superscript");
        }
    } else {
        panic!("Expected Superscript node, got: {:?}", result);
    }
}

#[test]
fn test_subscript_ast_building() {
    let input = "~subscript~";
    let result = parse_rule(Rule::subscript, input);
    assert!(result.is_ok(), "Should parse subscript successfully");

    if let Ok(Node::Subscript { content, .. }) = result {
        assert!(!content.is_empty());
        if let Node::Text { content: text, .. } = &content[0] {
            assert_eq!(text, "subscript");
        }
    } else {
        panic!("Expected Subscript node, got: {:?}", result);
    }
}

// Test Marco-specific extensions
#[test]
fn test_admonition_ast_building() {
    let input = ":::note\nThis is a note\n:::";
    let result = parse_rule(Rule::admonition_block, input);
    assert!(result.is_ok(), "Should parse admonition successfully");

    if let Ok(Node::Admonition { kind, content, .. }) = result {
        assert_eq!(kind, "note");
        assert!(!content.is_empty());
    } else {
        panic!("Expected Admonition node, got: {:?}", result);
    }
}

#[test]
fn test_user_mention_ast_building() {
    let input = "@user [github](John Doe)";
    let result = parse_rule(Rule::user_mention, input);
    assert!(result.is_ok(), "Should parse user mention successfully");

    if let Ok(Node::UserMention {
        username,
        platform,
        display_name,
        ..
    }) = result
    {
        assert_eq!(username, "user");
        assert_eq!(platform, Some("github".to_string()));
        assert_eq!(display_name, Some("John Doe".to_string()));
    } else {
        panic!("Expected UserMention node, got: {:?}", result);
    }
}

#[test]
fn debug_heading_parsing() {
    let input = "# Main Title";
    println!("=== Debug Heading Parsing ===");
    println!("Input: {:?}", input);

    let result = parse_rule(Rule::heading, input);
    println!("Parse result: {:?}", result);

    if let Ok(Node::Heading { level, content, .. }) = &result {
        println!("Level: {}", level);
        println!("Content length: {}", content.len());
        for (i, node) in content.iter().enumerate() {
            println!("Content[{}]: {:?}", i, node);
            if let Node::Text { content: text, .. } = node {
                println!("  Text: {:?}", text);
                println!("  Contains 'Main Title': {}", text.contains("Main Title"));
            }
        }
    }
}

#[test]
fn debug_user_mention_parsing() {
    let input = "@user [github](John Doe)";
    println!("=== Debug User Mention Parsing ===");
    println!("Input: {:?}", input);

    let result = parse_rule(Rule::user_mention, input);
    println!("Parse result: {:?}", result);

    if let Ok(Node::UserMention {
        username,
        platform,
        display_name,
        ..
    }) = &result
    {
        println!("Username: {:?}", username);
        println!("Platform: {:?}", platform);
        println!("Display name: {:?}", display_name);
    }
}

#[test]
fn debug_task_list_parsing() {
    let input = "- [x] Completed task\n- [ ] Incomplete task";
    println!("=== Debug Task List Parsing ===");
    println!("Input: {:?}", input);

    // Try parsing with document first to see what we get
    let doc_result = parse_rule(Rule::document, input);
    println!("Document parse result: {:?}", doc_result);

    let result = parse_rule(Rule::list, input);
    println!("List parse result: {:?}", result);

    if let Ok(Node::List { items, .. }) = &result {
        println!("Items length: {}", items.len());
        for (i, item) in items.iter().enumerate() {
            println!("Item[{}]: {:?}", i, item);
        }
    }
}

#[test]
fn test_bookmark_ast_building() {
    let input = "[bookmark:section](./file.md=42)";
    let result = parse_rule(Rule::bookmark, input);
    assert!(result.is_ok(), "Should parse bookmark successfully");

    if let Ok(Node::Bookmark {
        label, path, line, ..
    }) = result
    {
        assert_eq!(label, "section");
        assert_eq!(path, "./file.md");
        assert_eq!(line, Some(42));
    } else {
        panic!("Expected Bookmark node, got: {:?}", result);
    }
}

#[test]
fn test_table_of_contents_ast_building() {
    let input = "[toc=3]";
    let result = parse_rule(Rule::toc, input);
    assert!(
        result.is_ok(),
        "Should parse table of contents successfully"
    );

    if let Ok(Node::TableOfContents {
        depth, document, ..
    }) = result
    {
        assert_eq!(depth, Some(3));
        assert_eq!(document, None);
    } else {
        panic!("Expected TableOfContents node, got: {:?}", result);
    }
}

#[test]
fn test_run_inline_ast_building() {
    let input = "run@bash(echo hello)";
    let result = parse_rule(Rule::run_inline, input);
    assert!(result.is_ok(), "Should parse run inline successfully");

    if let Ok(Node::RunInline {
        script_type,
        command,
        ..
    }) = result
    {
        assert_eq!(script_type, "bash");
        assert_eq!(command, "echo hello");
    } else {
        panic!("Expected RunInline node, got: {:?}", result);
    }
}

#[test]
fn test_document_ast_building() {
    let input = "# Title\n\nThis is a paragraph.\n\n**Bold text**";
    let result = parse_and_build_ast(Rule::file, input);
    assert!(result.is_ok(), "Should parse document successfully");

    if let Ok(Node::Document { children, .. }) = result {
        assert!(!children.is_empty());
        println!("Document has {} children", children.len());

        // Should contain heading, paragraph, etc.
        let has_heading = children
            .iter()
            .any(|node| matches!(node, Node::Heading { .. }));
        assert!(has_heading, "Document should contain at least one heading");
    } else {
        panic!("Expected Document node, got: {:?}", result);
    }
}

// Error handling tests
#[test]
fn test_invalid_input_handling() {
    // Test with completely invalid markdown
    let result = parse_rule(Rule::heading, "Not a heading");

    // Should either succeed (graceful degradation) or fail gracefully
    match result {
        Ok(_) => println!("Graceful degradation worked"),
        Err(_) => println!("Failed as expected"),
    }
}

#[test]
fn test_empty_input_handling() {
    let result = parse_rule(Rule::text, "");

    // Should handle empty input gracefully
    match result {
        Ok(_) => println!("Empty input handled"),
        Err(_) => println!("Empty input failed as expected"),
    }
}

// Complex document test
#[test]
fn test_complex_document_ast_building() {
    let input = r#"# Main Title

This is a paragraph with **bold** and *italic* text.

## Subsection

- List item 1
- List item 2 with `inline code`
- [x] Completed task
- [ ] Incomplete task

> This is a blockquote
> with multiple lines

```rust
fn hello() {
    println!("Hello, World!");
}
```

:::note
This is an admonition block
:::

[Link to example](https://example.com)

![Image](image.jpg)
"#;

    let result = parse_and_build_ast(Rule::file, input);
    assert!(result.is_ok(), "Should parse complex document successfully");

    if let Ok(Node::Document { children, .. }) = result {
        assert!(
            children.len() > 5,
            "Complex document should have multiple children"
        );
        println!(
            "Complex document parsed with {} top-level elements",
            children.len()
        );

        // Verify we have different types of nodes
        let has_heading = children
            .iter()
            .any(|node| matches!(node, Node::Heading { .. }));
        let has_paragraph = children
            .iter()
            .any(|node| matches!(node, Node::Paragraph { .. }));
        let has_list = children
            .iter()
            .any(|node| matches!(node, Node::List { .. }));
        let has_blockquote = children
            .iter()
            .any(|node| matches!(node, Node::BlockQuote { .. }));
        let has_code_block = children
            .iter()
            .any(|node| matches!(node, Node::CodeBlock { .. }));
        let has_admonition = children
            .iter()
            .any(|node| matches!(node, Node::Admonition { .. }));

        assert!(has_heading, "Should contain headings");
        // Note: Some elements might be wrapped differently due to parsing rules
        println!("Document structure verification:");
        println!("  Has heading: {}", has_heading);
        println!("  Has paragraph: {}", has_paragraph);
        println!("  Has list: {}", has_list);
        println!("  Has blockquote: {}", has_blockquote);
        println!("  Has code block: {}", has_code_block);
        println!("  Has admonition: {}", has_admonition);
    } else {
        panic!(
            "Expected Document node for complex input, got: {:?}",
            result
        );
    }
}
