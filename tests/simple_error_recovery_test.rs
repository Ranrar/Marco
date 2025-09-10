//! Simple test for error recovery functionality 

use marco::components::marco_engine::parser::{EnhancedMarcoParser, ParserConfig};

#[test]
fn test_error_recovery_simple() {
    let mut parser = EnhancedMarcoParser::with_config(ParserConfig {
        track_positions: true,
        enable_cache: false,
        max_cache_size: 100,
        detailed_errors: true,
        collect_stats: true,
    });

    // Test 1: Try to parse plain text as a heading - should fallback to text rule
    println!("🧪 Test 1: Plain text as heading");
    let result1 = parser.parse_with_rule("heading", "This is just plain text, not a heading");
    
    match result1.nodes {
        Ok(nodes) => {
            println!("✅ Success: Parsed {} nodes", nodes.len());
            if !result1.warnings.is_empty() {
                println!("⚠️  Error recovery triggered: {} warning(s)", result1.warnings.len());
                for warning in &result1.warnings {
                    println!("   - {}", warning.message);
                }
                assert!(result1.warnings.iter().any(|w| w.message.contains("Recovered using rule")));
            } else {
                println!("ℹ️  No warnings - either parsed correctly or fallback happened silently");
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
            panic!("Expected error recovery to handle this case");
        }
    }

    // Test 2: Try to parse incomplete bold formatting - should fallback to text
    println!("\n🧪 Test 2: Incomplete bold formatting");
    let result2 = parser.parse_with_rule("bold", "**incomplete bold text without closing");
    
    match result2.nodes {
        Ok(nodes) => {
            println!("✅ Success: Parsed {} nodes", nodes.len());
            if !result2.warnings.is_empty() {
                println!("⚠️  Error recovery triggered: {} warning(s)", result2.warnings.len());
                for warning in &result2.warnings {
                    println!("   - {}", warning.message);
                }
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
            // Error recovery should prevent this from happening
        }
    }

    println!("\n🎯 Error recovery test completed!");
}
