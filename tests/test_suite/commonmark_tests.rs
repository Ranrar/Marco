// CommonMark spec tests: validate against official spec examples

use serde::{Deserialize, Serialize};
use core::{parser, render, grammar::{inline, block}};
use super::utils::{print_header, Span};

#[derive(Debug, Deserialize, Serialize)]
pub struct CommonMarkTest {
    pub example: u32,
    pub section: Option<String>,
    pub markdown: String,
    pub html: String,
    #[serde(rename = "start_line")]
    pub start_line: Option<u32>,
    #[serde(rename = "end_line")]
    pub end_line: Option<u32>,
}

pub fn load_commonmark_tests() -> Vec<CommonMarkTest> {
    let json = include_str!("spec/commonmark.json");
    serde_json::from_str(json).expect("Failed to parse commonmark.json")
}

pub fn run_commonmark_tests(section: Option<String>) {
    print_header("CommonMark Spec Tests");
    
    let tests = load_commonmark_tests();
    let total_tests = tests.len();
    println!("Loaded {} test examples from CommonMark spec", total_tests);
    
    if let Some(section_filter) = section {
        let filtered: Vec<_> = tests.iter()
            .filter(|t| t.section.as_ref().map(|s| s.contains(&section_filter)).unwrap_or(false))
            .collect();
        
        println!("Testing section: {}", section_filter);
        println!("Found {} examples in this section\n", filtered.len());
        
        let mut passed = 0;
        let mut failed = 0;
        let test_limit = 10;
        
        for test in filtered.iter().take(test_limit) {
            println!("Example {}: {:?}", test.example, test.markdown.trim());
            println!("  Expected: {:?}", test.html.trim());
            
            // Test with appropriate parser
            if section_filter.contains("Code span") {
                let input = Span::new(test.markdown.trim_end());
                match inline::code_span(input) {
                    Ok((_, content)) => {
                        println!("  ✓ Parsed: {:?}", content.fragment());
                        passed += 1;
                    }
                    Err(e) => {
                        println!("  ✗ Failed: {:?}", e);
                        failed += 1;
                    }
                }
            } else if section_filter.contains("ATX heading") {
                let input = Span::new(test.markdown.trim_end());
                
                // Check if expected HTML contains heading tags
                let should_be_heading = test.html.contains("<h1>") 
                    || test.html.contains("<h2>") 
                    || test.html.contains("<h3>") 
                    || test.html.contains("<h4>") 
                    || test.html.contains("<h5>") 
                    || test.html.contains("<h6>");
                
                match block::heading(input) {
                    Ok((_, (level, content))) => {
                        if should_be_heading {
                            println!("  ✓ Parsed level {} heading: {:?}", level, content.fragment());
                            passed += 1;
                        } else {
                            println!("  ✗ Parsed as heading but expected paragraph/code");
                            failed += 1;
                        }
                    }
                    Err(_) => {
                        if !should_be_heading {
                            println!("  ✓ Correctly rejected (not a heading)");
                            passed += 1;
                        } else {
                            println!("  ✗ Failed to parse valid heading");
                            failed += 1;
                        }
                    }
                }
            } else if section_filter.contains("Paragraph") {
                let input = Span::new(test.markdown.trim_end());
                
                // Check if expected HTML contains paragraph tags
                let should_be_paragraph = test.html.contains("<p>");
                
                match block::paragraph(input) {
                    Ok((_, content)) => {
                        if should_be_paragraph {
                            println!("  ✓ Parsed paragraph: {:?}", &content.fragment()[..content.fragment().len().min(40)]);
                            passed += 1;
                        } else {
                            println!("  ✗ Parsed as paragraph but expected something else");
                            failed += 1;
                        }
                    }
                    Err(_) => {
                        if !should_be_paragraph {
                            println!("  ✓ Correctly rejected (not a paragraph)");
                            passed += 1;
                        } else {
                            println!("  ✗ Failed to parse valid paragraph");
                            failed += 1;
                        }
                    }
                }
            } else if section_filter.contains("Fenced code") {
                let input = Span::new(test.markdown.trim_end());
                
                // Check if expected HTML contains <pre><code> tags
                let should_be_code = test.html.contains("<pre><code>");
                
                match block::fenced_code_block(input) {
                    Ok((_, (language, content))) => {
                        if should_be_code {
                            println!("  ✓ Parsed fenced code block (lang={:?}): {:?}", 
                                     language, &content.fragment()[..content.fragment().len().min(30)]);
                            passed += 1;
                        } else {
                            println!("  ✗ Parsed as code block but expected something else");
                            failed += 1;
                        }
                    }
                    Err(_) => {
                        if !should_be_code {
                            println!("  ✓ Correctly rejected (not a fenced code block)");
                            passed += 1;
                        } else {
                            println!("  ✗ Failed to parse valid fenced code block");
                            failed += 1;
                        }
                    }
                }
            }
            println!();
        }
        
        // Print summary
        let tested = passed + failed;
        let percentage = if tested > 0 {
            passed as f64 / tested as f64 * 100.0
        } else {
            0.0
        };
        
        println!("─────────────────────────────────────────────────────────");
        println!("Section Summary: {}/{} tests passed ({:.1}%)", passed, tested, percentage);
        println!("Total progress: {}/{} CommonMark examples ({:.1}%)", 
                 passed, total_tests, passed as f64 / total_tests as f64 * 100.0);
        println!("─────────────────────────────────────────────────────────");
    } else {
        // List all sections
        let mut sections: Vec<_> = tests.iter()
            .filter_map(|t| t.section.clone())
            .collect();
        sections.sort();
        sections.dedup();
        
        println!("Available sections ({} total):", sections.len());
        let mut total_examples = 0;
        for section in &sections {
            let count = tests.iter().filter(|t| t.section.as_deref() == Some(section)).count();
            println!("  - {} ({} examples)", section, count);
            total_examples += count;
        }
        
        println!("\n─────────────────────────────────────────────────────────");
        println!("Total: {} sections with {} examples", sections.len(), total_examples);
        println!("CommonMark coverage: {}/{} examples ({:.1}%)", 
                 0, total_tests, 0.0);
        println!("─────────────────────────────────────────────────────────");
        println!("\nRun specific section with:");
        println!("  cargo test --package core --test test_suite test_commonmark_code_spans -- --nocapture");
    }
}

// ============================================================================
// TEST SUITE SUMMARY
// ============================================================================

