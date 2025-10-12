# Marco Test Suite

Comprehensive testing infrastructure for Marco markdown editor with advanced parser utilities and grammar validation.

## Structure

- **`integration_test_suite.rs`** - 14 comprehensive parser and engine tests with parser utility functions
- **`test_runner/`** - CLI testing framework with JSON specification support
- **`parser_debug/`** - Interactive grammar debugging utilities
- **`spec/`** - JSON test specifications (CommonMark, GFM, Marco)
- **`markdown_showcase/`** - Real-world test documents demonstrating features
- **`integration/`** - Integration tests using Marco engine directly
- **`install/`** - Installation and deployment test scripts

## Quick Start

### Integration Tests with Parser Utilities
```bash
# Run comprehensive integration test suite (14 tests)
cargo test --package marco --test integration_test_suite

# Run specific parser tests module
cargo test --package marco --test integration_test_suite parser_tests

# Run with verbose output
cargo test --package marco --test integration_test_suite -- --nocapture
```

### CLI Testing Tools
```bash
# Test markdown strings (requires integration-tests feature)
cargo run --bin marco-test --features integration-tests -- string "# Hello **World**"

# Multi-line input
echo "Header
======" | cargo run --bin marco-test --features integration-tests -- string

# Run CommonMark specification tests
cargo run --bin marco-test --features integration-tests -- spec --file tests/spec/commonmark.json

# Test specific sections
cargo run --bin marco-test --features integration-tests -- spec --section "Headers"
```

### Grammar Debugging
```bash
# Debug parser issues (requires integration-tests feature)
echo "Header\n======" | cargo run --bin marco-parser-debug --features integration-tests pipeline

# Visualize parse tree
cargo run --bin marco-test --features integration-tests -- visualize "# Heading"

# Filter by rule type
cargo run --bin marco-test --features integration-tests -- \
    visualize "**bold** text" --rule "bold"

# Limit tree depth
cargo run --bin marco-test --features integration-tests -- \
    visualize "complex markdown" --depth 3

# Debug tab handling
cargo run --bin marco-test --features integration-tests -- \
    visualize $'\tindented code'

# Test specific grammar rules
echo "# Header" | cargo run --bin marco-parser-debug --features integration-tests grammar --rule heading

# Test nested code blocks
cargo run --bin marco-parser-debug --features integration-tests pipeline < tests/markdown_showcase/test_nested_code.md
```

## Parser Utility Functions

The integration test suite uses enhanced parser utility functions for clean, readable testing:

```rust
// Import from marco_core
use marco_core::components::marco_engine::{parse_to_html_cached, Rule};
use marco_core::{parse_document, parse_with_rule, ParseResult};

// Type alias for consistent error handling
type ParseResult<T> = Result<T, String>;

// Parse complete documents
let result = parse_document("# Header\n\nContent");

// Test specific grammar rules
let result = parse_with_rule("^superscript^", Rule::superscript);
```

### Integration Test Suite

The `integration_test_suite.rs` contains 14 comprehensive tests:

1. **Grammar Tests** - `test_setext_h1_grammar`, `test_setext_h2_grammar`
2. **Content Extraction** - `test_setext_content_extraction`
3. **Document Processing** - `test_document_with_setext_headers`
4. **Header Comparison** - `test_setext_vs_atx_headers`
5. **Engine Integration** - `test_marco_engine_setext_rendering`
6. **Error Handling** - `test_parser_error_handling_with_parse_result`
7. **Grammar Validation** - `test_grammar_rule_validation_suite`
8. **Document Parsing** - `test_parse_document_comprehensive`
9. **Marco Syntax** - `test_marco_specific_syntax`
10. **Performance Testing** - `test_parser_performance_with_parse_result`
11. **Binary Testing** - `test_marco_test_binary_basic_functionality`
12. **Failure Cases** - `test_marco_test_binary_failure_case`
13. **Smoke Tests** - `test_marco_engine_smoke_test`

## Parser Debugging

The `marco-parser-debug` tool provides interactive debugging (requires `--features integration-tests`):

```bash
# Build debug tool with integration-tests feature
cargo build --bin marco-parser-debug --features integration-tests

# Test specific grammar rules
echo "# Header" | cargo run --bin marco-parser-debug --features integration-tests grammar --rule heading

# Debug AST building
echo "Header\n======" | cargo run --bin marco-parser-debug --features integration-tests ast

# Full pipeline debug (grammar → AST → HTML)
echo "Header\n======" | cargo run --bin marco-parser-debug --features integration-tests pipeline

# Test nested code blocks
cargo run --bin marco-parser-debug --features integration-tests pipeline < tests/markdown_showcase/test_nested_code.md

# Setext header specialist
echo "" | cargo run --bin marco-parser-debug --features integration-tests setext
```

