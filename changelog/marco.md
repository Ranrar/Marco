# Changelog
All notable user-visible changes to **Marco** are documented here.

This project follows **Semantic Versioning** and uses the **Keep a Changelog** format.

**Dependency note:** Marco uses **Core** for parsing and rendering. Marco releases reference the Core version they ship with.

Version scheme note: versions are reconstructed as `0.YY.ZZ` from git history using date-based release groupings starting at the first point where Core, Marco, and Polo co-exist in the repository (2025-10-18).

## [Unreleased]

## [0.23.2] - 2026-04-28

**Uses:** Core 1.0.2

### Added
- Unified export and print pipeline shared between Linux and Windows, with a common state machine, cancel token, and progress reporting so PDF and HTML export behave consistently across platforms.
- New "Exporting…" modal progress dialog with indeterminate progress, phase reporting, and cancel-via-close support during long-running PDF / HTML exports.
- New "Export complete" success dialog offering one-click actions to open the exported file in the system default app, reveal it in the file manager, or dismiss.
- Windows: native PDF export via WebView2's `ICoreWebView2_7::PrintToPdf`, removing the previous dependency on a headless Chromium / Edge subprocess. The export runs entirely in-process and keeps the GTK / Win32 message loop responsive while the export completes.
- Windows: native print dialog support — File → Print now opens the system print UI directly from the embedded WebView2 preview, matching the Linux print flow.
- Shared print/export CSS in `marco-shared` so paged.js page-box layout, paper size, orientation, and dark-mode handling stay consistent between live print, PDF export, and HTML export on both platforms.
- HTML export now uses a shared static-wrap composer for byte-stable output across runs and platforms.
- New CI workflow for publishing the `marco-core` crate to crates.io.

### Changed
- Windows portable packaging script now resolves the repository root from the script location, so it works consistently from both GitHub Actions release workflows and manual invocation from arbitrary working directories.
- Workspace crate layout was refactored: `core` was renamed to `marco-core`, and shared app/platform logic/assets were extracted into `marco-shared` for clearer separation between reusable engine code and app-layer code.
- Export dialog wiring was reworked to drive the unified pipeline, share progress UI between platforms, and surface clearer per-phase status.
- Cross-platform packaging/build documentation and scripts were updated to reflect the refactored crate layout for both Linux and Windows release flows.
- Source file permission metadata was normalized to avoid accidental executable bits on non-executable source/content files across platform checkouts.

### Fixed
- Windows: PDF export no longer requires an external Chromium/Edge install or spawned subprocess; export now uses the in-process WebView2 backend.
- Windows: print and PDF export now apply the same paged.js / `@media print` rules used on Linux, fixing prior fidelity gaps in paper size, orientation, margins, and dark-mode handling.
- Print/export progress UI now stays responsive on Windows during long operations (message-loop pumping during the COM async call).
- Debian package dependency metadata now supports newer Ubuntu-family runtime naming by accepting `libxml2-16` as an alternative to `libxml2`.
- Linux package build script now correctly detects Cargo's configured target directory when copying built binaries into the `.deb` payload.
- Fixed a Linux first-run Welcome screen regression where the Next button could be missing due to assistant action-area/header-bar behavior.

### Removed
- Removed the legacy workspace `core` crate path in favor of `marco-core` + `marco-shared` split.
- Removed the headless Chromium / Edge subprocess code path previously used for Windows PDF export.

### Security
- Verified mitigation status for GHSA-82j2-j2ch-gfr8 on Linux and Windows release targets: dependency graph resolves to patched `rustls-webpki` 0.103.13.
- Updated transitive `rand` to 0.8.6 in the workspace lockfile.

## [0.23.1] - 2026-04-14

**Uses:** Core 0.23.1

### Changed
- Rust toolchain updated to 1.94.1 (MSRV bumped from 1.93.0).
- GTK ecosystem upgraded to gtk4 0.11.2 / glib 0.22.5 / sourceview5 0.11.0 / webkit6 0.6.1 series.

### Security
- Updated `rand` dependency to address unsound behavior (GHSA-cq8v-f236-94qc).

