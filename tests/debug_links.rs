// Debug script to find failing link examples
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct CommonMarkTest {
    markdown: String,
    html: String,
    example: usize,
    start_line: usize,
    end_line: usize,
    section: Option<String>,
}

fn normalize_html(html: &str) -> String {
    html.trim()
        .replace("\n\n", "\n")
        .replace("  ", " ")
}

fn main() {
    // Load CommonMark spec JSON
    let json_path = "tests/test_suite/spec/commonmark.json";
    let json_content = fs::read_to_string(json_path)
        .expect("Failed to read commonmark.json");
    
    let tests: Vec<CommonMarkTest> = serde_json::from_str(&json_content)
        .expect("Failed to parse commonmark.json");
    
    // Filter for Links section
    let link_tests: Vec<_> = tests.iter()
        .filter(|t| t.section.as_deref() == Some("Links"))
        .collect();
    
    println!("Found {} link examples", link_tests.len());
    println!("─────────────────────────────────────────────────────────\n");
    
    let mut passed = 0;
    let mut failed = 0;
    
    for test in link_tests {
        // Parse markdown
        let parse_result = core::parser::parse(&test.markdown);
        
        match parse_result {
            Ok(document) => {
                // Render to HTML
                let options = core::render::RenderOptions::default();
                match core::render::render(&document, &options) {
                    Ok(actual_html) => {
                        let normalized_expected = normalize_html(&test.html);
                        let normalized_actual = normalize_html(&actual_html);
                        
                        if normalized_expected == normalized_actual {
                            passed += 1;
                        } else {
                            failed += 1;
                            println!("❌ FAILED Example {}", test.example);
                            println!("   Lines: {}-{}", test.start_line, test.end_line);
                            println!("\n   Input Markdown:");
                            println!("   {}", test.markdown.replace("\n", "\\n"));
                            println!("\n   Expected HTML:");
                            println!("   {}", normalized_expected);
                            println!("\n   Actual HTML:");
                            println!("   {}", normalized_actual);
                            println!("\n   Difference:");
                            
                            // Show character-by-character diff for first few chars
                            let max_len = normalized_expected.len().max(normalized_actual.len());
                            for i in 0..max_len.min(200) {
                                let exp_char = normalized_expected.chars().nth(i);
                                let act_char = normalized_actual.chars().nth(i);
                                if exp_char != act_char {
                                    println!("   Position {}: expected {:?}, got {:?}", 
                                             i, exp_char, act_char);
                                    break;
                                }
                            }
                            println!("\n─────────────────────────────────────────────────────────\n");
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        println!("❌ RENDER ERROR Example {}: {:?}", test.example, e);
                        println!("   Input: {}\n", test.markdown.replace("\n", "\\n"));
                    }
                }
            }
            Err(e) => {
                failed += 1;
                println!("❌ PARSE ERROR Example {}: {:?}", test.example, e);
                println!("   Input: {}\n", test.markdown.replace("\n", "\\n"));
            }
        }
    }
    
    println!("\n═════════════════════════════════════════════════════════");
    println!("Links Section: {} passed, {} failed out of {}", 
             passed, failed, link_tests.len());
    println!("═════════════════════════════════════════════════════════");
}
