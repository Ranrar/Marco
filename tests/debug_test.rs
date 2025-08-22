use marco::components::marco_engine::render::{markdown_to_html, MarkdownOptions};

#[test]
fn debug_heading_parsing() {
    let md = "# Hello World\n\nThis is a **bold** text.";
    let opts = MarkdownOptions::default();
    let html = markdown_to_html(md, &opts);
    
    println!("Input: {:?}", md);
    println!("Output: {:?}", html);
    
    // Test if we get a heading and paragraph
    assert!(html.contains("<h1>Hello World</h1>"));
    assert!(html.contains("<p>This is a **bold** text.</p>"));
}

#[test]
fn debug_simple_heading() {
    let md = "# Test";
    let opts = MarkdownOptions::default();
    let html = markdown_to_html(md, &opts);
    
    println!("Simple input: {:?}", md);
    println!("Simple output: {:?}", html);
    
    // Test if we get a heading
    assert!(html.contains("<h1>Test</h1>"));
}
