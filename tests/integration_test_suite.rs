// ============================================================================
// MARCO INTEGRATION TEST SUITE
// ============================================================================
//
// Comprehensive test suite for Marco's two-stage parser architecture
//
// Test Organization:
// 1. Smoke Tests (API-level) - High-level functionality validation
// 2. Parser Tests - Grammar rule validation
// 3. Document Tests - Full document parsing
// 4. API Tests - Public API functionality
// 5. Caching Tests - Parser cache functionality
// 6. Edge Cases - Boundary conditions and special cases
// 7. Comprehensive Tests - Full integration scenarios
//
// Run all tests:
//   cargo test --package marco --test integration_test_suite
//
// Run specific module:
//   cargo test --package marco --test integration_test_suite smoke_tests
//
// ============================================================================

use marco_core::components::engine::{
    api::{parse_markdown, parse_and_render, render_to_html},
    parser_cache::global_parser_cache,
    renderers::HtmlOptions,
    parsers::block_parser::{BlockParser, Rule as BlockRule},
};
use marco_core::parse_document;

// ============================================================================
// SMOKE TESTS - API-LEVEL VALIDATION
// ============================================================================
//
// Quick validation tests for core functionality using the public API.
// These tests ensure basic features work without testing low-level details.
//
// ============================================================================

mod smoke_tests {
    use super::*;
    
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
        
        let html = render_to_html(&ast, HtmlOptions::default());
        assert!(html.contains("<h1>"), "Should contain h1 tag");
        assert!(html.contains("Hello World"), "Should contain heading text");
        assert!(html.contains("<strong>"), "Should contain strong tag for bold");
        assert!(html.contains("test"), "Should contain paragraph text");
    }
    
    #[test]
    fn smoke_test_parse_and_render() {
        let input = "# Hello World\n\nThis is a **test** document.";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should parse and render: {:?}", result.err());
        let html = result.unwrap();
        assert!(html.contains("<h1>"), "Should contain h1 tag");
        assert!(html.contains("Hello World"), "Should contain heading text");
    }
    
    #[test]
    fn smoke_test_setext_headings() {
        let input = "Setext Heading\n==============\n\nAnother heading\n---------------";
        let result = parse_and_render(input, HtmlOptions::default());
        
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
        let result = parse_and_render(input, HtmlOptions::default());
        
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
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should parse code blocks: {:?}", result.err());
        let html = result.unwrap();
        assert!(html.contains("<pre>") || html.contains("<code>"), "Should contain code tags");
        assert!(html.contains("fn main"), "Should contain code content");
    }
    
    #[test]
    fn smoke_test_lists() {
        let input = "- Item 1\n- Item 2\n- Item 3";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should parse lists: {:?}", result.err());
        let html = result.unwrap();
        assert!(html.contains("<ul>") || html.contains("<li>"), "Should contain list tags");
        assert!(html.contains("Item 1"), "Should contain list items");
    }
    
    #[test]
    fn smoke_test_blockquotes() {
        let input = "> This is a quote\n> Second line";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should parse blockquotes: {:?}", result.err());
        let html = result.unwrap();
        assert!(html.contains("<blockquote>"), "Should contain blockquote tag");
        assert!(html.contains("This is a quote"), "Should contain quote text");
    }
    
    #[test]
    fn smoke_test_inline_formatting() {
        let input = "This has **bold**, *italic*, and `code` formatting.";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should parse inline formatting: {:?}", result.err());
        let html = result.unwrap();
        assert!(html.contains("<strong>") || html.contains("<b>"), "Should contain bold tag");
        assert!(html.contains("<em>") || html.contains("<i>"), "Should contain italic tag");
        assert!(html.contains("<code>"), "Should contain code tag");
    }
    
    #[test]
    fn smoke_test_links() {
        let input = "[Link text](https://example.com)";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should parse links: {:?}", result.err());
        let html = result.unwrap();
        assert!(html.contains("<a"), "Should contain anchor tag");
        assert!(html.contains("href"), "Should contain href attribute");
        assert!(html.contains("example.com"), "Should contain link URL");
    }
    
    #[test]
    fn smoke_test_images() {
        let input = "![Alt text](image.png)";
        let result = parse_and_render(input, HtmlOptions::default());
        
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
        
        let result = parse_and_render(input, HtmlOptions::default());
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
            assert!(
                result.is_ok() || result.is_err(),
                "Should handle malformed input gracefully"
            );
        }
    }
}

