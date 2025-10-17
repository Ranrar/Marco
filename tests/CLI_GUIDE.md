# Marco Test Runner - Comprehensive CLI Guide

This document explains how to use the consolidated Marco test runner CLI tool.

## Overview

The `marco-test` CLI consolidates all testing, debugging, and benchmarking tools into a single interface:

- **Integration Tests**: Run via `cargo test --test integration_test_suite` (48 tests)
- **CLI Tool**: Run via `cargo run --bin marco-test --features integration-tests`

## Installation

Build the CLI tool:
```bash
cargo build --bin marco-test --features integration-tests
```

Or run directly:
```bash
cargo run --bin marco-test --features integration-tests -- <command>
```

## Available Commands

### 1. String Testing
Test markdown strings directly:
```bash
# Interactive (read from stdin)
echo "# Hello World" | marco-test string

# With expected output
marco-test string "# Test" --expected "<h1>Test</h1>"

# Side-by-side diff
marco-test string "# Test" --expected "<h1>Test</h1>" --side-by-side
```

### 2. File Processing
Process markdown files:
```bash
# Convert markdown to HTML
marco-test file input.md -o output.html

# Compare against expected output
marco-test file input.md --expected expected.html
```

### 3. Specification Testing
Run CommonMark/Marco spec tests:
```bash
# Run all spec tests
marco-test spec

# Run specific file
marco-test spec --file tests/spec/commonmark.json

# Filter by section
marco-test spec --section "ATX headings"

# Run specific example
marco-test spec --example 32

# Fail fast (stop on first failure)
marco-test spec --fail-fast
```

### 4. CSS Debugging
Debug GTK CSS generation:
```bash
# Show CSS analysis (default)
marco-test css

# Show full CSS output
marco-test css --full

# Analyze specific line range
marco-test css --range 420:430

# List all CSS selectors
marco-test css --selectors
```

**Use Cases:**
- Find GTK CSS parser errors
- Identify unsupported pseudo-classes (`:empty`, `:nth-child`)
- Locate problematic selectors
- Debug CSS generation issues

### 5. Parser Debugging
Debug parser internals:

#### Grammar Debugging
```bash
# Debug specific grammar rule
marco-test debug grammar "# Heading" --rule heading

# Debug full document parsing
marco-test debug grammar "**bold** text"
```

#### AST Debugging
```bash
# Show AST structure
marco-test debug ast "# Hello\n\nParagraph"
```

#### Full Pipeline Debugging
```bash
# Debug parse → AST → HTML pipeline
marco-test debug pipeline "**Bold** text"

# Useful for finding rendering issues
marco-test debug pipeline "Header\n====="
```

#### Setext Header Debugging
```bash
# Test setext headers with default cases
marco-test debug setext

# Test custom setext header
marco-test debug setext "My Header\n========="
```

**Use Cases:**
- Understand parser behavior
- Debug grammar rules
- Inspect AST structure
- Find rendering issues
- Test specific markdown patterns

### 6. Performance Benchmarking
Measure parsing performance:

#### Basic Benchmark
```bash
# Benchmark with 100 iterations (default)
echo "# Test" | marco-test benchmark

# Custom iteration count
echo "**Bold** text" | marco-test benchmark --iterations 1000
```

#### Benchmark Suite
```bash
# Run comprehensive benchmark suite
marco-test benchmark --suite
```

Tests various markdown patterns:
- Simple text
- Headers
- Bold/italic
- Lists
- Code blocks
- Links
- Complex documents

#### Cache Performance
```bash
# Test parser cache performance
echo "# Test" | marco-test benchmark --cache --iterations 100
```

**Output includes:**
- Total duration
- Average per iteration
- Min/Max times
- Throughput (iterations/sec)
- Performance assessment (Excellent/Good/Acceptable/Slow)

### 7. AST Visualization
Visualize AST structure:
```bash
# Show full AST
marco-test visualize "# Hello **world**"

# Filter by rule
marco-test visualize "# Test" --rule heading

# Limit depth
marco-test visualize "# Complex\n\n- List\n  - Nested" --depth 2
```

### 8. Interactive Mode
Manage test baselines interactively:
```bash
marco-test interactive
```

Features:
- Create new test baselines
- Update existing tests
- Review differences
- Accept/reject changes

### 9. Statistics
Show test specification statistics:
```bash
# All specification files
marco-test stats

# Specific file
marco-test stats tests/spec/commonmark.json
```

Shows:
- Total test count
- Tests per section
- Example number ranges

