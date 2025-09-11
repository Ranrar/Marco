use std::path::Path;

// Add this to the root of the project for quick debugging

fn main() {
    println!("Line Break Debug Test");
    
    // Test case 1: Single newline (should be soft break)
    let input1 = "Line one\nLine two";
    println!("\n=== Test 1: Single newline ===");
    println!("Input: {:?}", input1);
    test_line_breaks(input1, "normal");
    test_line_breaks(input1, "reversed");
    
    // Test case 2: Two spaces + newline (should be hard break)
    let input2 = "Line one  \nLine two";
    println!("\n=== Test 2: Two spaces + newline ===");
    println!("Input: {:?}", input2);
    test_line_breaks(input2, "normal");
    test_line_breaks(input2, "reversed");
    
    // Test case 3: Backslash + newline (should be hard break)
    let input3 = "Line one\\\nLine two";
    println!("\n=== Test 3: Backslash + newline ===");
    println!("Input: {:?}", input3);
    test_line_breaks(input3, "normal");
    test_line_breaks(input3, "reversed");
}

fn test_line_breaks(input: &str, mode: &str) {
    use marco::components::marco_engine::{MarcoParser, Rule, AstBuilder, HtmlRenderer, HtmlOptions};
    use pest::Parser;
    
    let options = HtmlOptions {
        line_break_mode: mode.to_string(),
        ..Default::default()
    };
    
    match MarcoParser::parse(Rule::document, input) {
        Ok(pairs) => {
            match AstBuilder::build(pairs) {
                Ok(ast) => {
                    let renderer = HtmlRenderer::new(options);
                    let html = renderer.render(&ast);
                    println!("{} mode HTML: {}", mode, html);
                    
                    let has_br = html.contains("<br>");
                    println!("{} mode has <br>: {}", mode, has_br);
                }
                Err(e) => println!("AST build error: {:?}", e),
            }
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
}
