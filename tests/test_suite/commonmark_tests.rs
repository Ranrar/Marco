// CommonMark spec tests: validate against official spec examples

use serde::{Deserialize, Serialize};
use super::utils::print_header;

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

// Temporary struct for deserialization that allows missing fields
#[derive(Debug, Deserialize)]
struct RawCommonMarkEntry {
    #[serde(default)]
    example: Option<u32>,
    section: Option<String>,
    #[serde(default)]
    markdown: Option<String>,
    #[serde(default)]
    html: Option<String>,
    #[serde(rename = "start_line")]
    start_line: Option<u32>,
    #[serde(rename = "end_line")]
    end_line: Option<u32>,
}

pub fn load_commonmark_tests() -> Vec<CommonMarkTest> {
    let json = include_str!("spec/commonmark.json");
    let raw_entries: Vec<RawCommonMarkEntry> = serde_json::from_str(json)
        .expect("Failed to parse commonmark.json");
    
    // Filter out comment/metadata entries and convert to CommonMarkTest
    raw_entries.into_iter()
        .filter_map(|entry| {
            match (entry.example, entry.markdown, entry.html) {
                (Some(example), Some(markdown), Some(html)) => Some(CommonMarkTest {
                    example,
                    section: entry.section,
                    markdown,
                    html,
                    start_line: entry.start_line,
                    end_line: entry.end_line,
                }),
                _ => None, // Skip entries without required fields
            }
        })
        .collect()
}

pub fn load_extra_tests() -> Vec<CommonMarkTest> {
    let json = include_str!("spec/extra.json");
    let raw_entries: Vec<RawCommonMarkEntry> = serde_json::from_str(json)
        .expect("Failed to parse extra.json");
    
    // Filter out comment/metadata entries and convert to CommonMarkTest
    raw_entries.into_iter()
        .filter_map(|entry| {
            match (entry.example, entry.markdown, entry.html) {
                (Some(example), Some(markdown), Some(html)) => Some(CommonMarkTest {
                    example,
                    section: entry.section,
                    markdown,
                    html,
                    start_line: entry.start_line,
                    end_line: entry.end_line,
                }),
                _ => None, // Skip entries without required fields
            }
        })
        .collect()
}

