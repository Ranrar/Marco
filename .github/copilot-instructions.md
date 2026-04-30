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
Marco uses **Rust 1.92.0** (stable) with the following components:
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
cargo test --workspace       # Run all workspace tests
cargo doc --workspace --open # Generate & view project docs
cargo llvm-cov --html --open # Generate code coverage report
rustup doc                   # View Rust standard library docs
```

**Code coverage**: Use `cargo llvm-cov` to analyze test coverage. UI coverage is typically low/0% for GTK apps. Pure-Rust parser/render coverage now lives in the separate [marco-core](https://github.com/Ranrar/marco-core) repository.

### VS Code workspaces (native OS)

This repo intentionally supports **native per-OS development**. Use the workspace file that matches the OS you are on:

- **Linux**: `marco-linux.code-workspace`
- **Windows (MSVC)**: `marco-windows.code-workspace`

Avoid configuring Rust Analyzer to use `x86_64-pc-windows-gnu` on Linux for this project: GTK/Glib sys crates (e.g. `glib-sys`) rely on `pkg-config` and require a full Windows/MinGW sysroot for cross compilation, which will produce noisy diagnostics that are not actionable for most contributions.

### Using Logs for Testing
Marco uses file-based logging as part of the development workflow:
- **Run the application**: `cargo run -p marco` or `cargo run -p polo`
- **Check the log**: Open `log/YYYYMM/YYMMDD.log` (e.g., `log/202510/251007.log`)
- **Verify behavior**: Look for errors, warnings, or debug messages
- **Part of testing**: Reading logs is essential before marking work complete

## Architecture Overview

Marco uses a **Cargo workspace** with three crates and depends on the
externally-published [`marco-core`](https://crates.io/crates/marco-core) crate
(developed in its own repository: https://github.com/Ranrar/marco-core).

### Workspace Structure
- **`marco-shared/`** - Shared app logic: buffer management, settings, paths, loaders, layout state. No GTK dependencies. Also owns the centralized assets and the `build.rs` that copies them into `target/*/marco_assets/`.
- **`marco/`** - Full-featured editor binary: GTK4 UI, SourceView5 text editing, WebKit6 preview. Depends on `marco-core` (crates.io) and `marco-shared`.
- **`polo/`** - Lightweight viewer binary: GTK4 UI, WebKit6 preview only (no SourceView5). Depends on `marco-core` (crates.io) and `marco-shared`.
- **`marco-shared/src/assets/`** - Centralized assets: themes, fonts, icons, settings.

### Core Components

#### marco-core (external crate, separate repo)
The parser, AST, HTML renderer, and intelligence/LSP features live in the
`marco-core` crate published on crates.io. The pinned version is declared in
the workspace `Cargo.toml` under `[workspace.dependencies.marco-core]`.

Key modules (in the `marco-core` repo, accessible via the published crate):
- **`grammar/`** - nom-based grammar parsers for block and inline Markdown elements
- **`parser/`** - AST building from grammar output
- **`render/`** - HTML renderer with entity escaping and syntax highlighting support
- **`intelligence/`** (formerly `lsp/`) - syntax highlighting, diagnostics, completion, hover
- **`logic/`** - Pure Rust business logic: cache, logging

#### marco-shared Library (`marco-shared/src/`)
- **`logic/`** - Shared app logic: buffer management, settings, layout state, loaders
- **`paths/`** - Cross-platform path resolution for assets and config

#### marco Binary (`marco/src/`)
- **`components/editor/`** - GTK4 editor UI with SourceView5 integration  
- **`components/viewer/`** - WebKit6-based preview rendering
- **`components/language/`** - Localization support
- **`logic/`** - UI-specific logic: GTK signal management, menu handlers
- **`ui/`** - GTK widgets and split view layout
- **`ui/css/`** - Programmatic CSS generation system

#### polo Binary (`polo/src/`)
- Viewer-focused application (read-only viewer companion)

### Parser Architecture (nom-based)
The parser is provided by the external `marco-core` crate:
```rust
// Core workflow: grammar → parser → AST → renderer
let document = marco_core::parser::parse(input)?;           // Parse to AST
let html = marco_core::render::render(&document, options)?; // Render HTML
```

### LSP / Intelligence Architecture
The `marco-core` crate provides Language Server Protocol features for editor
integration (syntax highlights, diagnostics, completion, hover) under
`marco_core::intelligence`.

```rust
use marco_core::intelligence::{compute_highlights, compute_diagnostics, get_completions};
```

### Project Structure Patterns
- `marco/src/main.rs` serves **only** as application gateway - UI logic lives in components
- **Import convention**: Use `marco_core::` for parser/render/intelligence (external crate), `marco_shared::` for shared app logic, `crate::` for local modules

## Development Workflows

### Build System
- **Workspace root**: `Cargo.toml` defines workspace members and shared dependencies
- **Asset build**: `marco-shared/build.rs` copies assets from `marco-shared/src/assets/` to `target/*/marco_assets/`
- Font loading uses absolute paths via `marco_shared::paths` helpers
- Cross-platform support is primarily handled via `marco_core::paths::platform` (OS-specific path resolution) and platform-gated UI/webview code.

**Cross-platform cfg annotations**: When adding platform-specific code or dependencies, annotate modules or items with `#[cfg(target_os = "linux")]` or `#[cfg(target_os = "windows")]` to make behavior explicit and to keep builds clean. Prefer conditional dependencies in `Cargo.toml` for platform-only crates:

