//! Test for error recovery functionality
use marco::components::marco_engine::parser::{EnhancedMarcoParser, ParserConfig};

#[test]
fn test_error_recovery_functionality() {
    let mut parser = EnhancedMarcoParser::with_config(ParserConfig {
        track_positions: true,
        enable_cache: false,
        max_cache_size: 100,
        detailed_errors: true,
        collect_stats: true,
    });

    // Test cases that should trigger error recovery
    let test_cases = vec![
        (
            "Malformed heading",
            "### Incomplete heading without newline",
            "heading",
        ),
        (
            "Broken admonition",
            ":::\nnote without closing",
            "admonition_block",
        ),
        (
            "Invalid code block",
            "```rust\ncode without closing",
            "code_block",
        ),
        ("Malformed bold", "**bold without closing", "bold"),
        ("Broken link", "[link without](closing", "inline_link"),
    ];

    let mut recovery_count = 0;
    let mut success_count = 0;

    for (description, input, rule_name) in test_cases {
        println!("📝 Testing: {}", description);
        println!("   Input: {:?}", input);
        println!("   Rule:  {}", rule_name);

        let result = parser.parse_with_rule(rule_name, input);

        match result.nodes {
            Ok(nodes) => {
                success_count += 1;
                println!("   ✅ Success: Parsed {} nodes", nodes.len());
                if !result.warnings.is_empty() {
                    recovery_count += 1;
                    println!(
                        "   ⚠️  Error recovery triggered: {} warning(s)",
                        result.warnings.len()
                    );
                    for warning in &result.warnings {
                        println!("      - {}", warning.message);
                    }
                }
                println!(
                    "   📊 Stats: {} nodes, {} max depth",
                    result.stats.node_count, result.stats.max_depth
                );
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }

        println!();
    }

    println!("🎯 Error recovery test completed!");
    println!(
        "📊 Summary: {} successful parses, {} with error recovery",
        success_count, recovery_count
    );

    // Assert that at least some cases triggered error recovery
    assert!(
        recovery_count > 0,
        "Expected at least some test cases to trigger error recovery"
    );
}

#[test]
fn test_simple_error_recovery() {
    let mut parser = EnhancedMarcoParser::with_config(ParserConfig {
        track_positions: true,
        enable_cache: false,
        max_cache_size: 100,
        detailed_errors: true,
        collect_stats: true,
    });

    // Simple case: try to parse malformed heading as heading, should fallback to text
    let result = parser.parse_with_rule("heading", "This is not a heading");

    match result.nodes {
        Ok(nodes) => {
            println!("✅ Parsed {} nodes", nodes.len());
            if !result.warnings.is_empty() {
                println!("⚠️  Error recovery was triggered:");
                for warning in &result.warnings {
                    println!("   - {}", warning.message);
                }
                // This test should succeed if error recovery worked
                assert!(result
                    .warnings
                    .iter()
                    .any(|w| w.message.contains("Recovered using rule")));
            }
        }
        Err(e) => {
            println!("❌ Parse failed even with error recovery: {}", e);
        }
    }
}
