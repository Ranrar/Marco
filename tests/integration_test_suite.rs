use std::process::Command;

// Grammar and parser tests
mod parser_tests {
    use marco_core::components::marco_engine::{parse_to_html_cached, Rule};
    use marco_core::{parse_document, parse_with_rule, ParseResult};

    #[test]
    fn test_setext_h1_grammar() {
        let input = "Alternative Setext H1\n=====================";

        let result = parse_with_rule(input, Rule::setext_h1);
        assert!(result.is_ok(), "Should parse setext H1 successfully");

        let pairs = result.unwrap();
        let pair = pairs.into_iter().next().unwrap();

        // Check structure
        assert_eq!(pair.as_rule(), Rule::setext_h1);

        // Look for setext_content child
        let mut has_content = false;
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::setext_content {
                has_content = true;
                assert_eq!(inner.as_str(), "Alternative Setext H1");
            }
        }
        assert!(has_content, "Should have setext_content child rule");
    }

    #[test]
    fn test_setext_h2_grammar() {
        let input = "Alternative Setext H2\n---------------------";

        let result = parse_with_rule(input, Rule::setext_h2);
        assert!(result.is_ok(), "Should parse setext H2 successfully");

        let pairs = result.unwrap();
        let pair = pairs.into_iter().next().unwrap();

        // Check structure
        assert_eq!(pair.as_rule(), Rule::setext_h2);

        // Look for setext_content child
        let mut has_content = false;
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::setext_content {
                has_content = true;
                assert_eq!(inner.as_str(), "Alternative Setext H2");
            }
        }
        assert!(has_content, "Should have setext_content child rule");
    }

    #[test]
    fn test_setext_content_extraction() {
        let input = "Simple Header\n=============";

        let result = parse_with_rule(input, Rule::setext_h1);
        assert!(result.is_ok(), "Should parse setext H1");

        let pairs = result.unwrap();
        let pair = pairs.into_iter().next().unwrap();

        // Debug: print the structure
        println!("Setext H1 structure:");
        print_parser_structure(pair.clone(), 0);

        // Extract content
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::setext_content {
                assert_eq!(inner.as_str().trim(), "Simple Header");
                return;
            }
        }
        panic!("No setext_content found in parsed structure");
    }

    #[test]
    fn test_document_with_setext_headers() {
        let input = "First Header\n============\n\nSecond Header\n-------------\n\nRegular text.";

        let result = parse_document(input);
        assert!(result.is_ok(), "Should parse document with setext headers");

        let pairs = result.unwrap();

        // Debug: print the full document structure
        for pair in pairs {
            println!("Document structure:");
            print_parser_structure(pair, 0);
        }
    }

    #[test]
    fn test_setext_vs_atx_headers() {
        // Test that setext and ATX headers both work
        let setext_input = "Setext Header\n=============";
        let atx_input = "# ATX Header";

        // Parse both
        let setext_result = parse_with_rule(setext_input, Rule::setext_h1);
        let atx_result = parse_with_rule(atx_input, Rule::H1);

        assert!(setext_result.is_ok(), "Should parse setext header");
        assert!(atx_result.is_ok(), "Should parse ATX header");
    }

    #[test]
    fn test_marco_engine_setext_rendering() {
        // Test the actual HTML rendering through Marco engine
        let setext_h1 = "Test Header H1\n==============";
        let setext_h2 = "Test Header H2\n--------------";

        let h1_result = parse_to_html_cached(setext_h1);
        let h2_result = parse_to_html_cached(setext_h2);

        assert!(h1_result.is_ok(), "Should render setext H1");
        assert!(h2_result.is_ok(), "Should render setext H2");

        let h1_html = h1_result.unwrap();
        let h2_html = h2_result.unwrap();

        // Check that underlines are not in the HTML output
        assert!(
            !h1_html.contains("=============="),
            "H1 HTML should not contain underline markers"
        );
        assert!(
            !h2_html.contains("--------------"),
            "H2 HTML should not contain underline markers"
        );

        // Check proper header tags and content
        assert!(
            h1_html.contains("<h1>Test Header H1</h1>"),
            "Should contain clean H1"
        );
        assert!(
            h2_html.contains("<h2>Test Header H2</h2>"),
            "Should contain clean H2"
        );
    }

    #[test]
    fn test_parser_error_handling_with_parse_result() {
        // Test using ParseResult for consistent error handling
        let valid_input = "# Valid Header";
        let malformed_input = "```\nUnclosed code block";

        // Test success case with clean error type
        let success: ParseResult<_> = parse_document(valid_input);
        assert!(success.is_ok(), "Valid input should parse successfully");

        // Test that we can handle errors uniformly
        let error_result: ParseResult<_> = parse_with_rule(malformed_input, Rule::code_block);
        match error_result {
            Ok(_) => println!("Parsing succeeded unexpectedly"),
            Err(e) => println!("Expected parse error: {}", e), // ParseResult provides String errors
        }
    }

    #[test]
    fn test_grammar_rule_validation_suite() {
        // Comprehensive test suite for various grammar rules using the new functions
        let test_cases = vec![
            ("# ATX Header", Rule::H1, true),
            ("## ATX H2", Rule::H2, true),
            ("Setext H1\n=========", Rule::setext_h1, true),
            ("Setext H2\n---------", Rule::setext_h2, true),
            ("**bold text**", Rule::bold, true),
            ("~~strikethrough~~", Rule::strikethrough, true),
            ("==highlight==", Rule::highlight, true),
            ("^superscript^", Rule::superscript, true),
            ("~subscript~", Rule::subscript, true),
            ("Invalid header\n===", Rule::H1, false), // Should fail for H1 rule
        ];

        for (input, rule, should_succeed) in test_cases {
            let result = parse_with_rule(input, rule);
            if should_succeed {
                assert!(
                    result.is_ok(),
                    "Expected '{}' to parse successfully with rule {:?}",
                    input,
                    rule
                );
            } else {
                assert!(
                    result.is_err(),
                    "Expected '{}' to fail parsing with rule {:?}",
                    input,
                    rule
                );
            }
        }
    }

    #[test]
    fn test_parse_document_comprehensive() {
        // Test the parse_document function with complex markdown
        let complex_markdown = r#"# Main Title

This is a paragraph with **bold text** and *emphasis*.

## Subsection

- List item 1
- List item 2 with `inline code`

```rust
fn example() {
    println!("Hello, world!");
}
```

> This is a blockquote with ==highlighted text==.
"#;

        let result: ParseResult<_> = parse_document(complex_markdown);
        assert!(result.is_ok(), "Complex document should parse successfully");

        let pairs = result.unwrap();
        let pair_count = pairs.count();
        assert!(pair_count > 0, "Document should contain parsed elements");
    }

    /// Grammar testing utility function
    fn test_grammar_rule(input: &str, rule: Rule, expected_success: bool) -> ParseResult<()> {
        let result = parse_with_rule(input, rule);
        match (result.is_ok(), expected_success) {
            (true, true) => Ok(()),
            (false, false) => Ok(()),
            (true, false) => Err(format!(
                "Expected '{}' to fail with rule {:?}, but it succeeded",
                input, rule
            )),
            (false, true) => Err(format!(
                "Expected '{}' to succeed with rule {:?}, but it failed: {}",
                input,
                rule,
                result.unwrap_err()
            )),
        }
    }

    #[test]
    fn test_marco_specific_syntax() {
        // Test Marco-specific extensions using the parser functions
        let test_cases = vec![
            // Basic Marco syntax - no spaces allowed in superscript/subscript
            ("^superscript^", Rule::superscript, true),
            ("~subscript~", Rule::subscript, true),
            ("˅subscript˅", Rule::subscript, true),
            ("==highlight text==", Rule::highlight, true),
            // Edge cases
            ("^", Rule::superscript, false), // Incomplete superscript
            ("~", Rule::subscript, false),   // Incomplete subscript
            ("=", Rule::highlight, false),   // Incomplete highlight
            ("^superscript text^", Rule::superscript, false), // Spaces not allowed in superscript
        ];

        for (input, rule, should_succeed) in test_cases {
            if let Err(e) = test_grammar_rule(input, rule, should_succeed) {
                panic!("Grammar test failed: {}", e);
            }
        }
    }

    #[test]
    fn test_parser_performance_with_parse_result() {
        // Test parser performance and consistent error handling
        use std::time::Instant;

        let large_document = "# Header\n\n".repeat(1000)
            + &"This is a paragraph with **bold** text.\n\n".repeat(500)
            + &"- List item\n".repeat(200);

        let start = Instant::now();
        let result: ParseResult<_> = parse_document(&large_document);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Large document should parse successfully");
        println!("Parsed large document in {:?}", duration);

        // Test that ParseResult provides consistent error messages
        let invalid_input = "```\nUnclosed code block without proper ending";
        let error_result: ParseResult<_> = parse_with_rule(invalid_input, Rule::code_block);

        if let Err(error_msg) = error_result {
            assert!(
                error_msg.contains("expected"),
                "Error message should be descriptive"
            );
            println!("Error message format: {}", error_msg);
        }
    }

    // Helper function to debug parser structure
    fn print_parser_structure(pair: pest::iterators::Pair<Rule>, indent: usize) {
        let indent_str = "  ".repeat(indent);
        println!(
            "{}Rule: {:?}, Text: {:?}",
            indent_str,
            pair.as_rule(),
            pair.as_str()
        );

        for inner_pair in pair.into_inner() {
            print_parser_structure(inner_pair, indent + 1);
        }
    }
}

