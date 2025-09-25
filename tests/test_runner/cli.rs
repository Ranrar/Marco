//! Command-line interface for the Marco test runner
//!
//! This module provides a CLI that supports different modes:
//! - String mode: Test markdown strings directly
//! - File mode: Process markdown files  
//! - HTML mode: Compare HTML against specs
//! - Interactive mode: Add new baselines
//! - Spec mode: Run all spec tests

use crate::diff::{calculate_diff_stats, create_side_by_side_diff, DiffConfig};
use crate::interactive::InteractiveManager;
use crate::runner::{RunnerConfig, TestRunner};
use crate::spec::{TestResult, TestSpec};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "marco-test")]
#[command(about = "Automated test suite for Marco Markdown engine")]
#[command(version)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_colors: bool,

    /// Normalize whitespace before comparison
    #[arg(long, default_value = "true")]
    pub normalize_whitespace: bool,

    /// Number of context lines in diffs
    #[arg(long, default_value = "3")]
    pub context_lines: usize,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Test a markdown string directly
    String {
        /// The markdown text to test (if not provided, reads from stdin)
        markdown: Option<String>,

        /// Expected HTML output (optional)
        #[arg(short, long)]
        expected: Option<String>,

        /// Show side-by-side diff instead of unified
        #[arg(long)]
        side_by_side: bool,
    },

    /// Process a markdown file through Marco
    File {
        /// Path to the markdown file
        input: PathBuf,

        /// Output path for HTML (optional, prints to stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Expected HTML file to compare against (optional)
        #[arg(short, long)]
        expected: Option<PathBuf>,
    },

    /// Compare HTML output against JSON specifications
    Html {
        /// HTML content to compare
        #[arg(short, long)]
        html: String,

        /// Specification file to check against
        #[arg(short, long)]
        spec: Option<PathBuf>,

        /// Example number to compare against (optional)
        #[arg(short, long)]
        example: Option<u32>,
    },

    /// Run tests from specification files
    Spec {
        /// Specification file to run (defaults to all)
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Run only tests matching this section
        #[arg(long)]
        section: Option<String>,

        /// Run only this example number
        #[arg(long)]
        example: Option<u32>,

        /// Stop on first failure
        #[arg(long)]
        fail_fast: bool,
    },

    /// Interactive mode for creating and managing test baselines
    Interactive,

    /// Show statistics about test specifications
    Stats {
        /// Specification file to analyze (defaults to all)
        spec: Option<PathBuf>,
    },
}

impl Cli {
    /// Create a runner configuration from CLI arguments
    pub fn create_runner_config(&self) -> RunnerConfig {
        RunnerConfig {
            html_options: marco::components::marco_engine::HtmlOptions::default(),
            use_cache: true,
            normalize_whitespace: self.normalize_whitespace,
            verbose: self.verbose,
            diff_config: DiffConfig {
                use_colors: !self.no_colors,
                context_lines: self.context_lines,
                show_line_numbers: true,
            },
        }
    }

    /// Run the CLI application
    pub fn run(&self) -> Result<()> {
        let runner = TestRunner::new(self.create_runner_config());

        match &self.command {
            Commands::String {
                markdown,
                expected,
                side_by_side,
            } => {
                let markdown_content = if let Some(content) = markdown {
                    content.clone()
                } else {
                    // Read from stdin
                    use std::io::{self, Read};
                    let mut buffer = String::new();
                    io::stdin()
                        .read_to_string(&mut buffer)
                        .context("Failed to read markdown from stdin")?;
                    buffer
                };
                self.run_string_mode(
                    &runner,
                    &markdown_content,
                    expected.as_deref(),
                    *side_by_side,
                )
            }

            Commands::File {
                input,
                output,
                expected,
            } => self.run_file_mode(&runner, input, output.as_ref(), expected.as_ref()),

            Commands::Html {
                html,
                spec,
                example,
            } => self.run_html_mode(&runner, html, spec.as_ref(), *example),

            Commands::Spec {
                file,
                section,
                example,
                fail_fast,
            } => self.run_spec_mode(
                &runner,
                file.as_ref(),
                section.as_deref(),
                *example,
                *fail_fast,
            ),

            Commands::Interactive => self.run_interactive_mode(&runner),

            Commands::Stats { spec } => self.run_stats_mode(spec.as_ref()),
        }
    }