## Global Options

Available for all commands:

```bash
--verbose              # Enable detailed output
--no-colors            # Disable colored output
--normalize-whitespace # Normalize whitespace in comparisons (default: true)
--context-lines N      # Number of diff context lines (default: 3)
```

Examples:
```bash
marco-test --verbose spec --file tests/spec/commonmark.json
marco-test --no-colors string "# Test"
marco-test --context-lines 5 file input.md --expected expected.html
```

## Integration with Cargo Test

The comprehensive integration test suite can be run with cargo:

```bash
# Run all 48 integration tests
cargo test --package marco --test integration_test_suite

# Run specific test module
cargo test --package marco --test integration_test_suite parser_tests

# Run specific test
cargo test --package marco --test integration_test_suite test_atx_heading_level1

# Verbose output
cargo test --package marco --test integration_test_suite -- --nocapture
```

Test modules:
- `parser_tests` - Grammar and parser validation (13 tests)
- `document_tests` - Document-level parsing (4 tests)
- `api_tests` - Public API functions (8 tests)
- `cache_tests` - Parser caching (2 tests)
- `edge_cases` - Boundary conditions (7 tests)
- `smoke_tests` - Basic functionality (13 tests)
- `comprehensive_test` - Full integration (1 test)

## Common Workflows

### Debugging a Parsing Issue
```bash
# 1. Run the parser debug to see what's happening
marco-test debug pipeline "problematic markdown"

# 2. Check the AST structure
marco-test debug ast "problematic markdown"

# 3. Test against spec if applicable
marco-test spec --section "relevant section"
```

### Performance Analysis
```bash
# 1. Run quick benchmark
echo "test content" | marco-test benchmark --iterations 100

# 2. Compare cache performance
echo "test content" | marco-test benchmark --cache --iterations 500

# 3. Run full suite for comprehensive analysis
marco-test benchmark --suite
```

### CSS Development
```bash
# 1. Check for issues
marco-test css

# 2. List all selectors
marco-test css --selectors

# 3. Inspect specific problem area
marco-test css --range 420:430
```

### Compliance Testing
```bash
# 1. Run all CommonMark tests
marco-test spec --file tests/spec/commonmark.json

# 2. Check specific section
marco-test spec --file tests/spec/commonmark.json --section "Links"

# 3. Debug specific failing example
marco-test spec --example 123 --verbose
```

## Exit Codes

- `0` - Success
- `1` - Test failures or errors
- Other - System/runtime errors

## Tips

1. **Use verbose mode** when debugging: `--verbose`
2. **Disable colors** for log files: `--no-colors`
3. **Fail fast** during development: `--fail-fast`
4. **Benchmark before/after** changes to measure performance impact
5. **Run integration tests** before committing: `cargo test --test integration_test_suite`

## Examples

### Complete Testing Workflow
```bash
# 1. Run unit tests
cargo test --package marco --test integration_test_suite

# 2. Run spec tests
cargo run --bin marco-test --features integration-tests -- spec

# 3. Benchmark performance
cargo run --bin marco-test --features integration-tests -- benchmark --suite

# 4. Debug any issues
cargo run --bin marco-test --features integration-tests -- debug pipeline "problem case"
```

### Pre-Commit Checklist
```bash
# Quick validation (< 1 second)
cargo test --package marco --test integration_test_suite

# Full validation (if needed)
cargo run --bin marco-test --features integration-tests -- spec
cargo run --bin marco-test --features integration-tests -- css
```

## See Also

- `tests/integration_test_suite.rs` - Main integration test file (48 tests)
- `tests/spec/commonmark.json` - CommonMark 0.31.2 specification (652 tests)
- `tests/spec/marco.json` - Marco-specific extensions
- `documentation/TEST_FILES_EXPLAINED.md` - Detailed test infrastructure explanation

## Architecture Notes

The test runner is organized into modules:

- `cli.rs` - Command-line interface and argument parsing
- `spec.rs` - Specification file loading and management
- `runner.rs` - Core test execution logic
- `diff.rs` - Diff generation and formatting
- `interactive.rs` - Interactive baseline management
- `css_debug.rs` - CSS debugging utilities
- `parser_debug.rs` - Parser debugging tools
- `benchmark.rs` - Performance benchmarking

All modules use the new two-stage parser API:
- `parse_markdown()` - Parse markdown to AST
- `render_to_html()` - Render AST to HTML
- `parse_and_render()` - One-step convenience function
