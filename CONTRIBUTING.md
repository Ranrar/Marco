# Contributing to Marco

Thank you for your interest in contributing to Marco. This document explains how the project is organized, how it works at a high level, and where to find the main integration points you might work with when adding features or fixing bugs.

## Intro & contributing

We welcome contributions of all sizes. Typical contributions include bug fixes, new editor features, additional themes, documentation improvements, and core parser enhancements.

## Suggested workflow

1. Open an issue describing the change or bug you want to address.
2. Fork the repository and create a feature branch.
3. Add tests where appropriate and keep changes small and focused.
4. If modifying the core grammar, test with various markdown samples.
5. Run `cargo build` and `cargo test` locally.
6. Open a pull request describing the change and link the related issue.

Code style and expectations

- Keep UI code in `src/components/` and `src/ui/` and business logic in `src/logic/`.
- Follow Rust idioms and project patterns (use `Result<T, E>`, avoid panics in library code, document public APIs).
- Add unit tests under the appropriate module and integration tests under `tests/`.

## Workspace Structure

Marco uses a Cargo workspace with three crates:

- **core** — Pure Rust library with nom-based parser, AST builder, HTML renderer, LSP features, and core logic (buffer management, settings, paths, cache, logging). No GTK dependencies. Located in `core/`.
- **marco** — Full-featured GTK4 editor binary with SourceView5 text editing and WebKit6 preview. Depends on core. Located in `marco/`.
- **polo** — Lightweight viewer-only binary with WebKit6 preview but no text editing (no SourceView5). Depends on core. Located in `polo/`.

Assets (themes, fonts, icons, settings) are centralized at the workspace root in `assets/`.

### Core Library Structure

The `core/` crate is organized into several key modules:

- **`grammar/`** — nom-based grammar parsers for block and inline Markdown elements
- **`parser/`** — AST building from grammar output (includes `ast.rs`, `block_parser.rs`, `inline_parser.rs`, `position.rs`)
- **`render/`** — HTML renderer with entity escaping and syntax highlighting support
- **`lsp/`** — LSP features: syntax highlighting, diagnostics, completion, hover
- **`logic/`** — Pure Rust business logic: buffer management, settings, paths, cache, logging

## How it works (concise)

Marco uses a three-layer design:

- **main** — application entry and glue (in `marco/src/main.rs`), responsible for initializing GTK, ThemeManager, and wiring UI to logic.
- **components** — GTK widgets, layout, and event wiring (in `marco/src/components/`). The primary editor component is created via `create_editor_with_preview_and_buffer`.
- **logic** — document buffer management, file operations, and settings. Core logic lives in `core/src/logic/` (pure Rust, no GTK). UI-specific logic lives in `marco/src/logic/` (GTK-dependent signal management and menu handlers).

The core library handles markdown parsing and HTML rendering using a nom-based parser. The editor is a split-pane composed of a SourceView-based text buffer and a WebKit6-based HTML preview. Changes in the buffer trigger live re-rendering using core's parser for Markdown-to-HTML conversion with proper image path resolution. The parser uses nom combinators in `core/src/grammar/` to build an AST which is then rendered to HTML by `core/src/render/`.

## Embedding & API (main integration points)

These functions are useful when embedding the editor widget or integrating with Marco programmatically. See the corresponding source files for details and type signatures.

- `create_editor_with_preview_and_buffer(preview_theme_filename, preview_theme_dir, theme_manager, theme_mode, document_buffer)`
  - Returns: `(Paned, WebView, css_rc, refresh_preview, update_editor_theme, update_preview_theme, buffer, insert_mode_state, set_view_mode)`
  - Notes: Add the returned `Paned` to your window. Call `refresh_preview()` to re-render and `update_editor_theme(scheme_id)` / `update_preview_theme(scheme_id)` to change themes at runtime. The `document_buffer` parameter should be a `DocumentBuffer` for file path management and WebKit6 base URI support.

- `render_editor_with_view(style_scheme, font_family, font_size_pt)`
  - Returns: `(container, buffer, source_view)`
  - Notes: Useful for embedding the editor view without the WebView preview.

- `wire_footer_updates(buffer, labels, insert_mode_state)`
  - Notes: Attaches debounced footer updates that compute cursor position, word/char counts, and syntax information using the core AST parser.

If you add public utilities, document small examples for how to call them from `main.rs` or tests.

## Configuration & themes

File locations used during development:

