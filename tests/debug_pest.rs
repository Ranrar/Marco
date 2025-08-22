use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../src/assets/markdown_schema/Marco/markdown.pest"]
pub struct MarkdownParser;

fn main() {
    let input = "# Hello World";
    
    println!("Testing input: '{}'", input);
    
    // Test parsing specific rules
    println!("\n=== Testing atx_heading rule ===");
    match MarkdownParser::parse(Rule::atx_heading, input) {
        Ok(pairs) => {
            for pair in pairs {
                println!("SUCCESS: {:?} -> {:?}", pair.as_rule(), pair.as_str());
            }
        }
        Err(e) => println!("FAILED to parse as atx_heading: {}", e),
    }
    
    println!("\n=== Testing setext_heading rule ===");
    match MarkdownParser::parse(Rule::setext_heading, "Heading\n======") {
        Ok(pairs) => {
            for pair in pairs {
                println!("SUCCESS: {:?} -> {:?}", pair.as_rule(), pair.as_str());
            }
        }
        Err(e) => println!("FAILED to parse as setext_heading: {}", e),
    }
    
    println!("\n=== Testing paragraph rule ===");
    match MarkdownParser::parse(Rule::paragraph, input) {
        Ok(pairs) => {
            for pair in pairs {
                println!("SUCCESS: {:?} -> {:?}", pair.as_rule(), pair.as_str());
            }
        }
        Err(e) => println!("FAILED to parse as paragraph: {}", e),
    }
    
    println!("\n=== Testing block rule ===");
    match MarkdownParser::parse(Rule::block, input) {
        Ok(pairs) => {
            for pair in pairs {
                println!("SUCCESS: {:?} -> {:?}", pair.as_rule(), pair.as_str());
                for inner in pair.into_inner() {
                    println!("  Inner: {:?} -> {:?}", inner.as_rule(), inner.as_str());
                }
            }
        }
        Err(e) => println!("FAILED to parse as block: {}", e),
    }
}
