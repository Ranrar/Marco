# Changelog
All notable user-visible changes to **Polo** are documented here.

This project follows **Semantic Versioning** and uses the **Keep a Changelog** format.

**Dependency note:** Polo uses **Core** for parsing and rendering. Polo releases reference the Core version they ship with.

Version scheme note: versions are reconstructed as `0.YY.ZZ` from git history using date-based release groupings starting at the first point where Core, Marco, and Polo co-exist in the repository (2025-10-18).

## [Unreleased]

### Added
- **Preview zoom**, ported over from Marco's editor: an in-page toolbar (`+`/`−`/reset, bottom-right corner of the preview, revealed on hover), `Ctrl+=`/`Ctrl+-`/`Ctrl+0` keyboard shortcuts, and `Ctrl+scroll wheel`. Deliberately session-only — the zoom level is not written to settings, since Polo opens a different document on every launch rather than one long-lived editing session. _(2026-08-25)_

## [0.25.0] - 2026-08-20

_Covers the `unified-webviewer` branch: commits 31a6aac (2026-07-14) through 2d34df7 (2026-08-18), plus the fixes dated 2026-08-20 below._

**Uses:** Core 1.3.2

> **Breaking change (Linux only).** Every installed path and per-user
> directory has been renamed to resolve the long-standing collision with the
> unrelated MATE window manager package also called `marco`
> ([#41](https://github.com/Ranrar/Marco/issues/41)). dpkg detects conflicts by
> file path rather than package name, so the previous layout made the `.deb`
> impossible to install on any MATE system. **There is no migration:** existing
> settings under the old directories are not read. Copy anything you want to
> keep across by hand.
>
> | | Before | After |
> |---|---|---|
> | Editor binary | `/usr/bin/marco` | `/usr/bin/markdowncomposer` |
> | Viewer binary | `/usr/bin/polo` | `/usr/bin/markdownviewer` |
> | Editor config / data / cache | `~/.config/marco/` etc. | `~/.config/markdowncomposer/` etc. |
> | Viewer config / data / cache | `~/.config/polo/` etc. | `~/.config/markdownviewer/` etc. |
> | Shared assets | `/usr/share/marco/` | `/usr/share/markdowncomposer/` |
> | Download | `marco-suite_<version>_…` | `markdown-composer-and-viewer_<version>_…` |
>
> Desktop entries, man pages and icons are renamed to match. The applications
> are still called Marco and Polo everywhere they are shown to you; only the
> file and directory names change. Windows paths are unaffected.


### Added
- **Windows installer.** The new `_setup.exe` installs Polo along with Marco, adds a Start Menu entry for each, offers optional desktop shortcuts, and registers an uninstaller. It installs per user by default and needs no administrator rights. The portable zip still ships too, unchanged. _(2026-08-18)_
- **Find in page.** A find bar drops down from the toolbar (search icon, or Ctrl+F) with Highlight All, Match Case, Match Diacritics and Whole Word options, previous/next buttons and a live match count. It is implemented in JavaScript (`find_engine`) against the rendered document and shared with Marco, so both apps search the preview the same way. _(2026-07-30)_
- **Drag and drop.** Dropping a Markdown file onto the window opens it, with a themed drop target shown while you drag — over the empty "no file loaded" state and over an already-rendered document alike. The handler sits on the webview itself rather than a GTK `DropTarget`, because the native webview would otherwise swallow the event before GTK ever saw it. _(2026-07-30)_
- Hovered links now show their full address in an in-page HTML/CSS tooltip instead of a native one. Native tooltips are suppressed by moving `title` attributes onto data attributes, which also avoids a crash this triggered on Windows, where the tooltip window collided with the WebView2 child HWND. _(2026-07-30)_
- File tree side panel: browse the directory containing the open document, expand folders inline, and click any Markdown file to open it. Includes an incremental search field that filters the tree as you type. _(2026-07-30)_
- Sidebar coordinator so the Table of Contents and the new file tree behave as one sidebar: opening either closes the other, and each toggle works independently, so the two panels can no longer fight over the same space. _(2026-07-30)_

### Changed
- Updated to `marco-core` 1.3.2. _(2026-08-17)_
- Updated to `marco-core` 1.3.1, and to the `gtk4-webkit6` fork of `wry` 0.56.1 (from 0.55.1). _(2026-08-16)_
- **Unified webview backend**, shared with Marco. Polo's preview is now the same `PlatformWebView` on both platforms — WebKitGTK as a native GTK4 widget on Linux, WebView2 as a child window on Windows — replacing the previous platform-specific implementations. _(2026-07-16 – 2026-07-18)_
- Rendered HTML and local assets are served over a `polo-preview://` custom protocol on both platforms, so relative image paths in a document resolve the same way everywhere without a base-URI call. _(2026-07-16)_
- Window-control, menu and toolbar SVG icons are now inline string constants rendered through the shared icon entry point, matching Marco's handling and rasterising at a fixed supersample factor for consistent output across display backends. _(2026-07-18)_
- Menu and toolbar styling moved into a dedicated `menu_and_toolbar` CSS module. _(2026-07-30)_
- Updated to `marco-core` 1.3.0. _(2026-07-18)_

### Fixed
- **Linux: a packaged install shows Polo's own icon again.** Shared with Marco: the window asked GTK for an icon named `polo`, while the rename above installs it as `markdownviewer`, so the window, taskbar and alt-tab entry fell back to a generic icon on an installed system. Both names now match what the package installs, confirmed at all eleven packaged sizes. Wayland sessions were unaffected, since the shell reads the icon from the `.desktop` file. _(2026-08-20)_
- A rendered document could get stuck on screen after an unrelated error. The HTML handed to the webview lives in a shared buffer behind a mutex; if a panic elsewhere poisoned that lock, an update was dropped with only a log line and the protocol handler answered later loads with a bare "Content unavailable" page, so the window kept showing the previous document with no way back short of a restart. Both sides now recover from a poisoned lock. _(2026-08-20)_
- **Windows: an installed copy no longer keeps its settings inside the install directory.** Shared with Marco: a per-user install lives somewhere writable by the installing user, which the portable-mode check mistook for a portable copy, so Polo's settings and recent files were written into the install folder and removed on uninstall. An installed copy is now recognised as such and stores its settings in `%APPDATA%\marco`, the directory both apps share on Windows. _(2026-08-18)_
- **The window title now names the file you are viewing** ([#37](https://github.com/Ranrar/Marco/issues/37)), as `Polo - filename.md`, and updates when you open a different one. Polo draws its own titlebar, so the native `set_title` the open handler was calling is never the text on screen — the visible label was built afterwards and never told. Both are now updated together. With no file open the title is just `Polo`. _(2026-07-30)_
- **Open in Marco** looked for an executable named `marco` next to Polo and then on `PATH`. Neither exists once installed under the new Linux names, so the button would have failed on a packaged install. It now tries the installed name and the development name, alongside the binary first and then on `PATH`. _(2026-08-16)_

## [0.24.1] - 2026-07-08

**Uses:** Core 1.2.0

### Added
- Four new HTML preview themes shared with Marco: **Nord**, **Gruvbox**, **Solarized**, and **Sepia**. They appear in Polo's theme selection alongside the existing themes.

### Changed
- Updated to `marco-core` 1.2.0.
- The shared `github` preview theme was recoloured to match GitHub's current Primer palette; documents rendered with this theme now use updated text, border, and secondary-text colours in both light and dark mode.
- All HTML preview themes now use `--body-max-width: 100%` instead of a fixed pixel cap, so preview content is no longer artificially width-constrained.
- Polo's theme menu now uses the same metadata-aware theme listing as Marco (`list_html_view_themes`), so it displays each theme's `--theme-name` (e.g. "GitHub" instead of "github") instead of a raw filename-derived label.

### Fixed
- Fixed unreadable code blocks in light mode for the `academic`, `gruvbox`, `nord`, `sepia`, and `solarized` themes, which affects Polo's rendering the same way it did Marco's since both share these theme files.
- Fixed `neutral.css`'s table-of-contents sidebar colours, which were leftover values from an old GitHub palette instead of neutral's own palette.

### Removed
- Removed the `minimal` HTML preview theme (redundant with `academic` and the new `sepia` theme).
- Removed the superseded `list_available_themes_from_path` helper in favor of the shared, metadata-aware `list_html_view_themes`.

## [0.24.0] - 2026-06-04

**Uses:** Core 1.1.0

### Added
- Icon toolbar below the titlebar with buttons for Open file, Open in Marco editor, Toggle TOC, Print, and Light/Dark mode toggle. Icons use the Tabler icon set and adapt to the active theme color.
- Table of Contents side panel: auto-populated from document headings, click any entry to scroll the preview to that section. Panel can be toggled from the toolbar or the View menu.
- File → Print (Ctrl+P): opens the native print dialog for the current document preview. On Linux uses WebKit's `PrintOperation`; on Windows uses the embedded WebView2 print API.
- File → Open Recent submenu: lists recently opened files; selecting one reopens the document immediately. The list can be cleared via File → Open Recent → Clear Recent Files.
- Rendering progress overlay shown over the preview while large documents are loading. The overlay displays a framed "Rendering…" indicator with an indeterminate progress bar in the app's blue accent color and stays visible until the preview has fully painted. The overlay follows the active light/dark theme.
- File-based logging, matching Marco's logger. Polo now writes daily log files under `log/YYYYMM/YYMMDD.log` so startup, file open, render, and error events can be inspected after the fact.

### Changed
- Updated to `marco-core` 1.1.0.
- The `marco-core` crate now lives in its own repository (https://github.com/Ranrar/marco-core) and is consumed from crates.io. No user-visible behavior change; pinned via `[workspace.dependencies.marco-core]` in the root `Cargo.toml`.
- CSS theming system was rewritten as a programmatic Rust generator, aligned with Marco's palette constants, so light and dark mode colors are consistent across the two apps.

### Fixed
- Opening large markdown files no longer leaves the preview blank or appears to hang. The viewer now waits for the WebView's load-finished signal before hiding the rendering indicator, so the progress overlay remains visible until the document is actually painted.

## [0.23.2] - 2026-04-28

**Uses:** Core 1.0.2

### Added
- New CI workflow for publishing the shared engine crate `marco-core` to crates.io.
- Polo now consumes the shared print/export CSS from `marco-shared`, keeping its rendering pipeline aligned with Marco's live print and export output.

### Changed
- Windows portable packaging script now resolves the repository root from the script location, so packaging works the same in CI release workflows and manual runs.
- Workspace crate layout was refactored: `core` was renamed to `marco-core`, and shared platform/app logic and assets were extracted into `marco-shared`, which Polo now consumes directly.
- Cross-platform packaging/build scripts and docs were aligned with the refactored crate layout for Linux and Windows artifacts.
- Source file permission metadata was normalized to avoid accidental executable bits on non-executable source/content files across platform checkouts.

### Fixed
- Debian package dependency metadata now supports newer Ubuntu-family runtime naming by accepting `libxml2-16` as an alternative to `libxml2`.
- Linux package build script now correctly detects Cargo's configured target directory when collecting built binaries for the package payload.

### Removed
- Removed the legacy workspace `core` crate path in favor of the `marco-core` + `marco-shared` split.

### Security
- Verified mitigation status for GHSA-82j2-j2ch-gfr8 on Linux and Windows release targets: dependency graph resolves to patched `rustls-webpki` 0.103.13.
- Updated transitive `rand` to 0.8.6 in the workspace lockfile.

## [0.23.1] - 2026-04-14

**Uses:** Core 0.23.1

### Changed
- Rust toolchain updated to 1.94.1 (MSRV bumped from 1.93.0).
- GTK ecosystem upgraded to webkit6 0.6.1 / gtk4 0.11.2 / glib 0.22.5 series.

### Security
- Updated `rand` dependency to address unsound behavior (GHSA-cq8v-f236-94qc).

## [0.23.0] - 2026-04-12

**Uses:** Core 0.23.0

### Changed
- Updated to Core 0.23.0.
- Inherited preview rendering updates from Core: heading text acts as the direct permalink target, decorative anchor/link icon adorners are removed, and line-break parsing/rendering is more consistent with CommonMark hard-break behavior.

### Fixed
- Inherited Core parser fixes for nested-bracket links (for example image-in-link syntax) and NBSP spacer paragraph handling.

## [0.22.0] - 2026-04-08

**Uses:** Core 0.22.0

### Added
- Local link prompt — clicking a local file link in the HTML preview prompts to open that file in Polo; the dialog's cancel action uses a distinct button style to differentiate it visually from the primary open action.

### Changed
- Updated to Core 0.22.0.

## [0.21.0] - 2026-03-13

**Uses:** Core 0.21.0

### Changed
- Updated to Core 0.21.0 (in-process intelligence engine replacing `lsp/`, corrected image and footnote definition parser spans, new `EditorSettings` fields for diagnostics feature control).

## [0.20.0] - 2026-03-04

**Uses:** Core 0.20.0

### Added
- Platform webview abstraction (`components/viewer/platform_webview.rs`) — unified interface over the underlying webview backend for cross-platform viewer support.
- Empty state UI (`components/viewer/empty_state.rs`) — visual placeholder shown when no document is loaded.

### Changed
- Updated to Core 0.20.0 (centralized settings manager, pure-Rust Mermaid and KaTeX rendering, unified HTML preview document builder).

## [0.18.0] - 2026-02-09

**Uses:** Core 0.18.0

### Changed
- Updated to Core 0.18.0 (more reliable portable-mode detection and improved system-locale detection used for default configuration behavior).

## [0.17.1] - 2026-02-04

### Added
- Platform-native file picker integration: Windows uses native OS file dialog (`rfd` crate), Linux uses GTK file chooser for consistent OS-appropriate user experience.

### Changed
- Enhanced cross-platform compilation with refined conditional import statements and explicit platform guards.

## [0.17.0] - 2026-02-03

### Added
- **Platform-specific workspace files** - separate VS Code configurations for Linux and Windows.
- **Enhanced platform support** via core library platform abstraction.

### Changed
- **Improved path resolution** using new core platform module for config/data directories.

## [0.16.0] - 2026-02-02

### Added
- **Full cross-platform support** for Windows and Linux.
- Windows builds now use `wry` (WebView2) for HTML rendering.
- Linux builds use `webkit6` for HTML rendering.
- Windows icon embedding using `embed-resource` crate with `polo.rc` resource script.
- Platform-specific conditional compilation for webview backends.

### Changed
- Build system now supports both x86_64-pc-windows-msvc and x86_64-unknown-linux-gnu targets.
- Updated dependencies to match core 0.16.0 and marco 0.16.0.

## [0.15.2] - 2026-01-30

### Added
- Replaced legacy IcoMoon icon-font glyphs with **inline SVG icons** in dialog controls and menu elements.
- Introduced SVG-based window control icons with hover/active states and HiDPI supersampling.

### Changed
- CSS and button factories updated to rely on SVG rendering helpers; colors and hover/pressed behavior aligned with Marco's palette.

### Fixed
- Resolved pixelation and hover/press color glitches by using 2x rasterization and consistent event-driven texture swaps.

### Removed
- Legacy icon-font usage removed; packaging updated to remove `ui_menu.ttf` from packaged assets.

### Security
- Nothing yet.

## [0.15.1] - 2026-01-26

**Uses:** Core 0.15.1

### Added
- SVG icon support for window controls (minimize, maximize/restore, close)
  - Crisp 2x rendering for HiDPI displays
  - Event-based hover and active color states (#2563eb blue hover, #1e40af active)
  - Centralized ICON_SIZE constant for easy maintenance

### Changed
- Consolidated duplicate SVG rendering code into shared `render_svg_icon()` function
- Improved code organization in menu.rs (reduced from ~850 to ~776 lines)
- Window control buttons now use Material Design 3 inspired color palette
  - Light mode: subtle gray-blue (#4a5568) to blue hover
  - Dark mode: light gray (#9ca3af) to blue hover
- Enhanced color palette in CSS constants with window control states

### Fixed
- Window control icon colors no longer conflict between CSS filters and event handlers
- Arc<ParentWindowHandle> clippy warning (changed to Rc for single-threaded Windows UI)
- SVG icon pixelation issue resolved with 2x supersampling

## [0.15.0] - 2026-01-25

**Uses:** Core 0.15.0

### Added
- Cross-platform path support for asset discovery and file operations

### Changed
- File operations now fully compatible with Windows file paths

### Fixed
- Nothing yet.

### Removed
- `anyhow` dependency removed

## [0.14.0] - 2026-01-18

**Uses:** Core 0.14.0

### Added
- Preview rendering support for Marco tab blocks (`:::tab` / `@tab ...`) via the shared Core HTML renderer.
- Preview styling for extended GitHub-style custom-header admonitions (quote-styled callouts with theme-primary title color).
- Preview rendering support for Marco_sliders slideshow decks (`@slidestart[:tN]` … `@slideend`) via the shared Core HTML renderer.

## [0.13.3] - 2026-01-17

**Uses:** Core 0.13.3

### Added
- New Polo logo (application icon), used in the titlebar and installed for desktop integration.

### Changed
- Debian packaging (`install/build_deb.sh`) was improved (dependency checks, deterministic `--locked` builds, icon installation/scaling, and additional build/versioning options).
- Linux desktop entry now uses the system icon name `polo`.

## [0.13.2] - 2026-01-15

**Uses:** Core 0.13.2

### Changed
- Updated to the latest Core engine (no Polo-specific changes documented).

## [0.13.1] - 2026-01-14

**Uses:** Core 0.13.1

### Changed
- Reduced build footprint by removing unused direct dependencies.

### Security
- Tuned DevSkim/code-scanning configuration to ignore vendored/spec fixture content (improves signal-to-noise in Security scans).

## [0.13.0] - 2026-01-14

**Uses:** Core 0.13.0

### Added
- Syntax-highlighted code rendering.
- Emoji shortcodes in rendered output.

## [0.12.0] - 2026-01-13

**Uses:** Core 0.12.0

### Changed
- Updated to the latest Core engine (no Polo-specific changes documented).

## [0.11.0] - 2026-01-12

**Uses:** Core 0.11.0

### Changed
- Packaging/build workflow for Linux installs was updated and simplified.

## [0.10.0] - 2026-01-11

**Uses:** Core 0.10.0

### Added
- GitHub Flavored Markdown tables (via Core).
- Additional inline formatting extensions (via Core).

## [0.9.0] - 2025-10-28

**Uses:** Core 0.9.0

### Fixed
- More robust handling of autolinks vs inline HTML (via Core).

## [0.8.0] - 2025-10-27

**Uses:** Core 0.8.0

### Changed
- Updated to the latest Core engine (no Polo-specific changes documented).

## [0.7.0] - 2025-10-25

**Uses:** Core 0.7.0

### Changed
- Updated to the latest Core engine (no Polo-specific changes documented).

## [0.6.0] - 2025-10-24

**Uses:** Core 0.6.0

### Changed
- Theme appearance was standardized for more consistent UI colors.

## [0.5.0] - 2025-10-23

**Uses:** Core 0.5.0

### Changed
- Linux install flow moved toward packaged installation.

### Removed
- Removed the user-local install/uninstall workflow in favor of packaged installation.

## [0.4.0] - 2025-10-21

**Uses:** Core 0.4.0

### Changed
- Updated to the latest Core engine (no Polo-specific changes documented).

## [0.3.0] - 2025-10-20

**Uses:** Core 0.3.0

### Changed
- Updated to the latest Core engine (no Polo-specific changes documented).

## [0.2.0] - 2025-10-19

**Uses:** Core 0.2.0

### Changed
- General improvements to behavior and stability (based on commit messaging; details not specified).

## [0.1.0] - 2025-10-18

**Uses:** Core 0.1.0

### Added
- Initial integration of the shared Core engine.
