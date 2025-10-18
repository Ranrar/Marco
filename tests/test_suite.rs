// Marco Test Suite - CLI entry point for all tests
// Usage: cargo test --package core --test test_suite -- --help

use clap::{Parser, Subcommand};

// Include test modules
#[path = "test_suite/utils.rs"]
mod utils;
#[path = "test_suite/grammar_tests.rs"]
mod grammar_tests;
#[path = "test_suite/parser_tests.rs"]
mod parser_tests;
#[path = "test_suite/render_tests.rs"]
mod render_tests;
#[path = "test_suite/commonmark_tests.rs"]
mod commonmark_tests;

use grammar_tests::{run_inline_tests, run_block_tests};
use parser_tests::run_parser_tests;
use render_tests::{run_render_tests, run_inline_pipeline_tests};
use commonmark_tests::run_commonmark_tests;

/// Marco Test Suite - Comprehensive testing for the nom-based Markdown parser
#[derive(Parser)]
#[command(name = "test-suite")]
#[command(about = "Marco Test Suite - Test grammar, parser, AST, renderer, and LSP", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test inline grammar parsers (code spans, emphasis, strong, links, images)
    Inline {
        /// Run only specific test by name
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Test block grammar parsers (headings, paragraphs, code blocks, lists)
    Block {
        /// Run only specific test by name
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Test parser orchestration (blocks → inlines → AST)
    Parser,
    /// Test AST node structures and traversal
    Ast,
    /// Test HTML renderer (AST → HTML)
    Render,
    /// Test LSP features (syntax highlighting, completion, diagnostics)
    Lsp,
    /// Test against CommonMark spec examples
    Commonmark {
        /// Section to test (e.g., "Code spans", "ATX headings")
        #[arg(short, long)]
        section: Option<String>,
    },
    /// Run all tests
    All,
    /// Show test suite summary
    Summary,
}

fn show_summary() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║              Marco Test Suite Summary                      ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    println!("Available test commands:");
    println!("  inline       - Test inline grammar (code spans, emphasis, strong, links)");
    println!("  block        - Test block grammar (headings, paragraphs, code blocks)");
    println!("  parser       - Test parser integration (blocks → inlines → AST)");
    println!("  ast          - Test AST node structures");
    println!("  render       - Test HTML renderer (AST → HTML)");
    println!("  lsp          - Test LSP features (highlighting, completion, diagnostics)");
    println!("  commonmark   - Test against CommonMark spec examples");
    println!("  all          - Run all tests");
    println!("  summary      - Show this help\n");
    
    println!("Examples:");
    println!("  cargo test --package core --test test_suite -- inline");
    println!("  cargo test --package core --test test_suite -- block --filter heading");
    println!("  cargo test --package core --test test_suite -- commonmark --section \"Code spans\"");
    println!();
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Inline { filter }) => {
            run_inline_tests(filter);
        }
        Some(Commands::Block { filter }) => {
            run_block_tests(filter);
        }
        Some(Commands::Parser) => {
            run_parser_tests();
        }
        Some(Commands::Ast) => {
            println!("AST tests not yet implemented");
        }
        Some(Commands::Render) => {
            run_render_tests();
            run_inline_pipeline_tests();
        }
        Some(Commands::Lsp) => {
            println!("LSP tests not yet implemented");
        }
        Some(Commands::Commonmark { section }) => {
            run_commonmark_tests(section);
        }
        Some(Commands::All) => {
            run_inline_tests(None);
            run_block_tests(None);
            run_parser_tests();
            run_render_tests();
            run_inline_pipeline_tests();
            run_commonmark_tests(None);
        }
        Some(Commands::Summary) | None => {
            show_summary();
        }
    }
}

// Rust test integration
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inline_elements_pipeline() {
        run_inline_pipeline_tests();
    }
    
    #[test]
    fn test_render_pipeline() {
        run_render_tests();
    }
}