> **Info:** Platform-specific conditional dependencies for `Cargo.toml`
>
> ```txt
> [target.'cfg(target_os = "linux")'.dependencies]
> # Linux: webkit6 (GTK4-native WebKit)
> webkit6
> [target.'cfg(target_os = "windows")'.dependencies]
> # Windows: wry (Chromium-based native Windows webview, e.g., Edge)
> wry
> tao
> gdk4-win32
> raw-window-handle
> urlencoding
> ```

#### Conditional Imports

When writing cross-platform Rust, apply conditional compilation to the **imports themselves** to avoid unused-import warnings on one OS and missing-import errors on the other.

Use **only** these forms for OS gating in this repo:
- `#[cfg(target_os = "windows")]`
- `#[cfg(target_os = "linux")]`

Do **not** use `cfg(any(...))` or `cfg(not(...))` for these platform import cases.

Example:

```rust
#[cfg(target_os = "windows")]
use gtk4::Window;

#[cfg(target_os = "linux")]
use gtk4::{Window, Label};
```

You can also alias imports to use a common name across platforms:

```rust
#[cfg(target_os = "windows")]
use gtk4::Window as GtkWindow;

#[cfg(target_os = "linux")]
use gtk4::{Window as GtkWindow, Label};
```

This keeps cross-platform code clean and compiler-friendly.

Build commands:
```bash
cargo build -p marco-shared # Shared library only
cargo build -p marco    # Full editor
cargo build -p polo     # Viewer only
cargo build --workspace # All workspace crates (marco-shared, marco, polo)
```

### Versioning, Changelogs, and Packaging

#### Version tracking (single source of truth)
- **Do not** hand-edit crate versions in multiple places.
- Use `build/version.json` as the single version source for the workspace crates (`marco-shared`, `marco`, `polo`).
- `build/linux/build_deb.sh` is responsible for (optionally) bumping versions and syncing them into:
    - `marco-shared/Cargo.toml`
    - `marco/Cargo.toml`
    - `polo/Cargo.toml`
- The `marco-core` dependency version is pinned in the workspace `Cargo.toml`
  under `[workspace.dependencies.marco-core]` and is bumped manually when
  upgrading to a newer published version.

