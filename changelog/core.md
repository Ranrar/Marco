# Changelog
All notable user-visible changes to **Core** are documented here.

This project follows **Semantic Versioning** and uses the **Keep a Changelog** format.

Version scheme note: versions are reconstructed as `0.YY.ZZ` from git history using date-based release groupings starting at the first point where Core, Marco, and Polo co-exist in the repository (2025-10-18).

## [Unreleased]

### Added
- Nothing yet.

### Changed
- Nothing yet.

### Fixed
- Nothing yet.

### Removed
- Nothing yet.

### Security
- Nothing yet.

## [0.21.0] - 2026-03-13

### Added
- In-process intelligence engine (`core/src/intelligence/`) — replaces the previous `lsp/` module; provides syntax highlighting, hover information, Markdown completions, and diagnostic analysis behind a clean public API boundary.
- Two diagnostics catalogs in RON format (`diagnostics_catalog_marco.ron`, `diagnostics_catalog_markdownlint.ron`) — map diagnostic codes to human-readable titles, descriptions, severity, and fix suggestions.
- `get_position_span` — returns the tightest AST node span enclosing a cursor position; supports hover suppression logic in the editor.
- `intelligence/markdown/` boundary module — structured reinterpretation of the AST for intelligence consumers (blocks and inlines classification).
- LSP protocol adapter stub (`intelligence/lsp_protocol.rs`) for future language server integration.
- `DiagnosticsFilterSettings` struct in `EditorSettings` — per-severity toggles (errors, warnings, hints, infos) for the diagnostics display; replaces the previous `linting` boolean.
- `diagnostics_underlines_enabled`, `diagnostics_hover_enabled`, and `markdown_hover_enabled` fields in `EditorSettings` for fine-grained intelligence feature control.
- `SettingsManager::reload_settings` is now `pub` to allow external callers to trigger a settings reload.

### Changed
- Replaced `core/src/lsp/` module with `core/src/intelligence/` — Markdown intelligence capabilities (highlights, diagnostics, completions, hover) are now organized under feature-area sub-modules (`editor/`, `markdown/`, `analysis/`, `catalog/`).
- `EditorSettings`: removed `auto_pairing` and `linting` fields; added `diagnostics_underlines_enabled`, `diagnostics_hover_enabled`, `markdown_hover_enabled`, and `diagnostics_filter`.

### Fixed
- Image nodes now carry the span of the full `![alt](url)` syntax rather than just the alt-text fragment; this prevents zero-length spans when alt text is empty.
- Footnote definition nodes had their span end set to end-of-document due to use of the inclusive span range helper; corrected to use the exclusive variant so subsequent content is not erroneously enclosed within a footnote definition span.

### Removed
- Old `core/src/lsp/` module (completion, diagnostics, highlights, hover) superseded by `core/src/intelligence/`.

## [0.20.0] - 2026-03-04

### Added
- Centralized settings manager (`SettingsManager` / `Settings` in `logic/swanson.rs`) — thread-safe RON-based settings with typed sub-structs for editor, appearance, layout, language, telemetry, window, file, and polo configuration; supports change listeners, file I/O, validation, and auto-repair on load.
- Bookmark and emoji history persistence in `Settings`: `get_bookmarks`, `set_bookmarks`, `record_emoji_usage`, `get_top_emoji_usage`, and `clean_recent_files`.
- `PoloSettings` and `PoloWindowSettings` structs for Polo-specific persistent configuration.
- Emoji shortcode completion API (`logic/text_completion.rs`) — OnceLock-cached static list of `EmojiCompletionItem` values with `normalize_completion_query` and `emoji_shortcode_matches_query` prefix-match helpers.
- Pure-Rust Mermaid diagram rendering (`render/diagram.rs`) — native diagram rendering via `mermaid_rs_renderer` with a `MERMAID_MAX_CHARS` safety limit; full GitHub-style dark and light themes via `create_mermaid_theme`.
- KaTeX math rendering (`render/math.rs`) — `render_inline_math` and `render_display_math` with MathML output and a global `OnceLock<KatexContext>` for performance.
- Unified HTML preview document builder (`render/preview_document.rs`) — `wrap_preview_html_document()` shared by Marco and Polo; embeds interactive table-resize CSS and JS, heading anchor link CSS and SVG, background-flash prevention, and the `window.MarcoPreview` JS API.
- Layout state enum (`logic/layoutstate.rs`) — `LayoutState { DualView, EditorOnly, ViewOnly, EditorAndViewSeparate }` with a `layout_state_label` string helper.
- Cross-platform detection helpers (`logic/crossplatforms.rs`) — `Platform { Linux, Windows, Unknown }` enum, `detect_platform()`, and `is_dark_mode_supported()`.

