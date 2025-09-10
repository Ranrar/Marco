//! Test for LRU cache functionality and performance

use marco::components::marco_engine::parser::{EnhancedMarcoParser, ParserConfig};
use std::time::Instant;

#[test]
fn test_lru_cache_functionality() {
    let mut parser = EnhancedMarcoParser::with_config(ParserConfig {
        track_positions: false,
        enable_cache: true,
        max_cache_size: 3, // Small cache for testing
        detailed_errors: false,
        collect_stats: true,
    });

    println!("🧪 Testing LRU Cache functionality");

    // Test items with identical rule but different content
    let test_cases = [
        ("paragraph", "First paragraph content"),
        ("paragraph", "Second paragraph content"),
        ("paragraph", "Third paragraph content"),
        ("paragraph", "Fourth paragraph content"), // This should evict the first item
    ];

    println!("📝 Initial cache size: {}", parser.cache_stats().size);

    // Add items to cache
    for (i, (rule, input)) in test_cases.iter().enumerate() {
        let start = Instant::now();
        let result = parser.parse_with_rule(rule, input);
        let parse_time = start.elapsed();

        match result.nodes {
            Ok(_nodes) => {
                let cache_hit = result.stats.cache_hit;
                println!(
                    "   Item {}: {} - Cache: {} - Time: {:?}",
                    i + 1,
                    if cache_hit { "HIT" } else { "MISS" },
                    if cache_hit { "✅" } else { "❌" },
                    parse_time
                );
            }
            Err(e) => {
                println!("   Item {}: ERROR - {}", i + 1, e);
            }
        }

        let stats = parser.cache_stats();
        println!("      Cache size: {}/{}", stats.size, stats.max_size);
    }

    // Test LRU eviction by accessing items again
    println!("\n🔄 Testing LRU eviction by re-accessing items:");

    // Access first item again (should be cache miss due to eviction)
    let result1 = parser.parse_with_rule("paragraph", "First paragraph content");
    println!(
        "   First item (should be evicted): Cache {}",
        if result1.stats.cache_hit {
            "HIT ✅"
        } else {
            "MISS ❌"
        }
    );

    // Access fourth item again (should be cache hit as it was most recent)
    let result4 = parser.parse_with_rule("paragraph", "Fourth paragraph content");
    println!(
        "   Fourth item (should be cached): Cache {}",
        if result4.stats.cache_hit {
            "HIT ✅"
        } else {
            "MISS ❌"
        }
    );

    println!("\n📊 Final cache stats: {:?}", parser.cache_stats());

    // Verify LRU behavior - adjust expectations based on LRU semantics
    assert!(
        !result1.stats.cache_hit,
        "First item should have been evicted from LRU cache"
    );
    assert!(
        result4.stats.cache_hit,
        "Fourth item should still be cached as it was most recent"
    );

    println!("✅ LRU cache test completed successfully!");
}

#[test]
fn test_cache_performance_improvement() {
    let mut parser = EnhancedMarcoParser::with_config(ParserConfig {
        track_positions: false,
        enable_cache: true,
        max_cache_size: 100,
        detailed_errors: false,
        collect_stats: true,
    });

    println!("🚀 Testing cache performance improvement");

    let test_input = "# This is a complex heading with **bold** and *italic* text";

    // First parse (cache miss)
    let start1 = Instant::now();
    let result1 = parser.parse_with_rule("heading", test_input);
    let time1 = start1.elapsed();

    // Second parse (cache hit)
    let start2 = Instant::now();
    let result2 = parser.parse_with_rule("heading", test_input);
    let time2 = start2.elapsed();

    match (result1.nodes, result2.nodes) {
        (Ok(_), Ok(_)) => {
            println!("   First parse (miss): {:?}", time1);
            println!("   Second parse (hit): {:?}", time2);
            println!("   Cache hit 1: {}", result1.stats.cache_hit);
            println!("   Cache hit 2: {}", result2.stats.cache_hit);

            // Verify cache behavior
            assert!(!result1.stats.cache_hit, "First parse should be cache miss");
            assert!(result2.stats.cache_hit, "Second parse should be cache hit");

            // Cache hit should generally be faster (though timing can vary)
            if time2 < time1 {
                println!(
                    "   ✅ Cache provided speedup: {}x faster",
                    time1.as_nanos() as f64 / time2.as_nanos() as f64
                );
            } else {
                println!("   ℹ️  Cache overhead present (normal for simple cases)");
            }
        }
        _ => panic!("Parse errors occurred"),
    }

    println!("✅ Cache performance test completed!");
}