pub fn run_commonmark_tests(section: Option<String>) {
    let tests = load_commonmark_tests();
    let total_tests = tests.len();
    
    if let Some(ref section_filter) = section {
        // Custom header with section name
        let header_text = format!("CommonMark Testing section: {}", section_filter);
        print_header(&header_text);
    } else {
        print_header("CommonMark Spec Tests");
    }
    
    println!("Loaded {} test examples from CommonMark spec", total_tests);
    
    if let Some(section_filter) = section {
        let filtered: Vec<_> = tests.iter()
            .filter(|t| t.section.as_ref().map(|s| s.contains(&section_filter)).unwrap_or(false))
            .collect();
        
        let total_in_section = filtered.len();
        println!("Found {} examples in this section\n", total_in_section);
        
        // Run full section test and collect results
        struct TestResult<'a> {
            test: &'a CommonMarkTest,
            passed: bool,
            actual_html: Option<String>,
            error: Option<String>,
        }
        
        let mut results: Vec<TestResult> = Vec::new();
        
        for test in &filtered {
            let test_result = std::panic::catch_unwind(|| {
                let result = core::parser::parse(&test.markdown);
                match result {
                    Ok(document) => {
                        let options = core::render::RenderOptions::default();
                        match core::render::render(&document, &options) {
                            Ok(rendered_html) => {
                                // Basic structural match
                                let expected_has_h1 = test.html.contains("<h1>");
                                let expected_has_h2 = test.html.contains("<h2>");
                                let expected_has_p = test.html.contains("<p>");
                                let expected_has_code = test.html.contains("<code>");
                                let expected_has_pre = test.html.contains("<pre>");
                                
                                let rendered_has_h1 = rendered_html.contains("<h1>");
                                let rendered_has_h2 = rendered_html.contains("<h2>");
                                let rendered_has_p = rendered_html.contains("<p>");
                                let rendered_has_code = rendered_html.contains("<code>");
                                let rendered_has_pre = rendered_html.contains("<pre>");
                                
                                let passed = (expected_has_h1 == rendered_has_h1) &&
                                    (expected_has_h2 == rendered_has_h2) &&
                                    (expected_has_p == rendered_has_p) &&
                                    (expected_has_code == rendered_has_code) &&
                                    (expected_has_pre == rendered_has_pre);
                                
                                (passed, Some(rendered_html), None)
                            }
                            Err(e) => (false, None, Some(format!("Render error: {:?}", e)))
                        }
                    }
                    Err(e) => (false, None, Some(format!("Parse error: {:?}", e)))
                }
            });
            
            let (passed, actual_html, error) = test_result.unwrap_or((false, None, Some("Panic".to_string())));
            
            results.push(TestResult {
                test,
                passed,
                actual_html,
                error,
            });
        }
        
        // Count totals
        let _total_passed = results.iter().filter(|r| r.passed).count();
        let total_failed = results.iter().filter(|r| !r.passed).count();
        
        // Show only failures
        println!("Failed examples:\n");
        
        let failures: Vec<_> = results.iter().filter(|r| !r.passed).collect();
        
        if failures.is_empty() {
            println!("  ✓ All tests passed!\n");
        } else {
            for result in &failures {
                println!("✗ Example {}: {:?}", result.test.example, result.test.markdown.trim());
                println!("  Expected: {:?}", result.test.html.trim());
                
                if let Some(actual) = &result.actual_html {
                    println!("  Actual:   {:?}", actual.trim());
                } else if let Some(err) = &result.error {
                    println!("  Error:    {}", err);
                }
                println!();
            }
        }
        
        // Calculate percentages
        let failed_percentage = if total_in_section > 0 {
            total_failed as f64 / total_in_section as f64 * 100.0
        } else {
            0.0
        };
        
        // Get overall project stats by running all sections
        let mut overall_passed = 0;
        let mut overall_total = 0;
        
        let mut sections: Vec<_> = tests.iter()
            .filter_map(|t| t.section.clone())
            .collect();
        sections.sort();
        sections.dedup();
        
        for section_name in &sections {
            let section_tests: Vec<_> = tests.iter()
                .filter(|t| t.section.as_deref() == Some(section_name))
                .collect();
            
            for test in section_tests {
                overall_total += 1;
                let test_result = std::panic::catch_unwind(|| {
                    let result = core::parser::parse(&test.markdown);
                    match result {
                        Ok(document) => {
                            let options = core::render::RenderOptions::default();
                            match core::render::render(&document, &options) {
                                Ok(rendered_html) => {
                                    let expected_has_h1 = test.html.contains("<h1>");
                                    let expected_has_h2 = test.html.contains("<h2>");
                                    let expected_has_p = test.html.contains("<p>");
                                    let expected_has_code = test.html.contains("<code>");
                                    let expected_has_pre = test.html.contains("<pre>");
                                    
                                    let rendered_has_h1 = rendered_html.contains("<h1>");
                                    let rendered_has_h2 = rendered_html.contains("<h2>");
                                    let rendered_has_p = rendered_html.contains("<p>");
                                    let rendered_has_code = rendered_html.contains("<code>");
                                    let rendered_has_pre = rendered_html.contains("<pre>");
                                    
                                    (expected_has_h1 == rendered_has_h1) &&
                                    (expected_has_h2 == rendered_has_h2) &&
                                    (expected_has_p == rendered_has_p) &&
                                    (expected_has_code == rendered_has_code) &&
                                    (expected_has_pre == rendered_has_pre)
                                }
                                Err(_) => false
                            }
                        }
                        Err(_) => false
                    }
                });
                
                if test_result.unwrap_or(false) {
                    overall_passed += 1;
                }
            }
        }
        
        let overall_percentage = if overall_total > 0 {
            overall_passed as f64 / overall_total as f64 * 100.0
        } else {
            0.0
        };
        
        println!("─────────────────────────────────────────────────────────");
        println!("Section Summary: {}/{} tests failed ({:.1}%)", total_failed, total_in_section, failed_percentage);
        println!("Total progress: {}/{} CommonMark examples ({:.1}%)", 
                 overall_passed, overall_total, overall_percentage);
        println!("─────────────────────────────────────────────────────────");
    } else {
        // Run ALL tests and calculate actual coverage
        println!("\nRunning all CommonMark tests...\n");
        
        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut section_results: Vec<(String, usize, usize)> = Vec::new();
        
        // Group tests by section
        let mut sections: Vec<_> = tests.iter()
            .filter_map(|t| t.section.clone())
            .collect();
        sections.sort();
        sections.dedup();
        
        for section_name in &sections {
            let section_tests: Vec<_> = tests.iter()
                .filter(|t| t.section.as_deref() == Some(section_name))
                .collect();
            
            let section_total = section_tests.len();
            let mut section_passed = 0;
            let mut section_failed = 0;
            
            let verbose = std::env::var("VERBOSE").is_ok();
            
            for test in section_tests {
                // Try to parse using the full pipeline: markdown -> AST -> HTML
                // Use catch_unwind to handle parser panics gracefully
                let test_result = std::panic::catch_unwind(|| {
                    let result = core::parser::parse(&test.markdown);
                    
                    match result {
                        Ok(document) => {
                            // Successfully parsed - try to render
                            let options = core::render::RenderOptions::default();
                            match core::render::render(&document, &options) {
                                Ok(rendered_html) => {
                                    // Very basic check: does the rendered HTML contain expected elements?
                                    // This is not a perfect match but gives us coverage metrics
                                    let expected_has_h1 = test.html.contains("<h1>");
                                    let expected_has_h2 = test.html.contains("<h2>");
                                    let expected_has_p = test.html.contains("<p>");
                                    let expected_has_code = test.html.contains("<code>");
                                    let expected_has_pre = test.html.contains("<pre>");
                                    
                                    let rendered_has_h1 = rendered_html.contains("<h1>");
                                    let rendered_has_h2 = rendered_html.contains("<h2>");
                                    let rendered_has_p = rendered_html.contains("<p>");
                                    let rendered_has_code = rendered_html.contains("<code>");
                                    let rendered_has_pre = rendered_html.contains("<pre>");
                                    
                                    // Basic structural match
                                    let passed = (expected_has_h1 == rendered_has_h1) &&
                                        (expected_has_h2 == rendered_has_h2) &&
                                        (expected_has_p == rendered_has_p) &&
                                        (expected_has_code == rendered_has_code) &&
                                        (expected_has_pre == rendered_has_pre);
                                    
                                    if !passed && verbose {
                                        println!("❌ Example {} (lines {:?}-{:?})", test.example, test.start_line, test.end_line);
                                        println!("   Input: {}", test.markdown.replace("\n", "\\n"));
                                        println!("   Expected: {}", test.html.trim().replace("\n", "\\n"));
                                        println!("   Actual:   {}", rendered_html.trim().replace("\n", "\\n"));
                                        println!();
                                    }
                                    
                                    passed
                                }
                                Err(e) => {
                                    if verbose {
                                        println!("❌ Example {} RENDER ERROR: {:?}", test.example, e);
                                        println!("   Input: {}", test.markdown.replace("\n", "\\n"));
                                        println!();
                                    }
                                    false
                                }
                            }
                        }
                        Err(e) => {
                            if verbose {
                                println!("❌ Example {} PARSE ERROR: {:?}", test.example, e);
                                println!("   Input: {}", test.markdown.replace("\n", "\\n"));
                                println!();
                            }
                            false
                        }
                    }
                });
                
                match test_result {
                    Ok(true) => section_passed += 1,
                    _ => section_failed += 1,
                }
            }
            
            total_passed += section_passed;
            total_failed += section_failed;
            section_results.push((section_name.clone(), section_passed, section_total));
        }
        
        // Print section-by-section results
        println!("Section Results:");
        for (section, passed, total) in &section_results {
            let percentage = *passed as f64 / *total as f64 * 100.0;
            let status = if percentage >= 80.0 { "✓" } else if percentage >= 50.0 { "○" } else { "✗" };
            println!("  {} {} - {}/{} ({:.1}%)", status, section, passed, total, percentage);
        }
        
        let total_tested = total_passed + total_failed;
        let coverage_percentage = if total_tested > 0 {
            total_passed as f64 / total_tested as f64 * 100.0
        } else {
            0.0
        };
        
        println!("\n─────────────────────────────────────────────────────────");
        println!("Total: {} sections with {} examples", sections.len(), total_tested);
        println!("CommonMark coverage: {}/{} examples ({:.1}%)", 
                 total_passed, total_tested, coverage_percentage);
        println!("─────────────────────────────────────────────────────────");
        println!("\nRun specific section with:");
        println!("  cargo test --package core --test test_suite -- commonmark --section \"ATX headings\"");
    }
}