// ============================================================================
// GRAMMAR AND PARSER TESTS
// ============================================================================

mod parser_tests {
    use super::*;
    use pest::Parser;
    
    #[test]
    fn test_atx_heading_basic() {
        let input = "# Heading\n";
        let result = BlockParser::parse(BlockRule::atx_heading, input);
        assert!(result.is_ok(), "Should parse ATX heading");
        
        let pairs = result.unwrap();
        let pair = pairs.into_iter().next().unwrap();
        assert_eq!(pair.as_rule(), BlockRule::atx_heading);
    }
    
    #[test]
    fn test_atx_heading_all_levels() {
        for level in 1..=6 {
            let hashes = "#".repeat(level);
            let input = format!("{} Level {} Heading\n", hashes, level);
            
            let result = BlockParser::parse(BlockRule::atx_heading, &input);
            assert!(
                result.is_ok(), 
                "Should parse ATX heading level {}: {:?}", 
                level, 
                result.err()
            );
        }
    }
    
    #[test]
    fn test_setext_heading_level1() {
        let input = "Setext Heading\n==============\n";
        let result = BlockParser::parse(BlockRule::setext_heading, input);
        
        assert!(result.is_ok(), "Should parse setext H1: {:?}", result.err());
        
        if let Ok(pairs) = result {
            let pair = pairs.into_iter().next().unwrap();
            assert_eq!(pair.as_rule(), BlockRule::setext_heading);
        }
    }
    
    #[test]
    fn test_setext_heading_level2() {
        let input = "Setext Heading\n--------------\n";
        let result = BlockParser::parse(BlockRule::setext_heading, input);
        
        assert!(result.is_ok(), "Should parse setext H2: {:?}", result.err());
    }
    
    #[test]
    fn test_thematic_break_variations() {
        let variations = vec![
            "---\n",
            "***\n",
            "___\n",
            "- - -\n",
            "* * *\n",
            "_ _ _\n",
        ];
        
        for input in variations {
            let result = BlockParser::parse(BlockRule::thematic_break, input);
            assert!(
                result.is_ok(), 
                "Should parse thematic break '{}': {:?}", 
                input.trim(), 
                result.err()
            );
        }
    }
    
    #[test]
    fn test_fenced_code_block_backticks() {
        let input = "```\ncode here\n```";
        let result = BlockParser::parse(BlockRule::fenced_code_block, input);
        
        assert!(result.is_ok(), "Should parse backtick fenced code: {:?}", result.err());
    }
    
    #[test]
    fn test_fenced_code_block_tildes() {
        let input = "~~~\ncode here\n~~~";
        let result = BlockParser::parse(BlockRule::fenced_code_block, input);
        
        assert!(result.is_ok(), "Should parse tilde fenced code: {:?}", result.err());
    }
    
    #[test]
    fn test_fenced_code_block_with_info() {
        let input = "```rust\nfn main() {}\n```";
        let result = BlockParser::parse(BlockRule::fenced_code_block, input);
        
        assert!(result.is_ok(), "Should parse fenced code with info string: {:?}", result.err());
    }
    
    #[test]
    fn test_blockquote_simple() {
        let input = "> This is a quote\n";
        let result = BlockParser::parse(BlockRule::blockquote, input);
        
        assert!(result.is_ok(), "Should parse simple blockquote: {:?}", result.err());
    }
    
    #[test]
    fn test_blockquote_multi_line() {
        let input = "> Line 1\n> Line 2\n> Line 3\n";
        let result = BlockParser::parse(BlockRule::blockquote, input);
        
        assert!(result.is_ok(), "Should parse multi-line blockquote: {:?}", result.err());
    }
    
