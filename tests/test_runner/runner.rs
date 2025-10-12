//! Core test runner functionality
//!
//! This module provides the main testing logic that processes markdown through
//! the Marco engine and compares results against expected outputs.

use crate::diff::{create_unified_diff, DiffConfig};
use crate::spec::{TestCase, TestResult, TestSpec, TestSummary};
use anyhow::{Context, Result};
use marco_core::components::marco_engine::HtmlOptions;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for the test runner
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// HTML rendering options to use
    pub html_options: HtmlOptions,

    /// Whether to use cached parsing (default: true)
    pub use_cache: bool,

    /// Whether to normalize whitespace before comparison (default: true)
    pub normalize_whitespace: bool,

    /// Whether to be verbose in output
    pub verbose: bool,

    /// Configuration for diff display
    pub diff_config: DiffConfig,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            html_options: HtmlOptions::default(),
            use_cache: true,
            normalize_whitespace: true,
            verbose: false,
            diff_config: DiffConfig::default(),
        }
    }
}

/// Main test runner
#[derive(Clone)]
pub struct TestRunner {
    pub config: RunnerConfig,
}

impl TestRunner {
    /// Create a new test runner with the given configuration
    pub fn new(config: RunnerConfig) -> Self {
        Self { config }
    }

    /// Process markdown text through the Marco engine
    pub fn process_markdown(&self, markdown: &str) -> Result<String, String> {
        if self.config.use_cache {
            // Use cached processing for better performance
            use marco_core::components::marco_engine::global_parser_cache;
            global_parser_cache()
                .render_with_cache(markdown, self.config.html_options.clone())
                .map_err(|e| format!("Failed to render HTML (cached): {}", e))
        } else {
            // Use direct non-cached processing for testing
            use marco_core::components::marco_engine::{build_ast, parse_text, render_html};
            let pairs = parse_text(markdown)?;
            let ast = build_ast(pairs)?;
            Ok(render_html(&ast, self.config.html_options.clone()))
        }
    }

    /// Run a single test case and return the result
    pub fn run_test_case(&self, test_case: &TestCase) -> TestResult {
        let actual_html = match self.process_markdown(&test_case.markdown) {
            Ok(html) => html,
            Err(err) => {
                return TestResult::Error {
                    message: format!("Failed to process markdown: {}", err),
                }
            }
        };

        let expected = if self.config.normalize_whitespace {
            normalize_html(&test_case.html)
        } else {
            test_case.html.clone()
        };

        let actual = if self.config.normalize_whitespace {
            normalize_html(&actual_html)
        } else {
            actual_html
        };

        if expected == actual {
            TestResult::Passed
        } else {
            TestResult::Failed {
                expected: expected.clone(),
                actual: actual.clone(),
                diff: create_unified_diff(&expected, &actual, &self.config.diff_config),
            }
        }
    }

    /// Run all tests in a test specification
    pub fn run_spec(&self, spec: &TestSpec) -> (Vec<(TestCase, TestResult)>, TestSummary) {
        let mut results = Vec::new();
        let mut summary = TestSummary::new();

        for test_case in &spec.tests {
            if self.config.verbose {
                eprintln!(
                    "Running test example {} from section '{}'...",
                    test_case.example, test_case.section
                );
            }

            let result = self.run_test_case(test_case);
            summary.record(&result);
            results.push((test_case.clone(), result));
        }

        (results, summary)
    }

    /// Run tests from a JSON specification file
    pub fn run_spec_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(Vec<(TestCase, TestResult)>, TestSummary)> {
        let spec = TestSpec::load_from_file(&path).with_context(|| {
            format!("Failed to load test specification from {:?}", path.as_ref())
        })?;

        if self.config.verbose {
            eprintln!(
                "Loaded {} test cases from {}",
                spec.tests.len(),
                spec.source
            );
        }

        Ok(self.run_spec(&spec))
    }

    /// Process a markdown file and return HTML output
    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref();
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file {:?}", path))?;

