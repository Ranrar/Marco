use pest::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

// ASCII tree visualization
extern crate ascii_tree;
extern crate escape_string;
use pest::iterators::Pairs;

// Import marco engine components
use marco::components::marco_engine::{MarcoParser, Rule};

// Grammar visualization module
mod grammar_visualizer;

// JSON Test Specification Structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub markdown: String,
    pub html: String,
    pub example: u32,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub section: String,
    pub rule: Option<String>,
    pub description: Option<String>,
    pub expected_failure: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub spec_file: String,
    pub description: String,
    pub enabled: bool,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestOptions {
    pub stop_on_first_failure: bool,
    pub verbose_output: bool,
    pub show_parse_trees: bool,
    pub benchmark_mode: bool,
    pub parallel_execution: bool,
    pub output_format: String,
    pub save_results: bool,
    pub results_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrammarSettings {
    pub grammar_file: String,
    pub main_rule: String,
    pub debug_mode: bool,
    pub trace_parsing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConfig {
    pub test_suites: Vec<TestSuite>,
    pub test_options: TestOptions,
    pub grammar_settings: GrammarSettings,
}

// Test execution results
#[derive(Debug, Clone)]
pub enum TestStatus {
    Passed,
    Failed,
    ExpectedFailure,
    UnknownRule,
    ParseError,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_case: TestCase,
    pub suite_name: String,
    pub status: TestStatus,
    pub parse_tree: Option<String>,
    pub error_message: Option<String>,
    pub parse_time: Duration,
    pub rule_used: Option<Rule>,
}

#[derive(Debug)]
pub struct SuiteResults {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub expected_failures: usize,
    pub unknown_rules: usize,
    pub parse_errors: usize,
    pub skipped: usize,
    pub total_time: Duration,
    pub results: Vec<TestResult>,
}

#[derive(Debug)]
pub struct OverallResults {
    pub suite_results: Vec<SuiteResults>,
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_expected_failures: usize,
    pub total_unknown_rules: usize,
    pub total_parse_errors: usize,
    pub total_skipped: usize,
    pub overall_time: Duration,
    pub success_rate: f64,
}

// Color codes for terminal output
const RESET: &str = "\x1b[0m";
const BRIGHT_BLUE: &str = "\x1b[94m"; // Rule names
const BRIGHT_GREEN: &str = "\x1b[92m"; // Content
const BRIGHT_YELLOW: &str = "\x1b[93m"; // Keywords
const BRIGHT_CYAN: &str = "\x1b[96m"; // Operators
const BRIGHT_MAGENTA: &str = "\x1b[95m"; // Special rules
const DIM_WHITE: &str = "\x1b[37m"; // Brackets
const BRIGHT_RED: &str = "\x1b[91m"; // Errors

// Enhanced Tree visualization functions
fn into_ascii_tree_nodes(mut pairs: Pairs<Rule>) -> Vec<ascii_tree::Tree> {
    let mut vec = Vec::new();

    while let Some(pair) = pairs.next() {
        let pair_content = pair.as_span().as_str();
        let pair_rule = pair.as_rule();
        let rule_name = format!("{:?}", pair_rule);

        // Skip EOI (End of Input) tokens as they're not useful in visualization
        if rule_name == "EOI" {
            continue;
        }

        // Recursively process inner pairs to build the complete tree
        let inner_pairs = into_ascii_tree_nodes(pair.into_inner());

        let node = if inner_pairs.is_empty() {
            // Leaf node: show rule name and content with colors
            let content = if pair_content.len() > 50 {
                format!("{}...", &pair_content[..47])
            } else {
                pair_content.to_string()
            };
            let leaf_text = format_tree_node(&rule_name, &content, true);
            ascii_tree::Tree::Leaf(vec![leaf_text])
        } else {
            // Branch node: show rule name with children
            let content = if pair_content.len() > 100 {
                format!("{}...", &pair_content[..97])
            } else {
                pair_content.to_string()
            };
            let node_text = format_tree_node(&rule_name, &content, false);
            ascii_tree::Tree::Node(node_text, inner_pairs)
        };

        vec.push(node);
    }

    vec
}

fn format_tree_node(rule_name: &str, content: &str, is_leaf: bool) -> String {
    // Escape content for display
    let escaped_content = escape_content_for_display(content);

    // Color the rule name based on its type
    let colored_rule = color_rule_name(rule_name);

    // Format content with brackets
    let formatted_content = format!(
        "{}⟨{}{}{}{}",
        BRIGHT_CYAN, DIM_WHITE, BRIGHT_GREEN, escaped_content, RESET
    );
    let closing_bracket = format!("{}⟩{}", DIM_WHITE, RESET);

    format!(
        "{} {} {}{}",
        colored_rule,
        if is_leaf { "→" } else { "→" },
        formatted_content,
        closing_bracket
    )
}

fn color_rule_name(rule_name: &str) -> String {
    let color = if rule_name.starts_with("KW_") {
        BRIGHT_YELLOW
    } else if matches!(
        rule_name,
        "file" | "document" | "block" | "inline" | "paragraph"
    ) {
        BRIGHT_MAGENTA
    } else if rule_name.contains("text") || rule_name.contains("word") {
        BRIGHT_CYAN
    } else {
        BRIGHT_BLUE
    };

    format!("{}{}{}", color, rule_name, RESET)
}

fn escape_content_for_display(content: &str) -> String {
    content
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .chars()
        .take(80)
        .collect()
}

fn into_ascii_tree(pairs: Pairs<Rule>) -> Result<String, std::fmt::Error> {
    let nodes = into_ascii_tree_nodes(pairs);

    let mut output = String::new();

    match nodes.len() {
        0 => {}
        1 => {
            ascii_tree::write_tree(&mut output, nodes.first().unwrap())?;
        }
        _ => {
            let root = ascii_tree::Tree::Node(String::new(), nodes);
            ascii_tree::write_tree(&mut output, &root)?;

            if output.starts_with(" \n") {
                output = output.split_off(2);
            }
        }
    };

    Ok(output)
}

// Load test configuration
fn load_test_config() -> Result<TestConfig, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("test_config.json")?;
    let config: TestConfig = serde_json::from_str(&config_content)?;
    Ok(config)
}

// Load test cases from JSON spec file
fn load_test_cases(spec_file: &str) -> Result<Vec<TestCase>, Box<dyn std::error::Error>> {
    let spec_content = fs::read_to_string(spec_file)?;
    let test_cases: Vec<TestCase> = serde_json::from_str(&spec_content)?;
    Ok(test_cases)
}

// Convert rule string to Rule enum
fn string_to_rule(rule_str: &str) -> Option<Rule> {
    match rule_str {
        // Keep exact case for rules that need it
        "H1" => Some(Rule::H1),
        "H2" => Some(Rule::H2),
        "H3" => Some(Rule::H3),
        "H4" => Some(Rule::H4),
        "H5" => Some(Rule::H5),
        "H6" => Some(Rule::H6),
        _ => match rule_str.to_lowercase().as_str() {
            "document" => Some(Rule::document),
            "file" => Some(Rule::file),
            "block" => Some(Rule::block),
            "paragraph" => Some(Rule::paragraph),
            "heading" => Some(Rule::heading),
            "admonition_block" => Some(Rule::admonition_block),
            "user_mention" => Some(Rule::user_mention),
            "tab_block" | "tabs_block" => Some(Rule::tab_block),
            "bookmark" => Some(Rule::bookmark),
            "page_tag" => Some(Rule::page_tag),
            "toc" => Some(Rule::toc),
            "run_inline" => Some(Rule::run_inline),
            "run_block_fenced" => Some(Rule::run_block_fenced),
            "bold" => Some(Rule::bold),
            "italic" => Some(Rule::italic),
            "strikethrough" => Some(Rule::strikethrough),
            "code_inline" => Some(Rule::code_inline),
            "code_block" => Some(Rule::code_block),
            "list" => Some(Rule::list),
            "table" => Some(Rule::table),
            "autolink" => Some(Rule::autolink),
            "inline_link" => Some(Rule::inline_link),
            "inline_image" => Some(Rule::inline_image),
            "math_inline" => Some(Rule::math_inline),
            "math_block" => Some(Rule::math_block),
            "hr" => Some(Rule::hr),
            "blockquote" => Some(Rule::blockquote),
            "hard_line_break" => Some(Rule::hard_line_break),
            "emphasis" => Some(Rule::emphasis),
            "highlight" => Some(Rule::highlight),
            "text" => Some(Rule::text),
            // Mappings for GFM/CommonMark compatibility
            "fenced_code_block" => Some(Rule::code_block),
            "task_list" => Some(Rule::list), // Task lists are part of regular lists in Marco
            "link" => Some(Rule::inline_link),
            "image" => Some(Rule::inline_image),
            "horizontal_rule" => Some(Rule::hr),
            "block_quote" => Some(Rule::blockquote),
            "fenced_div" => Some(Rule::admonition_block), // Map to closest equivalent
            "github_reference" => Some(Rule::autolink),
            "github_mention" => Some(Rule::user_mention),
            "commit_reference" => Some(Rule::autolink),
            // Rules that exist in Marco engine but weren't mapped before
            "emoji" => Some(Rule::emoji),
            "superscript" => Some(Rule::superscript),
            "subscript" => Some(Rule::subscript),
            "footnote_def" => Some(Rule::footnote_def),
            "footnote_ref" => Some(Rule::footnote_ref),
            "inline_footnote_ref" => Some(Rule::inline_footnote_ref),
            "def_list" => Some(Rule::def_list),
            "definition_list" => Some(Rule::def_list), // Map to def_list
            "setext_h1" => Some(Rule::setext_h1),
            "setext_h2" => Some(Rule::setext_h2),
            "heading_atx" => Some(Rule::heading), // Map ATX headings to general heading rule
            "heading_setext" => Some(Rule::heading), // Map setext headings to general heading rule
            "soft_line_break" => Some(Rule::soft_line_break),
            "run_command" => Some(Rule::run_inline), // Map to closest equivalent
            "reference_link" => Some(Rule::reference_link),
            "inline_html" => Some(Rule::inline_html),
            "block_html" => Some(Rule::block_html),
            "inline_comment" => Some(Rule::inline_comment),
            "block_comment" => Some(Rule::block_comment),
            "escaped_char" => Some(Rule::escaped_char),
            "unknown_block" => Some(Rule::unknown_block),
            "macro_block" => Some(Rule::macro_block),
            "macro_inline" => Some(Rule::macro_inline),
            "reference_definition" => Some(Rule::reference_definition),
            "reference_image" => Some(Rule::reference_image),
            "table_safe_text" => Some(Rule::table_safe_text),
            "table_cell_content" => Some(Rule::table_cell_content),
            "footnote_label" => Some(Rule::footnote_label),
            "fenced_code" => Some(Rule::fenced_code),
            "indented_code" => Some(Rule::indented_code),
            "language_id" => Some(Rule::language_id),
            "paragraph_line" => Some(Rule::paragraph_line),
            "inline" => Some(Rule::inline),
            "inline_core" => Some(Rule::inline_core),
            // Map generic footnote to the most commonly used footnote_ref
            "footnote" => Some(Rule::footnote_ref), // Map to footnote_ref instead of None
            _ => None,
        },
    }
}

// Execute a single test case
fn execute_test(test_case: &TestCase, suite_name: &str, options: &TestOptions) -> TestResult {
    let start_time = Instant::now();

    let rule = if let Some(rule_str) = &test_case.rule {
        string_to_rule(rule_str)
    } else {
        Some(Rule::document) // Default rule
    };

    let (status, parse_tree, error_message) = if rule.is_none() {
        (
            TestStatus::UnknownRule,
            None,
            Some("Unknown rule".to_string()),
        )
    } else {
        match MarcoParser::parse(rule.unwrap(), &test_case.markdown) {
            Ok(pairs) => {
                // Parsing succeeded - generate parse tree
                let parse_tree = if let Some(first_pair) = pairs.into_iter().next() {
                    Some(format_parse_tree_html(first_pair))
                } else {
                    None
                };

                if test_case.expected_failure.unwrap_or(false) {
                    // This was supposed to fail but passed - this is a failed expected failure
                    (TestStatus::Failed, parse_tree, None)
                } else {
                    (TestStatus::Passed, parse_tree, None)
                }
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if test_case.expected_failure.unwrap_or(false) {
                    // This was supposed to fail and did fail - this is an expected failure
                    (TestStatus::ExpectedFailure, None, Some(error_msg))
                } else {
                    // This was not supposed to fail but did - this is a regular failure
                    (TestStatus::Failed, None, Some(error_msg))
                }
            }
        }
    };

    let parse_time = start_time.elapsed();

    TestResult {
        test_case: test_case.clone(),
        suite_name: suite_name.to_string(),
        status,
        parse_tree,
        error_message,
        parse_time,
        rule_used: rule,
    }
}

// Execute all tests for a test suite
fn execute_test_suite(
    suite: &TestSuite,
    test_cases: Vec<TestCase>,
    options: &TestOptions,
) -> SuiteResults {
    let start_time = Instant::now();

    println!(
        "{}🧪 Running {} tests from {}...{}",
        BRIGHT_CYAN,
        test_cases.len(),
        suite.name,
        RESET
    );

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    let mut expected_failures = 0;
    let mut unknown_rules = 0;
    let mut parse_errors = 0;
    let mut skipped = 0;

    for test_case in test_cases.iter() {
        let result = execute_test(test_case, &suite.name, options);

        match result.status {
            TestStatus::Passed => passed += 1,
            TestStatus::Failed => failed += 1,
            TestStatus::ExpectedFailure => expected_failures += 1,
            TestStatus::UnknownRule => unknown_rules += 1,
            TestStatus::ParseError => parse_errors += 1,
            TestStatus::Skipped => skipped += 1,
        }

        if options.verbose_output {
            let status_icon = match result.status {
                TestStatus::Passed => "✅",
                TestStatus::Failed => "❌",
                TestStatus::ExpectedFailure => "⚠️",
                TestStatus::UnknownRule => "❓",
                TestStatus::ParseError => "💥",
                TestStatus::Skipped => "⏭️",
            };
            println!(
                "  {} {} ({})",
                status_icon,
                test_case
                    .description
                    .as_deref()
                    .unwrap_or(&format!("Example {}", test_case.example)),
                test_case.section
            );
        }

        results.push(result);

        if options.stop_on_first_failure
            && matches!(
                results.last().unwrap().status,
                TestStatus::Failed | TestStatus::ParseError
            )
        {
            break;
        }
    }

    let total_time = start_time.elapsed();

    SuiteResults {
        suite_name: suite.name.clone(),
        total_tests: test_cases.len(),
        passed,
        failed,
        expected_failures,
        unknown_rules,
        parse_errors,
        skipped,
        total_time,
        results,
    }
}

// Execute all test suites
fn execute_all_tests(config: &TestConfig) -> Result<OverallResults, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let mut suite_results = Vec::new();

    for suite in &config.test_suites {
        if !suite.enabled {
            println!(
                "{}⏭️  Skipping disabled suite: {}{}",
                BRIGHT_YELLOW, suite.name, RESET
            );
            continue;
        }

        match load_test_cases(&suite.spec_file) {
            Ok(test_cases) => {
                let results = execute_test_suite(suite, test_cases, &config.test_options);
                suite_results.push(results);
            }
            Err(e) => {
                eprintln!(
                    "{}❌ Failed to load test cases from {}: {}{}",
                    BRIGHT_RED, suite.spec_file, e, RESET
                );
            }
        }
    }

    let overall_time = start_time.elapsed();

    // Calculate overall statistics
    let total_tests = suite_results.iter().map(|r| r.total_tests).sum();
    let total_passed = suite_results.iter().map(|r| r.passed).sum();
    let total_failed = suite_results.iter().map(|r| r.failed).sum();
    let total_expected_failures = suite_results.iter().map(|r| r.expected_failures).sum();
    let total_unknown_rules = suite_results.iter().map(|r| r.unknown_rules).sum();
    let total_parse_errors = suite_results.iter().map(|r| r.parse_errors).sum();
    let total_skipped = suite_results.iter().map(|r| r.skipped).sum();

    let success_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    Ok(OverallResults {
        suite_results,
        total_tests,
        total_passed,
        total_failed,
        total_expected_failures,
        total_unknown_rules,
        total_parse_errors,
        total_skipped,
        overall_time,
        success_rate,
    })
}

