# Contributing to Marco

Thank you for your interest in contributing to Marco. This document explains how the project is organized, how it works at a high level, and where to find the main integration points you might work with when adding features or fixing bugs.

## Intro & contributing

We welcome contributions of all sizes. Typical contributions include bug fixes, new editor features, additional themes, documentation improvements, and new markdown/AST schemas.

Suggested workflow

1. Open an issue describing the change or bug you want to address.
2. Fork the repository and create a feature branch.
3. Add tests where appropriate and keep changes small and focused.
4. Run `cargo build` and `cargo test` locally.
5. Open a pull request describing the change and link the related issue.

Code style and expectations

- Keep UI code in `src/ui/` and business logic in `logic/`.
- Follow Rust idioms and project patterns (use `Result<T, E>`, avoid panics in library code, document public APIs).
- Add unit tests under the appropriate module and integration tests under `tests/`.

## How it works (concise)

Marco uses a three-layer design:

- main — application entry and glue (in `src/main.rs`), responsible for initializing GTK, ThemeManager, and wiring UI to logic.
- ui — GTK widgets, layout, and event wiring (in `src/ui/`). The primary editor component is created via `create_editor_with_preview`.
- logic — parsing, document buffer, loaders, and schema handling (in `logic/`). Parser and schema code power AST validation and syntax traces.

The editor is a split-pane composed of a SourceView-based text buffer and a WebKit-based HTML preview. Changes in the buffer trigger live re-rendering using Comrak for Markdown-to-HTML conversion.

## Embedding & API (main integration points)

These functions are useful when embedding the editor widget or integrating with Marco programmatically. See the corresponding source files for details and type signatures.

- `create_editor_with_preview(preview_theme_filename, preview_theme_dir, theme_manager, theme_mode, labels)`
  - Returns: `(Paned, WebView, css_rc, refresh_preview, update_editor_theme, update_preview_theme, buffer, insert_mode_state)`
  - Notes: Add the returned `Paned` to your window. Call `refresh_preview()` to re-render and `update_editor_theme(scheme_id)` / `update_preview_theme(scheme_id)` to change themes at runtime.

- `render_editor_with_view(style_scheme, font_family, font_size_pt)`
  - Returns: `(container, buffer, source_view)`
  - Notes: Useful for embedding the editor view without the WebView preview.

- `wire_footer_updates(buffer, labels, syntax_map, insert_mode_state)`
  - Notes: Attaches debounced footer updates that compute cursor position, word/char counts, and syntax traces.

If you add public utilities, document small examples for how to call them from `main.rs` or tests.

## Configuration & themes

File locations used during development:

- theme and assets: `src/assets/themes/` and `src/assets/`.
- Production themes: `themes/` in `editor` and `html_viewer`.
- Active markdown schemas: `src/assets/markdown_schema/` (the app will load an active schema when available).
- Settings file: `src/assets/settings.ron` (or `themes/` equivalent in prod setups).

Theme manager notes

- The application uses a `ThemeManager` to map editor schemes to preview theme modes. Changing themes from the settings dialog calls back into functions returned by `create_editor_with_preview`.

Adding a theme or schema

1. Add CSS files under `src/assets/themes/`
2. Place editor style schemes under `src/assets/themes/editor/`.
3. Place view style schemes under `src/assets/themes/html_view/`
3. Add schema files to `src/assets/markdown_schema/` and update the settings to point at an active schema if needed.

## Quickstart & dev commands

Build:

```bash
cargo build --release
```

Run locally (development):

```bash
cargo run --release
```

Run tests:

```bash
cargo test --lib --tests -- --nocapture
```

Run the AST/syntax checker (Python):

```bash
python3 tools/ast_syntax_checker/main.py
```

## Troubleshooting

- GTK CSS errors: Ensure you run from the repository root so relative theme paths resolve. Check `src/assets/themes/*` exists.
- Missing fonts or icons: Confirm `src/assets/fonts/` and `src/assets/icons/` are present and that `get_asset_dir_checked()` finds the repo asset path.
- Preview not updating: Verify the buffer change signal is firing and that `comrak` is available (it's a Rust dependency compiled into the app).
- Schema loading issues: The app looks for schemas under `src/assets/markdown_schema/` — check the settings RON file if no schema is detected.

If you hit a problem you can't resolve, open an issue with a short description, steps to reproduce, and the output of running the app in a terminal.

## High-value contributions

These are areas where an implemented contribution will have big impact. If you plan to work on any of these, open an issue first so we can coordinate and reserve the scope.

 - Collaborative editing (Yjs / CRDT)
  - Goal: Add a shared-document component that syncs buffer state across peers using a CRDT backend (Yjs, automerge, or similar).
  - Integration points:
    - Create a new `components/collab/` module that implements a `CollabBackend` trait (connect, disconnect, apply_remote_ops, get_local_patch).
    - Wire the component into the editor buffer event loop: when the local buffer changes, the component should produce and broadcast a patch; when remote patches arrive, they should be applied to the `DocumentBuffer` using documented public update methods.
    - Respect existing undo/redo and cursor/selection synchronization: treat remote changes as first-class edits and emit events the UI can use to update cursors.
  - Testing notes: add unit tests for concurrent patches, and an integration test using two in-process backends that exchange patches.

- More AST + syntax flavors
  - Goal: Expand `src/assets/markdown_schema/` with more flavors and generating `display_hints.ron`.
  - Integration points:
  - Use `tools/ast_syntax_checker/` to test a given schema in `src/assets/markdown_schema/"name"`.
  - Testing notes: Test all added/new AST/Syntax to se if they work in the `footer` and `Preview` if not open a issue.
  - Submit an PR, when `ast_syntax_checker` shows the file has passed all tests

 - AI-assisted tools
  - Goal: Provide a component API and example component that offers in-editor assistance (summaries, rewrite suggestions, universal spell checking, autocorrect).
  - Integration points:
    - Define a `components/ai/` interface that accepts a text range and returns suggested edits or annotations. Keep the component optional and behind a feature flag or runtime toggle.
    - Provide a small example implementation that uses an HTTP-based LLM adapter (local or remote) and demonstrates non-blocking requests using async tasks; always run requests off the UI thread and apply edits on the main loop.
    - Offer a CLI or developer test harness under `tests/ai/` to run the component against sample documents.
  - Security & privacy notes: document privacy expectations clearly. Components that call external APIs must expose where data is sent and provide opt-in configuration.

If you'd like, I can also add short templates for issues/PRs that guide contributors working on these features (scoped tasks, test checklist, and API contracts). Mark which item you want templates for and I will add them.

## Component docs & assets

Reference README and asset locations for contributors working on components and translations:

- [src/components/ai/README.md](src/components/ai/README.md) — AI component guidance and interface notes
- [src/components/collab/README.md](src/components/collab/README.md) — Collaboration integration notes and references
- [src/components/language/README.md](src/components/language/README.md) — Localization provider contract and workflow
- [src/assets/language/language matrix.md](src/assets/language/language%20matrix.md) — language implementation matrix (coverage & contributors)

If you add new component folders, please include a short `README.md` in the folder that explains the contract, tests, and how to run the component's dev harness.
