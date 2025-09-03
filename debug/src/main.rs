use pest::Parser;
use pest_derive::Parser;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[grammar = "../src/components/marco_engine/grammar/marco.pest"]
pub struct MarcoParser;

#[derive(Debug)]
#[allow(dead_code)] // All fields are used in constructors, but compiler doesn't detect it
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        // Batch mode - run all test cases
        run_batch_tests();
    } else if args.len() == 2 && args[1] == "--benchmark" {
        // Benchmark mode
        run_benchmark_tests();
    } else if args.len() == 3 {
        // Single test mode
        let rule_name = &args[1];
        let input = &args[2];
        run_single_test(rule_name, input);
    } else {
        eprintln!("Usage:");
        eprintln!("  {} <rule> <input>           - Test single rule", args[0]);
        eprintln!(
            "  {}                          - Run all test cases from test_cases.toml",
            args[0]
        );
        eprintln!(
            "  {} --benchmark              - Run performance benchmarks",
            args[0]
        );
        eprintln!("Example: {} emoji ':smile:'", args[0]);
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

    // Create output file
    let mut output = match fs::File::create("src/results/test_results.md") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("❌ Failed to create src/results/test_results.md: {}", e);
            std::process::exit(1);
        }
    };

    writeln!(output, "# Marco Grammar Test Results\n").unwrap();
    writeln!(output, "Generated automatically from test_cases.toml\n").unwrap();

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;

    for (section, cases) in test_cases {
        writeln!(output, "## {}\n", section).unwrap();

        for (test_name, input_text) in cases {
            total_tests += 1;

            // Determine rule from section or test name
            let rule_name = determine_rule(&section, &test_name);
            let rule = match get_rule(&rule_name) {
                Some(r) => r,
                None => {
                    writeln!(
                        output,
                        "❌ **{}**: Unknown rule `{}`\n",
                        test_name, rule_name
                    )
                    .unwrap();
                    failed_tests += 1;
                    continue;
                }
            };

            // Test the rule
            match MarcoParser::parse(rule, &input_text) {
                Ok(pairs) => {
                    writeln!(output, "✅ **{}**: `{}`", test_name, rule_name).unwrap();
                    writeln!(output, "   Input: `{}`", escape_markdown(&input_text)).unwrap();
                    writeln!(output, "   Parse Tree:").unwrap();
                    writeln!(output, "   ```").unwrap();
                    for pair in pairs {
                        write_pairs_to_string(&mut output, pair, 1);
                    }
                    writeln!(output, "   ```\n").unwrap();
                    passed_tests += 1;
                }
                Err(e) => {
                    // Check if this was expected to fail
                    if test_name.contains("failure")
                        || test_name.contains("invalid")
                        || test_name.contains("malformed")
                        || test_name.contains("empty")
                        || test_name.contains("no_")
                        || test_name.contains("missing")
                    {
                        writeln!(
                            output,
                            "✅ **{}**: `{}` (Expected failure)",
                            test_name, rule_name
                        )
                        .unwrap();
                        writeln!(output, "   Input: `{}`", escape_markdown(&input_text)).unwrap();
                        writeln!(output, "   Error: `{}`\n", e).unwrap();
                        passed_tests += 1;
                    } else {
                        writeln!(
                            output,
                            "❌ **{}**: `{}` (Unexpected failure)",
                            test_name, rule_name
                        )
                        .unwrap();
                        writeln!(output, "   Input: `{}`", escape_markdown(&input_text)).unwrap();
                        writeln!(output, "   Error: `{}`\n", e).unwrap();
                        failed_tests += 1;
                    }
                }
            }
        }
    }

    // Write summary
    writeln!(output, "## Summary\n").unwrap();
    writeln!(output, "- **Total tests**: {}", total_tests).unwrap();
    writeln!(output, "- **Passed**: {} ✅", passed_tests).unwrap();
    writeln!(output, "- **Failed**: {} ❌", failed_tests).unwrap();
    writeln!(
        output,
        "- **Success rate**: {:.1}%",
        (passed_tests as f64 / total_tests as f64) * 100.0
    )
    .unwrap();

    println!("✅ Batch testing complete!");
    println!(
        "📊 Results: {}/{} passed ({:.1}%)",
        passed_tests,
        total_tests,
        (passed_tests as f64 / total_tests as f64) * 100.0
    );
    println!("📝 Detailed results written to src/results/test_results.md");
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
            eprintln!("❌ Unknown rule: {}", rule_name);
            eprintln!("Available rules: file, document, paragraph, text, word, emoji, H1-H6, bold, italic, etc.");
            std::process::exit(1);
        }
    };

    match MarcoParser::parse(rule, input) {
        Ok(pairs) => {
            println!("🌳 Real Parse Tree:");
            for pair in pairs {
                print_pairs(pair, 0);
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
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
        "safe_inline" => Some(Rule::safe_inline),
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
        "inline_image" => Some(Rule::inline_image),
        "inline_link_text" => Some(Rule::inline_link_text),
        "inline_url" => Some(Rule::inline_url),
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
        "tabs_block" => Some(Rule::tabs_block),
        "tabs_header" => Some(Rule::tabs_header),
        "tab" => Some(Rule::tab),
        "tab_line" => Some(Rule::tab_line),
        "tab_title" => Some(Rule::tab_title),
        "tab_content" => Some(Rule::tab_content),
        "tabs_end" => Some(Rule::tabs_end),

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
        "KW_TABS" => Some(Rule::KW_TABS),
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
        "italic_formatting" => "italic".to_string(),
        "bold_italic_combinations" => "bold_italic".to_string(),
        "other_formatting" => {
            if test_name.starts_with("strike") {
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
        "code_inline" => "code_inline".to_string(),
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
        "tabs" => "tabs_block".to_string(),
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
                "tabs_block".to_string()
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
            let test_value = line[eq_pos + 3..].trim();

            // Remove quotes if present
            let test_value = if test_value.starts_with('"') && test_value.ends_with('"') {
                &test_value[1..test_value.len() - 1]
            } else {
                test_value
            };

            // Unescape basic sequences
            let test_value = test_value.replace("\\n", "\n").replace("\\t", "\t");

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

fn print_pairs(pair: pest::iterators::Pair<Rule>, depth: usize) {
    let indent = "  ".repeat(depth);
    let rule_name = format!("{:?}", pair.as_rule());
    let text = pair.as_str();

    // Clone the pair to check if it has children
    let has_children = pair.clone().into_inner().peekable().peek().is_some();

    if has_children {
        // Has children - show structure
        println!("{}├── {} > \"{}\"", indent, rule_name, text);
        for inner_pair in pair.into_inner() {
            print_pairs(inner_pair, depth + 1);
        }
    } else {
        // Leaf node - show value
        println!("{}└── {}: \"{}\"", indent, rule_name, text);
    }
}
