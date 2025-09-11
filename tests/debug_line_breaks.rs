use marco::components::marco_engine::{AstBuilder, HtmlOptions, HtmlRenderer, MarcoParser};
use pest::Parser;

fn main() {
    // Test hard line break (2 spaces + newline) in normal mode
    let mut options = HtmlOptions::default();
    options.line_break_mode = "normal".to_string();

    let input = "Line one  \nLine two";
    println!("Input: {:?}", input);

    match MarcoParser::parse(marco::components::marco_engine::Rule::document, input) {
        Ok(pairs) => {
            println!("Parsed successfully");
            for pair in pairs.clone() {
                println!("Rule: {:?}, Text: {:?}", pair.as_rule(), pair.as_str());
                for inner_pair in pair.into_inner() {
                    println!(
                        "  Inner Rule: {:?}, Text: {:?}",
                        inner_pair.as_rule(),
                        inner_pair.as_str()
                    );
                    for inner_inner_pair in inner_pair.into_inner() {
                        println!(
                            "    Inner Inner Rule: {:?}, Text: {:?}",
                            inner_inner_pair.as_rule(),
                            inner_inner_pair.as_str()
                        );
                    }
                }
            }

            let pairs =
                MarcoParser::parse(marco::components::marco_engine::Rule::document, input).unwrap();
            let ast = AstBuilder::build(pairs).unwrap();
            println!("AST built successfully");
            println!("AST: {:#?}", ast);

            let renderer = HtmlRenderer::new(options);
            let html = renderer.render(&ast);
            println!("HTML: {}", html);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}
