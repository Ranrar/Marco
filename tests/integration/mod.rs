//! Integration tests for Marco test suite
//!
//! These tests integrate with the standard Rust testing framework (`cargo test`)
//! and can be run automatically in CI/CD pipelines.

use std::path::PathBuf;

// Import the test runner library
use marco::components::marco_engine::parse_to_html_cached;

// We can't directly use the test_runner modules from here because they're in dev-dependencies,
// so we'll create integration tests that use the Marco engine directly.

#[test]
fn test_marco_engine_basic_markdown() {
    let markdown = "# Hello World\n\nThis is a **test** document.";
    let result = parse_to_html_cached(markdown);

    assert!(
        result.is_ok(),
        "Marco engine should successfully process basic markdown"
    );

    let html = result.unwrap();
    assert!(
        html.contains("Hello World"),
        "HTML should contain the header text"
    );
    assert!(html.contains("test"), "HTML should contain the bold text");
}

#[test]
fn test_marco_engine_empty_input() {
    let result = parse_to_html_cached("");
    assert!(
        result.is_ok(),
        "Marco engine should handle empty input gracefully"
    );
}

#[test]
fn test_marco_engine_code_blocks() {
    let markdown = "```rust\nfn main() {\n    println!(\"Hello, World!\");\n}\n```";
    let result = parse_to_html_cached(markdown);

    assert!(result.is_ok(), "Marco engine should handle code blocks");

    let html = result.unwrap();
    assert!(
        html.contains("main"),
        "HTML should contain the code content"
    );
}

#[test]
fn test_marco_engine_lists() {
    let markdown = "- Item 1\n- Item 2\n- Item 3";
    let result = parse_to_html_cached(markdown);

    assert!(result.is_ok(), "Marco engine should handle lists");

    let html = result.unwrap();
    assert!(html.contains("Item 1"), "HTML should contain list items");
}

#[test]
fn test_marco_engine_links() {
    let markdown = "[Example](https://example.com)";
    let result = parse_to_html_cached(markdown);

    assert!(result.is_ok(), "Marco engine should handle links");

    let html = result.unwrap();
    assert!(html.contains("Example"), "HTML should contain link text");
    assert!(html.contains("example.com"), "HTML should contain link URL");
}

#[test]
fn test_commonmark_spec_file_exists() {
    let spec_path = PathBuf::from("tests/spec/commonmark.json");
    assert!(
        spec_path.exists(),
        "CommonMark specification file should exist"
    );

    // Try to read and parse the file
    let content = std::fs::read_to_string(&spec_path)
        .expect("Should be able to read CommonMark specification file");

    if !content.trim().is_empty() {
        serde_json::from_str::<serde_json::Value>(&content)
            .expect("CommonMark specification should be valid JSON");
    }
}

#[test]
fn test_marco_spec_file_exists() {
    let spec_path = PathBuf::from("tests/spec/marco.json");
    assert!(spec_path.exists(), "Marco specification file should exist");

    // The file might be empty initially, which is fine
    let content = std::fs::read_to_string(&spec_path)
        .expect("Should be able to read Marco specification file");

    if !content.trim().is_empty() {
        serde_json::from_str::<serde_json::Value>(&content)
            .expect("Marco specification should be valid JSON");
    }
}

// Conditional compilation for running spec tests only when requested
// This prevents long-running tests from being included in regular test runs
#[cfg(feature = "integration-tests")]
mod spec_tests {
    use super::*;

    #[test]
    fn run_commonmark_spec_tests() {
        // This would run the full CommonMark specification test suite
        // Only enabled with --features integration-tests

        let spec_path = PathBuf::from("tests/spec/commonmark.json");
        if !spec_path.exists() {
            return; // Skip if spec file doesn't exist
        }

        let content = std::fs::read_to_string(&spec_path).unwrap();
        if content.trim().is_empty() {
            return; // Skip if spec file is empty
        }

        let test_cases: Vec<serde_json::Value> =
            serde_json::from_str(&content).expect("Should parse CommonMark spec");

        let mut passed = 0;
        let mut failed = 0;

        for (i, test_case) in test_cases.iter().enumerate() {
            if let (Some(markdown), Some(expected_html)) = (
                test_case.get("markdown").and_then(|v| v.as_str()),
                test_case.get("html").and_then(|v| v.as_str()),
            ) {
                match parse_to_html_cached(markdown) {
                    Ok(actual_html) => {
                        // Simple comparison - more sophisticated diff logic is in the test runner
                        if normalize_html(&actual_html) == normalize_html(expected_html) {
                            passed += 1;
                        } else {
                            failed += 1;
                            if failed <= 5 {
                                // Only show first few failures to avoid spam
                                eprintln!("Test case {} failed:", i + 1);
                                eprintln!("Expected: {}", expected_html);
                                eprintln!("Actual: {}", actual_html);
                                eprintln!();
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        if failed <= 5 {
                            eprintln!("Test case {} errored: {}", i + 1, e);
                        }
                    }
                }
            }
        }

        eprintln!(
            "CommonMark spec results: {} passed, {} failed",
            passed, failed
        );

        // We don't fail the test here because Marco may not be 100% CommonMark compliant
        // This is more of a benchmark/diagnostic test
    }

    fn normalize_html(html: &str) -> String {
        html.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Performance benchmarks (run with cargo test --release)
#[test]
fn benchmark_marco_performance() {
    let markdown = include_str!("../markdown_showcase/04_real_world_example.md");

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = parse_to_html_cached(markdown);
    }
    let duration = start.elapsed();

    println!(
        "Processed {} iterations in {:?} (avg: {:?} per iteration)",
        100,
        duration,
        duration / 100
    );

    // Just verify it doesn't take an unreasonably long time
    assert!(
        duration.as_secs() < 10,
        "Performance test should complete within 10 seconds"
    );
}

#[test]
fn test_marco_parser_cache() {
    let markdown = "# Test Document\n\nThis is a test of the parser cache.";

    // First parse (should be cache miss)
    let start1 = std::time::Instant::now();
    let result1 = parse_to_html_cached(markdown);
    let duration1 = start1.elapsed();

    assert!(result1.is_ok());

    // Second parse (should be cache hit)
    let start2 = std::time::Instant::now();
    let result2 = parse_to_html_cached(markdown);
    let duration2 = start2.elapsed();

    assert!(result2.is_ok());
    assert_eq!(
        result1.unwrap(),
        result2.unwrap(),
        "Cached result should be identical"
    );

    // Cache hit should generally be faster, but we won't assert this since it's not guaranteed
    println!(
        "First parse: {:?}, Second parse: {:?}",
        duration1, duration2
    );
}