## [0.23.0] - 2026-04-12

**Uses:** Core 0.23.0

### Added
- Live page-view print preview using paged.js integration in the preview pipeline, including paper size, orientation, margins, page numbers, and multi-column page layout options.
- New export workflow and dialog for PDF and HTML export, with per-export controls for theme, color mode, paper options, orientation, margins, and page numbers.
- Standalone HTML export mode options including paged output and paperless (`None`) output paths.
- Preview zoom UI (overlay controls and zoom state persistence), including reset and incremental zoom actions.
- Dedicated **Print Preview** settings tab for persistent page-view defaults.
- New **Application** settings tab consolidating UI/theme and preview behavior options.

### Changed
- Settings UI was remodeled: the previous Appearance-focused structure was reorganized around Application and Print Preview workflows.
- Viewer/render integration was refactored so print/export and page-view behavior share a consistent rendering pipeline.

### Fixed
- Fixed line-break behavior in live preview flows so authored hard-break patterns render consistently.
- Export dialog styling and layout consistency improved (light/dark theme parity, aligned control sizing, consistent bottom action area, and clearer locked-state visuals).

### Removed
- Removed the legacy `Appearance` settings tab implementation.
- Removed decorative anchor/link icon adorners in live preview link presentation (heading text remains directly linkable).

## [0.22.0] - 2026-04-08

**Uses:** Core 0.22.0

### Added
- TOC sidebar panel — collapsible table-of-contents drawer extracted from live document headings; configurable depth and click-to-scroll navigation.
- Table of contents insert — insert a Markdown TOC block at the cursor position via the Insert menu or toolbar.
- TOC depth setting — controls how many heading levels (H1-H6) appear in the TOC sidebar.
- Live preview link hover — hovering over a link in the HTML preview shows its target URL in the footer status bar; clears when the cursor leaves the link.
- Welcome screen theme selection — first-run wizard now offers a light/dark mode choice before opening the editor for the first time.
- Right-to-left (RTL) text direction support — full UI flip: editor layout, split-pane ordering, menus, toolbar, footer, scrollbar placement, line-number gutter migrated from left to right side, HTML preview body direction, and live JS toggle without a restart.
- Table auto-align — pipe tables are automatically reformatted and column-aligned when pressing Tab, Enter, or moving the cursor outside a table row; the same reformat can be triggered on demand via the right-click context menu ("Format Table") or the keyboard shortcut Ctrl+Alt+T. Auto-alignment can be turned on or off in Settings → Editor → "Auto-Align Tables".
- Local link prompt — clicking a local file link in the HTML preview prompts to open that file in Marco; if the current file has unsaved edits, the prompt additionally offers to save before opening or cancel.
- Heading anchor links on all headings — hover-anchor links (the chain-link icon) are now rendered next to every heading; previously they only appeared on headings with an explicit `{#id}` marker. Required by click-to-scroll in the TOC sidebar.

### Fixed
- Tools menu restored and fully wired — quick-toggle panel covering line wrap, line numbers, show invisibles, tabs-to-spaces, syntax colours, table auto-align, scroll sync, and text direction; each toggle reads live editor state, applies the change immediately, and persists it to settings.

## [0.21.0] - 2026-03-13

**Uses:** Core 0.21.0

### Added
- Native `GtkSourceHoverProvider` (`components/editor/hover_provider.rs`) — span-comparison logic selects the narrowest match when both a diagnostic and a Markdown insight apply at the cursor; when only a diagnostic is present, it is suppressed if a tighter AST node covers the cursor position.
- Diagnostic underline markers in the editor (`components/editor/intelligence.rs`) — underlines are applied in chunks of 400 via GLib idle callbacks to avoid main-thread frame stutter.
- Diagnostics panel in the footer — a button displays error and warning counts; clicking opens a popover with a filterable list of all document issues, each navigable by clicking.
- Diagnostics Reference dialog (`ui/dialogs/diagnostics_reference.rs`) — searchable, categorized reference of all diagnostic codes with severity, descriptions, and fix suggestions.
- Intelligence settings tab (`ui/settings/tabs/intelligence.rs`) — per-feature toggles for diagnostic underlines, Markdown insights hover, issue insights hover, and syntax highlighting.
- Hover popover CSS module (`ui/css/popover.rs`) and diagnostics issue list CSS module (`ui/css/issue.rs`).
- "Diagnostics Reference" item added to the Help menu.

