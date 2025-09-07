use crate::components::marco_engine::grammar::{MarcoParser, Rule};
use pest::Parser;

#[test]
fn test_simple_heading_parsing() {
    // Test parsing a simple heading with H1 rule
    let input = "# Hello World";
    let pairs = MarcoParser::parse(Rule::H1, input);
    println!("H1 parse result: {:?}", pairs);

    // Test parsing with heading rule
    let pairs = MarcoParser::parse(Rule::heading, input);
    println!("Heading parse result: {:?}", pairs);

    // Test parsing text
    let input_text = "regular text";
    let pairs = MarcoParser::parse(Rule::text, input_text);
    println!("Text parse result: {:?}", pairs);

    // Test parsing bold
    let input_bold = "**bold text**";
    let pairs = MarcoParser::parse(Rule::bold, input_bold);
    println!("Bold parse result: {:?}", pairs);
}

#[test]
fn test_file_parsing_structure() {
    let input = "# Hello World\n\nThis is a **bold** test.";

    // Test what the file rule actually parses
    let pairs = MarcoParser::parse(Rule::file, input).expect("Failed to parse file");

    println!("=== FILE RULE OUTPUT ===");
    for pair in pairs {
        println!("Top level: {:?}", pair.as_rule());
        println!("Content: {:?}", pair.as_str());

        for inner in pair.into_inner() {
            println!("  Inner: {:?} -> {:?}", inner.as_rule(), inner.as_str());

            for inner2 in inner.into_inner() {
                println!(
                    "    Inner2: {:?} -> {:?}",
                    inner2.as_rule(),
                    inner2.as_str()
                );

                for inner3 in inner2.into_inner() {
                    println!(
                        "      Inner3: {:?} -> {:?}",
                        inner3.as_rule(),
                        inner3.as_str()
                    );
                }
            }
        }
    }
}