// Display results summary
fn display_results_summary(results: &OverallResults) {
    println!("\n{}📊 Test Results Summary{}", BRIGHT_CYAN, RESET);
    println!(
        "{}═══════════════════════════════════════{}",
        BRIGHT_CYAN, RESET
    );

    for suite_result in &results.suite_results {
        let suite_success_rate = if suite_result.total_tests > 0 {
            (suite_result.passed as f64 / suite_result.total_tests as f64) * 100.0
        } else {
            0.0
        };

        println!("\n{}📋 {}{}", BRIGHT_BLUE, suite_result.suite_name, RESET);
        println!(
            "   Total: {}{}{}  Passed: {}{}{}  Failed: {}{}{}  Success Rate: {}{:.1}%{}",
            BRIGHT_CYAN,
            suite_result.total_tests,
            RESET,
            BRIGHT_GREEN,
            suite_result.passed,
            RESET,
            BRIGHT_RED,
            suite_result.failed + suite_result.parse_errors,
            RESET,
            if suite_success_rate >= 90.0 {
                BRIGHT_GREEN
            } else if suite_success_rate >= 70.0 {
                BRIGHT_YELLOW
            } else {
                BRIGHT_RED
            },
            suite_success_rate,
            RESET
        );

        if suite_result.expected_failures > 0 {
            println!(
                "   Expected Failures: {}{}{}",
                BRIGHT_YELLOW, suite_result.expected_failures, RESET
            );
        }
        if suite_result.unknown_rules > 0 {
            println!(
                "   Unknown Rules: {}{}{}",
                BRIGHT_MAGENTA, suite_result.unknown_rules, RESET
            );
        }
        if suite_result.skipped > 0 {
            println!("   Skipped: {}{}{}", DIM_WHITE, suite_result.skipped, RESET);
        }
    }

    println!("\n{}🎯 Overall Results{}", BRIGHT_CYAN, RESET);
    println!("{}───────────────────{}", BRIGHT_CYAN, RESET);
    println!(
        "Total Tests: {}{}{}",
        BRIGHT_CYAN, results.total_tests, RESET
    );
    println!("Passed: {}{}{}", BRIGHT_GREEN, results.total_passed, RESET);
    println!(
        "Failed: {}{}{}",
        BRIGHT_RED,
        results.total_failed + results.total_parse_errors,
        RESET
    );
    println!(
        "Success Rate: {}{:.1}%{}",
        if results.success_rate >= 90.0 {
            BRIGHT_GREEN
        } else if results.success_rate >= 70.0 {
            BRIGHT_YELLOW
        } else {
            BRIGHT_RED
        },
        results.success_rate,
        RESET
    );
    println!(
        "Total Time: {}{:.2}s{}",
        BRIGHT_CYAN,
        results.overall_time.as_secs_f64(),
        RESET
    );

    if results.total_expected_failures > 0 {
        println!(
            "Expected Failures: {}{}{}",
            BRIGHT_YELLOW, results.total_expected_failures, RESET
        );
    }
    if results.total_unknown_rules > 0 {
        println!(
            "Unknown Rules: {}{}{}",
            BRIGHT_MAGENTA, results.total_unknown_rules, RESET
        );
    }
}

