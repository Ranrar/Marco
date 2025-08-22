use pest::Parser;

// Import from the main project
use marco::components::marco_engine::parser::{MarkdownParser, Rule};

fn main() {
    println!("=== Testing space patterns ===");
    
    // Test different space patterns
    let test_cases = [
        "1. test\n",      // single space
        "1.  test\n",     // double space
        "1.\ttest\n",     // tab
        "1. \ttest\n",    // space + tab
    ];
    
    for test_case in test_cases {
        println!("\nTesting: {:?}", test_case);
        match MarkdownParser::parse(Rule::ordered_list_item, test_case) {
            Ok(pairs) => {
                println!("✅ Success:");
                for pair in pairs {
                    println!("  {:?}", pair);
                }
            }
            Err(e) => println!("❌ Failed: {}", e),
        }
    }
    
    println!("\n=== Testing parts manually ===");
    
    // Test ordered_marker
    match MarkdownParser::parse(Rule::ordered_marker, "1.") {
        Ok(_) => println!("✅ ordered_marker works"),
        Err(e) => println!("❌ ordered_marker failed: {}", e),
    }
    
    // Test space
    match MarkdownParser::parse(Rule::WHITESPACE, " ") {
        Ok(_) => println!("✅ WHITESPACE works"),
        Err(e) => println!("❌ WHITESPACE failed: {}", e),
    }
    
    // Test list_item_content
    match MarkdownParser::parse(Rule::list_item_content, "test") {
        Ok(_) => println!("✅ list_item_content works"),
        Err(e) => println!("❌ list_item_content failed: {}", e),
    }
    
    // Test NEWLINE
    match MarkdownParser::parse(Rule::NEWLINE, "\n") {
        Ok(_) => println!("✅ NEWLINE works"),
        Err(e) => println!("❌ NEWLINE failed: {}", e),
    }
}
