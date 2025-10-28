use core::{parser, lsp};

fn main() {
    // Test different HTML img scenarios
    let test_cases = vec![
        (
            "Simple img tag",
            r#"<img src="test.png" alt="test" />"#
        ),
        (
            "Paragraph with img",
            r#"This is a paragraph with an image: <img src="test.png" alt="test" />"#
        ),
        (
            "Multiple HTML tags",
            r#"Text with <span>HTML</span> and <img src="test.png" /> inline."#
        ),
        (
            "Opening tag only",
            r#"<span>Text inside span"#
        ),
        (
            "Closing tag",
            r#"Text</span> after"#
        ),
    ];
    
    for (name, input) in test_cases {
        println!("\n=== Test: {} ===", name);
        println!("Input: {:?}", input);
        
        match parser::parse(input) {
            Ok(doc) => {
                println!("✓ Parsed successfully");
                
                // Print AST
                fn print_node(node: &parser::Node, depth: usize) {
                    let indent = "  ".repeat(depth);
                    match &node.kind {
                        parser::NodeKind::InlineHtml(html) => {
                            println!("{}InlineHtml: {:?}", indent, html);
                        }
                        other => {
                            println!("{}{:?}", indent, other);
                        }
                    }
                    if let Some(span) = &node.span {
                        println!("{}  Span: L{}:C{}-L{}:C{} (offset {}-{})", 
                            indent,
                            span.start.line, span.start.column,
                            span.end.line, span.end.column,
                            span.start.offset, span.end.offset
                        );
                    }
                    for child in &node.children {
                        print_node(child, depth + 1);
                    }
                }
                
                for node in &doc.children {
                    print_node(node, 0);
                }
                
                // Print highlights
                let highlights = lsp::compute_highlights(&doc);
                println!("Highlights: {}", highlights.len());
                for hl in &highlights {
                    println!("  {:?} at L{}:C{}-L{}:C{}",
                        hl.tag,
                        hl.span.start.line, hl.span.start.column,
                        hl.span.end.line, hl.span.end.column
                    );
                }
            }
            Err(e) => {
                println!("✗ Parse failed: {:?}", e);
            }
        }
    }
}
