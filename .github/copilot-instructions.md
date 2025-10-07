# Marco Copilot Instructions

Marco is a GTK4-based Rust markdown editor with custom syntax extensions and a pest-based parser. This guide helps AI agents understand the project's architecture and workflows.

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

### Using Logs for Testing
Marco uses file-based logging as part of the development workflow:
- **Run the application**: `cargo run -p marco` or `cargo run -p polo`
- **Check the log**: Open `log/YYYYMM/YYMMDD.log` (e.g., `log/202510/251007.log`)
- **Verify behavior**: Look for errors, warnings, or debug messages
- **Part of testing**: Reading logs is essential before marking work complete

## Architecture Overview

Marco uses a **Cargo workspace** with three crates:

### Workspace Structure
- **`marco_core/`** - Pure Rust library: pest-based parser, AST builder, HTML renderer, and core logic (buffer management, settings, paths). No GTK dependencies.
- **`marco/`** - Full-featured editor binary: GTK4 UI, SourceView5 text editing, WebKit6 preview. Depends on `marco_core`.
- **`polo/`** - Lightweight viewer binary: GTK4 UI, WebKit6 preview only (no SourceView5). Depends on `marco_core`.
- **`assets/`** - Centralized at workspace root: themes, fonts, icons, settings.

### Core Components

#### marco_core Library (`marco_core/src/`)
- **`components/marco_engine/`** - The heart of the project: pest-based parser, AST builder, and HTML renderer
- **`components/syntax_highlighter/`** - Syntect-based code highlighting for preview
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

### Marco Engine (Essential Understanding)
The `marco_engine` (in marco_core) provides a simplified 3-function API:
```rust
// Core workflow: parse → build_ast → render_html
let pairs = parse_text(input)?;          // Pest parsing
let ast = build_ast(pairs)?;             // AST construction  
let html = render_html(&ast, options);   // HTML output
```

Key files in `marco_core/src/components/marco_engine/`:
- `marco_grammar.pest` - Custom markdown grammar with Marco extensions
- `ast_builder.rs` - Converts pest pairs to AST nodes
- `render_html.rs` - Outputs HTML from AST

### Project Structure Patterns
- `marco/src/main.rs` serves **only** as application gateway - UI logic lives in components
- `marco_core/src/lib.rs` re-exports public API for external tools and tests
- **Import convention**: Use `marco_core::` for core functionality, `crate::` for local modules

## Development Workflows

### Grammar Development
The project includes VS Code tasks for pest grammar work:
- "Debug Pest Grammar (Interactive)" - launches `pest_debugger`
- "Format Pest Grammar" - runs `pestfmt` on grammar files

### Build System
- **Workspace root**: `Cargo.toml` defines workspace members and shared dependencies
- **Core build**: `marco_core/build.rs` copies assets from workspace `assets/` to `target/*/marco_assets/`
- Font loading uses absolute paths via `logic::paths` helpers
- Cross-platform support handled in `logic::crossplatforms`

Build commands:
```bash
cargo build -p marco_core  # Core library only
cargo build -p marco       # Full editor
cargo build -p polo        # Viewer only
cargo build --workspace    # All crates
```

### Error Handling & Logging
- Panic hook installed early in `marco/src/main.rs` with logger flush on crash
- File-based logging via `marco_core::logic::logger::SimpleFileLogger`
- Parser errors return `Result<T, String>` (not custom error types)

### Code Organization Rules
1. **No logic in `marco/src/main.rs`** - only application setup and UI creation
2. **Component isolation** - each component directory is self-contained
3. **Core vs UI separation** - Pure Rust logic in `marco_core`, GTK-dependent code in `marco`
4. **Asset management** - fonts, themes, icons loaded via `logic::paths` from workspace `assets/`
5. **Library API** - `marco_core/src/lib.rs` exposes clean API for external tools and polo binary
6. **Import patterns**: 
   - Use `marco_core::logic::buffer::DocumentBuffer` from marco binary
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
- `DocumentBuffer` in `marco_core::logic::buffer` manages file state
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

### Secondary Testing Approaches:
- **Integration tests** in `tests/` directory use marco_core lib.rs exports
- **Parser testing** via bin tools with live content (`tests/parser_debug/`)
- **Manual testing** preferred over unit tests for UI components
- **Grammar testing** - always run `test_current_parser` and check `PARSER_ISSUES.md`

### Testing Guidelines:
1. **Smoke tests first** - Every new module should include smoke tests
2. **Test the public API** - Focus on interfaces other components use
3. **Avoid over-mocking** - Use real objects when possible
4. **Document test intent** - Clear comments explaining what is being verified
5. **Fast feedback** - Tests should complete quickly for development workflow
6. **Run workspace tests** - Use `cargo test --workspace` to test all crates together
7. **Verify with runtime testing** - Before completing work, run the application (`cargo run -p marco` or `cargo run -p polo`) and check the log file (e.g., `log/202510/251007.log`) to ensure no runtime errors or warnings