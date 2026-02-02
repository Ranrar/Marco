# Changelog
All notable user-visible changes to **Marco** are documented here.

This project follows **Semantic Versioning** and uses the **Keep a Changelog** format.

**Dependency note:** Marco uses **Core** for parsing and rendering. Marco releases reference the Core version they ship with.

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

## [0.16.0] - 2026-02-02

### Added
- **Full cross-platform support** for Windows and Linux.
- Windows builds now use `wry` (WebView2) for HTML preview rendering.
- Linux builds use `webkit6` for HTML preview rendering.
- Windows icon embedding using `embed-resource` crate with `marco.rc` resource script.
- Platform-specific conditional compilation for webview backends.

### Changed
- Migrated to webkit6 0.5.0 async API for Linux builds (`evaluate_javascript_future`).
- Updated JavaScript evaluation to use async/await pattern with `glib::spawn_future_local`.
- Build system now supports both x86_64-pc-windows-msvc and x86_64-unknown-linux-gnu targets.

### Fixed
- Fixed Windows icon embedding - marco.exe now displays icon correctly.
- Fixed Linux build compatibility with webkit6 0.5.0 (removed callback-based API).
- Fixed borrow lifetime issues in webkit6 async JavaScript execution.
- Removed unused imports from search navigation and replace modules.

## [0.15.1] - 2026-01-31

### Added
- Added Windows preview helpers using `wry` for embedded previews on Windows:
  - `wry.rs` — HTML document wrapping, base URI generation, and HTML viewer creation using `wry`/WebView2 when available
  - `wry_detached_window.rs` — Detached preview window implementation that can host a `wry` WebView and integrate with the GTK application lifecycle
  - `wry_platform_webview.rs` — Platform-specific WebView wrapper for Windows that manages background color, HTML loading, and safe fallbacks when WebView2 is unavailable
  - Included runtime-friendly fallbacks and defensive checks for missing WebView2 runtimes; the feature is gated per-platform and integrates with the existing preview reparenting and menu logic

## [0.15.1] - 2026-01-30

### Added
- Replaced legacy IcoMoon icon-font glyphs with **inline SVG icons** across the UI (titlebar window controls, layout popover, dialogs, detached preview). These use `gtk::Picture` textures for crisp rendering and HiDPI supersampling.
- Added helper functions to render inline SVGs to `gtk::Picture` with consistent theme-driven color states.
- Added `DualView` layout SVG to the shared Core icon loader (see Core changelog).

### Changed
- Window control and layout buttons now use Picture-backed SVGs with hover and press color states aligned to Polo's visual behavior.
- CSS generation updated to remove `.icon-font`/IcoMoon selectors; theme constants adjusted for SVG-driven icon states.
- Popover logic improved: pre-created popover buttons and unparent them before re-append to avoid GTK parent assertion warnings.

### Fixed
- Added robust error handling for SVG parse/rasterization failures; a transparent 1x1 fallback texture avoids runtime panics on malformed SVG input.
- Fixed GTK parent assertion warnings by unparenting widgets before reuse in popovers.

### Removed
- Dropped legacy icon-font support and removed references to `ui_menu.ttf` in the UI code and tests.
- Removed the old `icon_font()` usage patterns (core paths helper moved/removed).
- Packaging scripts were updated to defensively remove deprecated `ui_menu.ttf` from installer/package outputs.

## [0.15.0] - 2026-01-25

**Uses:** Core 0.15.0

### Added
- Cross-platform path support for asset discovery and file operations

### Changed
- File operations now fully compatible with Windows file paths
- Error handling updated to use standard Rust error types instead of `anyhow`

### Fixed
- Fixed Result type annotations in file dialogs, menu handlers, and editor components
- Fixed error type conversions for GTK threading safety (`Send` trait compatibility)
- Editor settings save operations now properly handle errors

### Removed
- `anyhow` dependency removed

## [0.14.0] - 2026-01-18

**Uses:** Core 0.14.0

