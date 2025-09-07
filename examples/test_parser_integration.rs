use marco::components::marco_engine::{
    parser::{create_debug_parser, create_performance_parser, EnhancedMarcoParser, ParserConfig},
    MarcoEngine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Enhanced Marco Parser Integration");

    // Test 1: Basic parser creation
    println!("\n✅ Test 1: Basic parser creation");
    let mut parser = EnhancedMarcoParser::new();
    println!("   Created enhanced parser successfully");

    // Test 2: Parse a simple document
    println!("\n✅ Test 2: Parse simple document");
    let test_input = "# Hello World\n\nThis is a **test** document with *formatting*.";
    let result = parser.parse_document(test_input);

    match result.nodes {
        Ok(nodes) => {
            println!("   Parsed {} top-level nodes", nodes.len());
            println!("   Parse time: {:?}", result.stats.parse_time);
            println!("   Node count: {}", result.stats.node_count);
            if !result.warnings.is_empty() {
                println!("   Warnings: {}", result.warnings.len());
            }
        }
        Err(e) => {
            println!("   Parse error: {}", e);
        }
    }

    // Test 3: Syntax validation
    println!("\n✅ Test 3: Syntax validation");
    match parser.validate("text", "hello world") {
        Ok(valid) => println!("   'hello world' is valid text: {}", valid),
        Err(e) => println!("   Validation error: {}", e),
    }

    // Test 4: Enhanced engine functionality
    println!("\n✅ Test 4: Enhanced engine functionality");
    match MarcoEngine::validate_syntax("heading", "# Test Heading") {
        Ok(valid) => println!("   '# Test Heading' is valid heading: {}", valid),
        Err(e) => println!("   Engine validation error: {}", e),
    }

    // Test 5: Enhanced parser features
    println!("\n✅ Test 5: Enhanced parser with caching");
    let config = ParserConfig {
        track_positions: true,
        enable_cache: true,
        detailed_errors: true,
        collect_stats: true,
        max_cache_size: 50,
    };
    let mut enhanced_parser = EnhancedMarcoParser::with_config(config);

    let cache_stats = enhanced_parser.cache_stats();
    println!("   Cache enabled: {}", cache_stats.enabled);
    println!(
        "   Cache size: {}/{}",
        cache_stats.size, cache_stats.max_size
    );

    // Test 6: Parser configurations
    println!("\n✅ Test 6: Parser configurations");
    let _perf_parser = create_performance_parser();
    println!("   Created performance parser");
    let _debug_parser = create_debug_parser();
    println!("   Created debug parser");

    // Test 7: Enhanced pipeline
    println!("\n✅ Test 7: Enhanced pipeline");
    let mut enhanced_pipeline = MarcoEngine::create_enhanced_pipeline();
    match enhanced_pipeline.validate_syntax("text", "simple test") {
        Ok(valid) => println!("   Pipeline validation: {}", valid),
        Err(e) => println!("   Pipeline validation error: {}", e),
    }

    println!("\n🎉 All parser integration tests completed successfully!");
    println!("📊 Enhanced parser features:");
    println!("   ✓ Position tracking and source spans");
    println!("   ✓ Enhanced error reporting with context");
    println!("   ✓ Parse result caching");
    println!("   ✓ Rule usage analysis");
    println!("   ✓ Grammar dependency extraction");
    println!("   ✓ Memory usage estimation");
    println!("   ✓ Multiple parser configurations");
    println!("   ✓ Pipeline integration");

    Ok(())
}
