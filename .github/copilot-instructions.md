# Marco Copilot Instructions

Marco is a GTK4-based Rust markdown editor with custom syntax extensions and a pest-based parser. This guide helps AI agents understand the project's architecture and workflows.

## Architecture Overview

Marco follows a clear 3-layer architecture:

### Core Components (`src/components/`)
- **`marco_engine/`** - The heart of the project: pest-based parser, AST builder, and HTML renderer
- **`editor/`** - GTK4 editor UI with SourceView integration  
- **`viewer/`** - WebKit-based preview rendering

### Marco Engine (Essential Understanding)
The `marco_engine` provides a simplified 3-function API:
```rust
// Core workflow: parse → build_ast → render_html
let pairs = parse_text(input)?;          // Pest parsing
let ast = build_ast(pairs)?;             // AST construction  
let html = render_html(&ast, options);   // HTML output
```

Key files:
- `marco_grammar.pest` - Custom markdown grammar with Marco extensions
- `ast_builder.rs` - Converts pest pairs to AST nodes
- `render_html.rs` - Outputs HTML from AST

### Project Structure Patterns
- `src/main.rs` serves **only** as application gateway - UI logic lives in components
- `src/lib.rs` re-exports public API for external tools and tests

## Development Workflows

### Grammar Development
The project includes VS Code tasks for pest grammar work:
- "Debug Pest Grammar (Interactive)" - launches `pest_debugger`
- "Format Pest Grammar" - runs `pestfmt` on grammar files

Key insight: Grammar issues are tracked in `src/bin/doc/PARSER_ISSUES.md` with resolution status.

### Build System
- `build.rs` automatically copies assets from `src/assets/` to `target/*/marco_assets/`
- Font loading uses absolute paths via `logic::paths` helpers
- Cross-platform support handled in `logic::crossplatforms`

## Marco-Specific Patterns

### Custom Markdown Extensions
Marco supports unique syntax beyond CommonMark:
- `@run(bash: command)` - Executable code snippets with sandbox options
- `[toc=2](@Page)` - Multi-page table of contents with depth control
- `[Page=A4]` - Document page splitting and navigation
- `[@Page](file.md)` - Cross-document navigation arrows
- Admonition blocks with custom icons

### Error Handling & Logging
- Panic hook installed early in `main.rs` with logger flush on crash
- File-based logging via `logic::logger::SimpleFileLogger`
- Parser errors return `Result<T, String>` (not custom error types)

### Code Organization Rules
1. **No logic in `main.rs`** - only application setup and UI creation
2. **Component isolation** - each component directory is self-contained
3. **Asset management** - fonts, themes, icons loaded via `logic::paths`
4. **Library API** - `lib.rs` exposes clean API for external tools

## Key Integration Points

### GTK4 + WebKit Integration
- Editor uses `sourceview5` for syntax highlighting
- Preview uses `webkit6` for HTML rendering
- Theme synchronization between editor and preview handled in `theme.rs`

### Cross-Component Communication
- `DocumentBuffer` in `logic::buffer` manages file state
- Footer updates wired through `editor::footer_updates`
- View mode switching handled in `viewer::viewmode`

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
- **Integration tests** in `tests/` directory use lib.rs exports
- **Parser testing** via bin tools with live content (`src/bin/test_*`)
- **Manual testing** preferred over unit tests for UI components
- **Grammar testing** - always run `test_current_parser` and check `PARSER_ISSUES.md`

### Testing Guidelines:
1. **Smoke tests first** - Every new module should include smoke tests
2. **Test the public API** - Focus on interfaces other components use
3. **Avoid over-mocking** - Use real objects when possible
4. **Document test intent** - Clear comments explaining what is being verified
5. **Fast feedback** - Tests should complete quickly for development workflow