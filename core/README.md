# Marco Core - nom-based Markdown Engine

## Architecture

```
Markdown Input
    ↓
Grammar (nom parsers)
    ↓
Parser (2-stage: block → inline)
    ↓
AST (central representation)
    ↓
    ├→ Renderer (HTML output)
    └→ LSP (highlights, completion, hover)
```

## Module Structure

### `grammar/`
- `block.rs` - Block-level parsers (headings, paragraphs, lists, code blocks)
- `inline.rs` - Inline-level parsers (emphasis, links, images, code spans)

### `parser/`
- `block_parser.rs` - Stage 1: Parse document into blocks
- `inline_parser.rs` - Stage 2: Parse inline elements within blocks
- `position.rs` - Position tracking for LSP integration

### `ast/`
- `nodes.rs` - AST node definitions with position info
- `traversal.rs` - DFS/BFS traversal utilities

### `render/`
- `html.rs` - HTML renderer with syntax highlighting
- `options.rs` - Rendering configuration

### `lsp/`
- `highlights.rs` - Syntax highlighting ranges
- `completion.rs` - Autocomplete suggestions
- `hover.rs` - Hover information
- `diagnostics.rs` - Parse errors and warnings

## Test Suite (`tests/test_suite/`)

- `grammar_tests.rs` - Grammar parser validation
- `parser_tests.rs` - Two-stage parser tests
- `ast_tests.rs` - AST structure and traversal
- `render_tests.rs` - HTML output validation
- `lsp_tests.rs` - LSP feature tests
- `commonmark_tests.rs` - CommonMark spec compliance
- `integration_tests.rs` - End-to-end pipeline tests

## Usage

```rust
use core::{parse, render, RenderOptions};

// Parse markdown
let doc = parse("# Hello\n\nWorld!")?;

// Render to HTML
let html = render(&doc, &RenderOptions::default())?;

// LSP integration
use core::lsp::{LspProvider, compute_highlights};
let mut lsp = LspProvider::new();
lsp.update_document(doc);
let highlights = compute_highlights(&doc);
```

## Running Tests

```bash
cargo test --package core              # Core tests
cargo test --test test_suite           # Full test suite
cargo test commonmark                  # CommonMark compliance
```
