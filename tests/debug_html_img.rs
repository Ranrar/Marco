/// Debug test for HTML img highlighting issues
use core::{grammar, parser, lsp};

fn main() {
    // Initialize logger
    env_logger::init();
    
    let test_cases = vec![
        "<img src=\"test.png\" alt=\"test\" />",
        "<span>HTML</span>",
        "<div class=\"container\">",
        "Text with <span>HTML</span> inline",
        "This is a paragraph with an image: <img src=\"test.png\" alt=\"test\" />",
    ];
    
    println!("\n=== Testing HTML/IMG Grammar Parsing ===\n");
    
    for (i, input) in test_cases.iter().enumerate() {
        println!("Test case {}: {:?}", i + 1, input);
        
        // Test grammar parsing
        let span = grammar::Span::new(input);
        match grammar::inline_html(span) {
            Ok((rest, content)) => {
                println!("  ✓ Grammar parsed successfully");
                println!("    Content: {:?}", content.fragment());
                println!("    Remaining: {:?}", rest.fragment());
            }
            Err(e) => {
                println!("  ✗ Grammar parsing failed: {:?}", e);
            }
        }
        println!();
    }
    
    println!("\n=== Testing Full Document Parsing ===\n");
    
    let full_doc = r#"# Test HTML IMG Highlighting

This is a paragraph with an image: <img src="test.png" alt="test" />

<img src="another.png" />

Text with <span>HTML</span> inline.
"#;
    
    println!("Document:\n{}\n", full_doc);
    
    match parser::parse(full_doc) {
        Ok(doc) => {
            println!("✓ Document parsed successfully");
            println!("  Root nodes: {}", doc.children.len());
            
            // Walk the AST and print nodes
            fn print_node(node: &parser::Node, depth: usize) {
                let indent = "  ".repeat(depth);
                println!("{}Node: {:?}", indent, node.kind);
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
            
            // Test LSP highlights
            println!("\n=== Testing LSP Highlights ===\n");
            let highlights = lsp::compute_highlights(&doc);
            println!("Generated {} highlights:", highlights.len());
            for (i, hl) in highlights.iter().enumerate() {
                println!("  {}: {:?} at L{}:C{}-L{}:C{} (offset {}-{})",
                    i + 1,
                    hl.tag,
                    hl.span.start.line, hl.span.start.column,
                    hl.span.end.line, hl.span.end.column,
                    hl.span.start.offset, hl.span.end.offset
                );
            }
        }
        Err(e) => {
            println!("✗ Document parsing failed: {:?}", e);
        }
    }
}