### Changed
- Settings system expanded from minimal telemetry and window structs (0.18.0) to a full application settings hub with typed per-component structs, audit-logged file I/O, and per-component typed change listeners.

## [0.18.0] - 2026-02-09

### Added
- System locale detection helper (`detect_system_locale_iso639_1`) with ISO 639-1 normalization.
- Telemetry settings support (persisted settings fields for enabling telemetry + tracking whether the first-run dialog has been shown).
- Inline SVG icons for About dialog link buttons (GitHub, Link, Bug, Help).

### Changed
- Portable mode detection on Linux and Windows now prefers a writable `config/` directory next to the executable (more reliable portable installs).
- Asset root validation now requires `language/` alongside `icons/` and `themes/`.
- Settings change listeners are now stored as `Arc` and notified outside the listener lock (reduces lock contention and avoids re-entrancy hazards).

### Fixed
- Reduced false-positive portable-mode detection in development environments by tightening Linux portable heuristics.

## [0.17.1] - 2026-02-04

### Changed
- Improved cross-platform compilation with refined conditional compilation attributes for Linux and Windows builds.
- Enhanced platform-specific code organization using explicit `cfg(target_os)` gates.

### Fixed
- Resolved Linux build compilation error in font loader module (missing `HashMap` import under platform-specific code path).

## [0.17.0] - 2026-02-03

### Added
- **Platform abstraction module** (`core::paths::platform`) for OS-specific path implementations.
- **Windows portable mode detection** - automatically uses local config/data when exe directory is writable.
- **Linux platform paths** - XDG-compliant user directories (`~/.local/share/marco`, `~/.config/marco`).
- **Windows platform paths** - Standard Windows locations (`%LOCALAPPDATA%\Marco`) with portable fallback.
- **Asset root validation** - verifies asset bundles contain required directories before accepting paths.

### Changed
- **Refactored path system** to use platform-specific modules with explicit `cfg` attributes.
- **Install location detection** now properly distinguishes between system/user/portable installs.

## [0.16.0] - 2026-02-02

### Added
- **Full cross-platform support** for Windows and Linux.
- Platform-agnostic core library works identically on both platforms.

### Changed
- Migrated logger from `static mut` to `OnceLock<T>` for Rust 2024 compatibility.
- Removed all unsafe blocks for logger access in favor of safe initialization pattern.

### Fixed
- Fixed `static_mut_refs` warnings to comply with Rust 2024 edition.

## [0.15.2] - 2026-01-30

### Added
- Added `DualView` layout **inline SVG** to the Core icon loader.
- Icon loader documentation updated to describe inline SVG usage and HiDPI rasterization expectations.

### Changed
- Icon loader and related docs updated to formally deprecate icon-font usage across the workspace and to prefer inline SVG assets.

### Fixed
- N/A

### Removed
- Removed the legacy `icon_font()` helper from `core::paths::SharedPaths` (icon-font helper was no longer used).

### Security
- Nothing yet.

## [0.15.1] - 2026-01-26

### Changed
- Icon loader now supports inline SVG rendering for window controls
  - Added window icon SVG generation (minimize, maximize, restore, close)
  - Integrated with rsvg/librsvg for high-quality SVG rasterization

## [0.15.0] - 2026-01-25

### Added
- Cross-platform file path support for Windows and Linux
  - Asset root discovery now supports Windows paths (`%LOCALAPPDATA%`, `%PROGRAMFILES%`, `%PROGRAMDATA%`)
  - Install location detection works for both Linux (`/usr/share`, `~/.local/share`) and Windows (Program Files, AppData)
  - Platform-appropriate log directories (Linux: `~/.cache/marco/logs`, Windows: `%LOCALAPPDATA%\Marco\logs`)
  - Config and data directories use platform-specific locations via `dirs` crate

### Changed
- Logger now uses platform-appropriate cache directory instead of hardcoded `cwd/log`
- Path detection uses conditional compilation for Linux and Windows specific logic

