# Architecture

## Workspace Structure

Marco uses a Cargo workspace with three crates plus an external dependency:

- **marco-core** — Pure Rust library with the nom-based parser, AST builder, HTML renderer, and language-intelligence features (highlights, diagnostics, completions, hover). **Lives in its own repository** ([Ranrar/marco-core](https://github.com/Ranrar/marco-core)) and is consumed from crates.io. The pinned version is declared in the workspace `Cargo.toml` under `[workspace.dependencies.marco-core]`.
- **marco-shared** — Shared application logic: buffer management, settings, paths, file loaders, and layout state. Depends on `marco-core`. Located in `marco-shared/`. Also owns the centralized assets and the `build.rs` that copies them into `target/*/marco_assets/`.
- **marco** — Full-featured GTK4 editor binary with SourceView5 text editing and WebKit6 preview. Depends on `marco-core` and `marco-shared`. Located in `marco/`.
- **polo** — Lightweight viewer-only binary with WebKit6 preview but no text editing (no SourceView5). Depends on `marco-core` and `marco-shared`. Located in `polo/`.

Assets (themes, icons, settings, language files) live in `marco-shared/src/assets/`.

### Polo (viewer) notes

`polo/` is the viewer-focused sibling of Marco. It is intended to be a smaller GTK app that:

- Renders Markdown via `core` (parser + HTML renderer)
- Displays the result in a WebKit-based preview
- Does **not** include a full editor surface (no SourceView)

Key files:

- Entry point: `polo/src/main.rs`
- UI components: `polo/src/components/`

Good contribution areas for Polo:

- Preview performance improvements (incremental refresh, caching)
- Theme parity with Marco (HTML preview themes)
- File opening / reload behavior
- Cross-platform windowing and webview integration (keeping UI code isolated in `polo/`; parser/renderer changes belong in [`marco-core`](https://github.com/Ranrar/marco-core))

### marco-core Library Structure

The `marco-core` crate (external repository) is organized into several key modules:

- **`grammar/`** — nom-based grammar parsers for block and inline Markdown elements
- **`parser/`** — AST building from grammar output (includes `ast.rs`, `block_parser.rs`, `inline_parser.rs`, `position.rs`)
- **`render/`** — HTML renderer with entity escaping and syntax highlighting support
- **`intelligence/`** — syntax highlighting, diagnostics, completion, hover
- **`logic/`** — Pure Rust business logic: cache, logging, utf8 sanitization

To work on these modules, clone https://github.com/Ranrar/marco-core and develop there.

## How it works (concise)

Marco uses a three-layer design:

- **main** — application entry and glue (in `marco/src/main.rs`), responsible for initializing GTK, the theme manager, and wiring UI to logic.
- **components** — GTK widgets, layout, and event wiring (in `marco/src/components/`). The primary editor component is created via `create_editor_with_preview_and_buffer`.
- **logic** — document buffer management, file operations, and settings.
  - Pure parser / cache / logging logic lives in the external [`marco-core`](https://github.com/Ranrar/marco-core) crate.
  - Shared (GTK-free) app logic lives in `marco-shared/src/` (buffer, settings, paths, loaders).
  - UI-specific logic lives in `marco/src/logic/` (GTK-dependent signal management and menu handlers).

The `marco-core` crate handles markdown parsing and HTML rendering using a nom-based parser. The editor is a split-pane composed of a SourceView-based text buffer and a WebKit6-based HTML preview. Changes in the buffer trigger live re-rendering: text is fed into `marco_core::parser::parse` to build an AST, which is then rendered to HTML by `marco_core::render::render`, with proper image path resolution applied by `marco-shared`/`marco`.

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

## Configuration

File locations used during development:

- **Settings template (packaging / reference)**: `marco-shared/src/assets/settings_org.ron`.
- **Settings used at runtime**: resolved via `marco_shared::paths`.
  - Dev mode uses `tests/settings/settings.ron`.
  - Installed builds use the per-OS config directory (for example `~/.config/marco/settings.ron` on Linux; on Windows this may be `%APPDATA%\marco\settings.ron` or a portable `config\settings.ron` next to the executable).
- **Markdown engine**: the `marco-core` crate ([repo](https://github.com/Ranrar/marco-core)) provides the nom-based parser (`grammar/`, `parser/`), HTML renderer (`render/`), and intelligence features. Pinned via `[workspace.dependencies.marco-core]` in the root `Cargo.toml`.
- **Themes**: see [Themes](themes.md).
- **Languages**: see [Localization (UI language)](language.md).
