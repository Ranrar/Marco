// Marco Test Suite - CLI entry point for all tests
//
// USAGE EXAMPLES:
//   cargo test --package core --test test_suite -- --help       # Show all available commands
//   cargo test --package core --test test_suite -- all          # Run all tests
//   cargo test --package core --test test_suite -- summary      # Show test summary
//   cargo test --package core --test test_suite -- inline       # Test inline grammar
//   cargo test --package core --test test_suite -- block        # Test block grammar
//   cargo test --package core --test test_suite -- parser       # Test parser
//   cargo test --package core --test test_suite -- render       # Test HTML renderer
//   cargo test --package core --test test_suite -- commonmark   # Test CommonMark spec
//   cargo test --package core --test test_suite -- extra        # Test extra custom cases
//   cargo test --package core --test test_suite -- benchmark    # Run performance benchmarks
//   cargo test --package core --test test_suite -- inspect -e 307,318  # Inspect specific examples
//
// NOTE: Use subcommands (e.g., "extra") NOT flags (e.g., "--extra")

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
#[path = "test_suite/example_runner.rs"]
mod example_runner;
#[path = "test_suite/benchmark_tests.rs"]
mod benchmark_tests;

use grammar_tests::{run_inline_tests, run_block_tests};
use parser_tests::run_parser_tests;
use render_tests::{run_render_tests, run_inline_pipeline_tests};
use commonmark_tests::{run_commonmark_tests, run_extra_tests};
use example_runner::run_example_inspection;
use benchmark_tests::run_performance_benchmarks;

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
    /// Test extra custom test cases (beyond CommonMark spec)
    Extra,
    /// Deep inspection of specific examples (show grammar, AST, render pipeline)
    Inspect {
        /// Example numbers to inspect (comma-separated, e.g., "307,318,653")
        #[arg(short, long)]
        examples: String,
    },
    /// Run performance benchmarks (parse, render, full pipeline)
    Benchmark {
        /// Number of iterations per benchmark (default: 100)
        #[arg(short, long, default_value_t = 100)]
        iterations: usize,
        /// Custom file to benchmark (optional, uses built-in test files if not provided)
        #[arg(short, long)]
        file: Option<String>,
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
    
    println!("BASIC COMMANDS (no arguments needed):");
    println!("  all          - Run all tests");
    println!("  extra        - Test extra custom cases (beyond CommonMark)");
    println!("  inline       - Test inline grammar (emphasis, links, code spans)");
    println!("  block        - Test block grammar (headings, lists, code blocks)");
    println!("  parser       - Test parser integration (blocks → inlines → AST)");
    println!("  ast          - Test AST node structures");
    println!("  render       - Test HTML renderer (AST → HTML)");
    println!("  lsp          - Test LSP features (highlighting, completion)");
    println!("  commonmark   - Test against CommonMark spec (652 examples)");
    println!("  benchmark    - Run performance benchmarks (parse, render, pipeline)");
    println!("  summary      - Show this help\n");
    
    println!("COMMANDS WITH OPTIONS:");
    println!("  inline --filter <name>           - Filter inline tests by name");
    println!("  block --filter <name>            - Filter block tests by name");
    println!("  commonmark --section <name>      - Test specific CommonMark section");
    println!("  benchmark --iterations <n>       - Run benchmarks with n iterations (default: 100)");
    println!("  benchmark --file <path>          - Benchmark a custom markdown file");
    println!("  inspect -e <nums>                - Inspect specific examples (comma-separated)\n");
    
    println!("USAGE EXAMPLES:");
    println!("  # Run all tests");
    println!("  cargo test -p core --test test_suite -- all\n");
    
    println!("  # Run extra custom tests");
    println!("  cargo test -p core --test test_suite -- extra\n");
    
    println!("  # Filter tests by name");
    println!("  cargo test -p core --test test_suite -- block --filter heading\n");
    
    println!("  # Test specific CommonMark section");
    println!("  cargo test -p core --test test_suite -- commonmark --section \"Code spans\"\n");
    
    println!("  # Run performance benchmarks");
    println!("  cargo test -p core --test test_suite -- benchmark");
    println!("  cargo test -p core --test test_suite -- benchmark --iterations 500");
    println!("  cargo test -p core --test test_suite -- benchmark --file ../README.md  # Use ../ for workspace root\n");
    
    println!("  # Inspect specific examples (NOTE: use -e or --examples flag!)");
    println!("  cargo test -p core --test test_suite -- inspect -e 307,318,654");
    println!("  cargo test -p core --test test_suite -- inspect --examples 654\n");
    
    println!("TIP: For more details, run: cargo test -p core --test test_suite -- --help");
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
        Some(Commands::Extra) => {
            run_extra_tests();
        }
        Some(Commands::Inspect { examples }) => {
            // Parse comma-separated example numbers
            let example_numbers: Vec<u32> = examples
                .split(',')
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect();
            
            run_example_inspection(example_numbers);
        }
        Some(Commands::Benchmark { iterations, file }) => {
            run_performance_benchmarks(iterations, file.as_deref());
        }
        Some(Commands::All) => {
            run_inline_tests(None);
            run_block_tests(None);
            run_parser_tests();
            run_render_tests();
            run_inline_pipeline_tests();
            run_commonmark_tests(None);
            run_extra_tests();
        }
        Some(Commands::Summary) | None => {
            show_summary();
        }
    }
}
