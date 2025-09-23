# Marco Test Suite

Automated testing for Marco markdown editor.

## Structure

- **`test_runner/`** - Core test suite
- **`parser_debug/`** - Parser debugging CLI
- **`spec/`** - JSON test specifications
- **`integration_test_suite.rs`** - Integration tests

## Quick Start

```bash
# Test strings
cargo run --bin marco-test -- string "# Hello **World**"

# Multi-line input
echo "Header
======" | cargo run --bin marco-test -- string

# Run specs
cargo run --bin marco-test -- spec --file tests/spec/commonmark.json

# Debug parser issues
echo "Header
======" | ./target/debug/marco-parser-debug pipeline
```

## Parser Debugging

The `marco-parser-debug` tool helps debug parsing issues:

```bash
# Build tool
cargo build --bin marco-parser-debug

# Debug setext headers
echo "" | ./target/debug/marco-parser-debug setext

# Test specific grammar rules
echo "# Header" | ./target/debug/marco-parser-debug grammar --rule heading

# Debug AST building
echo "Header
======" | ./target/debug/marco-parser-debug ast

# Full pipeline debug
echo "Header
======" | ./target/debug/marco-parser-debug pipeline
```

### Commands

- `grammar --rule RULE` - Test grammar rules
- `ast` - Debug AST building
- `pipeline` - Full grammar → AST → HTML debug
- `setext` - Setext header specialist

### Debugging Workflow

1. Find issue: `marco-test string "problem" --expected "output"`
2. Debug: `echo "problem" | marco-parser-debug pipeline`
3. Fix parser code
4. Validate: Run both tools again

## Test Specs

JSON format:
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

## Tips

```bash
# Fail fast
cargo run --bin marco-test -- spec --file tests/spec/commonmark.json --fail-fast

# Specific sections
cargo run --bin marco-test -- spec --section "Headers"

# Multi-line with heredoc
cargo run --bin marco-test -- string << 'EOF'
Complex markdown
================
EOF
```