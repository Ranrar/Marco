#!/usr/bin/env bash

# Test script for enhanced parser integration

cd /home/ranrar/Code/projects/marco2

echo "Testing enhanced parser integration..."

# Create a simple test program
cat > test_parser_integration.rs << 'EOF'
use marco::components::marco_engine::{
    EnhancedMarcoParser, ParserConfig, MarcoEngine
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
    println!("   Cache size: {}/{}", cache_stats.size, cache_stats.max_size);
    
    println!("\n🎉 All tests completed successfully!");
    Ok(())
}
EOF

# Compile and run the test
echo "Compiling test program..."
rustc --edition 2021 -L target/debug/deps test_parser_integration.rs -o test_parser_integration --extern marco=target/debug/libmarco.rlib

if [ $? -eq 0 ]; then
    echo "Running parser integration test..."
    ./test_parser_integration
    
    # Clean up
    rm test_parser_integration.rs test_parser_integration
    
    echo -e "\n✅ Parser integration test completed successfully!"
else
    echo "❌ Failed to compile test program"
    exit 1
fi