### Added
- Preview styling for extended GitHub-style custom-header admonitions (quote-styled callouts with theme-primary title color).
- Editor syntax highlighting for Marco tab block markers (`:::tab`, `@tab ...`, closing `:::`).
- Preview support + styling for Marco_sliders slideshow decks (`@slidestart[:tN]` … `@slideend` with `---` / `--` separators).
- Editor syntax highlighting for Marco_sliders marker/separator lines.

## [0.13.3] - 2026-01-17

**Uses:** Core 0.13.3

### Added
- New Marco logo (application icon), used in the titlebar and installed for desktop integration.

### Changed
- Debian packaging (`install/build_deb.sh`) was improved (dependency checks, deterministic `--locked` builds, icon installation/scaling, and additional build/versioning options).
- Linux desktop entry now uses the system icon name `marco`.

## [0.13.2] - 2026-01-15

**Uses:** Core 0.13.2

### Added
- Editor syntax highlighting coverage for additional structural elements (reference-style link placeholders and extended definition lists).

### Changed
- LSP highlight application is now chunked to reduce UI stutter on large documents.
- LSP tag cleanup uses a centralized authoritative tag list to keep UI and Core highlight tags in sync.

## [0.13.1] - 2026-01-14

**Uses:** Core 0.13.1

### Changed
- Reduced build footprint by removing unused direct dependencies.
- External links that start with `www.` are now opened as `https://…` by default.

### Fixed
- Prevented intermittent GTK/WebKit warnings by deferring WebView loads/updates until the widget is mapped and has an allocation.

### Security
- Tuned DevSkim/code-scanning configuration to ignore vendored/spec fixture content (improves signal-to-noise in Security scans).

## [0.13.0] - 2026-01-14

**Uses:** Core 0.13.0

### Added
- Syntax-highlighted code rendering.
- Emoji shortcodes in rendered output.
- Footnotes.
- Extended heading identifiers.

## [0.12.0] - 2026-01-13

**Uses:** Core 0.12.0

### Added
- Editor/LSP support for task list checkboxes and tables.

## [0.11.0] - 2026-01-12

**Uses:** Core 0.11.0

### Changed
- Packaging/build workflow for Linux installs was updated and simplified.

## [0.10.0] - 2026-01-11

**Uses:** Core 0.10.0

### Added
- GitHub Flavored Markdown tables.
- Additional inline formatting extensions.

## [0.9.0] - 2025-10-28

**Uses:** Core 0.9.0

### Fixed
- More robust handling of autolinks vs inline HTML (reduces false-positive autolinks around common tags).

## [0.8.0] - 2025-10-27

**Uses:** Core 0.8.0

### Fixed
- Improved consistency for some Markdown parsing edge cases (thematic breaks and inline spans).

## [0.7.0] - 2025-10-25

**Uses:** Core 0.7.0

### Added
- Syntax highlighting support in editor integrations.

## [0.6.0] - 2025-10-24

**Uses:** Core 0.6.0

### Changed
- Theme appearance was standardized for more consistent UI colors.

## [0.5.0] - 2025-10-23

**Uses:** Core 0.5.0

### Added
- Editor assistance (completions and diagnostics) for common Markdown structures.

### Changed
- Linux install flow moved toward packaged installation.

### Removed
- Removed the user-local install/uninstall workflow in favor of packaged installation.

## [0.4.0] - 2025-10-21

**Uses:** Core 0.4.0

### Changed
- Core parsing pipeline was integrated more directly to improve stability.

## [0.3.0] - 2025-10-20

**Uses:** Core 0.3.0

### Added
- Support for link reference definitions and HTML blocks (via Core).

## [0.2.0] - 2025-10-19

**Uses:** Core 0.2.0

### Changed
- General improvements to behavior and stability (based on commit messaging; details not specified).

## [0.1.0] - 2025-10-18

**Uses:** Core 0.1.0

### Added
- Initial integration of the shared Core engine.