    /// Run string testing mode
    fn run_string_mode(
        &self,
        runner: &TestRunner,
        markdown: &str,
        expected: Option<&str>,
        side_by_side: bool,
    ) -> Result<()> {
        println!("{}", "Testing markdown string...".blue().bold());
        println!();

        let result = runner.test_string(markdown, expected);

        match result {
            TestResult::Passed => {
                println!("{}", "✓ Test passed!".green().bold());
            }

            TestResult::Failed {
                expected,
                actual,
                diff,
            } => {
                println!("{}", "✗ Test failed!".red().bold());
                println!();

                if side_by_side {
                    let side_diff =
                        create_side_by_side_diff(&expected, &actual, &runner.config.diff_config);
                    println!("{}", side_diff);
                } else {
                    println!("{}", diff);
                }

                // Show stats
                let stats = calculate_diff_stats(&expected, &actual);
                println!();
                println!("Similarity: {:.1}%", stats.similarity_ratio * 100.0);
                return Err(anyhow::anyhow!("Test failed"));
            }

            TestResult::NoBaseline { actual } => {
                println!(
                    "{}",
                    "No expected result provided. Here's the actual output:"
                        .yellow()
                        .bold()
                );
                println!();
                println!("{}", actual);
            }

            TestResult::Error { message } => {
                println!("{}: {}", "Error".red().bold(), message);
                return Err(anyhow::anyhow!("Test execution failed: {}", message));
            }
        }

        Ok(())
    }

    /// Run file processing mode
    fn run_file_mode(
        &self,
        runner: &TestRunner,
        input: &PathBuf,
        output: Option<&PathBuf>,
        expected: Option<&PathBuf>,
    ) -> Result<()> {
        println!("{} {:?}...", "Processing file".blue().bold(), input);

        let html_result = runner
            .process_file(input)
            .with_context(|| format!("Failed to process file {:?}", input))?;

        // Write output if requested
        if let Some(output_path) = output {
            std::fs::write(output_path, &html_result)
                .with_context(|| format!("Failed to write output to {:?}", output_path))?;
            println!("{} {:?}", "Output written to".green(), output_path);
        } else {
            println!("{}", "HTML Output:".blue().bold());
            println!("{}", html_result);
        }

        // Compare against expected if provided
        if let Some(expected_path) = expected {
            let expected_content = std::fs::read_to_string(expected_path)
                .with_context(|| format!("Failed to read expected file {:?}", expected_path))?;

            let test_result =
                runner.test_string(&std::fs::read_to_string(input)?, Some(&expected_content));

            match test_result {
                TestResult::Passed => {
                    println!("{}", "✓ Output matches expected!".green().bold());
                }
                TestResult::Failed { diff, .. } => {
                    println!("{}", "✗ Output differs from expected!".red().bold());
                    println!();
                    println!("{}", diff);
                }
                _ => {} // Other cases don't apply here
            }
        }

        Ok(())
    }

    /// Run HTML comparison mode
    fn run_html_mode(
        &self,
        runner: &TestRunner,
        html: &str,
        spec_file: Option<&PathBuf>,
        example: Option<u32>,
    ) -> Result<()> {
        println!(
            "{}",
            "Comparing HTML against specifications...".blue().bold()
        );

        let spec_path = spec_file
            .cloned()
            .unwrap_or_else(|| PathBuf::from("tests/spec/commonmark.json"));
        let spec = TestSpec::load_from_file(&spec_path)
            .with_context(|| format!("Failed to load specification from {:?}", spec_path))?;

        if let Some(example_num) = example {
            // Compare against specific example
            if let Some(test_case) = spec.find_by_example(example_num) {
                let result = runner.test_string(&test_case.markdown, Some(html));

                match result {
                    TestResult::Passed => {
                        println!(
                            "{} example {}",
                            "✓ HTML matches".green().bold(),
                            example_num
                        );
                    }
                    TestResult::Failed { diff, .. } => {
                        println!(
                            "{} example {}",
                            "✗ HTML differs from".red().bold(),
                            example_num
                        );
                        println!();
                        println!("{}", diff);
                    }
                    _ => {} // Other cases don't apply
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Example {} not found in specification",
                    example_num
                ));
            }
        } else {
            // Find closest match
            println!("{}", "Finding closest matches...".yellow());
            // This could be enhanced to find the most similar test cases
            println!(
                "{}",
                "HTML comparison without specific example not yet implemented".yellow()
            );
        }

