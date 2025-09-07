use pest::Parser;
use pest_derive::Parser;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

// ASCII tree visualization
extern crate ascii_tree;
extern crate escape_string;
use pest::iterators::Pairs;

// Grammar visualization module
mod grammar_visualizer;

// Use the shared grammar file from the main project
#[derive(Parser)]
#[grammar = "../src/components/marco_engine/grammar/marco.pest"]
pub struct MarcoParser;

#[derive(Debug)]
struct TestResult {
    name: String,
    rule: String,
    input: String,
    success: bool,
    error: Option<String>,
    parse_time: Duration,
    memory_estimate: usize,
}

#[derive(Debug)]
struct TestResultItem {
    test_name: String,
    rule_name: String,
    input_text: String,
    status: TestStatus,
    parse_tree: Option<String>,
    error_message: Option<String>,
}

#[derive(Debug, Clone)]
enum TestStatus {
    Passed,
    Failed,
    ExpectedFailure,
    UnknownRule,
}

#[derive(Debug)]
struct BenchmarkStats {
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    total_time: Duration,
    average_time: Duration,
    slowest_test: Option<(String, Duration)>,
    fastest_test: Option<(String, Duration)>,
    memory_total: usize,
}

// Enhanced Tree visualization functions with colors and better formatting
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

// Color codes for terminal output
const RESET: &str = "\x1b[0m";
const BRIGHT_BLUE: &str = "\x1b[94m"; // Rule names
const BRIGHT_GREEN: &str = "\x1b[92m"; // Content
const BRIGHT_YELLOW: &str = "\x1b[93m"; // Keywords
const BRIGHT_CYAN: &str = "\x1b[96m"; // Operators
const BRIGHT_MAGENTA: &str = "\x1b[95m"; // Special rules
const DIM_WHITE: &str = "\x1b[37m"; // Brackets

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
        .take(80) // Limit content length
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
                eprintln!("{}❌ Tree formatting error: {}{}", "\x1b[91m", e, RESET);
            }
        },
        Err(e) => {
            eprintln!("{}❌ Parse error: {}{}", "\x1b[91m", e, RESET);
        }
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
            "  {}                          - Run all test cases from test_cases.toml",
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

