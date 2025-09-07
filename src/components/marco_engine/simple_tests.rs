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
