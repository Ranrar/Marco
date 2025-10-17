//! Marco Test Runner Library
//!
//! This library provides testing functionality for the Marco markdown engine.
//! It can be used both as a standalone CLI tool and integrated into other test suites.

pub mod spec;
pub mod runner;
pub mod diff;
pub mod interactive;
pub mod cli;
pub mod css_debug;
pub mod parser_debug;
pub mod benchmark;

// Re-export main types for external use
pub use spec::{TestCase, TestSpec, TestResult, TestSummary};
pub use runner::{TestRunner, RunnerConfig};
pub use diff::{DiffConfig, create_unified_diff, create_compact_diff, create_side_by_side_diff};
pub use css_debug::{dump_css_analysis, dump_full_css, analyze_css_range, list_css_selectors};
pub use parser_debug::{debug_grammar_rule, debug_ast_building, debug_full_pipeline, debug_setext_headers};
pub use benchmark::{benchmark_markdown, run_benchmark_suite, benchmark_parser_cache, quick_benchmark, BenchmarkResult};