fn run_batch_tests() {
    println!("🧪 Running batch tests from test_cases.toml...");

    // Read test cases file
    let test_content = match fs::read_to_string("test_cases.toml") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read test_cases.toml: {}", e);
            std::process::exit(1);
        }
    };

    // Parse TOML content manually (simple parser)
    let test_cases = parse_test_cases(&test_content);

    // Collect test results in memory first
    let mut test_results: Vec<TestResult> = Vec::new();
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut expected_failures = 0;
    let mut unknown_rules = 0;
    let mut group_failures: HashMap<String, usize> = HashMap::new();
    let mut section_results: HashMap<String, Vec<TestResultItem>> = HashMap::new();

    for (section, cases) in test_cases {
        let mut section_items = Vec::new();

        for (test_name, input_text) in cases {
            total_tests += 1;

            // Determine rule from section or test name
            let rule_name = determine_rule(&section, &test_name);
            let rule = match get_rule(&rule_name) {
                Some(r) => r,
                None => {
                    let result_item = TestResultItem {
                        test_name: test_name.clone(),
                        rule_name: rule_name.clone(),
                        input_text: input_text.clone(),
                        status: TestStatus::UnknownRule,
                        parse_tree: None,
                        error_message: Some(format!("Unknown rule `{}`", rule_name)),
                    };
                    section_items.push(result_item);
                    failed_tests += 1;
                    unknown_rules += 1;
                    *group_failures.entry(section.clone()).or_insert(0) += 1;
                    continue;
                }
            };

            // Test the rule
            match MarcoParser::parse(rule, &input_text) {
                Ok(pairs) => {
                    let mut tree_output = String::new();
                    for pair in pairs {
                        format_parse_tree_html(&mut tree_output, pair, 0);
                    }

                    let result_item = TestResultItem {
                        test_name: test_name.clone(),
                        rule_name: rule_name.clone(),
                        input_text: input_text.clone(),
                        status: TestStatus::Passed,
                        parse_tree: Some(tree_output),
                        error_message: None,
                    };
                    section_items.push(result_item);
                    passed_tests += 1;
                }
                Err(e) => {
                    // Check if this was expected to fail
                    let is_expected_failure = test_name.contains("failure")
                        || test_name.contains("invalid")
                        || test_name.contains("malformed")
                        || test_name.contains("empty")
                        || test_name.contains("no_")
                        || test_name.contains("missing");

                    let status = if is_expected_failure {
                        TestStatus::ExpectedFailure
                    } else {
                        TestStatus::Failed
                    };

                    let result_item = TestResultItem {
                        test_name: test_name.clone(),
                        rule_name: rule_name.clone(),
                        input_text: input_text.clone(),
                        status,
                        parse_tree: None,
                        error_message: Some(e.to_string()),
                    };
                    section_items.push(result_item);

                    if is_expected_failure {
                        passed_tests += 1;
                        expected_failures += 1;
                    } else {
                        failed_tests += 1;
                        *group_failures.entry(section.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        section_results.insert(section, section_items);
    }

    // Generate HTML report
    generate_test_results_html(
        &section_results,
        total_tests,
        passed_tests,
        failed_tests,
        expected_failures,
        unknown_rules,
        &group_failures,
    );

    println!("✅ Batch testing complete!");

    // Show unknown rules count first
    if unknown_rules > 0 {
        println!("❌ Unknown rules: {}", unknown_rules);
    } else {
        println!("✅ Unknown rules: 0");
    }

    // Show passed and failed tests separately
    println!("✅ Passed tests: {}", passed_tests);
    if expected_failures > 0 {
        println!("✅ (Including {} expected failures)", expected_failures);
    }
    println!("❌ Failed tests: {} (unexpected failures)", failed_tests);

    println!(
        "📊 Results: {}/{} passed ({:.1}%)",
        passed_tests,
        total_tests,
        (passed_tests as f64 / total_tests as f64) * 100.0
    );

    // Show group failures (only groups that have failures)
    if !group_failures.is_empty() {
        let total_group_failures: usize = group_failures.values().sum();
        println!("\nGroup failures (total: {}):", total_group_failures);
        let mut sorted_failures: Vec<_> = group_failures.iter().collect();
        sorted_failures.sort_by(|a, b| b.1.cmp(a.1)); // Sort by failure count descending

        for (group, count) in sorted_failures {
            println!("  {}: {}", group, count);
        }
    }

    println!("📝 Detailed results written to src/results/test_results.html");
}

fn generate_test_results_html(
    section_results: &HashMap<String, Vec<TestResultItem>>,
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    expected_failures: usize,
    unknown_rules: usize,
    _group_failures: &HashMap<String, usize>,
) {
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
        .summary-card.total {{ border-left-color: #569cd6; }}
        .summary-card.rate {{ border-left-color: #dcdcaa; }}
        
        .summary-card .number {{
            font-size: 2em;
            font-weight: bold;
            margin-bottom: 10px;
        }}
        
        .summary-card.passed .number {{ color: #4ec9b0; }}
        .summary-card.failed .number {{ color: #f44747; }}
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
            transition: background-color 0.2s;
        }}
        
        .test-item:hover {{
            background: #3c3c3c;
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
        
        .test-details {{
            margin: 10px 0;
        }}
        
        .test-rule {{
            color: #569cd6;
            font-family: 'Consolas', monospace;
            background: #3c3c3c;
            padding: 2px 6px;
            border-radius: 3px;
        }}
        
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
        
        .hidden {{
            display: none !important;
        }}
        
        .stats-bar {{
            background: #252525;
            padding: 10px 20px;
            text-align: center;
            font-size: 0.9em;
            color: #9cdcfe;
        }}
    </style>
    <script>
        let currentFilter = 'all';
        
        function filterTests(status) {{
            currentFilter = status;
            
            // Update active button
            document.querySelectorAll('.filter-btn').forEach(btn => {{
                btn.classList.remove('active');
            }});
            document.querySelector(`[onclick="filterTests('${{status}}')"]`).classList.add('active');
            
            // Filter test items
            document.querySelectorAll('.test-item').forEach(item => {{
                const testStatus = item.dataset.status;
                if (status === 'all' || testStatus === status) {{
                    item.classList.remove('hidden');
                }} else {{
                    item.classList.add('hidden');
                }}
            }});
            
            updateSectionVisibility();
        }}
        
        function searchTests() {{
            const query = document.getElementById('search').value.toLowerCase();
            
            document.querySelectorAll('.test-item').forEach(item => {{
                const testName = item.querySelector('.test-name').textContent.toLowerCase();
                const testRule = item.querySelector('.test-rule').textContent.toLowerCase();
                const testInput = item.querySelector('.test-input').textContent.toLowerCase();
                
                const matches = testName.includes(query) || testRule.includes(query) || testInput.includes(query);
                const statusVisible = currentFilter === 'all' || item.dataset.status === currentFilter;
                
                if (matches && statusVisible) {{
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
        <div class="subtitle">Generated automatically from test_cases.toml • {timestamp}</div>
    </div>
    
    <div class="summary-grid">
        <div class="summary-card total">
            <div class="number">{total_tests}</div>
            <div>Total Tests</div>
        </div>
        <div class="summary-card passed">
            <div class="number">{passed_tests}</div>
            <div>Passed</div>
        </div>
        <div class="summary-card failed">
            <div class="number">{failed_tests}</div>
            <div>Failed</div>
        </div>
        <div class="summary-card rate">
            <div class="number">{success_rate:.1}%</div>
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
        </div>
    </div>
    
    {sections_html}
    
    <div class="stats-bar">
        <strong>Summary:</strong> {passed_tests} passed • {failed_tests} failed • {expected_failures} expected failures • {unknown_rules} unknown rules
    </div>
</body>
</html>"#,
        timestamp = get_current_timestamp(),
        total_tests = total_tests,
        passed_tests = passed_tests,
        failed_tests = failed_tests,
        success_rate = (passed_tests as f64 / total_tests as f64) * 100.0,
        expected_failures = expected_failures,
        unknown_rules = unknown_rules,
        sections_html = generate_sections_html(section_results),
    );

    match fs::write("src/results/test_results.html", html_content) {
        Ok(_) => println!("📄 HTML test results written to src/results/test_results.html"),
        Err(e) => eprintln!("❌ Failed to write HTML file: {}", e),
    }
}

fn generate_sections_html(section_results: &HashMap<String, Vec<TestResultItem>>) -> String {
    let mut sections_html = String::new();

    // Sort sections by name
    let mut sorted_sections: Vec<_> = section_results.iter().collect();
    sorted_sections.sort_by_key(|(name, _)| *name);

    for (section_name, items) in sorted_sections {
        let total_items = items.len();
        let passed_items = items
            .iter()
            .filter(|item| {
                matches!(
                    item.status,
                    TestStatus::Passed | TestStatus::ExpectedFailure
                )
            })
            .count();

        sections_html.push_str(&format!(
            r#"<div class="section">
        <div class="section-header" onclick="toggleSection(this)">
            <span>▶ {}</span>
            <span class="section-stats">{}/{} passed</span>
        </div>
        <div class="collapsible-content">
            {}</div>
    </div>"#,
            section_name,
            passed_items,
            total_items,
            generate_test_items_html(items)
        ));
    }

    sections_html
}

fn generate_test_items_html(items: &[TestResultItem]) -> String {
    let mut items_html = String::new();

    for item in items {
        let (status_class, status_text, status_data) = match item.status {
            TestStatus::Passed => ("status-passed", "✅ Passed", "passed"),
            TestStatus::Failed => ("status-failed", "❌ Failed", "failed"),
            TestStatus::ExpectedFailure => ("status-expected", "⚠️ Expected", "expected"),
            TestStatus::UnknownRule => ("status-unknown", "❓ Unknown", "unknown"),
        };

        let content = if let Some(tree) = &item.parse_tree {
            format!(
                r#"<div class="parse-tree">{}</div>"#,
                tree // Don't escape - tree already contains proper HTML spans
            )
        } else if let Some(error) = &item.error_message {
            format!(
                r#"<div class="error-message">{}</div>"#,
                escape_html_content(error)
            )
        } else {
            String::new()
        };

        items_html.push_str(&format!(
            r#"<div class="test-item" data-status="{}">
                <div class="test-header">
                    <span class="test-name">{}</span>
                    <span class="test-status {}">{}</span>
                </div>
                <div class="test-details">
                    <strong>Rule:</strong> <span class="test-rule">{}</span>
                </div>
                <div class="test-input">{}</div>
                {}
            </div>"#,
            status_data,
            escape_html_content(&item.test_name),
            status_class,
            status_text,
            escape_html_content(&item.rule_name),
            escape_html_content(&item.input_text),
            content
        ));
    }

    items_html
}

fn format_parse_tree_html(output: &mut String, pair: pest::iterators::Pair<Rule>, depth: usize) {
    format_parse_tree_html_enhanced(output, pair, depth);
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

fn escape_html_preserve_structure(text: &str) -> String {
    // Only escape HTML characters, preserve actual newlines for tree structure
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn get_current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simple date formatting (YYYY-MM-DD)
    let days_since_epoch = now / 86400;
    let years_since_1970 = days_since_epoch / 365;
    let year = 1970 + years_since_1970;

    format!("{}-09-07", year) // Simplified for demo
}

fn run_benchmark_tests() {
    println!("🚀 Running performance benchmarks...");

    // Read test cases file
    let test_content = match fs::read_to_string("test_cases.toml") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read test_cases.toml: {}", e);
            std::process::exit(1);
        }
    };

    let test_cases = parse_test_cases(&test_content);
    let mut results = Vec::new();
    let overall_start = Instant::now();

    // Focus on benchmark_tests section and other performance-relevant sections
    let benchmark_sections = [
        "benchmark_tests",
        "memory_stress",
        "performance_tests",
        "pathological_inputs",
    ];

    for section_name in &benchmark_sections {
        if let Some(cases) = test_cases.get(*section_name) {
            println!("🔥 Benchmarking section: {}", section_name);

            for (test_name, input_text) in cases {
                let rule_name = determine_rule(section_name, test_name);
                let rule = match get_rule(&rule_name) {
                    Some(r) => r,
                    None => {
                        println!("⚠️  Skipping unknown rule: {}", rule_name);
                        continue;
                    }
                };

                // Warm up (3 runs)
                for _ in 0..3 {
                    let _ = MarcoParser::parse(rule, input_text);
                }

                // Benchmark (10 runs)
                let mut times = Vec::new();
                for _ in 0..10 {
                    let start = Instant::now();
                    let parse_result = MarcoParser::parse(rule, input_text);
                    let duration = start.elapsed();
                    times.push(duration);

                    match parse_result {
                        Ok(_) => {
                            results.push(TestResult {
                                name: test_name.clone(),
                                rule: rule_name.clone(),
                                input: input_text.clone(),
                                success: true,
                                error: None,
                                parse_time: duration,
                                memory_estimate: input_text.len(),
                            });
                        }
                        Err(e) => {
                            results.push(TestResult {
                                name: test_name.clone(),
                                rule: rule_name.clone(),
                                input: input_text.clone(),
                                success: false,
                                error: Some(e.to_string()),
                                parse_time: duration,
                                memory_estimate: input_text.len(),
                            });
                        }
                    }
                }

                // Calculate statistics
                let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
                let min_time = *times.iter().min().unwrap();
                let max_time = *times.iter().max().unwrap();

                println!(
                    "  ⏱️  {}: avg={:.2}μs, min={:.2}μs, max={:.2}μs",
                    test_name,
                    avg_time.as_nanos() as f64 / 1000.0,
                    min_time.as_nanos() as f64 / 1000.0,
                    max_time.as_nanos() as f64 / 1000.0
                );
            }
        }
    }

    let total_time = overall_start.elapsed();

    // Calculate benchmark statistics
    let passed_tests = results.iter().filter(|r| r.success).count();
    let failed_tests = results.len() - passed_tests;

    let times: Vec<Duration> = results.iter().map(|r| r.parse_time).collect();
    let average_time = if !times.is_empty() {
        Duration::from_nanos(
            times.iter().map(|d| d.as_nanos() as u64).sum::<u64>() / times.len() as u64,
        )
    } else {
        Duration::ZERO
    };

    let slowest_test = results
        .iter()
        .max_by_key(|r| r.parse_time)
        .map(|r| (r.name.clone(), r.parse_time));

    let fastest_test = results
        .iter()
        .min_by_key(|r| r.parse_time)
        .map(|r| (r.name.clone(), r.parse_time));

    let memory_total: usize = results.iter().map(|r| r.memory_estimate).sum();

    let stats = BenchmarkStats {
        total_tests: results.len(),
        passed_tests,
        failed_tests,
        total_time,
        average_time,
        slowest_test,
        fastest_test,
        memory_total,
    };

    // Generate benchmark report
    generate_benchmark_report(&results, &stats);

    println!("✅ Benchmark complete! Report written to src/results/benchmark_results.md");
}

fn generate_benchmark_report(results: &[TestResult], stats: &BenchmarkStats) {
    let mut output = match fs::File::create("src/results/benchmark_results.md") {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "❌ Failed to create src/results/benchmark_results.md: {}",
                e
            );
            return;
        }
    };

    writeln!(output, "# Marco Grammar Performance Benchmark Report\n").unwrap();
    writeln!(output, "Generated automatically from benchmark tests\n").unwrap();

    writeln!(output, "## Summary\n").unwrap();
    writeln!(output, "- **Total Tests**: {}", stats.total_tests).unwrap();
    writeln!(output, "- **Passed**: {} ✅", stats.passed_tests).unwrap();
    writeln!(output, "- **Failed**: {} ❌", stats.failed_tests).unwrap();
    writeln!(
        output,
        "- **Total Time**: {:.2}ms",
        stats.total_time.as_nanos() as f64 / 1_000_000.0
    )
    .unwrap();
    writeln!(
        output,
        "- **Average Parse Time**: {:.2}μs",
        stats.average_time.as_nanos() as f64 / 1000.0
    )
    .unwrap();
    writeln!(
        output,
        "- **Memory Estimate**: {} bytes",
        stats.memory_total
    )
    .unwrap();

    // Performance extremes
    if let Some((name, time)) = &stats.slowest_test {
        writeln!(
            output,
            "- **Slowest Test**: {} ({:.2}μs)",
            name,
            time.as_nanos() as f64 / 1000.0
        )
        .unwrap();
    }

    if let Some((name, time)) = &stats.fastest_test {
        writeln!(
            output,
            "- **Fastest Test**: {} ({:.2}μs)",
            name,
            time.as_nanos() as f64 / 1000.0
        )
        .unwrap();
    }

    writeln!(output, "\n## Performance Analysis\n").unwrap();

    // Group by input size for throughput analysis
    let mut size_groups: HashMap<String, Vec<&TestResult>> = HashMap::new();
    for result in results {
        let size_category = match result.input.len() {
            0..=50 => "Small (0-50 chars)",
            51..=200 => "Medium (51-200 chars)",
            201..=1000 => "Large (201-1000 chars)",
            _ => "Extra Large (1000+ chars)",
        };
        size_groups
            .entry(size_category.to_string())
            .or_insert_with(Vec::new)
            .push(result);
    }

    for (category, group) in size_groups {
        let avg_time = group.iter().map(|r| r.parse_time).sum::<Duration>() / group.len() as u32;
        let avg_size = group.iter().map(|r| r.input.len()).sum::<usize>() / group.len();
        let throughput = if avg_time.as_nanos() > 0 {
            (avg_size as f64) / (avg_time.as_secs_f64() * 1_000_000.0) // MB/s
        } else {
            0.0
        };

        writeln!(output, "### {}", category).unwrap();
        writeln!(output, "- Tests: {}", group.len()).unwrap();
        writeln!(
            output,
            "- Average Time: {:.2}μs",
            avg_time.as_nanos() as f64 / 1000.0
        )
        .unwrap();
        writeln!(output, "- Average Size: {} chars", avg_size).unwrap();
        writeln!(output, "- Throughput: {:.2} MB/s\n", throughput).unwrap();
    }

    writeln!(output, "## Detailed Results\n").unwrap();
    writeln!(
        output,
        "| Test Name | Rule | Time (μs) | Input Size | Status |"
    )
    .unwrap();
    writeln!(
        output,
        "|-----------|------|-----------|------------|--------|"
    )
    .unwrap();

    for result in results {
        let status = if result.success { "✅" } else { "❌" };
        writeln!(
            output,
            "| {} | {} | {:.2} | {} | {} |",
            result.name,
            result.rule,
            result.parse_time.as_nanos() as f64 / 1000.0,
            result.input.len(),
            status
        )
        .unwrap();
    }

    writeln!(
        output,
        "\n---\n*Report generated by Marco Grammar Test Suite*"
    )
    .unwrap();
}

fn run_single_test(rule_name: &str, input: &str) {
    // Map rule name to Rule enum
    let rule = match get_rule(rule_name) {
        Some(r) => r,
        None => {
            eprintln!("{}❌ Unknown rule: {}{}", "\x1b[91m", rule_name, RESET);
            eprintln!("Available rules: file, document, paragraph, text, word, emoji, H1-H6, bold, italic, etc.");
            std::process::exit(1);
        }
    };

    match MarcoParser::parse(rule, input) {
        Ok(pairs) => {
            println!("{}🌳 Enhanced Parse Tree:{}", BRIGHT_CYAN, RESET);
            for pair in pairs {
                print_enhanced_pairs(pair, 0);
            }
        }
        Err(e) => {
            eprintln!("{}❌ Parse error: {}{}", "\x1b[91m", e, RESET);
        }
    }
}

fn run_tree_test(rule_name: &str, input: &str) {
    // Map rule name to Rule enum
    let rule = match get_rule(rule_name) {
        Some(r) => r,
        None => {
            eprintln!("{}❌ Unknown rule: {}{}", "\x1b[91m", rule_name, RESET);
            eprintln!("Available rules: file, document, paragraph, text, word, emoji, H1-H6, bold, italic, etc.");
            std::process::exit(1);
        }
    };

    println!(
        "{}🔍 Testing rule: {}{}{} with input: \"{}{}{}",
        BRIGHT_CYAN,
        BRIGHT_BLUE,
        rule_name,
        RESET,
        BRIGHT_GREEN,
        escape_content_for_display(input),
        RESET
    );
    println!();
    print_ascii_tree_result(MarcoParser::parse(rule, input));
}

fn get_rule(rule_name: &str) -> Option<Rule> {
    match rule_name {
        // Main structure
        "file" => Some(Rule::file),
        "document" => Some(Rule::document),
        "section" => Some(Rule::section),
        "block" => Some(Rule::block),
        "paragraph" => Some(Rule::paragraph),
        "paragraph_line" => Some(Rule::paragraph_line),

        // Text and characters
        "text" => Some(Rule::text),
        "word" => Some(Rule::word),
        "safe_punct" => Some(Rule::safe_punct),
        "math_symbol" => Some(Rule::math_symbol),
        "unicode_letter" => Some(Rule::unicode_letter),
        "inner_char" => Some(Rule::inner_char),

        // Headings
        "heading" => Some(Rule::heading),
        "heading_content" => Some(Rule::heading_content),
        "H1" => Some(Rule::H1),
        "H2" => Some(Rule::H2),
        "H3" => Some(Rule::H3),
        "H4" => Some(Rule::H4),
        "H5" => Some(Rule::H5),
        "H6" => Some(Rule::H6),
        "setext_h1" => Some(Rule::setext_h1),
        "setext_h2" => Some(Rule::setext_h2),

        // Formatting
        "emphasis" => Some(Rule::emphasis),
        "bold" => Some(Rule::bold),
        "bold_asterisk" => Some(Rule::bold_asterisk),
        "bold_underscore" => Some(Rule::bold_underscore),
        "italic" => Some(Rule::italic),
        "italic_asterisk" => Some(Rule::italic_asterisk),
        "italic_underscore" => Some(Rule::italic_underscore),
        "bold_italic" => Some(Rule::bold_italic),
        "bold_italic_triple_asterisk" => Some(Rule::bold_italic_triple_asterisk),
        "bold_italic_triple_underscore" => Some(Rule::bold_italic_triple_underscore),
        "bold_italic_mixed_ast_under" => Some(Rule::bold_italic_mixed_ast_under),
        "bold_italic_mixed_under_ast" => Some(Rule::bold_italic_mixed_under_ast),
        "strikethrough" => Some(Rule::strikethrough),
        "strikethrough_tilde" => Some(Rule::strikethrough_tilde),
        "strikethrough_dash" => Some(Rule::strikethrough_dash),
        "highlight" => Some(Rule::highlight),
        "superscript" => Some(Rule::superscript),
        "subscript" => Some(Rule::subscript),
        "emoji" => Some(Rule::emoji),

        // Code and math
        "code_inline" => Some(Rule::code_inline),
        "code_block" => Some(Rule::code_block),
        "fenced_code" => Some(Rule::fenced_code),
        "indented_code" => Some(Rule::indented_code),
        "language_id" => Some(Rule::language_id),
        "math_inline" => Some(Rule::math_inline),
        "math_block" => Some(Rule::math_block),

        // Links and images
        "inline_link" => Some(Rule::inline_link),
        "bracket_link_with_title" => Some(Rule::bracket_link_with_title),
        "bracket_link_without_title" => Some(Rule::bracket_link_without_title),
        "autolink" => Some(Rule::autolink),
        "autolink_email" => Some(Rule::autolink_email),
        "autolink_url" => Some(Rule::autolink_url),
        "inline_image" => Some(Rule::inline_image),
        "inline_link_text" => Some(Rule::inline_link_text),
        "inline_url" => Some(Rule::link_url), // Legacy alias
        "link_url" => Some(Rule::link_url),
        "link_title" => Some(Rule::link_title),
        "reference_link" => Some(Rule::reference_link),
        "reference_image" => Some(Rule::reference_image),
        "reference_definition" => Some(Rule::reference_definition),
        "block_image" => Some(Rule::block_image),
        "block_youtube" => Some(Rule::block_youtube),
        "block_caption" => Some(Rule::block_caption),
        "ref_title" => Some(Rule::ref_title),

        // URLs
        "http_url" => Some(Rule::http_url),
        "https_url" => Some(Rule::http_url), // Alias
        "www_url" => Some(Rule::www_url),
        "mailto" => Some(Rule::mailto),
        "local_path" => Some(Rule::local_path),
        "youtube_url" => Some(Rule::youtube_url),
        "image_url" => Some(Rule::image_url),
        "image_ext" => Some(Rule::image_ext),

        // Lists
        "list" => Some(Rule::list),
        "list_item" => Some(Rule::list_item),
        "regular_list_item" => Some(Rule::regular_list_item),
        "task_list_item" => Some(Rule::task_list_item),
        "list_item_content" => Some(Rule::list_item_content),
        "list_marker" => Some(Rule::list_marker),
        "unordered_marker" => Some(Rule::unordered_marker),
        "ordered_marker" => Some(Rule::ordered_marker),
        "task_marker" => Some(Rule::task_marker),
        "task_metadata" => Some(Rule::task_metadata),
        "inline_task_item" => Some(Rule::inline_task_item),

        // Definition lists
        "def_list" => Some(Rule::def_list),
        "term_line" => Some(Rule::term_line),
        "def_line" => Some(Rule::def_line),

        // Tables
        "table" => Some(Rule::table),
        "table_header" => Some(Rule::table_header),
        "table_sep" => Some(Rule::table_sep),
        "table_row" => Some(Rule::table_row),
        "table_cell" => Some(Rule::table_cell),
        "table_sep_cell" => Some(Rule::table_sep_cell),

        // Blockquotes
        "blockquote" => Some(Rule::blockquote),
        "blockquote_line" => Some(Rule::blockquote_line),

        // Horizontal rules
        "hr" => Some(Rule::hr),

        // Footnotes
        "footnote_ref" => Some(Rule::footnote_ref),
        "footnote_def" => Some(Rule::footnote_def),
        "footnote_label" => Some(Rule::footnote_label),
        "inline_footnote_ref" => Some(Rule::inline_footnote_ref),

        // HTML and comments
        "inline_html" => Some(Rule::inline_html),
        "block_html" => Some(Rule::block_html),
        "inline_comment" => Some(Rule::inline_comment),
        "block_comment" => Some(Rule::block_comment),

        // Inline elements
        "inline" => Some(Rule::inline),
        "inline_core" => Some(Rule::inline_core),
        "escaped_char" => Some(Rule::escaped_char),
        "line_break" => Some(Rule::line_break),

        // Marco extensions
        "macro_inline" => Some(Rule::macro_inline),
        "macro_block" => Some(Rule::macro_block),
        "user_mention" => Some(Rule::user_mention),
        "username" => Some(Rule::username),
        "platform" => Some(Rule::platform),
        "display_name" => Some(Rule::display_name),

        // Admonitions
        "admonition_block" => Some(Rule::admonition_block),
        "admonition_type" => Some(Rule::admonition_type),
        "admonition_open" => Some(Rule::admonition_open),
        "admonition_emoji" => Some(Rule::admonition_emoji),
        "admonition_close" => Some(Rule::admonition_close),

        // Page and document
        "page_tag" => Some(Rule::page_tag),
        "page_format" => Some(Rule::page_format),
        "doc_ref" => Some(Rule::doc_ref),
        "bookmark" => Some(Rule::bookmark),

        // Table of contents
        "toc" => Some(Rule::toc),
        "toc_depth" => Some(Rule::toc_depth),
        "toc_doc" => Some(Rule::toc_doc),

        // Run commands
        "run_inline" => Some(Rule::run_inline),
        "run_block_fenced" => Some(Rule::run_block_fenced),
        "script_type" => Some(Rule::script_type),

        // Diagrams
        "diagram_fenced" => Some(Rule::diagram_fenced),
        "diagram_type" => Some(Rule::diagram_type),

        // Tabs
        "tab_block" => Some(Rule::tab_block),
        "tab_header" => Some(Rule::tab_header),
        "tab_title" => Some(Rule::tab_title),
        "tabs_content_I" => Some(Rule::tabs_content_I),
        "tab_content_line" => Some(Rule::tab_content_line),
        "tab" => Some(Rule::tab),
        "tab_line" => Some(Rule::tab_line),
        "tab_name" => Some(Rule::tab_name),
        "tab_content_II" => Some(Rule::tab_content_II),
        "tab_end" => Some(Rule::tab_end),

        // Keywords (case-insensitive)
        "KW_NOTE" => Some(Rule::KW_NOTE),
        "KW_TIP" => Some(Rule::KW_TIP),
        "KW_WARNING" => Some(Rule::KW_WARNING),
        "KW_DANGER" => Some(Rule::KW_DANGER),
        "KW_INFO" => Some(Rule::KW_INFO),
        "KW_BOOKMARK" => Some(Rule::KW_BOOKMARK),
        "KW_PAGE" => Some(Rule::KW_PAGE),
        "KW_DOC" => Some(Rule::KW_DOC),
        "KW_TOC" => Some(Rule::KW_TOC),
        "KW_TAB" => Some(Rule::KW_TAB),
        "KW_BASH" => Some(Rule::KW_BASH),
        "KW_ZSH" => Some(Rule::KW_ZSH),
        "KW_SH" => Some(Rule::KW_SH),
        "KW_BAT" => Some(Rule::KW_BAT),
        "KW_POWERSHELL" => Some(Rule::KW_POWERSHELL),
        "KW_PS" => Some(Rule::KW_PS),
        "KW_PYTHON" => Some(Rule::KW_PYTHON),
        "KW_PY" => Some(Rule::KW_PY),
        "KW_RUN" => Some(Rule::KW_RUN),
        "KW_MERMAID" => Some(Rule::KW_MERMAID),
        "KW_GRAPHVIZ" => Some(Rule::KW_GRAPHVIZ),

        // Error recovery
        "unknown_block" => Some(Rule::unknown_block),

        _ => None,
    }
}

fn determine_rule(section: &str, test_name: &str) -> String {
    // Try to determine rule from section first
    match section {
        "text_and_words" => {
            if test_name.contains("word") {
                "word".to_string()
            } else if test_name.contains("math_symbol") {
                "math_symbol".to_string()
            } else {
                "text".to_string()
            }
        }
        "headings_atx" => {
            if test_name.starts_with("h1") {
                "H1".to_string()
            } else if test_name.starts_with("h2") {
                "H2".to_string()
            } else if test_name.starts_with("h3") {
                "H3".to_string()
            } else if test_name.starts_with("h4") {
                "H4".to_string()
            } else if test_name.starts_with("h5") {
                "H5".to_string()
            } else if test_name.starts_with("h6") {
                "H6".to_string()
            } else {
                "heading".to_string()
            }
        }
        "headings_setext" => {
            if test_name.contains("h1") {
                "setext_h1".to_string()
            } else if test_name.contains("h2") {
                "setext_h2".to_string()
            } else {
                "heading".to_string()
            }
        }
        "bold_formatting" => "bold".to_string(),
        "bold_italic_combinations" => "bold_italic".to_string(),
        "other_formatting" => {
            // Handle mixed content tests that should use broader parsing
            if test_name.contains("_mixed") {
                "inline".to_string()
            } else if test_name.starts_with("strike") {
                "strikethrough".to_string()
            } else if test_name.starts_with("highlight") {
                "highlight".to_string()
            } else if test_name.starts_with("superscript") {
                "superscript".to_string()
            } else if test_name.starts_with("subscript") {
                "subscript".to_string()
            } else {
                "emphasis".to_string()
            }
        }
        "code_inline" => {
            // Handle mixed content tests that should use broader parsing
            if test_name.contains("_mixed") {
                "inline".to_string()
            } else {
                "code_inline".to_string()
            }
        }
        "italic_formatting" => {
            // Handle mixed content tests that should use broader parsing
            if test_name.contains("_mixed") {
                "inline".to_string()
            } else {
                "italic".to_string()
            }
        }
        "math_inline" => "math_inline".to_string(),
        "code_blocks" => {
            if test_name.contains("fenced") {
                "fenced_code".to_string()
            } else if test_name.contains("indented") {
                "indented_code".to_string()
            } else {
                "code_block".to_string()
            }
        }
        "math_blocks" => "math_block".to_string(),
        "urls" => {
            if test_name.starts_with("http") {
                "http_url".to_string()
            } else if test_name.starts_with("https") {
                "http_url".to_string()
            } else if test_name.starts_with("www") {
                "www_url".to_string()
            } else if test_name.starts_with("mailto") {
                "mailto".to_string()
            } else if test_name.starts_with("local") {
                "local_path".to_string()
            } else if test_name.starts_with("youtube") {
                "youtube_url".to_string()
            } else if test_name.starts_with("image") {
                "image_url".to_string()
            } else {
                "inline_url".to_string()
            }
        }
        "inline_links" => "inline_link".to_string(),
        "link_title" => "link_title".to_string(),
        "inline_images" => "inline_image".to_string(),
        "reference_links" => {
            if test_name.starts_with("ref_def") {
                "reference_definition".to_string()
            } else if test_name.starts_with("ref_image") {
                "reference_image".to_string()
            } else {
                "reference_link".to_string()
            }
        }
        "unordered_lists" | "ordered_lists" => "list".to_string(),
        "task_lists" => {
            if test_name.starts_with("inline_task") {
                "inline_task_item".to_string()
            } else {
                "task_list_item".to_string()
            }
        }
        "definition_lists" => "def_list".to_string(),
        "tables" => "table".to_string(),
        "blockquotes" => "blockquote".to_string(),
        "horizontal_rules" => "hr".to_string(),
        "footnotes" => {
            if test_name.starts_with("footnote_def") {
                "footnote_def".to_string()
            } else if test_name.starts_with("inline_footnote") {
                "inline_footnote_ref".to_string()
            } else {
                "footnote_ref".to_string()
            }
        }
        "html_elements" => {
            if test_name.starts_with("comment") {
                "inline_comment".to_string()
            } else {
                "inline_html".to_string()
            }
        }
        "user_mentions" => "user_mention".to_string(),
        "admonitions" => "admonition_block".to_string(),
        "tab" => "tab_block".to_string(),
        "tabs" => "tab_block".to_string(),
        "page_and_doc" => {
            if test_name.starts_with("page") {
                "page_tag".to_string()
            } else if test_name.starts_with("doc") {
                "doc_ref".to_string()
            } else if test_name.starts_with("toc") {
                "toc".to_string()
            } else {
                "macro_inline".to_string()
            }
        }
        "bookmarks" => "bookmark".to_string(),
        "run_commands" => {
            if test_name.contains("block") {
                "run_block_fenced".to_string()
            } else {
                "run_inline".to_string()
            }
        }
        "diagrams" => "diagram_fenced".to_string(),
        "escaped_characters" => "escaped_char".to_string(),
        "edge_cases" => "text".to_string(),

        // New comprehensive test categories
        "pathological_inputs" => {
            if test_name.contains("nested_quotes") {
                "blockquote".to_string()
            } else if test_name.contains("nested_lists") {
                "list".to_string()
            } else if test_name.contains("nested_emphasis") {
                "emphasis".to_string()
            } else {
                "text".to_string()
            }
        }
        "commonmark_edge_cases" => {
            if test_name.contains("link") {
                "inline_link".to_string()
            } else if test_name.contains("emphasis") {
                "emphasis".to_string()
            } else if test_name.contains("autolink") {
                "inline_html".to_string()
            } else if test_name.contains("hr") {
                "hr".to_string()
            } else if test_name.contains("list") {
                "list".to_string()
            } else if test_name.contains("heading") {
                "heading".to_string()
            } else if test_name.contains("setext") {
                if test_name.contains("h1") {
                    "setext_h1".to_string()
                } else {
                    "setext_h2".to_string()
                }
            } else {
                "text".to_string()
            }
        }
        "security_vectors" => {
            if test_name.contains("script") || test_name.contains("html") {
                "inline_html".to_string()
            } else if test_name.contains("url") || test_name.contains("link") {
                "inline_link".to_string()
            } else {
                "text".to_string()
            }
        }
        "unicode_advanced" => "text".to_string(),
        "marco_stress_tests" => {
            if test_name.contains("admonition") {
                "admonition_block".to_string()
            } else if test_name.contains("run") {
                "run_block_fenced".to_string()
            } else if test_name.contains("user_mention") {
                "user_mention".to_string()
            } else if test_name.contains("tabs") {
                "tab_block".to_string()
            } else if test_name.contains("bookmark") {
                "bookmark".to_string()
            } else if test_name.contains("toc") {
                "toc".to_string()
            } else {
                "text".to_string()
            }
        }
        "performance_tests" => {
            if test_name.contains("table") {
                "table".to_string()
            } else if test_name.contains("footnote") {
                "footnote_ref".to_string()
            } else if test_name.contains("ref") {
                "reference_link".to_string()
            } else {
                "text".to_string()
            }
        }
        "boundary_conditions" => {
            if test_name.contains("list") {
                "list".to_string()
            } else if test_name.contains("heading") {
                "ordered_marker".to_string()
            } else if test_name.contains("url") {
                "inline_url".to_string()
            } else {
                "text".to_string()
            }
        }
        "real_world_cases" => "text".to_string(),
        "regression_tests" => {
            if test_name.contains("emphasis") {
                "emphasis".to_string()
            } else if test_name.contains("link") {
                "inline_link".to_string()
            } else if test_name.contains("quote") {
                "blockquote".to_string()
            } else if test_name.contains("table") {
                "table".to_string()
            } else if test_name.contains("list") {
                "list".to_string()
            } else if test_name.contains("setext") {
                if test_name.contains("h1") {
                    "setext_h1".to_string()
                } else {
                    "setext_h2".to_string()
                }
            } else if test_name.contains("html") {
                "inline_html".to_string()
            } else {
                "text".to_string()
            }
        }

        // New advanced test categories
        "commonmark_conformance" => {
            if test_name.contains("atx") {
                if test_name.contains("h1") {
                    "H1".to_string()
                } else if test_name.contains("h2") {
                    "H2".to_string()
                } else if test_name.contains("h3") {
                    "H3".to_string()
                } else if test_name.contains("h4") {
                    "H4".to_string()
                } else if test_name.contains("h5") {
                    "H5".to_string()
                } else if test_name.contains("h6") {
                    "H6".to_string()
                } else {
                    "heading".to_string()
                }
            } else if test_name.contains("setext") {
                if test_name.contains("h1") {
                    "setext_h1".to_string()
                } else {
                    "setext_h2".to_string()
                }
            } else if test_name.contains("emphasis") || test_name.contains("strong") {
                "emphasis".to_string()
            } else if test_name.contains("link") {
                "inline_link".to_string()
            } else if test_name.contains("autolink") {
                "inline_html".to_string()
            } else if test_name.contains("code") {
                "code_inline".to_string()
            } else {
                "text".to_string()
            }
        }
        "fuzzing_tests" => "text".to_string(),
        "memory_stress" => {
            if test_name.contains("list") {
                "list".to_string()
            } else if test_name.contains("table") {
                "table".to_string()
            } else if test_name.contains("footnote") {
                "footnote_ref".to_string()
            } else {
                "text".to_string()
            }
        }
        "benchmark_tests" => {
            if test_name.contains("formatting") {
                "emphasis".to_string()
            } else if test_name.contains("quote") {
                "blockquote".to_string()
            } else if test_name.contains("github") || test_name.contains("academic") {
                "document".to_string()
            } else {
                "text".to_string()
            }
        }
        "specification_compliance" => {
            if test_name.contains("table") {
                "table".to_string()
            } else if test_name.contains("strikethrough") {
                "strikethrough".to_string()
            } else if test_name.contains("autolink") {
                "http_url".to_string()
            } else if test_name.contains("task") {
                "task_list_item".to_string()
            } else if test_name.contains("subscript") {
                "subscript".to_string()
            } else if test_name.contains("superscript") {
                "superscript".to_string()
            } else if test_name.contains("definition") {
                "def_list".to_string()
            } else {
                "text".to_string()
            }
        }
        "error_recovery" => {
            if test_name.contains("bold") {
                "bold".to_string()
            } else if test_name.contains("link") {
                "inline_link".to_string()
            } else if test_name.contains("table") {
                "table".to_string()
            } else if test_name.contains("code") {
                "fenced_code".to_string()
            } else if test_name.contains("admonition") {
                "admonition_block".to_string()
            } else {
                "text".to_string()
            }
        }
        "integration_tests" => {
            if test_name.contains("blog")
                || test_name.contains("technical")
                || test_name.contains("readme")
            {
                "document".to_string()
            } else {
                "text".to_string()
            }
        }

        "failure_cases" => {
            // Try to determine the most appropriate rule for failure cases
            if test_name.contains("link") {
                "inline_link".to_string()
            } else if test_name.contains("image") {
                "inline_image".to_string()
            } else if test_name.contains("bold") {
                "bold".to_string()
            } else if test_name.contains("italic") {
                "italic".to_string()
            } else if test_name.contains("code") {
                "code_inline".to_string()
            } else if test_name.contains("math") {
                "math_inline".to_string()
            } else if test_name.contains("emoji") {
                "emoji".to_string()
            } else if test_name.contains("html") {
                "inline_html".to_string()
            } else if test_name.contains("heading") {
                "heading".to_string()
            } else if test_name.contains("table") {
                "table".to_string()
            } else if test_name.contains("footnote") {
                "footnote_ref".to_string()
            } else if test_name.contains("user") {
                "user_mention".to_string()
            } else if test_name.contains("bookmark") {
                "bookmark".to_string()
            } else if test_name.contains("admonition") {
                "admonition_block".to_string()
            } else if test_name.contains("script") {
                "run_inline".to_string()
            } else {
                "text".to_string()
            }
        }

        // Legacy support for old test categories
        "emoji" => "emoji".to_string(),
        "headings" => {
            if test_name.starts_with("h1") {
                "H1".to_string()
            } else if test_name.starts_with("h2") {
                "H2".to_string()
            } else if test_name.starts_with("h3") {
                "H3".to_string()
            } else if test_name.starts_with("h4") {
                "H4".to_string()
            } else if test_name.starts_with("h5") {
                "H5".to_string()
            } else if test_name.starts_with("h6") {
                "H6".to_string()
            } else {
                "heading".to_string()
            }
        }
        "formatting" => {
            if test_name.starts_with("bold") {
                "bold".to_string()
            } else if test_name.starts_with("italic") {
                "italic".to_string()
            } else if test_name.starts_with("code") {
                "code_inline".to_string()
            } else if test_name.starts_with("strike") {
                "strikethrough".to_string()
            } else if test_name.starts_with("highlight") {
                "highlight".to_string()
            } else if test_name.starts_with("superscript") {
                "superscript".to_string()
            } else if test_name.starts_with("subscript") {
                "subscript".to_string()
            } else {
                "emphasis".to_string()
            }
        }
        "links" => {
            if test_name.starts_with("image") {
                "inline_image".to_string()
            } else {
                "inline_link".to_string()
            }
        }
        "lists" => "list".to_string(),
        "text" => {
            if test_name.contains("word") {
                "word".to_string()
            } else {
                "text".to_string()
            }
        }
        "math" => "math_inline".to_string(),
        "comments_and_html" => "inline_html".to_string(),
        "marco_extensions" => {
            if test_name.starts_with("user") {
                "user_mention".to_string()
            } else if test_name.starts_with("page") {
                "page_tag".to_string()
            } else if test_name.starts_with("toc") {
                "toc".to_string()
            } else if test_name.starts_with("run") {
                "run_inline".to_string()
            } else {
                "macro_inline".to_string()
            }
        }

        _ => "text".to_string(),
    }
}

fn parse_test_cases(content: &str) -> HashMap<String, Vec<(String, String)>> {
    let mut sections = HashMap::new();
    let mut current_section = String::new();
    let mut current_cases = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous section
            if !current_section.is_empty() {
                sections.insert(current_section.clone(), current_cases.clone());
            }

            current_section = line[1..line.len() - 1].to_string();
            current_cases.clear();
        }
        // Test case
        else if let Some(eq_pos) = line.find(" = ") {
            let test_name = line[..eq_pos].trim().to_string();
            let mut test_value = line[eq_pos + 3..].trim();

            // Handle comments - find # that's not inside quotes
            let mut in_quotes = false;
            let mut comment_pos = None;
            for (i, ch) in test_value.char_indices() {
                if ch == '"' && (i == 0 || test_value.chars().nth(i - 1) != Some('\\')) {
                    in_quotes = !in_quotes;
                } else if ch == '#' && !in_quotes {
                    comment_pos = Some(i);
                    break;
                }
            }

            // Remove comment if found
            if let Some(pos) = comment_pos {
                test_value = test_value[..pos].trim();
            }

            // Remove quotes if present
            let test_value = if test_value.starts_with('"') && test_value.ends_with('"') {
                &test_value[1..test_value.len() - 1]
            } else {
                test_value
            };

            // Unescape basic sequences
            let test_value = test_value
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\\"", "\"")
                .replace("\\'", "'");

            current_cases.push((test_name, test_value));
        }
    }

    // Save last section
    if !current_section.is_empty() {
        sections.insert(current_section, current_cases);
    }

    sections
}

fn escape_markdown(text: &str) -> String {
    text.replace('`', "\\`")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

fn write_pairs_to_string(output: &mut fs::File, pair: pest::iterators::Pair<Rule>, depth: usize) {
    let indent = "  ".repeat(depth);
    let rule_name = format!("{:?}", pair.as_rule());
    let text = pair.as_str();

    // Clone the pair to check if it has children
    let has_children = pair.clone().into_inner().peekable().peek().is_some();

    if has_children {
        // Has children - show structure
        writeln!(output, "{}├── {} > \"{}\"", indent, rule_name, text).unwrap();
        for inner_pair in pair.into_inner() {
            write_pairs_to_string(output, inner_pair, depth + 1);
        }
    } else {
        // Leaf node - show value
        writeln!(output, "{}└── {}: \"{}\"", indent, rule_name, text).unwrap();
    }
}

fn print_enhanced_pairs(pair: pest::iterators::Pair<Rule>, depth: usize) {
    let rule_name = format!("{:?}", pair.as_rule());
    let text = pair.as_str();
    let escaped_text = escape_content_for_display(text);

    // Clone the pair to check if it has children
    let has_children = pair.clone().into_inner().peekable().peek().is_some();

    // Create indentation with proper tree characters
    let indent = if depth == 0 {
        String::new()
    } else {
        let mut indent_str = String::new();
        for i in 0..depth {
            if i == depth - 1 {
                indent_str.push_str("├─ ");
            } else {
                indent_str.push_str("│  ");
            }
        }
        indent_str
    };

    // Format the output line
    let colored_rule = color_rule_name(&rule_name);
    let content_part = format!(
        "{}⟨{}{}{}{}",
        BRIGHT_CYAN, DIM_WHITE, BRIGHT_GREEN, escaped_text, RESET
    );
    let closing_part = format!("{}⟩{}", DIM_WHITE, RESET);

    if depth == 0 {
        // Root node gets a special checkmark
        println!(
            "{}✅ {} → {}{}",
            BRIGHT_GREEN, colored_rule, content_part, closing_part
        );
    } else {
        println!(
            "{}{} → {}{}",
            indent, colored_rule, content_part, closing_part
        );
    }

    if has_children {
        let inner_pairs: Vec<_> = pair.into_inner().collect();
        for (i, inner_pair) in inner_pairs.iter().enumerate() {
            // For the last child, we need different tree characters
            if i == inner_pairs.len() - 1 {
                print_enhanced_pairs_last(inner_pair.clone(), depth + 1, depth + 1);
            } else {
                print_enhanced_pairs(inner_pair.clone(), depth + 1);
            }
        }
    }
}

fn print_enhanced_pairs_last(
    pair: pest::iterators::Pair<Rule>,
    depth: usize,
    current_depth: usize,
) {
    let rule_name = format!("{:?}", pair.as_rule());
    let text = pair.as_str();
    let escaped_text = escape_content_for_display(text);

    // Clone the pair to check if it has children
    let has_children = pair.clone().into_inner().peekable().peek().is_some();

    // Create indentation with proper tree characters for the last item
    let mut indent_str = String::new();
    for i in 0..depth {
        if i == depth - 1 {
            indent_str.push_str("└─ ");
        } else if i < current_depth - 1 {
            indent_str.push_str("│  ");
        } else {
            indent_str.push_str("   ");
        }
    }

    // Format the output line
    let colored_rule = color_rule_name(&rule_name);
    let content_part = format!(
        "{}⟨{}{}{}{}",
        BRIGHT_CYAN, DIM_WHITE, BRIGHT_GREEN, escaped_text, RESET
    );
    let closing_part = format!("{}⟩{}", DIM_WHITE, RESET);

    println!(
        "{}{} → {}{}",
        indent_str, colored_rule, content_part, closing_part
    );

    if has_children {
        let inner_pairs: Vec<_> = pair.into_inner().collect();
        for (i, inner_pair) in inner_pairs.iter().enumerate() {
            if i == inner_pairs.len() - 1 {
                print_enhanced_pairs_last(inner_pair.clone(), depth + 1, current_depth);
            } else {
                print_enhanced_pairs(inner_pair.clone(), depth + 1);
            }
        }
    }
}