    #[test]
    fn test_list_bullet_dash() {
        let input = "- Item 1\n- Item 2\n";
        let result = BlockParser::parse(BlockRule::list, input);
        
        assert!(result.is_ok(), "Should parse dash bullet list: {:?}", result.err());
    }
    
    #[test]
    fn test_list_ordered() {
        let input = "1. First\n2. Second\n3. Third\n";
        let result = BlockParser::parse(BlockRule::list, input);
        
        assert!(result.is_ok(), "Should parse ordered list: {:?}", result.err());
    }
    
    #[test]
    fn test_reference_definition_basic() {
        let input = "[foo]: /url\n";
        let result = BlockParser::parse(BlockRule::reference_definition, input);
        
        assert!(result.is_ok(), "Should parse basic reference definition: {:?}", result.err());
    }
    
    #[test]
    fn test_reference_definition_with_title() {
        let input = "[foo]: /url \"title\"\n";
        let result = BlockParser::parse(BlockRule::reference_definition, input);
        
        assert!(result.is_ok(), "Should parse reference with title: {:?}", result.err());
    }
}

// ============================================================================
// DOCUMENT PARSING TESTS
// ============================================================================

mod document_tests {
    use super::*;
    
    #[test]
    fn test_parse_document_simple() {
        let input = "# Heading\n\nThis is a paragraph.\n";
        let result = parse_document(input);
        
        assert!(result.is_ok(), "Should parse simple document: {:?}", result.err());
    }
    
    #[test]
    fn test_parse_document_complex() {
        let input = r#"# Title

This is a paragraph with **bold** and *italic* text.

## Subtitle

```rust
fn main() {
    println!("Hello");
}
```

> A blockquote

- List item 1
- List item 2

---

Another paragraph.
"#;
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Should parse complex document: {:?}", result.err());
    }
    
    #[test]
    fn test_parse_document_with_setext_headings() {
        let input = "First Header\n============\n\nSecond Header\n-------------\n\nRegular text.\n";
        
        let result = parse_document(input);
        assert!(result.is_ok(), "Should parse document with setext headers: {:?}", result.err());
    }
    
    #[test]
    fn test_parse_document_without_trailing_newline() {
        let input = "# Heading\n\nThis is a paragraph.";
        let result = parse_document(input);
        
        assert!(result.is_ok(), "Should parse document without trailing newline: {:?}", result.err());
    }
}

// ============================================================================
// API TESTS (parse_markdown, render_to_html, parse_and_render)
// ============================================================================

mod api_tests {
    use super::*;
    
    #[test]
    fn test_parse_markdown_basic() {
        let input = "# Hello World\n\nThis is a **test** document.";
        let result = parse_markdown(input);
        
        assert!(result.is_ok(), "Should parse markdown: {:?}", result.err());
        
        let ast = result.unwrap();
        let ast_debug = format!("{:?}", ast);
        assert!(ast_debug.contains("Hello World"), "AST should contain heading text");
    }
    
    #[test]
    fn test_parse_and_render_basic() {
        let input = "# Hello World\n\nThis is a **test** document.";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should parse and render: {:?}", result.err());
        
        let html = result.unwrap();
        assert!(html.contains("<h1>"), "HTML should contain h1 tag");
        assert!(html.contains("Hello World"), "HTML should contain heading text");
        assert!(html.contains("<strong>"), "HTML should contain strong tag");
        assert!(html.contains("test"), "HTML should contain paragraph text");
    }
    
    #[test]
    fn test_parse_and_render_setext_headings() {
        let setext_h1 = "Test Header H1\n==============\n";
        let h1_result = parse_and_render(setext_h1, HtmlOptions::default());
        
        assert!(h1_result.is_ok(), "Should render setext H1: {:?}", h1_result.err());
        
        let h1_html = h1_result.unwrap();
        assert!(
            !h1_html.contains("=============="),
            "H1 HTML should not contain underline markers"
        );
        assert!(
            h1_html.contains("<h1>") && h1_html.contains("Test Header H1"),
            "Should contain proper H1 tag and content"
        );
    }
    