### Changed
- Replaced `lsp_integration.rs` with `intelligence.rs` backed by `core::intelligence`; all previous LSP symbols removed.
- Intelligence settings moved to a dedicated Intelligence tab; Auto Pairing and Markdown Linting controls removed from the Editor settings tab.
- CSS system extended with `footer.rs` module for diagnostic badge and popover styles; `menu.rs` and `dialog.rs` updated with new component styles.
- Updated translations (`en.toml`, `de.toml`) with intelligence settings keys, "Diagnostics Reference" menu label, and Intelligence tab keys.
- Disabled unfinished controls so users can clearly see they are not available yet: Text Direction, UI Font, UI Font Size, Send Anonymous User Data, and File → Export.

### Fixed
- Hover provider no longer shows a diagnostic popover for text visually below the last diagnostic; the span-comparison logic correctly identifies the narrowest applicable insight at the cursor position.
- Package installer now creates a `libxml2.so.2` compatibility symlink automatically on distributions that ship libxml2 2.12+ (soname `libxml2.so.16`), such as AnduinOS 1.4.2 and Ubuntu 24.10+, preventing a startup failure due to the missing shared library.
- Added `libxml2 (>= 2.9)` to the `.deb` package `Depends` field; it was a direct runtime dependency that was previously undeclared.

### Removed
- `ui/menu_items/tools.rs` — Tools menu removed; its actions were migrated or deferred to other menus.
- Auto Pairing and Markdown Linting settings removed from the Editor tab (superseded by Intelligence tab controls).
- Removed the "Custom CSS for Preview" button from the Appearance settings tab.

## [0.20.0] - 2026-03-04

**Uses:** Core 0.20.0

### Added
- Bookmark system (`components/bookmarks/BookmarkManager`) — full CRUD operations backed by `SettingsManager`; automatic line-position shifting after text insertions; bookmarks grouped by current and other files for menu display.
- Interactive Markdown table editing (`components/editor/table_edit.rs`) — parse, navigate, and modify tables inline with full row/column insert, delete, move, and alignment operations; `TableActionAvailability` struct drives context-sensitive menu state.
- Rich editor right-click context menu (`components/editor/contextmenu.rs`) — `GtkPopoverMenu` with clipboard actions (cut, copy, paste, delete, select all), undo/redo, indentation, nested table sub-menu, and bookmark toggle.
- Mermaid diagram insert dialog (`ui/dialogs/mermaid.rs`) — 6 diagram type templates with live pure-Rust preview, 350 ms debounced updates, and inline error display.
- Table insert dialog (`ui/dialogs/tables.rs`) — configurable column and row count, optional header row, per-column alignment selection, and Markdown output.
- Slider deck insert dialog (`ui/dialogs/sliderdeck.rs`) — GTK `ListView`-based slide manager supporting up to 20 slides, optional auto-advance timer, and Markdown output.
- Platform mention insert dialog (`ui/dialogs/mention.rs`) — platform-aware input validation for GitHub, GitLab, Reddit, and Mastodon; renders platform profile links in the preview.
- Welcome screen wizard (`ui/dialogs/welcome_screen.rs`) — GTK `Assistant`-based first-run flow with language selection and telemetry opt-in.
- Expanded settings dialog with dedicated tabs: Appearance, Editor, Layout, Language, Markdown, Debug, and Advanced.
- Editor font and display configuration manager (`components/editor/display_config.rs`) — `EditorConfiguration` wrapping `EditorDisplaySettings` with cached monospace font loading.
- Chunked LSP syntax highlighting (`components/editor/lsp_integration.rs`) — highlights applied in batches of 400 via GLib idle callbacks to prevent main-thread frame stutter.
- Window size and position persistence — window state is saved and restored via `SettingsManager` on startup (`logic/window_state.rs`).
- Split pane ratio persistence — saved split ratio is restored with retry logic on startup (`logic/split_state.rs`).
- Mermaid diagram CSS module (`ui/css/mermaid.rs`) — theme-aware stylesheet for rendered diagrams.
- AI component scaffold (`components/ai/`) — reserved module with an `AiAssistant` trait specification for future in-editor AI assistance.
- Collaboration component scaffold (`components/collab/`) — reserved module with a `CollabBackend` trait specification for future real-time collaboration.

