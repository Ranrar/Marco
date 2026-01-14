// Parser tests: validate block → inline parsing and AST building

use super::utils::{print_header, print_section};

pub fn run_parser_tests() {
    use core::parser;

    print_header("Parser → AST Integration Tests");

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Simple heading
    print_section("Parse Heading to AST");
    total += 1;
    let input = "# Hello World";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 1 {
                if let core::parser::NodeKind::Heading { level, text, .. } = &doc.children[0].kind {
                    if *level == 1 && text == "Hello World" {
                        println!("  ✓ Heading parsed: level={}, text={:?}", level, text);
                        passed += 1;
                    } else {
                        println!(
                            "  ✗ Heading data mismatch: level={}, text={:?}",
                            level, text
                        );
                        failed += 1;
                    }
                } else {
                    println!("  ✗ Expected Heading node, got {:?}", doc.children[0].kind);
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 1 node, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }

    // Test 2: Paragraph
    print_section("Parse Paragraph to AST");
    total += 1;
    let input = "This is a paragraph.";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 1 {
                if let core::parser::NodeKind::Paragraph = &doc.children[0].kind {
                    if !doc.children[0].children.is_empty() {
                        println!(
                            "  ✓ Paragraph parsed with {} child nodes",
                            doc.children[0].children.len()
                        );
                        passed += 1;
                    } else {
                        println!("  ✗ Paragraph has no children");
                        failed += 1;
                    }
                } else {
                    println!("  ✗ Expected Paragraph node");
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 1 node, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }

    // Test 3: Fenced code block
    print_section("Parse Code Block to AST");
    total += 1;
    let input = "```rust\nfn main() {}\n```\n";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 1 {
                if let core::parser::NodeKind::CodeBlock { language, code } = &doc.children[0].kind
                {
                    if language == &Some("rust".to_string()) && code.contains("fn main") {
                        println!(
                            "  ✓ Code block parsed: lang={:?}, {} bytes",
                            language,
                            code.len()
                        );
                        passed += 1;
                    } else {
                        println!("  ✗ Code block data mismatch");
                        failed += 1;
                    }
                } else {
                    println!("  ✗ Expected CodeBlock node");
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 1 node, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }

    // Test 4: Mixed content
    print_section("Parse Mixed Content to AST");
    total += 1;
    let input = "# Title\n\nA paragraph.\n\n```\ncode\n```\n";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 3 {
                let has_heading =
                    matches!(doc.children[0].kind, core::parser::NodeKind::Heading { .. });
                let has_paragraph =
                    matches!(doc.children[1].kind, core::parser::NodeKind::Paragraph);
                let has_code = matches!(
                    doc.children[2].kind,
                    core::parser::NodeKind::CodeBlock { .. }
                );

                if has_heading && has_paragraph && has_code {
                    println!(
                        "  ✓ Mixed content parsed: {} nodes (Heading, Paragraph, CodeBlock)",
                        doc.children.len()
                    );
                    passed += 1;
                } else {
                    println!("  ✗ Node types mismatch");
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 3 nodes, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }

    // Summary
    println!("\n{}", "─".repeat(60));
    println!(
        "Parser Integration Summary: {}/{} tests passed ({:.1}%)",
        passed,
        total,
        (passed as f64 / total as f64) * 100.0
    );
    if failed > 0 {
        println!("  ! {} test(s) failed", failed);
    }
}

// ============================================================================
// RENDER TESTS (Parser → AST → HTML)
// ============================================================================
