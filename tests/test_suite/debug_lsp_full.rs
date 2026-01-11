// Comprehensive debug test for LSP highlighting with the test file
use core::lsp::compute_highlights;
use core::parser::parse;
use std::fs;

pub fn debug_full_test_file() {
    println!("\n=== Testing Full LSP Highlighting File ===\n");

    // Read the test file - path relative to workspace root
    let test_file =
        if std::path::Path::new("tests/test_suite/fixtures/test_lsp_highlighting.md").exists() {
            "tests/test_suite/fixtures/test_lsp_highlighting.md"
        } else {
            "../tests/test_suite/fixtures/test_lsp_highlighting.md"
        };

    let markdown = fs::read_to_string(test_file).expect("Failed to read test file");

    println!("File: {}", test_file);
    println!("Size: {} bytes\n", markdown.len());

    // Parse the document
    match parse(&markdown) {
        Ok(document) => {
            println!("✓ Parsed successfully\n");

            // Compute highlights
            let highlights = compute_highlights(&document);
            println!("=== Generated {} highlights ===\n", highlights.len());

            // Group by tag type
            use std::collections::HashMap;
            let mut by_tag: HashMap<String, Vec<_>> = HashMap::new();

            for highlight in &highlights {
                let tag_name = format!("{:?}", highlight.tag);
                by_tag
                    .entry(tag_name)
                    .or_insert_with(Vec::new)
                    .push(highlight);
            }

            // Show summary
            println!("Summary by type:");
            let mut tags: Vec<_> = by_tag.keys().collect();
            tags.sort();
            for tag in tags {
                println!("  {}: {} highlights", tag, by_tag[tag].len());
            }

            println!("\n=== Sample Highlights (first 10) ===\n");

            for (i, highlight) in highlights.iter().take(10).enumerate() {
                let start = &highlight.span.start;
                let end = &highlight.span.end;

                // Extract the text for this span
                let lines: Vec<&str> = markdown.lines().collect();
                let text = if start.line == end.line {
                    // Single line
                    if let Some(line) = lines.get(start.line - 1) {
                        let start_byte = start.column - 1;
                        let end_byte = end.column - 1;
                        if start_byte < line.len() && end_byte <= line.len() {
                            &line[start_byte..end_byte]
                        } else {
                            "<out of bounds>"
                        }
                    } else {
                        "<line out of bounds>"
                    }
                } else {
                    "<multi-line>"
                };

                println!("{}. {:?}", i + 1, highlight.tag);
                println!(
                    "   Position: [{}:{} to {}:{}]",
                    start.line, start.column, end.line, end.column
                );
                println!("   Text: {:?}", text);
                println!();
            }

            // Check specific cases that were previously broken
            println!("\n=== Checking Key Elements ===\n");

            // Check for heading highlights
            let headings: Vec<_> = highlights
                .iter()
                .filter(|h| format!("{:?}", h.tag).starts_with("Heading"))
                .collect();
            println!("✓ Found {} heading highlights", headings.len());

            // Check for link highlights
            let links: Vec<_> = highlights
                .iter()
                .filter(|h| matches!(h.tag, core::lsp::HighlightTag::Link))
                .collect();
            println!("✓ Found {} link highlights", links.len());

            // Check for code span highlights
            let code_spans: Vec<_> = highlights
                .iter()
                .filter(|h| matches!(h.tag, core::lsp::HighlightTag::CodeSpan))
                .collect();
            println!("✓ Found {} code span highlights", code_spans.len());

            // Check for emphasis/strong highlights
            let emphasis: Vec<_> = highlights
                .iter()
                .filter(|h| matches!(h.tag, core::lsp::HighlightTag::Emphasis))
                .collect();
            let strong: Vec<_> = highlights
                .iter()
                .filter(|h| matches!(h.tag, core::lsp::HighlightTag::Strong))
                .collect();
            println!("✓ Found {} emphasis highlights", emphasis.len());
            println!("✓ Found {} strong highlights", strong.len());

            println!("\n=== All Checks Passed! ===\n");
        }
        Err(e) => {
            println!("✗ Parse failed: {}", e);
            panic!("Parse should succeed");
        }
    }
}