#[test]
fn test_marco_test_binary_basic_functionality() {
    let output = Command::new("./target/debug/marco-test")
        .args([
            "string",
            "# Hello World",
            "--expected",
            "<h1>Hello World</h1>",
        ])
        .output()
        .expect("Failed to execute marco-test");

    assert!(
        output.status.success(),
        "marco-test should pass for correct input"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓ Test passed!"),
        "Should show success message"
    );
}

#[test]
fn test_marco_test_binary_failure_case() {
    let output = Command::new("./target/debug/marco-test")
        .args([
            "string",
            "# Hello World",
            "--expected",
            "<h2>Hello World</h2>",
        ])
        .output()
        .expect("Failed to execute marco-test");

    assert!(
        !output.status.success(),
        "marco-test should fail for incorrect input"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✗ Test failed!"),
        "Should show failure message"
    );
    assert!(
        stdout.contains("Similarity"),
        "Should show similarity percentage"
    );
}

#[test]
fn test_marco_engine_smoke_test() {
    // Basic smoke test for Marco engine through test runner
    use marco_core::components::marco_engine::parse_to_html_cached;

    let result = parse_to_html_cached("# Test Header");
    assert!(result.is_ok(), "Marco engine should parse basic markdown");
    assert!(result.unwrap().contains("h1"), "Should produce header HTML");
}