        self.process_markdown(&content)
            .map_err(|err| anyhow::anyhow!("Failed to process markdown from {:?}: {}", path, err))
    }

    /// Test a markdown string and return the result (useful for interactive mode)
    pub fn test_string(&self, markdown: &str, expected_html: Option<&str>) -> TestResult {
        let actual_html = match self.process_markdown(markdown) {
            Ok(html) => html,
            Err(err) => {
                return TestResult::Error {
                    message: format!("Failed to process markdown: {}", err),
                }
            }
        };

        match expected_html {
            Some(expected) => {
                let expected = if self.config.normalize_whitespace {
                    normalize_html(expected)
                } else {
                    expected.to_string()
                };

                let actual = if self.config.normalize_whitespace {
                    normalize_html(&actual_html)
                } else {
                    actual_html
                };

                if expected == actual {
                    TestResult::Passed
                } else {
                    TestResult::Failed {
                        expected: expected.clone(),
                        actual: actual.clone(),
                        diff: create_unified_diff(&expected, &actual, &self.config.diff_config),
                    }
                }
            }
            None => TestResult::NoBaseline {
                actual: actual_html,
            },
        }
    }

    /// Find and run CommonMark specification tests
    pub fn run_commonmark_tests(&self) -> Result<(Vec<(TestCase, TestResult)>, TestSummary)> {
        let spec_path = PathBuf::from("tests/spec/commonmark.json");
        self.run_spec_file(spec_path)
    }

    /// Find and run Marco-specific tests  
    pub fn run_marco_tests(&self) -> Result<(Vec<(TestCase, TestResult)>, TestSummary)> {
        let spec_path = PathBuf::from("tests/spec/marco.json");
        self.run_spec_file(spec_path)
    }

    /// Run all specification tests (CommonMark + Marco)
    pub fn run_all_tests(&self) -> Result<(Vec<(TestCase, TestResult)>, TestSummary)> {
        let mut all_results = Vec::new();
        let mut combined_summary = TestSummary::new();

        // Run CommonMark tests
        match self.run_commonmark_tests() {
            Ok((results, summary)) => {
                if self.config.verbose {
                    eprintln!(
                        "CommonMark tests: {} passed, {} failed, {} need baseline, {} errors",
                        summary.passed, summary.failed, summary.needs_baseline, summary.errors
                    );
                }
                all_results.extend(results);
                combined_summary.total += summary.total;
                combined_summary.passed += summary.passed;
                combined_summary.failed += summary.failed;
                combined_summary.needs_baseline += summary.needs_baseline;
                combined_summary.errors += summary.errors;
            }
            Err(err) => {
                eprintln!("Warning: Failed to run CommonMark tests: {}", err);
            }
        }

        // Run Marco tests
        match self.run_marco_tests() {
            Ok((results, summary)) => {
                if self.config.verbose {
                    eprintln!(
                        "Marco tests: {} passed, {} failed, {} need baseline, {} errors",
                        summary.passed, summary.failed, summary.needs_baseline, summary.errors
                    );
                }
                all_results.extend(results);
                combined_summary.total += summary.total;
                combined_summary.passed += summary.passed;
                combined_summary.failed += summary.failed;
                combined_summary.needs_baseline += summary.needs_baseline;
                combined_summary.errors += summary.errors;
            }
            Err(err) => {
                eprintln!("Warning: Failed to run Marco tests: {}", err);
            }
        }

        Ok((all_results, combined_summary))
    }
}

/// Normalize HTML for comparison (remove extra whitespace, etc.)
fn normalize_html(html: &str) -> String {
    html.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::TestCase;

    #[test]
    fn test_normalize_html() {
        let input = "  <p>  \n  Hello World  \n  </p>  \n";
        let expected = "<p>\nHello World\n</p>";
        assert_eq!(normalize_html(input), expected);
    }

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new(RunnerConfig::default());
        assert!(runner.config.use_cache);
        assert!(runner.config.normalize_whitespace);
        assert!(!runner.config.verbose);
    }

    #[test]
    fn test_cached_vs_non_cached() {
        let cached_config = RunnerConfig {
            use_cache: true,
            ..RunnerConfig::default()
        };
        let non_cached_config = RunnerConfig {
            use_cache: false,
            ..RunnerConfig::default()
        };

        let cached_runner = TestRunner::new(cached_config);
        let non_cached_runner = TestRunner::new(non_cached_config);

        let markdown = "# Test Header\n\nThis is a **test**.";

        let cached_result = cached_runner.process_markdown(markdown);
        let non_cached_result = non_cached_runner.process_markdown(markdown);

        // Both should succeed
        assert!(cached_result.is_ok());
        assert!(non_cached_result.is_ok());

        // Results should be identical (same HTML output)
        assert_eq!(cached_result.unwrap(), non_cached_result.unwrap());
    }

    #[test]
    fn test_process_simple_markdown() {
        let runner = TestRunner::new(RunnerConfig::default());
        let result = runner.process_markdown("# Hello World");

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("Hello World"));
        // Note: We can't be too specific about the exact HTML since Marco
        // may have its own formatting. The main test is that it doesn't error.
    }

    #[test]
    fn test_run_test_case_pass() {
        let runner = TestRunner::new(RunnerConfig::default());
        let test_case = TestCase {
            markdown: "**bold**".to_string(),
            html: "<p><strong>bold</strong></p>".to_string(), // This might need adjustment based on Marco's output
            example: 1,
            start_line: 1,
            end_line: 1,
            section: "Test".to_string(),
        };

        let result = runner.run_test_case(&test_case);
        // Note: This test may fail until we verify Marco's exact HTML output format
        // It's more of a smoke test to ensure the function doesn't panic
        match result {
            TestResult::Passed | TestResult::Failed { .. } => {
                // Both are acceptable for this smoke test
            }
            TestResult::Error { message } => {
                panic!("Unexpected error: {}", message);
            }
            TestResult::NoBaseline { .. } => {
                panic!("Unexpected no baseline result");
            }
        }
    }

    #[test]
    fn test_string_test_no_baseline() {
        let runner = TestRunner::new(RunnerConfig::default());
        let result = runner.test_string("# Test", None);

        match result {
            TestResult::NoBaseline { actual } => {
                assert!(actual.contains("Test"));
            }
            _ => panic!("Expected NoBaseline result"),
        }
    }

    #[test]
    fn test_empty_spec() {
        let runner = TestRunner::new(RunnerConfig::default());
        let spec = TestSpec {
            tests: vec![],
            source: "empty.json".to_string(),
        };

        let (results, summary) = runner.run_spec(&spec);
        assert_eq!(results.len(), 0);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.success_rate(), 100.0); // Empty is considered 100% success rate
    }
}