        Ok(())
    }

    /// Run specification testing mode
    fn run_spec_mode(
        &self,
        runner: &TestRunner,
        file: Option<&PathBuf>,
        section: Option<&str>,
        example: Option<u32>,
        fail_fast: bool,
    ) -> Result<()> {
        println!("{}", "Running specification tests...".blue().bold());
        println!();

        let (results, summary) = if let Some(spec_file) = file {
            runner.run_spec_file(spec_file)?
        } else {
            runner.run_all_tests()?
        };

        // Filter results if needed
        let filtered_results: Vec<_> = results
            .into_iter()
            .filter(|(test_case, _)| {
                if let Some(sec) = section {
                    test_case.section.eq_ignore_ascii_case(sec)
                } else {
                    true
                }
            })
            .filter(|(test_case, _)| {
                if let Some(ex) = example {
                    test_case.example == ex
                } else {
                    true
                }
            })
            .collect();

        // Display results
        let mut _failed_count = 0;
        for (test_case, result) in &filtered_results {
            match result {
                TestResult::Passed => {
                    if self.verbose {
                        println!(
                            "{} Example {} ({})",
                            "✓".green(),
                            test_case.example,
                            test_case.section
                        );
                    }
                }
                TestResult::Failed { diff, .. } => {
                    _failed_count += 1;
                    println!(
                        "{} Example {} ({}):",
                        "✗".red().bold(),
                        test_case.example,
                        test_case.section
                    );
                    if self.verbose {
                        println!("{}", diff);
                        println!();
                    }

                    if fail_fast {
                        return Err(anyhow::anyhow!(
                            "Stopping on first failure (fail-fast mode)"
                        ));
                    }
                }
                TestResult::NoBaseline { .. } => {
                    println!(
                        "{} Example {} ({}): No baseline",
                        "?".yellow().bold(),
                        test_case.example,
                        test_case.section
                    );
                }
                TestResult::Error { message } => {
                    println!(
                        "{} Example {} ({}): {}",
                        "!".red().bold(),
                        test_case.example,
                        test_case.section,
                        message
                    );

                    if fail_fast {
                        return Err(anyhow::anyhow!("Stopping on error (fail-fast mode)"));
                    }
                }
            }
        }

        // Print summary
        println!();
        println!("{}", "Test Results Summary:".blue().bold());
        println!("Total tests: {}", summary.total);
        println!("{}: {}", "Passed".green(), summary.passed);
        println!("{}: {}", "Failed".red(), summary.failed);
        println!("{}: {}", "Need baseline".yellow(), summary.needs_baseline);
        println!("{}: {}", "Errors".red(), summary.errors);
        println!("Success rate: {:.1}%", summary.success_rate());

        if summary.failed > 0 || summary.errors > 0 {
            std::process::exit(1);
        }

        Ok(())
    }

    /// Run interactive mode for managing baselines
    fn run_interactive_mode(&self, runner: &TestRunner) -> Result<()> {
        let manager = InteractiveManager::new(runner.clone());
        manager.run()
    }

    /// Run statistics mode
    fn run_stats_mode(&self, spec_file: Option<&PathBuf>) -> Result<()> {
        println!("{}", "Test Specification Statistics".blue().bold());
        println!();

        if let Some(file) = spec_file {
            self.show_spec_stats(file)?;
        } else {
            // Show stats for all spec files
            for spec_file in ["tests/spec/commonmark.json", "tests/spec/marco.json"] {
                let path = PathBuf::from(spec_file);
                if path.exists() {
                    self.show_spec_stats(&path)?;
                    println!();
                }
            }
        }

        Ok(())
    }

    /// Show statistics for a single specification file
    fn show_spec_stats(&self, spec_file: &PathBuf) -> Result<()> {
        let spec = TestSpec::load_from_file(spec_file)
            .with_context(|| format!("Failed to load specification from {:?}", spec_file))?;

        println!("{}: {}", "File".blue().bold(), spec.source);
        println!("Total tests: {}", spec.tests.len());

        // Group by section
        let mut sections = std::collections::HashMap::new();
        for test in &spec.tests {
            *sections.entry(&test.section).or_insert(0) += 1;
        }

        println!("Sections:");
        for (section, count) in sections {
            println!("  {}: {} tests", section, count);
        }

        // Example number range
        if !spec.tests.is_empty() {
            let min_example = spec.tests.iter().map(|t| t.example).min().unwrap();
            let max_example = spec.tests.iter().map(|t| t.example).max().unwrap();
            println!("Example range: {} - {}", min_example, max_example);
        }

        Ok(())
    }
}

/// Entry point for the CLI application
pub fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}
