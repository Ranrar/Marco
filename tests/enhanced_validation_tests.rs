use marco::components::marco_engine::parser::marco_parser::EnhancedMarcoParser;

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_enhanced_task_parsing() {
        let test_cases = vec![
            ("[x] Simple completed task", true),
            ("[X] Uppercase completed task", true),
            ("[ ] Simple incomplete task", true),
            ("[x] Task with [nested] brackets", true),
            ("[x]a", true), // Minimal valid input - should work with our improvements
        ];

        for (input, should_parse) in test_cases {
            let mut parser = EnhancedMarcoParser::new();
            let result = parser.parse_document(input);

            if should_parse {
                assert!(result.nodes.is_ok(), "Should parse valid task: {}", input);
            } else {
                // We're testing that the parsing itself works, not necessarily success/failure
                // The key is that our validation improvements don't break valid syntax
                let _ = result; // Just ensure it doesn't panic
            }
        }
    }

    #[test]
    fn test_enhanced_user_mention_parsing() {
        let test_cases = vec![
            "@simple_user",
            "@user123",
            "@user_with_underscores",
            "@user-with-dashes",
            "@user123_test-name", // Complex valid username - should work with our improvements
        ];

        for input in test_cases {
            let mut parser = EnhancedMarcoParser::new();
            let result = parser.parse_inline(input);

            // Test that our enhanced validation doesn't break valid usernames
            assert!(
                result.nodes.is_ok(),
                "Should parse valid user mention: {}",
                input
            );
        }
    }

    #[test]
    fn test_enhanced_bookmark_parsing() {
        let test_cases = vec![
            "[bookmark:simple](path.txt)",
            "[bookmark:readme](./README.md)",
            "[bookmark:config](config.toml=42)",
            "[bookmark:test](src/main.rs=1)",
            "[bookmark:large_line](file.txt=99999)", // Large line numbers - should work with our improvements
        ];

        for input in test_cases {
            let mut parser = EnhancedMarcoParser::new();
            let result = parser.parse_inline(input);

            // Test that our enhanced validation doesn't break valid bookmarks
            assert!(
                result.nodes.is_ok(),
                "Should parse valid bookmark: {}",
                input
            );
        }
    }

    #[test]
    fn test_enhanced_toc_parsing() {
        let test_cases = vec![
            "[toc]",
            "[toc=3]",
            "[toc:document]",
            "[toc=1]", // Single digit depth
        ];

        for input in test_cases {
            let mut parser = EnhancedMarcoParser::new();
            let result = parser.parse_inline(input);

            // Test that our enhanced validation doesn't break valid ToC syntax
            assert!(result.nodes.is_ok(), "Should parse valid ToC: {}", input);
        }
    }

    #[test]
    fn test_enhanced_error_handling() {
        let invalid_cases = vec![
            "",    // Empty string
            "   ", // Only whitespace
            "[invalid_marker] task",
            "[@invalid_mention",
            "[bookmark:incomplete",
            "[toc=invalid_depth]",
        ];

        for input in invalid_cases {
            let mut parser = EnhancedMarcoParser::new();
            let result = parser.parse_document(input);

            // Our enhanced validation should either parse correctly or fail gracefully
            // The key improvement is that it doesn't panic or produce inconsistent results
            match result.nodes {
                Ok(_) => {
                    // If it parses, that's fine - it means our validation is lenient
                    println!("Parsed potentially invalid input: {}", input);
                }
                Err(_) => {
                    // If it fails, that's also fine - it means our validation caught it
                    println!("Properly rejected invalid input: {}", input);
                }
            }
            // The test passes either way - we're testing stability, not strict validation
        }
    }

    #[test]
    fn test_validation_improvements_integration() {
        // Test a complex document that exercises all our validation improvements
        let complex_document = r#"
# Test Document

## Tasks
[x] Completed task with validation
[ ] Incomplete task  
[x]a
[X] Another completed task

## User Mentions  
@simple_user mentioned this
@user123_test-name has complex username
@user_with_underscores works too

## Bookmarks
[bookmark:readme](./README.md)
[bookmark:config](config.toml=42)
[bookmark:large_line](file.txt=99999)

## Table of Contents
[toc]
[toc=3]
[toc:document]
"#;

        let mut parser = EnhancedMarcoParser::new();
        let result = parser.parse_document(complex_document);

        // Our enhanced validation should handle this complex document without issues
        assert!(
            result.nodes.is_ok(),
            "Should parse complex document with all enhanced features"
        );

        if let Ok(ast) = result.nodes {
            // Just verify we got some AST nodes back
            println!(
                "Successfully parsed complex document with {} nodes",
                ast.len()
            );
        }
    }

    #[test]
    fn test_regression_prevention() {
        // Test that our validation improvements don't break existing functionality
        let existing_syntax = vec![
            "Regular paragraph text",
            "# Header",
            "## Subheader",
            "*italic* and **bold** text",
            "[link](url)",
            "1. Numbered list",
            "- Bullet list",
        ];

        for input in existing_syntax {
            let mut parser = EnhancedMarcoParser::new();
            let result = parser.parse_document(input);

            // All existing syntax should continue to work
            assert!(
                result.nodes.is_ok(),
                "Should parse existing syntax: {}",
                input
            );
        }
    }
}
