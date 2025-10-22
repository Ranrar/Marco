# Marco Copilot Instructions

Marco is a GTK4-based Rust markdown editor with nom-based parser. This guide helps AI agents understand the project's architecture and workflows.

## Communication Style

When completing work, **DO NOT create markdown documentation files**. Instead:
- Write summaries directly in chat responses
- Use simple tables for data
- Keep text blocks small and focused
- Be concise and to-the-point

## Problem-Solving Approach

When facing an issue or problem:
1. **Review existing code** - Check how similar issues are handled elsewhere in the codebase
2. **Search online** - Use web search to find solutions, best practices, and documentation
3. **Analyze the problem** - Break down complex issues into smaller, manageable parts
4. **Test solutions** - Verify fixes work before considering the task complete

## Development Workflow

### Rust Toolchain
Marco uses **Rust 1.90.0** (stable, released September 2025) with the following components:
- **rustfmt** - Code formatting (`cargo fmt`)
- **clippy** - Linting and code quality (`cargo clippy`)
- **rust-src** - Source code for standard library (required for rust-analyzer)
- **rust-docs** - Standard library documentation (`rustup doc --std`)
- **llvm-tools** - LLVM utilities for profiling and code coverage

**Toolchain file**: `rust-toolchain.toml` pins the version across all machines

**Development commands**:
```bash
cargo fmt                    # Format code
cargo clippy                 # Run linter
cargo test --workspace       # Run all tests (546 total)
cargo doc --workspace --open # Generate & view project docs
cargo llvm-cov --html --open # Generate code coverage report
rustup doc                   # View Rust standard library docs
```

**Code coverage**: Use `cargo llvm-cov` to analyze test coverage. Current coverage: **51.75% line coverage** (grammar/parser/LSP well-covered, UI code at 0% which is normal for GTK apps).

### Using Logs for Testing
Marco uses file-based logging as part of the development workflow:
- **Run the application**: `cargo run -p marco` or `cargo run -p polo`
- **Check the log**: Open `log/YYYYMM/YYMMDD.log` (e.g., `log/202510/251007.log`)
- **Verify behavior**: Look for errors, warnings, or debug messages
- **Part of testing**: Reading logs is essential before marking work complete

## Architecture Overview

Marco uses a **Cargo workspace** with three crates:

### Workspace Structure
- **`core/`** - Pure Rust library: nom-based parser, AST, HTML renderer, and core logic (buffer management, settings, paths, cache, logging). No GTK dependencies.
- **`marco/`** - Full-featured editor binary: GTK4 UI, SourceView5 text editing, WebKit6 preview. Depends on `core`.
- **`polo/`** - Lightweight viewer binary: GTK4 UI, WebKit6 preview only (no SourceView5). Depends on `core`.
- **`assets/`** - Centralized at workspace root: themes, fonts, icons, settings.

### Core Components

#### core Library (`core/src/`)
- **`grammar/`** - nom-based grammar parsers for block and inline Markdown elements
- **`parser/`** - AST building from grammar output (includes `ast.rs`, `block_parser.rs`, `inline_parser.rs`, `position.rs`)
- **`render/`** - HTML renderer with entity escaping and syntax highlighting support
- **`lsp/`** - LSP features: syntax highlighting, diagnostics, completion, hover
- **`logic/`** - Pure Rust business logic: buffer management, settings, paths, cache, logging

#### marco Binary (`marco/src/`)
- **`components/editor/`** - GTK4 editor UI with SourceView5 integration  
- **`components/viewer/`** - WebKit6-based preview rendering
- **`components/language/`** - Localization support
- **`logic/`** - UI-specific logic: GTK signal management, menu handlers
- **`ui/`** - GTK widgets and split view layout
- **`ui/css/`** - Programmatic CSS generation system

#### polo Binary (`polo/src/`)
- Viewer-only application (implementation pending)

### Parser Architecture (nom-based)
The core parser uses **nom combinators** for Markdown parsing:
```rust
// Core workflow: grammar → parser → AST → renderer
let document = parser::parse(input)?;           // Parse to AST
let html = render::render(&document, options)?; // Render HTML
```

