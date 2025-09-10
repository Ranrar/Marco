//! Simple demonstration of LRU cache benefits

use marco::components::marco_engine::parser::{EnhancedMarcoParser, ParserConfig};
use std::time::Instant;

#[test]
fn demo_lru_cache_benefits() {
    println!("🧪 LRU Cache Benefits Demonstration\n");

    // Parser with LRU cache enabled
    let mut cached_parser = EnhancedMarcoParser::with_config(ParserConfig {
        track_positions: false,
        enable_cache: true,
        max_cache_size: 50,
        detailed_errors: false,
        collect_stats: true,
    });

    // Parser without cache for comparison
    let mut uncached_parser = EnhancedMarcoParser::with_config(ParserConfig {
        track_positions: false,
        enable_cache: false,
        max_cache_size: 0,
        detailed_errors: false,
        collect_stats: true,
    });

    let test_input =
        "# Main Heading\n\nThis is a **complex** paragraph with *italic* text and `code` elements.";

    println!("📄 Test input: \"{}\"", test_input);
    println!("🔧 Parsing with heading rule...\n");

    // Warm up - first parse with cache
    let _warmup = cached_parser.parse_with_rule("heading", test_input);

    // Measure cached performance
    let start = Instant::now();
    let cached_result = cached_parser.parse_with_rule("heading", test_input);
    let cached_time = start.elapsed();

    // Measure uncached performance
    let start = Instant::now();
    let uncached_result = uncached_parser.parse_with_rule("heading", test_input);
    let uncached_time = start.elapsed();

    match (cached_result.nodes, uncached_result.nodes) {
        (Ok(_), Ok(_)) => {
            println!("✅ Both parses successful!");
            println!("🏃 Uncached time: {:?}", uncached_time);
            println!("🚀 Cached time:   {:?}", cached_time);
            println!("📊 Cache hit: {}", cached_result.stats.cache_hit);

            if cached_time < uncached_time {
                let speedup = uncached_time.as_nanos() as f64 / cached_time.as_nanos() as f64;
                println!("🎯 Speedup: {:.1}x faster with LRU cache!", speedup);
            } else {
                println!("ℹ️  Cache overhead minimal for simple cases");
            }

            let cache_stats = cached_parser.cache_stats();
            println!(
                "📈 Cache stats: {} items, max {}, enabled: {}",
                cache_stats.size, cache_stats.max_size, cache_stats.enabled
            );
        }
        _ => {
            println!("❌ Parse errors occurred");
        }
    }

    println!("\n✅ LRU Cache demonstration completed!");
}
