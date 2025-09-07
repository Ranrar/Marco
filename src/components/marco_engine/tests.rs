use crate::components::marco_engine::{
    ast::AstBuilder,
    grammar::{MarcoParser, Rule},
    render::{markdown_to_html, HtmlOptions, MarkdownExtensions, MarkdownOptions},
    render::{HtmlRenderer, JsonRenderer, TextOptions, TextRenderer},
};
use pest::Parser;

#[test]
fn test_marco_basic_parsing() {
    let input = "# Hello World\n\nThis is a **bold** test.";

    // Test pest parsing with the file rule
    let pairs = MarcoParser::parse(Rule::file, input).expect("Failed to parse");

    // Test AST building
    let ast = AstBuilder::build(pairs).expect("Failed to build AST");

    // Test HTML rendering
    let html_renderer = HtmlRenderer::new(HtmlOptions::default());
    let html = html_renderer.render(&ast);

    println!("HTML output: {}", html);
    println!("AST: {:?}", ast);

    assert!(html.contains("<h1 class=\"marco-heading-1\">"));
    assert!(html.contains("Hello World")); // Check heading content has proper spacing

    // Test JSON rendering
    let json_renderer = JsonRenderer::new(false);
    let json = json_renderer.render(&ast).expect("Failed to render JSON");
    assert!(json.contains("Document"));

    // Test text rendering
    let text_renderer = TextRenderer::new(TextOptions::default());
    let text = text_renderer.render(&ast);
    assert!(text.contains("Hello World"));
    assert!(text.contains("bold"));
}
#[test]
fn test_legacy_markdown_to_html() {
    let input = "# Test\n\nA **bold** paragraph.";
    let options = MarkdownOptions {
        html_options: HtmlOptions::default(),
        extension: MarkdownExtensions::default(),
    };

    let result = markdown_to_html(input, &options);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("<h1>"));
    assert!(html.contains("<strong>"));
}