Key modules in `core/src/`:
- `grammar/{block,inline}.rs` - nom-based grammar parsers (headings, code blocks, emphasis, links, etc.)
- `parser/{block_parser,inline_parser}.rs` - AST builders calling grammar functions
- `parser/ast.rs` - Document, Node, NodeKind definitions
- `render/html.rs` - HTML output with entity escaping

### LSP Architecture
The core library provides **Language Server Protocol features** for editor integration:

**Key LSP modules** (`core/src/lsp/`):
- `highlights.rs` - Syntax highlighting tags (11 types: Heading1-6, Emphasis, Strong, Link, CodeSpan, CodeBlock)
- `diagnostics.rs` - Parse validation (4 severity levels: Error, Warning, Info, Hint)
- `completion.rs` - Context-aware suggestions (headings, code blocks, links, emphasis, strong)
- `hover.rs` - Hover information (stub for future implementation)
- `mod.rs` - LspProvider coordinator

**Usage example**:
```rust
use core::lsp::{compute_highlights, compute_diagnostics, get_completions};

let highlights = compute_highlights(&document);  // Returns Vec<Highlight>
let diagnostics = compute_diagnostics(&document); // Returns Vec<Diagnostic>
let completions = get_completions(position, context); // Returns Vec<CompletionItem>
```

### Project Structure Patterns
- `marco/src/main.rs` serves **only** as application gateway - UI logic lives in components
- `core/src/lib.rs` re-exports public API for external tools and tests
- **Import convention**: Use `core::` for core functionality, `crate::` for local modules

## Development Workflows

### Build System
- **Workspace root**: `Cargo.toml` defines workspace members and shared dependencies
- **Core build**: `core/build.rs` copies assets from workspace `assets/` to `target/*/marco_assets/`
- Font loading uses absolute paths via `logic::paths` helpers
- Cross-platform support handled in `logic::crossplatforms`

Build commands:
```bash
cargo build -p core     # Core library only
cargo build -p marco    # Full editor
cargo build -p polo     # Viewer only
cargo build --workspace # All crates
```

### Error Handling & Logging
- Panic hook installed early in `marco/src/main.rs` with logger flush on crash
- File-based logging via `core::logic::logger::SimpleFileLogger`
- Parser errors return `Result<T, anyhow::Error>`

### Code Organization Rules
1. **No logic in `marco/src/main.rs`** - only application setup and UI creation
2. **Component isolation** - each component directory is self-contained
3. **Core vs UI separation** - Pure Rust logic in `core`, GTK-dependent code in `marco`
4. **Asset management** - fonts, themes, icons loaded via `logic::paths` from workspace `assets/`
5. **Library API** - `core/src/lib.rs` exposes clean API for external tools and polo binary
6. **Import patterns**: 
   - Use `core::logic::buffer::DocumentBuffer` from marco binary
   - Use `crate::components::editor::...` for local marco modules
   - Never use absolute paths like `marco::...` from within marco binary

## Key Integration Points

### GTK4 + WebKit Integration
- Editor uses `sourceview5` for syntax highlighting
- Preview uses `webkit6` for HTML rendering
- Theme synchronization between editor and preview handled in `theme.rs`

### GTK CSS System
Marco uses **programmatic CSS generation** in Rust, applied via GTK's `CssProvider`.

**Structure** (`marco/src/ui/css/`): `mod.rs` (loader), `constants.rs` (colors/spacing), `menu.rs`, `toolbar.rs`, `footer.rs`

**Usage**: `crate::ui::css::load_css();` in `main.rs` - single call generates and applies all CSS

**Global Application**: CSS is applied to the entire GTK display (window-level), not individual widgets. Uses `gtk4::style_context_add_provider_for_display()` with `PRIORITY_APPLICATION`, so all widgets automatically inherit styles via CSS class selectors (`.titlebar`, `.toolbar-button`, etc.)

**Adding Styles**: Edit color in `constants.rs` → update generator function in menu/toolbar/footer module → run `cargo test -p marco --lib ui::css`