### Fixed
- Removed `anyhow` dependency, replaced with standard `Result<T, Box<dyn std::error::Error>>`
- Fixed all Result type annotations throughout parser and logic modules
- Fixed thread safety issues for error types used with GTK's `gio::spawn_blocking`

### Removed
- `anyhow` dependency removed from core library

## [0.14.0] - 2026-01-18

### Added
- Extended GitHub-style admonitions with custom headers, e.g. `> [:joy: Happy Header]` (quote-styled, with optional emoji/icon + custom title).
- Marco_Extended tab blocks (`:::tab` + `@tab ...` + closing `:::`), parsed into `TabGroup`/`TabItem` and rendered as a no-JS tab UI in HTML preview.
- Marco_sliders slideshow decks (`@slidestart[:tN]` … `@slideend` with `---` / `--` separators), parsed into a dedicated AST and rendered as an interactive slideshow in the preview.
- Source-aware LSP highlighting helper (`compute_highlights_with_source`) to color structural marker lines like tab block markers.

## [0.13.3] - 2026-01-17

### Added
- Per-application icon/logo helpers in `SharedPaths` (Marco vs Polo), with backwards-compatible fallbacks for older shared icon filenames.
- New Marco and Polo logo/icon assets are now supported by the shared asset resolution layer.

### Changed
- Shared asset icon resolution now prefers the Marco/Polo-specific icon/favicons when present.

## [0.13.2] - 2026-01-15

### Added
- Extended definition lists (Markdown Guide / Markdown Extra-style).
- Headerless pipe tables (delimiter-first, no header row).
- Inline footnotes (`^[...]`) (inline definitions collected into the document footnotes section).
- Platform mentions (`@username[platform]` with optional display name) rendered as profile links for known platforms.
- Inline task checkbox markers (`[ ]` / `[x]` / `[X]`) inside paragraphs (including mid-paragraph markers and after hard line breaks).

### Changed
- Emoji shortcode handling now uses a full shortcode dataset for broader support.
- LSP highlight output is now sorted and de-duplicated for deterministic application.

### Fixed
- Prevented a UTF-8 boundary slicing panic when scanning for emoji shortcode candidates.

## [0.13.1] - 2026-01-14

### Changed
- Reduced build footprint by removing unused direct dependencies.
- Made external-link CSS selectors in the preview document more tolerant (match `http:`/`https:` prefixes).

### Security
- Reduced DevSkim/code-scanning noise by avoiding insecure URL literals in non-user-facing examples/tests.

## [0.13.0] - 2026-01-14

### Added
- Syntax-highlighted code rendering.
- Emoji shortcodes in rendered output.
- Footnotes.
- Extended heading identifiers.

## [0.12.0] - 2026-01-13

### Added
- Editor/LSP support for task list checkboxes and tables.

## [0.11.0] - 2026-01-12

### Changed
- Packaging/build workflow for Linux installs was updated and simplified.

## [0.10.0] - 2026-01-11

### Added
- GitHub Flavored Markdown tables.
- Additional inline formatting extensions.

## [0.9.0] - 2025-10-28

### Fixed
- More robust handling of autolinks vs inline HTML (reduces false-positive autolinks around common tags).

## [0.8.0] - 2025-10-27

### Fixed
- More consistent parsing of thematic breaks.

## [0.7.0] - 2025-10-25

### Added
- Syntax highlighting support in editor integrations.

## [0.6.0] - 2025-10-24

### Changed
- Theme color definitions were standardized for consistent appearance.

## [0.5.0] - 2025-10-23

### Added
- Editor/LSP completions and diagnostics for common Markdown structures (including lists, blockquotes, thematic breaks, and HTML blocks).

### Changed
- Linux install flow moved toward packaged installation.

## [0.4.0] - 2025-10-21

### Changed
- Parsing pipeline was modularized and legacy components were removed to improve maintainability and stability.

## [0.3.0] - 2025-10-20

### Added
- Unicode normalization for more consistent parsing of equivalent text.
- Link reference definitions.
- HTML blocks.

### Changed
- Block parsing improved for nested structures and blank-line edge cases.

## [0.2.0] - 2025-10-19

### Fixed
- Improved handling of invalid or tricky UTF-8 input.

## [0.1.0] - 2025-10-18

### Added
- Initial Core crate integration as a shared library used by Marco and Polo.

### Changed
- Parsing engine was rebuilt around a nom-based pipeline.