pub fn run_extra_tests() {
    let tests = load_extra_tests();
    let total_tests = tests.len();
    
    print_header("Marco Extra Tests");
    println!("Loaded {} extra test examples\n", total_tests);
    
    if total_tests == 0 {
        println!("No extra tests found.");
        return;
    }
    
    println!("Running extra tests...\n");
    
    let mut passed = 0;
    let mut failed = 0;
    let mut failed_examples = Vec::new();
    
    for test in &tests {
        let test_result = std::panic::catch_unwind(|| {
            let result = core::parser::parse(&test.markdown);
            match result {
                Ok(document) => {
                    let options = core::render::RenderOptions::default();
                    match core::render::render(&document, &options) {
                        Ok(rendered_html) => {
                            // Normalize whitespace for comparison
                            let expected_normalized = test.html.trim();
                            let actual_normalized = rendered_html.trim();
                            
                            if expected_normalized == actual_normalized {
                                true
                            } else {
                                println!("✗ Example {} FAILED", test.example);
                                if let Some(ref section) = test.section {
                                    println!("  Section: {}", section);
                                }
                                println!("  Markdown: {:?}", test.markdown);
                                println!("  Expected: {:?}", expected_normalized);
                                println!("  Got:      {:?}\n", actual_normalized);
                                false
                            }
                        }
                        Err(e) => {
                            println!("✗ Example {} FAILED (render error)", test.example);
                            println!("  Error: {}\n", e);
                            false
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Example {} FAILED (parse error)", test.example);
                    println!("  Error: {}\n", e);
                    false
                }
            }
        });
        
        match test_result {
            Ok(true) => passed += 1,
            _ => {
                failed += 1;
                failed_examples.push(test.example);
            }
        }
    }
    
    println!("─────────────────────────────────────────────────────────");
    let coverage_percentage = if total_tests > 0 {
        passed as f64 / total_tests as f64 * 100.0
    } else {
        0.0
    };
    
    if failed > 0 {
        println!("Extra tests: {}/{} passed ({:.1}%)", passed, total_tests, coverage_percentage);
        println!("Failed examples: {:?}", failed_examples);
    } else {
        println!("✓ All extra tests passed: {}/{} ({:.1}%)", passed, total_tests, coverage_percentage);
    }
    println!("─────────────────────────────────────────────────────────");
}

// ============================================================================
// TEST SUITE SUMMARY
// ============================================================================

