# Marco Grammar Workbench

A comprehensive testing and validation platform for the Marco markdown engine. This tool provides real-time grammar testing, performance benchmarking, and parse tree visualization using the actual Marco engine parser.

## Overview

The Grammar Workbench is designed to validate Marco's parsing capabilities across:
- **1,237 comprehensive test cases** covering all grammar rules
- **Real-time individual rule testing** with parse tree visualization  
- **Performance benchmarking** with microsecond precision
- **Marco-specific extensions** (admonitions, tabs, user mentions, etc.)
- **CommonMark compliance testing** with edge cases

## Quick Start

```bash
cd tests/grammar_workbench

# Run all test cases (1,237 tests)
cargo run

# Test individual grammar rules
cargo run -- text "hello world"
cargo run -- admonition_block $':::note\nContent\n:::'
cargo run -- user_mention "@user [github](John Doe)"

# Performance benchmarks
cargo run -- --benchmark

# Parse tree visualization
cargo run -- --tree bold "**text**"
```

## Features

### 🧪 **Comprehensive Testing**
- **91.8% test pass rate** (1,136/1,237 tests passing)
- **Automated test execution** from `test_cases.toml`
- **HTML report generation** with detailed results
- **Expected vs unexpected failure tracking**

### ⚡ **Performance Analysis**
- **Microsecond-level timing** for all parse operations
- **Memory usage estimation** and optimization insights
- **Benchmark categories**: Simple text, complex formatting, pathological inputs
- **Performance regression detection**

### 🌳 **Parse Tree Visualization**
- **Enhanced ASCII tree display** showing rule hierarchy
- **Real-time grammar debugging** with immediate feedback
- **Individual rule isolation** for focused testing
- **Error location pinpointing** with context

### 🎯 **Marco Extensions Support**
- **Admonition blocks**: `:::note`, `:::warning`, `:::tip`, etc.
- **User mentions**: `@username [platform](Display Name)`
- **Tabs functionality**: Multi-tab content organization
- **Bookmarks**: `[bookmark:label](path=line)`
- **Custom Marco syntax** validation

## Usage Reference

### Command Line Interface

```bash
# Individual rule testing
cargo run -- <rule> <input>

# Parse tree visualization  
cargo run -- --tree <rule> <input>

# Performance benchmarks
cargo run -- --benchmark

# Grammar structure analysis
cargo run -- --grammar

# Detailed analysis reports
cargo run -- --analyze

# Multiline paragraph testing
cargo run -- --multiline

# Full test suite execution
cargo run
```

### Examples

```bash
# Test basic text parsing
cargo run -- text "Hello, world!"

# Visualize emphasis parsing
cargo run -- --tree emphasis "**bold** and *italic*"

# Test Marco admonitions
cargo run -- admonition_block $':::warning\nImportant note\n:::'

# Benchmark performance
cargo run -- --benchmark

# Test user mentions
cargo run -- user_mention "@johndoe [github](John Doe)"

# Show complete grammar structure
cargo run -- --grammar
```

## Test Categories

### Core Grammar (CommonMark)
- **Text & Words**: Basic text, punctuation, unicode
- **Headings**: H1-H6 with ATX and Setext styles
- **Emphasis**: Bold, italic, strikethrough combinations
- **Code**: Inline and block code with syntax highlighting
- **Links & Images**: URLs, references, inline links
- **Lists**: Ordered, unordered, task lists, nested structures
- **Tables**: Simple and complex table structures
- **Math**: Inline and block mathematical expressions

### Marco Extensions  
- **Admonitions**: `:::note`, `:::warning`, `:::tip`, `:::danger`, `:::info`
- **Tabs**: Multi-tab content blocks with syntax highlighting
- **User Mentions**: Platform-specific user references
- **Bookmarks**: File and line-specific bookmarks
- **Run Blocks**: Executable code with language specification
- **Document Tags**: Page formatting and metadata

### Performance Tests
- **Benchmark Tests**: Standard parsing scenarios
- **Memory Stress**: Large document handling
- **Pathological Inputs**: Edge cases that could cause performance issues
- **Backtracking Tests**: Complex parsing scenarios

## Generated Reports

### HTML Test Results (`src/results/test_results.html`)
- **Visual test matrix** with pass/fail indicators
- **Parse tree visualization** for each test case
- **Error messages** with context for failures
- **Test categorization** and filtering options

### Benchmark Report (`src/results/benchmark_results.md`)
- **Performance timing** analysis by category
- **Memory usage** estimates and trends
- **Throughput calculations** (MB/s processing)
- **Slowest/fastest test identification**

## Performance Metrics

Current performance benchmarks:
- **Simple text**: ~19μs average
- **Complex formatting**: ~3.7μs average
- **Academic papers**: ~1.16ms average
- **GitHub READMEs**: ~745μs average
- **Large tables**: ~1.6ms average

## Development Workflow

### Testing New Grammar Rules
```bash
# 1. Add test cases to test_cases.toml
# 2. Run focused tests
cargo run -- your_new_rule "test input"

# 3. Visualize parsing
cargo run -- --tree your_new_rule "test input"

# 4. Run full test suite
cargo run

# 5. Check performance impact
cargo run -- --benchmark
```

### Debugging Parse Failures
```bash
# Test specific failing rule
cargo run -- problematic_rule "failing input"

# Get detailed parse tree
cargo run -- --tree problematic_rule "failing input"

# Check error context in HTML report
open src/results/test_results.html
```

## Project Structure

```
tests/grammar_workbench/
├── Cargo.toml              # Project dependencies
├── README.md               # This file
├── test_cases.toml         # 1,237 comprehensive test cases
├── src/
│   ├── main.rs            # Main testing application
│   ├── grammar_visualizer.rs # Parse tree visualization
│   └── results/           # Generated reports
│       ├── test_results.html
│       └── benchmark_results.md
└── target/                # Compiled binaries
```

## Contributing

When adding new test cases to `test_cases.toml`:

1. **Group by category** (e.g., `[admonitions]`, `[code_blocks]`)
2. **Include edge cases** and known problematic inputs
3. **Test both success and expected failure cases**
4. **Add performance-sensitive cases** to benchmark sections
5. **Document expected behavior** in comments

## Integration

The Grammar Workbench integrates directly with the main Marco engine:
- Uses `marco::components::marco_engine::MarcoParser`
- Leverages real AST building with `build_ast()`
- Tests actual HTML rendering with `render_html()`
- Validates production parsing pipeline end-to-end

This ensures that testing results accurately reflect real-world Marco engine behavior and performance characteristics.