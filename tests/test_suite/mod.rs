// Test suite entry point - runs all test modules

pub mod utils;
pub mod grammar_tests;
pub mod parser_tests;
pub mod ast_tests;
pub mod render_tests;
pub mod lsp_tests;
pub mod commonmark_tests;
pub mod integration_tests;
pub mod example_runner;

// Re-export test runner functions for CLI
pub use grammar_tests::{run_inline_tests, run_block_tests};
pub use parser_tests::run_parser_tests;
pub use render_tests::{run_render_tests, run_inline_pipeline_tests};
pub use commonmark_tests::{run_commonmark_tests, run_extra_tests};
pub use example_runner::run_example_inspection;

// Test suite runs automatically via cargo test
