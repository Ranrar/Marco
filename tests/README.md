# Marco Test Suite

Comprehensive testing infrastructure for Marco markdown editor with parser debugging, benchmarking, and CommonMark compliance testing.

## Quick Start

```bash
# Run integration tests (fastest, use this most often)
cargo test --package marco --test integration_test_suite

# Test markdown string
echo "# Hello" | cargo run --bin marco-test --features integration-tests -- string

# Debug parser
cargo run --bin marco-test --features integration-tests -- debug pipeline "**text**"

# Check CSS
cargo run --bin marco-test --features integration-tests -- css

# Run benchmarks
cargo run --bin marco-test --features integration-tests -- benchmark --suite
```

**📖 See [CLI_GUIDE.md](CLI_GUIDE.md) for comprehensive documentation**

## Structure

```
tests/
├── integration_test_suite.rs    # 48 integration tests (main entry point)
├── test_runner/                 # Unified CLI tool (marco-test)
│   ├── cli.rs                   # Command handlers
│   ├── css_debug.rs             # CSS debugging
│   ├── parser_debug.rs          # Parser debugging
│   ├── benchmark.rs             # Performance testing
│   └── ...
├── spec/
│   ├── commonmark.json          # CommonMark 0.31.2 (652 tests)
│   └── marco.json               # Marco extensions
└── markdown_showcase/           # Real-world test documents
```

## Integration Test Suite

**Primary test file:** `integration_test_suite.rs` (48 tests, < 0.02s)

```bash
cargo test --package marco --test integration_test_suite
```

**Test modules:**
- `parser_tests` - Grammar validation (13 tests)
- `document_tests` - Document parsing (4 tests)
- `api_tests` - Public API (8 tests)
- `cache_tests` - Parser caching (2 tests)
- `edge_cases` - Boundary conditions (7 tests)
- `smoke_tests` - Basic functionality (13 tests)
- `comprehensive_test` - Full integration (1 test)

## CLI Tool (marco-test)

Unified testing, debugging, and benchmarking tool:

```bash
cargo run --bin marco-test --features integration-tests -- <command>
```

**Main commands:**
- `string` - Test markdown strings
- `spec` - Run specification tests (CommonMark, Marco)
- `css` - Debug GTK CSS generation
- `debug` - Parser debugging (grammar, AST, pipeline, setext)
- `benchmark` - Performance testing
- `visualize` - AST visualization
- `interactive` - Manage test baselines

**Examples:**
```bash
# Test string with expected output
marco-test string "# Test" --expected "<h1>Test</h1>"

# Run CommonMark spec
marco-test spec --file tests/spec/commonmark.json

# Debug CSS issues
marco-test css

# Debug parser pipeline
marco-test debug pipeline "**Bold** text"

# Show AST
marco-test debug ast "# Header"

# Benchmark performance
marco-test benchmark --suite
```

## Development Workflow

```bash
# 1. Run quick tests (< 1 second)
cargo test --test integration_test_suite

# 2. Debug if needed
cargo run --bin marco-test --features integration-tests -- debug pipeline "case"

# 3. Commit if passing
git commit -m "Your message"
```

## API Usage

Marco uses a clean two-stage parser API:

```rust
use marco_core::{parse_markdown, render_to_html, parse_and_render, HtmlOptions};

// Parse to AST
let ast = parse_markdown("# Hello")?;

// Render AST to HTML
let html = render_to_html(&ast, HtmlOptions::default());

// Or one-step convenience
let html = parse_and_render("# Hello", HtmlOptions::default())?;
```

## Architecture

Marco workspace structure:
- **`marco_core/`** - Pure Rust library (parser, AST, renderer)
- **`marco/`** - Full editor (GTK4, depends on marco_core)
- **`polo/`** - Viewer (depends on marco_core)

Test binaries use `marco_core` API and require `--features integration-tests` to build.

## Tips

- Use integration tests for fast validation
- Use `marco-test debug` for parser issues
- Use `marco-test css` for GTK CSS problems
- Use `marco-test benchmark` for performance analysis
- See [CLI_GUIDE.md](CLI_GUIDE.md) for detailed usage