    #[test]
    fn test_parse_and_render_code_blocks() {
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should render code block: {:?}", result.err());
        
        let html = result.unwrap();
        assert!(html.contains("<pre>") || html.contains("<code>"), "Should contain code tags");
        assert!(html.contains("fn main"), "Should contain code content");
    }
    
    #[test]
    fn test_parse_and_render_lists() {
        let input = "- Item 1\n- Item 2\n- Item 3\n";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should render list: {:?}", result.err());
        
        let html = result.unwrap();
        assert!(html.contains("<ul>") || html.contains("<li>"), "Should contain list tags");
    }
    
    #[test]
    fn test_parse_and_render_blockquote() {
        let input = "> This is a quote\n> with multiple lines\n";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should render blockquote: {:?}", result.err());
        
        let html = result.unwrap();
        assert!(html.contains("<blockquote>"), "Should contain blockquote tag");
    }
    
    #[test]
    fn test_parse_and_render_emphasis() {
        let input = "This has *emphasis* and **strong** text.\n";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should render emphasis: {:?}", result.err());
        
        let html = result.unwrap();
        assert!(html.contains("<em>") || html.contains("<i>"), "Should contain emphasis tag");
        assert!(html.contains("<strong>") || html.contains("<b>"), "Should contain strong tag");
    }
    
    #[test]
    fn test_reference_links() {
        let input = r#"[google]: https://google.com "Google"

Visit [Google][google] for search.
"#;
        
        let result = parse_and_render(input, HtmlOptions::default());
        assert!(result.is_ok(), "Should render reference links: {:?}", result.err());
        
        let html = result.unwrap();
        assert!(html.contains("https://google.com"), "Should contain resolved URL");
    }
}

// ============================================================================
// CACHING TESTS
// ============================================================================

mod cache_tests {
    use super::*;
    
    #[test]
    fn test_cached_parsing() {
        let input = "# Cached Test\n\nThis tests the cache.";
        
        // First call - cache miss
        let result1 = global_parser_cache().render_with_cache(input, HtmlOptions::default());
        assert!(result1.is_ok(), "First parse should succeed");
        
        // Second call - should hit cache
        let result2 = global_parser_cache().render_with_cache(input, HtmlOptions::default());
        assert!(result2.is_ok(), "Cached parse should succeed");
        
        // Results should be identical
        assert_eq!(
            result1.unwrap(), 
            result2.unwrap(), 
            "Cached results should match"
        );
    }
    
    #[test]
    fn test_cache_invalidation() {
        let input1 = "# First Version\n";
        let input2 = "# Second Version\n";
        
        let result1 = global_parser_cache().render_with_cache(input1, HtmlOptions::default());
        let result2 = global_parser_cache().render_with_cache(input2, HtmlOptions::default());
        
        assert!(result1.is_ok() && result2.is_ok());
        assert_ne!(
            result1.unwrap(),
            result2.unwrap(),
            "Different inputs should produce different outputs"
        );
    }
}

// ============================================================================
// EDGE CASES AND ERROR HANDLING
// ============================================================================

mod edge_cases {
    use super::*;
    
    #[test]
    fn test_empty_document() {
        let input = "";
        let result = parse_document(input);
        
        // Empty document should parse successfully (as empty document node)
        assert!(result.is_ok(), "Empty document should parse: {:?}", result.err());
    }
    
    #[test]
    fn test_only_whitespace() {
        let input = "   \n\n  \n";
        let result = parse_document(input);
        
        assert!(result.is_ok(), "Whitespace-only document should parse: {:?}", result.err());
    }
    
    #[test]
    fn test_mixed_line_endings() {
        // Test with different line endings (LF, CRLF)
        let input = "# Header\r\n\r\nParagraph\n\nAnother paragraph\r\n";
        let result = parse_document(input);
        
        assert!(result.is_ok(), "Should handle mixed line endings: {:?}", result.err());
    }
    
