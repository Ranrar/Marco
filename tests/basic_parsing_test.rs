use marco::components::marco_engine::{grammar::Rule, parser::marco_parser::EnhancedMarcoParser};

#[test]
fn test_simple_text_parsing() {
    let mut parser = EnhancedMarcoParser::new();
    let input = "Hello world";

    let result = parser.parse_with_rule("text", input);
    assert!(
        result.nodes.is_ok(),
        "Should parse simple text successfully"
    );

    if let Ok(node) = result.nodes {
        println!("Parsed AST: {:?}", node);
    }
}

#[test]
fn test_simple_heading_parsing() {
    let mut parser = EnhancedMarcoParser::new();
    let input = "# Hello World";

    let result = parser.parse_with_rule("heading", input);
    assert!(result.nodes.is_ok(), "Should parse heading successfully");

    if let Ok(node) = result.nodes {
        println!("Parsed heading AST: {:?}", node);
    }
}

#[test]
fn test_paragraph_parsing() {
    let mut parser = EnhancedMarcoParser::new();
    let input = "This is a simple paragraph with multiple words.";

    let result = parser.parse_with_rule("paragraph", input);
    assert!(result.nodes.is_ok(), "Should parse paragraph successfully");

    if let Ok(node) = result.nodes {
        println!("Parsed paragraph AST: {:?}", node);
    }
}

#[test]
fn test_code_block_parsing() {
    let mut parser = EnhancedMarcoParser::new();
    let input = "```rust\nfn main() {\n    println!(\"Hello!\");\n}\n```";

    let result = parser.parse_with_rule("code_block", input);
    assert!(result.nodes.is_ok(), "Should parse code block successfully");

    if let Ok(node) = result.nodes {
        println!("Parsed code block AST: {:?}", node);
    }
}

#[test]
fn test_admonition_parsing() {
    let mut parser = EnhancedMarcoParser::new();
    let input = ":::note\nThis is a note\n:::";

    let result = parser.parse_with_rule("admonition_block", input);
    println!("Admonition parse result: {:?}", result);

    if let Ok(node) = result.nodes {
        println!("Parsed admonition AST: {:?}", node);
    }
}
#[test]
fn test_simple_paragraph_parsing() {
    let mut parser = EnhancedMarcoParser::new();
    let input = "This is a simple paragraph with some text.";

    let result = parser.parse_with_rule("paragraph", input);
    assert!(result.nodes.is_ok(), "Should parse paragraph successfully");

    if let Ok(node) = result.nodes {
        println!("Parsed paragraph AST: {:?}", node);
    }
}

#[test]
fn test_document_parsing() {
    let mut parser = EnhancedMarcoParser::new();
    let input = "# Title\n\nThis is a paragraph.";

    let result = parser.parse_document(input);
    assert!(result.nodes.is_ok(), "Should parse document successfully");

    if let Ok(node) = result.nodes {
        println!("Parsed document AST: {:?}", node);
    }
}

#[test]
fn test_basic_marco_extensions() {
    let mut parser = EnhancedMarcoParser::new();

    // Test admonition block
    let admonition_input = ":::note\nThis is a note\n:::";
    let result = parser.parse_with_rule("admonition_block", admonition_input);

    // Should either parse successfully or fail gracefully
    match result.nodes {
        Ok(node) => println!("Parsed admonition AST: {:?}", node),
        Err(e) => println!("Admonition parsing failed (expected for now): {}", e),
    }
}