### Changed
- CSS system expanded from 5 to 14 modules: added `buttons`, `controls`, `dialog`, `list`, `mermaid`, `radio`, `settings`, `syntax`, and `textfield` modules.
- Preview code syntax highlighting now uses Syntect (Solarized Light / Monokai Dark themes) via `logic/syntax_highlighter.rs`.

## [0.18.0] - 2026-02-09

**Uses:** Core 0.18.0

### Added
- UI localization system backed by `assets/language/*.toml`, with per-key fallback to built-in English defaults.
- German (de) UI translation.
- Localization documentation for translators/contributors (language guide + language matrix).
- First-run Welcome screen with language selection and telemetry information.
- New Settings tabs (Editor, Layout, Appearance, Language, Markdown, Advanced, Debug) with live UI language switching.
- Reusable custom titlebar component for dialogs/aux windows, with SVG window controls.

### Changed
- Settings dialog now updates labels/tooltips in-place when the UI language changes (avoids rebuilding the widget tree).
- Search & Replace window was refactored and restyled (match count overlay, translated UI; Windows uses a no-WebView version).
- Save changes confirmation dialog was redesigned and now uses the shared custom titlebar + translated text/tooltips.
- Windows portable packaging script now ships `config/` + `data/` folders alongside the executable for portable mode.

### Fixed
- Reduced instability when switching UI language at runtime by avoiding widget-tree rebuilds in settings-related UI.

## [0.17.1] - 2026-02-04

### Added
- Platform-agnostic scroll synchronization API ensuring consistent behavior across Windows (wry/WebView2) and Linux (webkit6).
- Enhanced conditional compilation guards to eliminate cross-platform build warnings.

### Changed
- Optimized preview scroll event handling with reduced JavaScript overhead for improved performance on both platforms.
- Refined cross-platform compilation with explicit `cfg(target_os)` attributes throughout the codebase.
- Improved WRY WebView integration with proper API stub implementations for Windows-Linux feature parity.

### Fixed
- Resolved Windows preview mouse-wheel scrolling issue when cursor hovers over heading elements (H1-H6).
- Corrected Windows portable build script OS detection logic to handle PowerShell version differences.
- Eliminated unused import warnings on Linux builds through targeted conditional compilation.

## [0.17.0] - 2026-02-03

### Added
- **Platform-specific workspace files** - separate VS Code configurations for Linux and Windows.
- **Windows native file dialogs** using `rfd` crate (replaces GTK dialogs on Windows).
- **Enhanced editor UI module** with platform-conditional WebView implementations.
- **Bidirectional scroll synchronization** between editor and preview.
- **Dynamic CSS theming** for scrollbars and paned separators based on editor theme colors.
- **Smooth HTML updates** - reduced flickering during editing with debounced rendering.

### Changed
- **Refactored editor UI** into dedicated `components/editor/ui.rs` module (1527 lines).
- **Debounced processing** - preview rendering (400ms), LSP highlighting (250ms), extension processing (400ms).
- **All `cfg` attributes** now use explicit `target_os` conditions instead of negative conditions.
- **WebView implementation** is now platform-specific: `webkit6` on Linux, `wry` on Windows.

### Fixed
- **Removed duplicated `cfg` attributes** in webkit6 modules.
- **Eliminated unnecessary clone operations** on Copy types.
- **Replaced lazy evaluation** with direct values where appropriate.
- **Fixed useless format! macros** replaced with `.to_string()`.

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