### Available Commands

- **`grammar --rule RULE`** - Test specific grammar rules with detailed output
- **`ast`** - Debug AST building and structure validation
- **`pipeline`** - Full grammar → AST → HTML transformation debug
- **`setext`** - Specialized setext header parsing and validation

### Debugging Workflow

1. **Identify Issue**: `cargo run --bin marco-test --features integration-tests -- string "problem" --expected "output"`
2. **Debug Grammar**: `echo "problem" | cargo run --bin marco-parser-debug --features integration-tests grammar --rule specific_rule`
3. **Analyze Pipeline**: `echo "problem" | cargo run --bin marco-parser-debug --features integration-tests pipeline`
4. **Fix Code**: Update parser, AST builder, or renderer in `marco_core`
5. **Validate Fix**: Re-run integration tests and CLI tools

## Test Specifications

### JSON Specification Format
```json
[
  {
    "example": 1,
    "markdown": "# Header",
    "html": "<h1>Header</h1>",
    "section": "Headers"
  }
]
```

### Available Specification Files
- **`commonmark.json`** - CommonMark specification compliance tests
- **`gfm.json`** - GitHub Flavored Markdown tests
- **`marco.json`** - Marco-specific syntax extensions
- **`markdown_extra.json`** - Extended markdown features

## Advanced Testing

### Performance & Benchmarking
```bash
# Performance testing with integration suite
cargo test --package marco --test integration_test_suite test_parser_performance_with_parse_result -- --nocapture

# Benchmark parsing different document sizes
cargo test --package marco --test integration_test_suite --release
```

### Grammar Rule Testing
```rust
// Test specific grammar rules with utility functions from marco_core
use marco_core::{parse_with_rule, ParseResult};
use marco_core::components::marco_engine::Rule;

fn test_custom_grammar() {
    let result = parse_with_rule("^superscript^", Rule::superscript);
    assert!(result.is_ok(), "Should parse superscript successfully");
    
    // Validate parse tree structure
    let pairs = result.unwrap();
    let pair = pairs.into_iter().next().unwrap();
    assert_eq!(pair.as_rule(), Rule::superscript);
}
```

### Error Handling Validation
```rust
// Test error conditions with ParseResult from marco_core
use marco_core::{parse_document, ParseResult};

fn test_error_handling() {
    let result = parse_document("invalid markdown ]]");
    assert!(result.is_err(), "Should fail on invalid syntax");
    
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("parsing error"));
}
```

## Testing Tips & Best Practices

### Command Line Usage
```bash
# Fail fast on specification tests (requires integration-tests feature)
cargo run --bin marco-test --features integration-tests -- spec --file tests/spec/commonmark.json --fail-fast

# Test specific sections only
cargo run --bin marco-test --features integration-tests -- spec --section "Headers"

# Multi-line input with heredoc
cargo run --bin marco-test --features integration-tests -- string << 'EOF'
Complex markdown
================
With multiple lines
EOF

# Interactive specification testing
cargo run --bin marco-test --features integration-tests -- interactive
```

### Integration Testing Guidelines
- **Use parser utilities** for clean, readable test code
- **Test grammar rules individually** before testing full documents
- **Validate parse tree structure** not just success/failure
- **Include error handling tests** to ensure robust parsing
- **Test performance** with various document sizes

### Smoke Testing Approach
Marco prioritizes **smoke tests** for reliable, fast validation:

```rust
use marco_core::{parse_document, components::marco_engine::parse_to_html_cached};

#[test]
fn smoke_test_parser_functionality() {
    // Test basic parsing without complex setup
    let result = parse_document("# Hello World");
    assert!(result.is_ok(), "Should parse basic markdown");
    
    // Verify core functionality works
    let html = parse_to_html_cached("**bold**");
    assert!(html.unwrap().contains("strong"), "Should render bold text");
}
```

### Development Workflow
1. **Write failing test** using parser utilities from `marco_core`
2. **Debug with CLI tools** (requires `--features integration-tests`) to understand parsing issues
3. **Fix grammar/parser** incrementally in `marco_core/src/components/marco_engine/`
4. **Validate with integration tests** to ensure no regressions
5. **Add smoke tests** for new functionality

## Architecture

Marco is organized as a Cargo workspace with three crates:

- **`marco_core`** - Pure Rust library with parser, AST builder, HTML renderer (no GTK dependencies)
- **`marco`** - Full-featured editor binary (GTK4 UI, depends on marco_core)
- **`polo`** - Lightweight viewer binary (depends on marco_core)

Test binaries (`marco-test`, `marco-parser-debug`) are defined in `marco/Cargo.toml` but use `marco_core` for all parsing/rendering functionality. They require the `integration-tests` feature flag to build.