**GTK Limitations**: Avoid `:empty` pseudo-class (not supported), use explicit classes instead

### Cross-Component Communication
- `DocumentBuffer` in `core::logic::buffer` manages file state
- Footer updates wired through `marco/src/components/editor/footer_updates.rs`
- View mode switching handled in `marco/src/components/viewer/viewmode.rs`
- Theme synchronization between editor and preview in `marco/src/theme.rs`

## Testing Approach

### Primary Testing Strategy: Smoke Tests
Marco prioritizes **smoke tests** as the primary testing methodology. Smoke tests verify core functionality works correctly without extensive mocking or complex setup.

#### Smoke Test Principles:
- **Fast execution** - Complete in milliseconds, suitable for frequent runs
- **Core functionality focus** - Test the happy path and essential features
- **Real integration** - Use actual components together, not mocked dependencies
- **Clear assertions** - Verify observable behavior and expected outputs
- **Self-contained** - Each test includes its own data and cleanup

#### Smoke Test Examples:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_parser_cache() {
        let cache = SimpleParserCache::new();
        let content = "# Hello World\n\nThis is a **test** document.";
        
        // Test AST caching - first call should be cache miss
        let ast1 = cache.parse_with_cache(content).expect("Parse failed");
        let stats = cache.stats();
        assert_eq!(stats.ast_misses, 1);
        assert_eq!(stats.ast_hits, 0);
        
        // Second call should be cache hit
        let ast2 = cache.parse_with_cache(content).expect("Parse failed");
        let stats = cache.stats();
        assert_eq!(stats.ast_hits, 1);
        
        // Verify functionality works
        assert!(format!("{:?}", ast1).contains("Hello World"));
    }
}
```

#### When to Add Smoke Tests:
- **New components or modules** - Add smoke tests immediately after implementation
- **Core functionality changes** - Update existing smoke tests to reflect new behavior  
- **Bug fixes** - Add smoke test to verify fix and prevent regression
- **Performance optimizations** - Ensure smoke tests still pass after changes
- **Parser features** - Every grammar rule should have smoke tests (see `grammar/inline.rs`, `grammar/block.rs`)
- **LSP features** - Each LSP function needs smoke tests (highlights, diagnostics, completion)
- **Render changes** - HTML output changes require render smoke tests
- **Integration points** - Test where modules interact (parser→AST, AST→renderer, AST→LSP)

### Secondary Testing Approaches:
- **Integration tests** in `tests/test_suite/` directory - modular test suite with CLI interface
- **Test modules**: `grammar_tests.rs`, `parser_tests.rs`, `render_tests.rs`, `commonmark_tests.rs`, `lsp_tests.rs`, `ast_tests.rs`
- **Manual testing** preferred over unit tests for UI components
- **CommonMark compliance** - Test against official spec examples

### Testing Guidelines:
1. **Smoke tests first** - Every new module should include smoke tests
2. **Test the public API** - Focus on interfaces other components use
3. **Avoid over-mocking** - Use real objects when possible
4. **Document test intent** - Clear comments explaining what is being verified
5. **Fast feedback** - Tests should complete quickly for development workflow
6. **Run workspace tests** - Use `cargo test --workspace` to test all crates together
7. **Verify with runtime testing** - Before completing work, run the application (`cargo run -p marco` or `cargo run -p polo`) and check the log file (e.g., `log/202510/251007.log`) to ensure no runtime errors or warnings

### Test Results
```
Core Library Tests:   85/85 passing (100%)
Integration Tests:     2/2 passing (100%)
Total:                87/87 passing (100%)
```

Test suite structure:
- `tests/test_suite.rs` - CLI entry point (145 lines)
- `tests/test_suite/grammar_tests.rs` - Inline + block grammar tests
- `tests/test_suite/parser_tests.rs` - Parser integration tests
- `tests/test_suite/render_tests.rs` - Render + inline pipeline tests
- `tests/test_suite/commonmark_tests.rs` - CommonMark spec tests
- `tests/test_suite/lsp_tests.rs` - LSP feature tests