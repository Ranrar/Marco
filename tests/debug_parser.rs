use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/assets/markdown_schema/Marco/markdown.pest"]
pub struct MarkdownParser;

fn main() {
    let inputs = vec![
        "# Hello World",
        "# Hello World\n",
        "# Hello World\n\n",
        "# Hello World\n\nParagraph",
        "# Hello World\n\nParagraph\n",
    ];

    for input in inputs {
        println!("\n=== Testing input: {:?} ===", input);
        match MarkdownParser::parse(Rule::document, input) {
            Ok(pairs) => {
                for pair in pairs {
                    println!("Success: {:?}", pair);
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }

        // Also try parsing as individual blocks
        println!("--- Testing as individual rules ---");
        match MarkdownParser::parse(Rule::atx_heading, "# Hello World\n") {
            Ok(pairs) => {
                for pair in pairs {
                    println!("ATX Heading Success: {:?}", pair);
                }
            }
            Err(e) => {
                println!("ATX Heading Error: {:?}", e);
            }
        }
    }
}