#### Version scheme: library vs apps
`marco-core` is published to crates.io from its own repository
(https://github.com/Ranrar/marco-core) and follows **independent semver**.
`marco-shared`, `marco`, and `polo` share an **app version track** in this repo:

| Crate | Version track | Rationale |
|---|---|---|
| `marco-core` | `1.x.y` — semver | Published library in separate repo; not bumped here |
| `marco-shared` | `0.x.y` — app versioning | Internal shared lib; versioned with apps |
| `marco` | `0.x.y` — app versioning | Binary release; no API contract |
| `polo` | `0.x.y` — app versioning | Binary release; no API contract |

#### Automated version bump (recommended)
- Prefer using `build/linux/build_deb.sh` to bump versions and sync app `Cargo.toml` files.
- For version changes without building a `.deb`, use:
    - `bash build/linux/build_deb.sh --version-only` (patch bump)
    - `bash build/linux/build_deb.sh --version-only --bump minor|major`
    - `bash build/linux/build_deb.sh --version-only --set X.Y.Z`
- To upgrade the consumed `marco-core` version, edit the workspace `Cargo.toml`
  manually, run `cargo update -p marco-core`, and add a note to `changelog/marco.md` / `changelog/polo.md`.

#### Release workflow (repo practice)
For a real release commit:
1. Update the changelogs (`changelog/marco.md`, `changelog/polo.md`) — only update the ones that changed.
2. Bump versions via `build/linux/build_deb.sh` (recommended: `--version-only`).
   - `--version-only --set X.Y.Z`
3. Run tests (`cargo test --workspace --locked`).
4. Commit the changelog + version changes.
5. Tag the release (for example `vX.Y.Z`) and push.

#### Cargo/SemVer Zero-Padding Policy (Simple)

* **No leading zeros** in major, minor, or patch numbers.

    * ✅ Correct: `1.2.3`, `0.1.0`, `1.0.0-rc.1`
    * ❌ Incorrect: `01.2.3`, `1.02.3`, `1.2.03`

* **Zero is allowed** if it's the only digit (`0`) in major, minor, or patch.

* **Pre-release tags** (like `-rc.1`) and **build metadata** (like `+build.123`) are allowed, but numeric parts must still have no leading zeros.

**Example valid versions:**

```toml
version = "1.2.3"
version = "0.9.1-rc.2"
version = "2.0.0+build.123"
```

#### Changelogs
- Changelogs live in `changelog/`:
    - `changelog/marco.md`
    - `changelog/polo.md`
  (`marco-core` keeps its own changelog in its own repository.)
- Format: **Keep a Changelog** sections (`Added`, `Changed`, `Fixed`, `Removed`, `Security`).
- Entries should be **user-visible** and avoid commit hashes/file names.
- If details are ambiguous, prefer neutral wording and avoid guessing.

#### Debian packaging (Linux)
Debian packaging assets and scripts live in `build/linux/`.
Primary entry point: `build/linux/build_deb.sh`
    - Builds the workspace and produces a `.deb` (marco-suite package) using the versions from `build/version.json`.
    - Supports flags to control version bumping (for example, CI uses a no-bump mode so builds don't mutate versions).

**Package naming policy:** the repo uses a fixed `amd64` suffix for produced package filenames.

#### Windows portable packaging
Windows packaging assets and scripts live in `build/windows/`.
Primary entry point: `build/windows/build_portable.ps1`
    - Builds the workspace with `--target x86_64-pc-windows-gnu` and produces a portable `.zip` using the version from `build/version.json`.
    - Must be run on Windows (not cross-compiled from Linux).
    - Creates a self-contained directory structure with `marco.exe`, `polo.exe`, `assets/`, and empty `config/` + `data/` folders for portable mode.

**Package naming:** 
- `marco-suite_<version>_windows_amd64.zip`

**Portable mode detection:** The Windows build automatically detects it's running in portable mode (writable directory next to the executable) and stores config/data locally instead of `%LOCALAPPDATA%`.

### GitHub Actions / CI workflows
- Workflows live in `.github/workflows/`.
- The Debian release workflow builds the `.deb` and publishes it as a release asset.
- CI should build deterministically:
    - Avoid changing `build/version.json` or `Cargo.toml` during CI runs.
    - Ensure required build tools are installed (notably `python3` is used by the packaging/version script).

Release assets are published per version tag in CI.

### Error Handling & Logging
- Panic hook installed early in `marco/src/main.rs` with logger flush on crash
- File-based logging via `marco_core::logic::logger::SimpleFileLogger`
- Parser errors return `Result<T, Box<dyn std::error::Error>>`

### Code Organization Rules
1. **No logic in `marco/src/main.rs`** - only application setup and UI creation
2. **Component isolation** - each component directory is self-contained
3. **Core vs UI separation** - Pure Rust logic in `marco-core` (external crate) and `marco-shared`, GTK-dependent code in `marco`
4. **Asset management** - fonts, themes, icons loaded via `marco_shared::paths` from `marco-shared/src/assets/`
5. **Library API** - `marco-core` is consumed as a published crate; bump the workspace pin in the root `Cargo.toml` to upgrade
6. **Import patterns**: 
   - Use `marco_core::` for parser/render/LSP functionality
   - Use `marco_shared::` for buffer, paths, settings from marco/polo binaries
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
- **Parser features** - Every grammar rule should have smoke tests (see `grammar/inline.rs`, `grammar/block.rs`)
- **LSP features** - Each LSP function needs smoke tests (highlights, diagnostics, completion)
- **Render changes** - HTML output changes require render smoke tests
- **Integration points** - Test where modules interact (parser→AST, AST→renderer, AST→LSP)

### Secondary Testing Approaches:
- **Parser/renderer test suite** lives in the external [`marco-core`](https://github.com/Ranrar/marco-core) repository (grammar, parser, render, CommonMark, intelligence). Run it from a clone of that repo with `cargo test`.
- **App-level integration tests** can be added under `marco/tests/` or `polo/tests/` if/when needed.
- **Manual testing** preferred over unit tests for UI components.
- **CommonMark compliance** — validated in the `marco-core` repository.

### Testing Guidelines:
1. **Smoke tests first** - Every new module should include smoke tests
2. **Test the public API** - Focus on interfaces other components use
3. **Avoid over-mocking** - Use real objects when possible
4. **Document test intent** - Clear comments explaining what is being verified
5. **Fast feedback** - Tests should complete quickly for development workflow
6. **Run workspace tests** - Use `cargo test --workspace` to test all app crates (`marco-shared`, `marco`, `polo`) together
7. **Verify with runtime testing** - Before completing work, run the application (`cargo run -p marco` or `cargo run -p polo`) and check the log file (e.g., `log/202510/251007.log`) to ensure no runtime errors or warnings
8. **Parser changes** - File parser/render/intelligence work against the [`marco-core`](https://github.com/Ranrar/marco-core) repository; bump the workspace pin in this repo's root `Cargo.toml` once a new version is published.