    #[test]
    fn test_unicode_content() {
        let input = "# 你好世界\n\nこんにちは 🌍\n";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should handle Unicode: {:?}", result.err());
        
        let html = result.unwrap();
        assert!(html.contains("你好世界"), "Should preserve Chinese characters");
        assert!(html.contains("こんにちは"), "Should preserve Japanese characters");
        assert!(html.contains("🌍"), "Should preserve emoji");
    }
    
    #[test]
    fn test_deeply_nested_blockquotes() {
        let input = "> Level 1\n> > Level 2\n> > > Level 3\n";
        let result = parse_and_render(input, HtmlOptions::default());
        
        assert!(result.is_ok(), "Should handle nested blockquotes: {:?}", result.err());
    }
    
    #[test]
    fn test_unclosed_code_block() {
        // Unclosed code block should still parse (as paragraph or remaining content)
        let input = "```\nunclosed code block";
        let result = parse_document(input);
        
        // The parser should handle this gracefully
        assert!(result.is_ok(), "Should handle unclosed code block: {:?}", result.err());
    }
}

// ============================================================================
// COMPREHENSIVE INTEGRATION TEST
// ============================================================================

#[test]
fn test_comprehensive_document() {
    let input = r#"# Main Document Title

This is the introduction paragraph with **bold**, *italic*, and `inline code`.

## Section 1: Code Examples

Here's a Rust code block:

```rust
fn main() {
    println!("Hello, Marco!");
}
```

And a Python example:

```python
def hello():
    print("Hello, World!")
```

## Section 2: Lists

Unordered list:
- First item
- Second item with **emphasis**
- Third item

Ordered list:
1. Step one
2. Step two
3. Step three

## Section 3: Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> And have multiple paragraphs.

## Section 4: Links and References

[google]: https://google.com "Google Search"
[github]: https://github.com

Visit [Google][google] or check out [GitHub][github].

---

## Conclusion

This document demonstrates various markdown features supported by Marco.
"#;
    
    // Test parsing
    let parse_result = parse_markdown(input);
    assert!(
        parse_result.is_ok(), 
        "Should parse comprehensive document: {:?}", 
        parse_result.err()
    );
    
    // Test rendering
    let render_result = parse_and_render(input, HtmlOptions::default());
    assert!(
        render_result.is_ok(),
        "Should render comprehensive document: {:?}",
        render_result.err()
    );
    
    let html = render_result.unwrap();
    
    // Verify key elements are present
    assert!(html.contains("<h1>"), "Should have H1 heading");
    assert!(html.contains("<h2>"), "Should have H2 headings");
    assert!(html.contains("<code>"), "Should have code elements");
    assert!(html.contains("<ul>") || html.contains("<li>"), "Should have unordered list");
    assert!(html.contains("<ol>"), "Should have ordered list");
    assert!(html.contains("<blockquote>"), "Should have blockquote");
    assert!(html.contains("<strong>"), "Should have bold text");
    assert!(html.contains("<em>") || html.contains("<i>"), "Should have italic text");
    assert!(html.contains("https://google.com"), "Should have resolved reference links");
    
    println!("✅ Comprehensive document test passed!");
    println!("HTML length: {} bytes", html.len());
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[cfg(not(debug_assertions))]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_large_document_performance() {
        // Generate a large document
        let mut document = String::new();
        for i in 0..1000 {
            document.push_str(&format!("# Heading {}\n\n", i));
            document.push_str("This is a paragraph with **bold** text.\n\n");
            document.push_str("- List item\n");
        }
        
        let start = Instant::now();
        let result = parse_and_render(&document, HtmlOptions::default());
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Should parse large document");
        println!("⏱️  Large document (1000 blocks) parsed in {:?}", duration);
        
        // Performance target: should complete within reasonable time
        assert!(
            duration.as_secs() < 5,
            "Large document should parse within 5 seconds (took {:?})",
            duration
        );
    }
}
