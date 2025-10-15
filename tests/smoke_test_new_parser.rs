// Smoke tests for new two-stage parser
// These tests validate core functionality without relying on low-level grammar rules

use marco_core::components::engine::{parse_to_html_cached, api::{parse_markdown, render_to_html, parse_and_render}};

#[test]
fn smoke_test_parse_markdown() {
    let input = "# Hello World\n\nThis is a **test** document.";
    let result = parse_markdown(input);
    
    assert!(result.is_ok(), "Should parse basic markdown: {:?}", result.err());
    let ast = result.unwrap();
    assert!(format!("{:?}", ast).contains("Hello World"), "AST should contain heading text");
}

#[test]
fn smoke_test_render_to_html() {
    let input = "# Hello World\n\nThis is a **test** document.";
    let ast = parse_markdown(input).expect("Parse should succeed");
    
    let html = render_to_html(&ast, None);
    assert!(html.contains("<h1>"), "Should contain h1 tag");
    assert!(html.contains("Hello World"), "Should contain heading text");
    assert!(html.contains("<strong>"), "Should contain strong tag for bold");
    assert!(html.contains("test"), "Should contain paragraph text");
}

#[test]
fn smoke_test_parse_and_render() {
    let input = "# Hello World\n\nThis is a **test** document.";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse and render: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<h1>"), "Should contain h1 tag");
    assert!(html.contains("Hello World"), "Should contain heading text");
}

#[test]
fn smoke_test_cached_parsing() {
    let input = "# Cached Test\n\nThis tests the cache.";
    
    // First call - cache miss
    let result1 = parse_to_html_cached(input);
    assert!(result1.is_ok(), "First parse should succeed");
    
    // Second call - should hit cache
    let result2 = parse_to_html_cached(input);
    assert!(result2.is_ok(), "Cached parse should succeed");
    
    // Results should be identical
    assert_eq!(result1.unwrap(), result2.unwrap(), "Cached results should match");
}

#[test]
fn smoke_test_setext_headings() {
    let input = "Setext Heading\n==============\n\nAnother heading\n---------------";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse setext headings: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<h1>"), "Should contain h1 for level 1 setext");
    assert!(html.contains("<h2>"), "Should contain h2 for level 2 setext");
    assert!(html.contains("Setext Heading"), "Should contain first heading text");
    assert!(html.contains("Another heading"), "Should contain second heading text");
}

#[test]
fn smoke_test_atx_headings() {
    let input = "# Level 1\n## Level 2\n### Level 3\n#### Level 4\n##### Level 5\n###### Level 6";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse ATX headings: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<h1>"), "Should contain h1");
    assert!(html.contains("<h2>"), "Should contain h2");
    assert!(html.contains("<h3>"), "Should contain h3");
    assert!(html.contains("<h4>"), "Should contain h4");
    assert!(html.contains("<h5>"), "Should contain h5");
    assert!(html.contains("<h6>"), "Should contain h6");
}

#[test]
fn smoke_test_code_blocks() {
    let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse code blocks: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<pre>") || html.contains("<code>"), "Should contain code tags");
    assert!(html.contains("fn main"), "Should contain code content");
}

#[test]
fn smoke_test_lists() {
    let input = "- Item 1\n- Item 2\n- Item 3";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse lists: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<ul>") || html.contains("<li>"), "Should contain list tags");
    assert!(html.contains("Item 1"), "Should contain list items");
}

#[test]
fn smoke_test_blockquotes() {
    let input = "> This is a quote\n> Second line";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse blockquotes: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<blockquote>"), "Should contain blockquote tag");
    assert!(html.contains("This is a quote"), "Should contain quote text");
}

#[test]
fn smoke_test_inline_formatting() {
    let input = "This has **bold**, *italic*, and `code` formatting.";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse inline formatting: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<strong>") || html.contains("<b>"), "Should contain bold tag");
    assert!(html.contains("<em>") || html.contains("<i>"), "Should contain italic tag");
    assert!(html.contains("<code>"), "Should contain code tag");
}

#[test]
fn smoke_test_links() {
    let input = "[Link text](https://example.com)";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse links: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<a"), "Should contain anchor tag");
    assert!(html.contains("href"), "Should contain href attribute");
    assert!(html.contains("example.com"), "Should contain link URL");
}

#[test]
fn smoke_test_images() {
    let input = "![Alt text](image.png)";
    let result = parse_and_render(input, None);
    
    assert!(result.is_ok(), "Should parse images: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<img"), "Should contain img tag");
    assert!(html.contains("src"), "Should contain src attribute");
}

#[test]
fn smoke_test_complex_document() {
    let input = r#"# Main Title

This is a paragraph with **bold** and *italic* text.

## Section 1

- List item 1
- List item 2

```rust
fn example() {
    println!("Code block");
}
```

### Subsection

> A blockquote with [a link](https://example.com)

---

Another paragraph after thematic break.
"#;
    
    let result = parse_and_render(input, None);
    assert!(result.is_ok(), "Should parse complex document: {:?}", result.err());
    
    let html = result.unwrap();
    assert!(html.contains("<h1>"), "Should contain main title");
    assert!(html.contains("<h2>"), "Should contain section heading");
    assert!(html.contains("<h3>"), "Should contain subsection");
    assert!(html.contains("<li>"), "Should contain list items");
    assert!(html.contains("<pre>") || html.contains("<code>"), "Should contain code block");
    assert!(html.contains("<blockquote>"), "Should contain blockquote");
    assert!(html.contains("<a"), "Should contain link");
    assert!(html.contains("<hr"), "Should contain thematic break");
}

#[test]
fn smoke_test_error_handling() {
    // Test that malformed input doesn't panic
    let malformed_inputs = vec![
        "",  // Empty input
        "\n\n\n",  // Only newlines
        "   \n   \n   ",  // Only whitespace
        "# ",  // Heading with no content
    ];
    
    for input in malformed_inputs {
        let result = parse_markdown(input);
        // Should either parse successfully or return error, but not panic
        match result {
            Ok(ast) => println!("Parsed malformed input into AST: {:?}", ast),
            Err(e) => println!("Got expected error for malformed input: {}", e),
        }
    }
}