// Generate markdown report
fn generate_markdown_report(
    results: &OverallResults,
    config: &TestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.test_options.save_results {
        return Ok(());
    }

    let results_dir = &config.test_options.results_dir;
    fs::create_dir_all(results_dir)?;

    let timestamp = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    let report_path = format!("{}/test_results.md", results_dir);

    let mut report = String::new();

    // Header
    report.push_str("# Marco Grammar Test Results\n\n");
    report.push_str(&format!("**Generated:** {}\n", timestamp));
    report.push_str(&format!("**Total Tests:** {}\n", results.total_tests));
    report.push_str(&format!(
        "**Overall Success Rate:** {:.1}%\n\n",
        results.success_rate
    ));

    // Suite summaries
    report.push_str("## Test Suite Summary\n\n");
    report.push_str(
        "| Suite | Total | Passed | Failed | Expected Failures | Unknown Rules | Success Rate |\n",
    );
    report.push_str(
        "|-------|-------|--------|--------|-------------------|---------------|-------------|\n",
    );

    for suite_result in &results.suite_results {
        let suite_success_rate = if suite_result.total_tests > 0 {
            (suite_result.passed as f64 / suite_result.total_tests as f64) * 100.0
        } else {
            0.0
        };

        report.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {:.1}% |\n",
            suite_result.suite_name,
            suite_result.total_tests,
            suite_result.passed,
            suite_result.failed + suite_result.parse_errors,
            suite_result.expected_failures,
            suite_result.unknown_rules,
            suite_success_rate
        ));
    }

    // Detailed results for each suite
    for suite_result in &results.suite_results {
        report.push_str(&format!("\n## {}\n\n", suite_result.suite_name));

        let mut categories: HashMap<String, Vec<&TestResult>> = HashMap::new();
        for result in &suite_result.results {
            categories
                .entry(result.test_case.section.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        for (category, test_results) in categories {
            report.push_str(&format!("### {}\n\n", category));

            for result in test_results {
                let status_icon = match result.status {
                    TestStatus::Passed => "✅",
                    TestStatus::Failed => "❌",
                    TestStatus::ExpectedFailure => "⚠️",
                    TestStatus::UnknownRule => "❓",
                    TestStatus::ParseError => "💥",
                    TestStatus::Skipped => "⏭️",
                };

                let description = result.test_case.description.as_deref();
                let fallback = format!("Example {}", result.test_case.example);
                let description = description.unwrap_or(&fallback);

                report.push_str(&format!("- {} **{}**\n", status_icon, description));

                if !matches!(
                    result.status,
                    TestStatus::Passed | TestStatus::ExpectedFailure
                ) {
                    // Show markdown input for failed tests
                    let markdown_preview = if result.test_case.markdown.len() > 100 {
                        format!("{}...", &result.test_case.markdown[..97])
                    } else {
                        result.test_case.markdown.clone()
                    };

                    report.push_str(&format!("  ```\n  {}\n  ```\n", markdown_preview));

                    if let Some(error) = &result.error_message {
                        report.push_str(&format!("  *Error: {}*\n", error));
                    }
                }
            }
            report.push_str("\n");
        }
    }

    fs::write(report_path, report)?;
    println!(
        "{}📝 Detailed report saved to {}/test_results.md{}",
        BRIGHT_GREEN, results_dir, RESET
    );

    // Also generate HTML version
    generate_html_report(results, config)?;

    Ok(())
}

// Generate HTML test results report
fn generate_html_report(
    results: &OverallResults,
    config: &TestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.test_options.save_results {
        return Ok(());
    }

    let results_dir = &config.test_options.results_dir;
    fs::create_dir_all(results_dir)?;

    let timestamp = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    let report_path = format!("{}/test_results.html", results_dir);

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Marco Grammar Test Results</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #1e1e1e;
            color: #d4d4d4;
            margin: 0;
            padding: 20px;
            line-height: 1.6;
        }}
        
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 30px;
            border-radius: 12px;
            text-align: center;
            margin-bottom: 30px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.3);
        }}
        
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            color: white;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.5);
        }}
        
        .header .subtitle {{
            margin: 10px 0 0 0;
            font-size: 1.1em;
            color: rgba(255,255,255,0.9);
        }}
        
        .summary-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        
        .summary-card {{
            background: #2d2d30;
            padding: 20px;
            border-radius: 8px;
            border-left: 4px solid;
            text-align: center;
            transition: transform 0.2s;
        }}
        
        .summary-card:hover {{
            transform: translateY(-2px);
        }}
        
        .summary-card.passed {{ border-left-color: #4ec9b0; }}
        .summary-card.failed {{ border-left-color: #f44747; }}
        .summary-card.expected {{ border-left-color: #ffcc02; }}
        .summary-card.unknown {{ border-left-color: #c586c0; }}
        .summary-card.total {{ border-left-color: #569cd6; }}
        .summary-card.rate {{ border-left-color: #dcdcaa; }}
        
        .summary-card .number {{
            font-size: 2em;
            font-weight: bold;
            margin-bottom: 10px;
        }}
        
        .summary-card.passed .number {{ color: #4ec9b0; }}
        .summary-card.failed .number {{ color: #f44747; }}
        .summary-card.expected .number {{ color: #ffcc02; }}
        .summary-card.unknown .number {{ color: #c586c0; }}
        .summary-card.total .number {{ color: #569cd6; }}
        .summary-card.rate .number {{ color: #dcdcaa; }}
        
        .controls {{
            background: #252525;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            display: flex;
            gap: 15px;
            align-items: center;
            flex-wrap: wrap;
        }}
        
        .search-box {{
            flex: 1;
            min-width: 200px;
            padding: 10px;
            background: #3c3c3c;
            border: 1px solid #555;
            border-radius: 4px;
            color: #d4d4d4;
            font-size: 14px;
        }}
        
        .filter-buttons {{
            display: flex;
            gap: 10px;
        }}
        
        .filter-btn {{
            padding: 8px 16px;
            background: #3c3c3c;
            border: 1px solid #555;
            border-radius: 4px;
            color: #d4d4d4;
            cursor: pointer;
            transition: all 0.2s;
        }}
        
        .filter-btn:hover {{
            background: #4c4c4c;
        }}
        
        .filter-btn.active {{
            background: #007acc;
            border-color: #007acc;
        }}
        
        .spec-filter {{
            padding: 8px 12px;
            background: #3c3c3c;
            border: 1px solid #555;
            border-radius: 4px;
            color: #d4d4d4;
            cursor: pointer;
            transition: all 0.2s;
            min-width: 150px;
        }}
        
        .spec-filter:hover {{
            background: #4c4c4c;
        }}
        
        .spec-filter:focus {{
            outline: none;
            border-color: #007acc;
        }}
        
        .hidden {{
            display: none !important;
        }}
        
        .section {{
            background: #2d2d30;
            margin-bottom: 20px;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 2px 8px rgba(0,0,0,0.2);
        }}
        
        .section-header {{
            background: #3c3c3c;
            padding: 15px 20px;
            font-weight: bold;
            font-size: 1.2em;
            border-bottom: 1px solid #555;
            cursor: pointer;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        
        .section-header:hover {{
            background: #4c4c4c;
        }}
        
        .section-stats {{
            font-size: 0.9em;
            color: #9cdcfe;
        }}
        
        .test-item {{
            padding: 15px 20px;
            border-bottom: 1px solid #3c3c3c;
        }}
        
        .test-item:last-child {{
            border-bottom: none;
        }}
        
        .test-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 10px;
        }}
        
        .test-name {{
            font-weight: bold;
            font-size: 1.1em;
        }}
        
        .test-status {{
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.9em;
            font-weight: bold;
        }}
        
        .status-passed {{ background: #4ec9b0; color: #000; }}
        .status-failed {{ background: #f44747; color: #fff; }}
        .status-expected {{ background: #dcdcaa; color: #000; }}
        .status-unknown {{ background: #c586c0; color: #fff; }}
        .status-error {{ background: #f44747; color: #fff; }}
        
        .test-input {{
            background: #252525;
            padding: 10px;
            border-radius: 4px;
            border-left: 3px solid #007acc;
            margin: 10px 0;
            font-family: 'Consolas', monospace;
            white-space: pre-wrap;
            word-break: break-word;
        }}
        
        .stats-bar {{
            background: #252525;
            padding: 10px 20px;
            text-align: center;
            font-size: 0.9em;
            color: #9cdcfe;
        }}
        
        .collapsible-content {{
            max-height: 0;
            overflow: hidden;
            transition: max-height 0.3s ease;
        }}
        
        .collapsible-content.expanded {{
            max-height: 50000px;
        }}
        
        .expand-icon {{
            transition: transform 0.3s ease;
        }}
        
        .expand-icon.rotated {{
            transform: rotate(90deg);
        }}
        
        .parse-tree {{
            background: #1e1e1e;
            padding: 15px;
            border-radius: 4px;
            border-left: 3px solid #4ec9b0;
            margin: 10px 0;
            font-family: 'Consolas', monospace;
            font-size: 0.9em;
            white-space: pre;
            overflow-x: auto;
        }}
        
        /* Terminal-style syntax highlighting */
        .rule-name {{
            color: #569cd6; /* Bright blue - default rule color */
            font-weight: bold;
        }}
        
        .rule-name.keyword {{
            color: #dcdcaa; /* Bright yellow - for KW_ rules */
        }}
        
        .rule-name.structural {{
            color: #c586c0; /* Bright magenta - for main structure rules */
        }}
        
        .rule-name.text {{
            color: #9cdcfe; /* Bright cyan - for text/word rules */
        }}
        
        .tree-arrow {{
            color: #9cdcfe; /* Bright cyan - for arrows */
        }}
        
        .tree-brackets {{
            color: #d4d4d4; /* Dim white - for angle brackets */
        }}
        
        .tree-content {{
            color: #4ec9b0; /* Bright green - for content */
        }}
        
        .tree-structure {{
            color: #d4d4d4; /* White - for tree lines ├─ └─ │ */
        }}
        
        .error-message {{
            background: #3c1e1e;
            padding: 10px;
            border-radius: 4px;
            border-left: 3px solid #f44747;
            margin: 10px 0;
            font-family: 'Consolas', monospace;
            color: #ff6b6b;
        }}
    </style>
    <script>
        let currentFilter = 'all';
        let currentSpec = 'all';
        
        function filterTests(status) {{
            currentFilter = status;
            
            // Update active button
            document.querySelectorAll('.filter-btn').forEach(btn => {{
                btn.classList.remove('active');
            }});
            document.querySelector(`[onclick="filterTests('${{status}}')"]`).classList.add('active');
            
            applyFilters();
        }}
        
        function searchTests() {{
            const query = document.getElementById('search').value.toLowerCase();
            
            document.querySelectorAll('.test-item').forEach(item => {{
                const testName = item.querySelector('.test-name').textContent.toLowerCase();
                const testRule = item.querySelector('.test-rule') ? item.querySelector('.test-rule').textContent.toLowerCase() : '';
                const testInput = item.querySelector('.test-input') ? item.querySelector('.test-input').textContent.toLowerCase() : '';
                
                const matches = testName.includes(query) || testRule.includes(query) || testInput.includes(query);
                const statusVisible = currentFilter === 'all' || item.dataset.status === currentFilter;
                const specVisible = currentSpec === 'all' || item.dataset.spec === currentSpec;
                
                if (matches && statusVisible && specVisible) {{
                    item.classList.remove('hidden');
                }} else {{
                    item.classList.add('hidden');
                }}
            }});
            
            updateSectionVisibility();
        }}
        
        function filterBySpec(spec) {{
            currentSpec = spec;
            applyFilters();
        }}
        
        function applyFilters() {{
            document.querySelectorAll('.test-item').forEach(item => {{
                const testStatus = item.dataset.status;
                const testSpec = item.dataset.spec || '';
                
                const statusMatch = currentFilter === 'all' || testStatus === currentFilter;
                const specMatch = currentSpec === 'all' || testSpec === currentSpec;
                
                if (statusMatch && specMatch) {{
                    item.classList.remove('hidden');
                }} else {{
                    item.classList.add('hidden');
                }}
            }});
            
            updateSectionVisibility();
        }}
        
        function updateSectionVisibility() {{
            document.querySelectorAll('.section').forEach(section => {{
                const visibleItems = section.querySelectorAll('.test-item:not(.hidden)');
                if (visibleItems.length === 0) {{
                    section.classList.add('hidden');
                }} else {{
                    section.classList.remove('hidden');
                }}
            }});
        }}
        
        function toggleSection(header) {{
            const content = header.nextElementSibling;
            const icon = header.querySelector('.expand-icon');
            
            content.classList.toggle('expanded');
            icon.classList.toggle('rotated');
        }}
        
        // Initialize
        document.addEventListener('DOMContentLoaded', function() {{
            // Expand all sections by default
            document.querySelectorAll('.collapsible-content').forEach(content => {{
                content.classList.add('expanded');
            }});
            document.querySelectorAll('.expand-icon').forEach(icon => {{
                icon.classList.add('rotated');
            }});
        }});
    </script>
</head>
<body>
    <div class="header">
        <h1>🧪 Marco Grammar Test Results</h1>
        <div class="subtitle">Generated from JSON specifications • {}</div>
    </div>
    
    <div class="summary-grid">
        <div class="summary-card total">
            <div class="number">{}</div>
            <div>Total Tests</div>
        </div>
        <div class="summary-card passed">
            <div class="number">{}</div>
            <div>Passed</div>
        </div>
        <div class="summary-card failed">
            <div class="number">{}</div>
            <div>Failed</div>
        </div>
        <div class="summary-card expected">
            <div class="number">{}</div>
            <div>Expected Failures</div>
        </div>
        <div class="summary-card unknown">
            <div class="number">{}</div>
            <div>Unknown Rules</div>
        </div>
        <div class="summary-card rate">
            <div class="number">{:.1}%</div>
            <div>Success Rate</div>
        </div>
    </div>
    
    <div class="controls">
        <input type="text" id="search" class="search-box" placeholder="Search tests, rules, or input..." onkeyup="searchTests()">
        <div class="filter-buttons">
            <button class="filter-btn active" onclick="filterTests('all')">All</button>
            <button class="filter-btn" onclick="filterTests('passed')">✅ Passed</button>
            <button class="filter-btn" onclick="filterTests('failed')">❌ Failed</button>
            <button class="filter-btn" onclick="filterTests('expected')">⚠️ Expected</button>
            <button class="filter-btn" onclick="filterTests('unknown')">❓ Unknown</button>
            <select class="spec-filter" onchange="filterBySpec(this.value)">
                <option value="all">All Specs</option>
                <option value="CommonMark">CommonMark</option>
                <option value="GitHub Flavored Markdown">GitHub Flavored Markdown</option>
                <option value="Pandoc Extensions">Pandoc Extensions</option>
                <option value="Marco Extensions">Marco Extensions</option>
                <option value="Custom Tests">Custom Tests</option>
            </select>
        </div>
    </div>
    
    {}
    
    <div class="stats-bar">
        <strong>Summary:</strong> {} passed • {} failed • {} expected failures • {} unknown rules
    </div>
</body>
</html>"#,
        timestamp,
        results.total_tests,
        results.total_passed,
        results.total_failed,
        results.total_expected_failures,
        results.total_unknown_rules,
        results.success_rate,
        generate_sections_html(results),
        results.total_passed,
        results.total_failed,
        results.total_expected_failures,
        results.total_unknown_rules,
    );

    fs::write(report_path, html_content)?;
    println!(
        "{}📄 HTML report saved to {}/test_results.html{}",
        BRIGHT_GREEN, results_dir, RESET
    );

    Ok(())
}

// Generate HTML sections for the detailed results
fn generate_sections_html(results: &OverallResults) -> String {
    let mut sections_html = String::new();

    for suite_result in &results.suite_results {
        let mut categories: std::collections::HashMap<String, Vec<&TestResult>> =
            std::collections::HashMap::new();
        for result in &suite_result.results {
            categories
                .entry(result.test_case.section.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        for (category, test_results) in categories {
            let total_items = test_results.len();
            let passed_items = test_results
                .iter()
                .filter(|item| matches!(item.status, TestStatus::Passed))
                .count();

            sections_html.push_str(&format!(
                r#"<div class="section">
        <div class="section-header" onclick="toggleSection(this)">
            <span><span class="expand-icon">▶</span> {} - {}</span>
            <span class="section-stats">{}/{} passed</span>
        </div>
        <div class="collapsible-content">
            {}</div>
    </div>"#,
                suite_result.suite_name,
                category,
                passed_items,
                total_items,
                generate_test_items_html(&test_results, &suite_result.suite_name)
            ));
        }
    }

    sections_html
}

// Generate HTML for individual test items
fn generate_test_items_html(test_results: &[&TestResult], suite_name: &str) -> String {
    let mut items_html = String::new();

    for result in test_results {
        let (status_class, status_text, data_status) = match result.status {
            TestStatus::Passed => ("status-passed", "✅ PASSED", "passed"),
            TestStatus::Failed => ("status-failed", "❌ FAILED", "failed"),
            TestStatus::ExpectedFailure => ("status-expected", "⚠️ EXPECTED", "expected"),
            TestStatus::UnknownRule => ("status-unknown", "❓ UNKNOWN", "unknown"),
            TestStatus::ParseError => ("status-error", "💥 ERROR", "failed"),
            TestStatus::Skipped => ("status-unknown", "⏭️ SKIPPED", "unknown"),
        };

        let description = result.test_case.description.as_deref();
        let fallback = format!("Example {}", result.test_case.example);
        let description = description.unwrap_or(&fallback);

        let parse_tree_html = if let Some(tree) = &result.parse_tree {
            format!(r#"<div class="parse-tree">{}</div>"#, tree)
        } else {
            String::new()
        };

        let error_html = if let Some(error) = &result.error_message {
            format!(
                r#"<div class="error-message">{}</div>"#,
                error.replace('<', "&lt;").replace('>', "&gt;")
            )
        } else {
            String::new()
        };

        items_html.push_str(&format!(
            r#"<div class="test-item" data-status="{}" data-spec="{}">
                <div class="test-header">
                    <div class="test-name">{}</div>
                    <div class="test-status {}">{}</div>
                </div>
                <div class="test-details">
                    <strong>Rule:</strong> <span class="test-rule">{}</span>
                </div>
                <div class="test-input">{}</div>
                {}{}
            </div>"#,
            data_status,
            suite_name,
            description,
            status_class,
            status_text,
            result.test_case.rule.as_deref().unwrap_or("document"),
            result
                .test_case
                .markdown
                .replace('<', "&lt;")
                .replace('>', "&gt;"),
            parse_tree_html,
            error_html
        ));
    }

    items_html
}

// Generate benchmark results report
fn generate_benchmark_report(
    results: &OverallResults,
    total_time: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("src/results")?;

    let mut output = fs::File::create("src/results/benchmark_results.md")?;
    use std::io::Write;

    writeln!(output, "# Marco Grammar Performance Benchmark Report\n")?;
    writeln!(
        output,
        "Generated automatically from JSON specification tests\n"
    )?;

    writeln!(output, "## Summary\n")?;
    writeln!(output, "- **Total Tests**: {}", results.total_tests)?;
    writeln!(output, "- **Passed**: {} ✅", results.total_passed)?;
    writeln!(output, "- **Failed**: {} ❌", results.total_failed)?;
    writeln!(
        output,
        "- **Expected Failures**: {} ⚠️",
        results.total_expected_failures
    )?;
    writeln!(
        output,
        "- **Unknown Rules**: {} ❓",
        results.total_unknown_rules
    )?;
    writeln!(output, "- **Total Time**: {:.3}s", total_time.as_secs_f64())?;
    writeln!(
        output,
        "- **Average Time per Test**: {:.3}ms",
        total_time.as_secs_f64() * 1000.0 / results.total_tests as f64
    )?;

    // Add summary line like the old system
    writeln!(
        output,
        "\n**Summary**: {} passed • {} failed • {} expected failures • {} unknown rules\n",
        results.total_passed,
        results.total_failed,
        results.total_expected_failures,
        results.total_unknown_rules
    )?;

    writeln!(output, "\n## Performance by Suite\n")?;
    writeln!(
        output,
        "| Suite | Tests | Passed | Time (ms) | Avg (ms) | Success Rate |"
    )?;
    writeln!(
        output,
        "|-------|-------|--------|-----------|----------|--------------|"
    )?;

    for suite_result in &results.suite_results {
        let suite_time_ms = total_time.as_secs_f64() * 1000.0 / results.suite_results.len() as f64;
        let avg_time_ms = suite_time_ms / suite_result.total_tests as f64;
        let success_rate = if suite_result.total_tests > 0 {
            (suite_result.passed as f64 / suite_result.total_tests as f64) * 100.0
        } else {
            0.0
        };

        writeln!(
            output,
            "| {} | {} | {} | {:.3} | {:.3} | {:.1}% |",
            suite_result.suite_name,
            suite_result.total_tests,
            suite_result.passed,
            suite_time_ms,
            avg_time_ms,
            success_rate
        )?;
    }

    writeln!(output, "\n## Performance Analysis\n")?;

    // Analyze by test category
    for suite_result in &results.suite_results {
        writeln!(output, "### {}\n", suite_result.suite_name)?;

        let mut categories: std::collections::HashMap<String, Vec<&TestResult>> =
            std::collections::HashMap::new();
        for result in &suite_result.results {
            categories
                .entry(result.test_case.section.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        writeln!(output, "| Category | Tests | Passed | Input Size (avg) |")?;
        writeln!(output, "|----------|-------|--------|------------------|")?;

        for (category, tests) in categories {
            let avg_size = tests
                .iter()
                .map(|t| t.test_case.markdown.len())
                .sum::<usize>()
                / tests.len().max(1);
            let passed = tests
                .iter()
                .filter(|t| matches!(t.status, TestStatus::Passed))
                .count();

            writeln!(
                output,
                "| {} | {} | {} | {} chars |",
                category,
                tests.len(),
                passed,
                avg_size
            )?;
        }
        writeln!(output)?;
    }

    writeln!(
        output,
        "\n---\n*Report generated by Marco Grammar Test Suite*"
    )?;

    println!(
        "{}📊 Benchmark report saved to src/results/benchmark_results.md{}",
        BRIGHT_GREEN, RESET
    );

    Ok(())
}

// Test individual rule with input
fn test_individual_rule(rule_name: &str, input: &str) {
    println!(
        "{}🔍 Testing rule '{}' with input:{}",
        BRIGHT_CYAN, rule_name, RESET
    );
    println!(
        "{}Input:{} {}",
        BRIGHT_BLUE,
        RESET,
        input.replace('\n', "\\n")
    );

    if let Some(rule) = string_to_rule(rule_name) {
        match MarcoParser::parse(rule, input) {
            Ok(pairs) => {
                println!("{}✅ Parse successful!{}", BRIGHT_GREEN, RESET);

                match into_ascii_tree(pairs) {
                    Ok(tree) => {
                        println!("\n{}🌳 Parse Tree:{}", BRIGHT_CYAN, RESET);
                        println!("{}", tree);
                    }
                    Err(e) => {
                        eprintln!("{}❌ Tree formatting error: {}{}", BRIGHT_RED, e, RESET);
                    }
                }
            }
            Err(e) => {
                println!("{}❌ Parse failed:{}", BRIGHT_RED, RESET);
                println!("{}", e);
            }
        }
    } else {
        println!("{}❓ Unknown rule: {}{}", BRIGHT_YELLOW, rule_name, RESET);
        println!("Available rules: document, block, paragraph, heading, admonition_block, user_mention, etc.");
    }
}

// Legacy function mapping for backward compatibility
fn get_rule(rule_name: &str) -> Option<Rule> {
    string_to_rule(rule_name)
}

// Original batch test runner using JSON specs
fn run_batch_tests() {
    println!("🧪 Running batch tests from JSON specifications...");

    match load_test_config() {
        Ok(config) => match execute_all_tests(&config) {
            Ok(results) => {
                display_results_summary(&results);

                if let Err(e) = generate_markdown_report(&results, &config) {
                    eprintln!(
                        "{}⚠️  Failed to generate report: {}{}",
                        BRIGHT_YELLOW, e, RESET
                    );
                }
            }
            Err(e) => {
                eprintln!("{}❌ Test execution failed: {}{}", BRIGHT_RED, e, RESET);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!(
                "{}❌ Failed to load test configuration: {}{}",
                BRIGHT_RED, e, RESET
            );
            eprintln!("Make sure test_config.json exists and is valid.");
            std::process::exit(1);
        }
    }
}

// Original single test runner with exact formatting
fn run_single_test(rule_name: &str, input: &str) {
    // Map rule name to Rule enum
    let rule = match get_rule(rule_name) {
        Some(r) => r,
        None => {
            println!("==========================================================");
            println!(
                "🔍 Testing rule: {} with input: \"{}\"",
                rule_name,
                escape_content_for_display(input)
            );
            println!("💬️ Expected Pass");
            println!("❓️ Note: unknown rule(s)");
            println!("--------------------------------------");
            println!("❓️ Parse error: Unknown rule: {}", rule_name);
            println!("Available rules: file, document, paragraph, text, word, emoji, H1-H6, bold, italic, etc.");
            println!("==========================================================");
            std::process::exit(1);
        }
    };

    // Determine if this test pattern is expected to fail
    let expected_to_fail = determine_expected_failure_for_single_test(rule_name, input);

    println!("==========================================================");
    println!(
        "🔍 Testing rule: {} with input: \"{}\"",
        rule_name,
        escape_content_for_display(input)
    );

    // Display expectation
    if expected_to_fail {
        println!("💬️ Expected Failure");
    } else {
        println!("💬️ Expected Pass");
    }

    let result = MarcoParser::parse(rule, input);
    match &result {
        Ok(pairs) => {
            let pairs_clone = pairs.clone();
            let count = pairs_clone.count();
            println!("✅ Parse successful! Generated {} pairs", count);

            if expected_to_fail {
                println!("❌ Status: Unexpected Success (Expected to Fail)");
            } else {
                println!("✅ Status: Success");
            }
        }
        Err(e) => {
            if expected_to_fail {
                println!("✅ Status: Expected Failure");
            } else {
                println!("❌ Status: Unexpected Failure");
            }
            println!("❌ Parse error: {}", e);
        }
    }
    println!("==========================================================");
}

// Original tree test runner with ASCII visualization
fn run_tree_test(rule_name: &str, input: &str) {
    // Map rule name to Rule enum
    let rule = match get_rule(rule_name) {
        Some(r) => r,
        None => {
            println!("==========================================================");
            println!(
                "🔍 Testing rule: {} with input: \"{}\"",
                rule_name,
                escape_content_for_display(input)
            );
            println!("💬️ Expected Pass");
            println!("❓️ Note: unknown rule(s)");
            println!("--------------------------------------");
            println!("❓️ Parse error: Unknown rule: {}", rule_name);
            println!("Available rules: file, document, paragraph, text, word, emoji, H1-H6, bold, italic, etc.");
            println!("==========================================================");
            std::process::exit(1);
        }
    };

    // Determine if this test pattern is expected to fail
    let expected_to_fail = determine_expected_failure_for_single_test(rule_name, input);

    println!("==========================================================");
    println!(
        "🔍 Testing rule: {} with input: \"{}\"",
        rule_name,
        escape_content_for_display(input)
    );

    // Display expectation
    if expected_to_fail {
        println!("💬️ Expected Failure");
    } else {
        println!("💬️ Expected Pass");
    }

    let result = MarcoParser::parse(rule, input);
    match &result {
        Ok(pairs) => {
            let pairs_clone = pairs.clone();
            let count = pairs_clone.count();
            println!("✅ Parse successful! Generated {} pairs", count);

            if expected_to_fail {
                println!("❌ Status: Unexpected Success (Expected to Fail)");
            } else {
                println!("✅ Status: Success");
            }

            // Show ASCII tree
            print_ascii_tree_result(Ok(pairs.clone()));
        }
        Err(e) => {
            if expected_to_fail {
                println!("✅ Status: Expected Failure");
            } else {
                println!("❌ Status: Unexpected Failure");
            }
            println!("❌ Parse error: {}", e);
        }
    }
    println!("==========================================================");
}

// Original ASCII tree printing function
fn print_ascii_tree_result(parsing_result: Result<Pairs<Rule>, pest::error::Error<Rule>>) {
    match parsing_result {
        Ok(pairs) => match into_ascii_tree(pairs) {
            Ok(output) => {
                println!(
                    "{}🌳 Enhanced Parse Tree Visualization:{}",
                    BRIGHT_CYAN, RESET
                );
                println!("{}", output);
            }
            Err(e) => {
                eprintln!("{}❌ Tree formatting error: {}{}", BRIGHT_RED, e, RESET);
            }
        },
        Err(e) => {
            eprintln!("{}❌ Parse error: {}{}", BRIGHT_RED, e, RESET);
        }
    }
}

// Expected failure determination for backward compatibility
fn determine_expected_failure_for_single_test(rule_name: &str, input: &str) -> bool {
    // Simple heuristics based on the original logic
    if input.contains("expected_failure") || input.contains("should_fail") {
        return true;
    }

    // Check for patterns that are known to fail in Marco grammar
    match rule_name {
        "emoji" if !input.starts_with(':') || !input.ends_with(':') => true,
        "user_mention" if input == "@" => true,
        "admonition_block"
            if input.starts_with(":::")
                && !input.contains("note")
                && !input.contains("tip")
                && !input.contains("warning")
                && !input.contains("danger")
                && !input.contains("info") =>
        {
            true
        }
        _ => false,
    }
}

// Benchmark testing function
fn run_benchmark_tests() {
    println!("🏁 Running performance benchmarks...");

    match load_test_config() {
        Ok(config) => {
            let start_time = Instant::now();

            match execute_all_tests(&config) {
                Ok(results) => {
                    let total_time = start_time.elapsed();

                    println!("\n{}📈 Benchmark Results{}", BRIGHT_CYAN, RESET);
                    println!("{}═══════════════════════{}", BRIGHT_CYAN, RESET);

                    println!(
                        "Total Tests: {}{}{}",
                        BRIGHT_CYAN, results.total_tests, RESET
                    );
                    println!(
                        "Total Time: {}{:.3}s{}",
                        BRIGHT_CYAN,
                        total_time.as_secs_f64(),
                        RESET
                    );
                    println!(
                        "Average Time per Test: {}{:.3}ms{}",
                        BRIGHT_CYAN,
                        (total_time.as_secs_f64() * 1000.0) / results.total_tests as f64,
                        RESET
                    );

                    // Find the slowest suite
                    if let Some(slowest_suite) = results
                        .suite_results
                        .iter()
                        .max_by(|a, b| a.total_time.cmp(&b.total_time))
                    {
                        println!(
                            "Slowest Suite: {}{}{} ({:.3}s)",
                            BRIGHT_YELLOW,
                            slowest_suite.suite_name,
                            RESET,
                            slowest_suite.total_time.as_secs_f64()
                        );
                    }

                    // Performance analysis
                    println!("\n{}Performance Analysis:{}", BRIGHT_BLUE, RESET);
                    for suite_result in &results.suite_results {
                        let avg_time =
                            suite_result.total_time.as_secs_f64() / suite_result.total_tests as f64;
                        println!(
                            "  {}: {:.3}ms per test",
                            suite_result.suite_name,
                            avg_time * 1000.0
                        );
                    }

                    // Generate benchmark report
                    if let Err(e) = generate_benchmark_report(&results, total_time) {
                        eprintln!(
                            "{}❌ Failed to generate benchmark report: {}{}",
                            BRIGHT_RED, e, RESET
                        );
                    }
                }
                Err(e) => {
                    eprintln!("{}❌ Benchmark failed: {}{}", BRIGHT_RED, e, RESET);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!(
                "{}❌ Failed to load test configuration: {}{}",
                BRIGHT_RED, e, RESET
            );
            std::process::exit(1);
        }
    }
}

// Multiline paragraph testing function
fn test_multiline_paragraphs() {
    println!("📝 Testing multiline paragraph parsing...");

    let test_cases = vec![
        "This is a single line paragraph.",
        "This is a\nmultiline paragraph\nwith three lines.",
        "Line one\n\nLine two after blank line",
        "Paragraph with **bold**\nand *italic* text\nacross multiple lines.",
        "# Heading\n\nParagraph after heading\nwith multiple lines.",
    ];

    for (i, test_case) in test_cases.iter().enumerate() {
        println!("\n--- Test Case {} ---", i + 1);
        run_tree_test("paragraph", test_case);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        // Batch mode - run all test cases
        run_batch_tests();
    } else if args.len() == 2 && args[1] == "--benchmark" {
        // Benchmark mode
        run_benchmark_tests();
    } else if args.len() == 2 && args[1] == "--grammar" {
        // Grammar visualization mode - show entire grammar structure
        grammar_visualizer::show_grammar_tree();
    } else if args.len() == 2 && args[1] == "--analyze" {
        // Grammar analysis mode - detailed breakdown
        grammar_visualizer::analyze_grammar_structure();
    } else if args.len() == 2 && args[1] == "--multiline" {
        // Test multiline paragraph parsing
        test_multiline_paragraphs();
    } else if args.len() == 2 && args[1] == "--tree" {
        eprintln!("Usage for --tree mode:");
        eprintln!(
            "  {} --tree <rule> <input>    - Show ASCII tree visualization",
            args[0]
        );
        eprintln!("Example: {} --tree text 'Hello **world**'", args[0]);
        std::process::exit(1);
    } else if args.len() == 4 && args[1] == "--tree" {
        // Tree mode - single test with ASCII tree visualization
        let rule_name = &args[2];
        let input = &args[3];
        run_tree_test(rule_name, input);
    } else if args.len() == 3 {
        // Single test mode
        let rule_name = &args[1];
        let input = &args[2];
        run_single_test(rule_name, input);
    } else {
        eprintln!("Usage:");
        eprintln!("  {} <rule> <input>           - Test single rule", args[0]);
        eprintln!(
            "  {} --tree <rule> <input>    - Show ASCII tree visualization",
            args[0]
        );
        eprintln!(
            "  {} --grammar                - Show complete grammar structure",
            args[0]
        );
        eprintln!(
            "  {} --analyze                - Detailed grammar analysis",
            args[0]
        );
        eprintln!(
            "  {} --multiline              - Test multiline paragraph parsing",
            args[0]
        );
        eprintln!(
            "  {}                          - Run all test cases",
            args[0]
        );
        eprintln!(
            "  {} --benchmark              - Run performance benchmarks",
            args[0]
        );
        eprintln!("Example: {} emoji ':smile:'", args[0]);
        eprintln!("Example: {} --tree bold '**hello**'", args[0]);
        eprintln!("Example: {} --grammar", args[0]);
        std::process::exit(1);
    }
}

// Parse tree formatting functions (copied from old TOML main)
fn format_parse_tree_html(pair: pest::iterators::Pair<Rule>) -> String {
    let mut output = String::new();
    format_parse_tree_html_enhanced(&mut output, pair, 0);
    output
}

fn format_parse_tree_html_enhanced(
    output: &mut String,
    pair: pest::iterators::Pair<Rule>,
    depth: usize,
) {
    format_parse_tree_html_recursive(output, pair, depth, &vec![false; depth]);
}

fn format_parse_tree_html_recursive(
    output: &mut String,
    pair: pest::iterators::Pair<Rule>,
    depth: usize,
    is_last_at_level: &Vec<bool>, // true if this is the last child at each level
) {
    let rule_name = format!("{:?}", pair.as_rule());
    let text = pair.as_str();
    let escaped_text = escape_html_content(text);

    // Clone the pair to check if it has children
    let children: Vec<_> = pair.clone().into_inner().collect();
    let has_children = !children.is_empty();

    // Create indentation with proper tree characters
    let indent = if depth == 0 {
        " ".to_string() // Root node gets a space
    } else {
        let mut indent_parts = Vec::new();

        // For each level before the current one
        for level in 0..(depth - 1) {
            if is_last_at_level.get(level).unwrap_or(&false) == &false {
                // Parent is not last, so draw vertical line
                indent_parts.push("<span class=\"tree-structure\">│  </span>");
            } else {
                // Parent is last, so just spaces
                indent_parts.push("   ");
            }
        }

        // For the current level
        if is_last_at_level.get(depth - 1).unwrap_or(&false) == &false {
            // This is not the last child
            indent_parts.push("<span class=\"tree-structure\">├─ </span>");
        } else {
            // This is the last child
            indent_parts.push("<span class=\"tree-structure\">└─ </span>");
        }

        indent_parts.join("")
    };

    // Determine rule CSS class based on rule name
    let rule_class = if rule_name.starts_with("KW_") {
        "rule-name keyword"
    } else if matches!(
        rule_name.as_str(),
        "file" | "document" | "block" | "inline" | "paragraph"
    ) {
        "rule-name structural"
    } else if rule_name.contains("text") || rule_name.contains("word") {
        "rule-name text"
    } else {
        "rule-name"
    };

    // Format the output line with color classes
    output.push_str(&format!(
        "{}<span class=\"{}\">{}</span> <span class=\"tree-arrow\">→</span> <span class=\"tree-brackets\">⟨</span><span class=\"tree-content\">{}</span><span class=\"tree-brackets\">⟩</span>\n",
        indent, rule_class, rule_name, escaped_text
    ));

    if has_children {
        for (i, inner_pair) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            let mut new_is_last_at_level = is_last_at_level.clone();

            // Extend to include this level
            if new_is_last_at_level.len() <= depth {
                new_is_last_at_level.resize(depth + 1, false);
            }
            new_is_last_at_level[depth] = is_last_child;

            format_parse_tree_html_recursive(
                output,
                inner_pair.clone(),
                depth + 1,
                &new_is_last_at_level,
            );
        }
    }
}

fn escape_html_content(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
}