- **Themes and assets**: `assets/themes/` at workspace root.
- **Settings file**: `assets/settings.ron` (with defaults in `settings_default.ron`).
- **Languages**: `assets/language/` for localization files.
- **Core library**: `core/src/` contains the nom-based markdown parser (`grammar/`, `parser/`), HTML renderer (`render/`), and LSP features (`lsp/`).

## Theme manager notes

- The application uses a `ThemeManager` to map editor schemes to preview theme modes. Changing themes from the settings dialog calls back into functions returned by `create_editor_with_preview_and_buffer`.

## Adding a theme

1. Add CSS files under `assets/themes/`
2. Place editor style schemes under `assets/themes/editor/`.
3. Place view style schemes under `assets/themes/html_viewer/`
4. Update the theme manager to include your new theme.

## Quickstart & dev commands

Build all crates:

```bash
cargo build --release --workspace
```

Build specific crates:

```bash
cargo build --release -p core  # Core library only
cargo build --release -p marco       # Full editor
cargo build --release -p polo        # Viewer only
```

Run the full editor (development):

```bash
cargo run --release -p marco
```

Run the viewer only:

```bash
cargo run --release -p polo
```

Run tests for all crates:

```bash
cargo test --workspace --lib --tests -- --nocapture
```

Run tests for specific crate:

```bash
cargo test -p core -- --nocapture
```

## Troubleshooting

- **GTK CSS errors**: Ensure you run from the repository root so relative theme paths resolve. Check `assets/themes/*` exists.
- **Missing fonts or icons**: Confirm `assets/fonts/` and `assets/icons/` are present and that `get_asset_dir_checked()` finds the repo asset path.
- **Preview not updating**: Verify the buffer change signal is firing and that the core parser is working correctly. Check the WebKit6 console for base URI issues with local images.
- **Core parsing issues**: The app uses a custom nom-based parser in `core/src/grammar/` and `core/src/parser/` — check the grammar modules and AST builder if markdown isn't rendering correctly. Run `cargo test -p core` to validate parser behavior.
- **Local images not displaying**: Ensure WebKit6 security settings are enabled and DocumentBuffer is providing correct base URIs for file:// protocol access.
- **Import errors**: Use `core::` for core functionality (parser, buffer, logic), `crate::` for local modules within marco or polo binaries.

If you hit a problem you can't resolve, open an issue with a short description, steps to reproduce, and the output of running the app in a terminal.

## High-value contributions

These are areas where an implemented contribution will have big impact. If you plan to work on any of these, open an issue first so we can coordinate and reserve the scope.

### Collaborative editing (Yjs / CRDT)
- **Goal**: Add a shared-document component that syncs buffer state across peers using a CRDT backend (Yjs, automerge, or similar).
- **Integration points**:
  - Create a new `src/components/collab/` module that implements a `CollabBackend` trait (connect, disconnect, apply_remote_ops, get_local_patch).
  - Wire the component into the editor buffer event loop: when the local buffer changes, the component should produce and broadcast a patch; when remote patches arrive, they should be applied to the `DocumentBuffer` using documented public update methods.
  - Respect existing undo/redo and cursor/selection synchronization: treat remote changes as first-class edits and emit events the UI can use to update cursors.
- **Testing notes**: add unit tests for concurrent patches, and an integration test using two in-process backends that exchange patches.

### AI-assisted tools
- **Goal**: Provide a component API and example component that offers in-editor assistance (summaries, rewrite suggestions, universal spell checking, autocorrect).
- **Integration points**:
  - Define a `src/components/ai/` interface that accepts a text range and returns suggested edits or annotations. Keep the component optional and behind a feature flag or runtime toggle.
  - Provide a small example implementation that uses an HTTP-based LLM adapter (local or remote) and demonstrates non-blocking requests using async tasks; always run requests off the UI thread and apply edits on the main loop.
  - Offer a CLI or developer test harness under `tests/ai/` to run the component against sample documents.
- **Security & privacy notes**: document privacy expectations clearly. Components that call external APIs must expose where data is sent and provide opt-in configuration.

## Component docs & assets

Reference README and asset locations for contributors working on components and translations:

- [marco/src/components/ai/README.md](marco/src/components/ai/README.md) — AI component guidance and interface notes
- [marco/src/components/collab/README.md](marco/src/components/collab/README.md) — Collaboration integration notes and references
- [marco/src/components/language/README.md](marco/src/components/language/README.md) — Localization provider contract and workflow
- [assets/language/language matrix.md](assets/language/language%20matrix.md) — language implementation matrix (coverage & contributors)

If you add new component folders, please include a short `README.md` in the folder that explains the contract, tests, and how to run the component's dev harness.
