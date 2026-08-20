# Changelog
All notable user-visible changes to **Marco** are documented here.

This project follows **Semantic Versioning** and uses the **Keep a Changelog** format.

**Dependency note:** Marco uses **Core** for parsing and rendering. Marco releases reference the Core version they ship with.

Version scheme note: versions are reconstructed as `0.YY.ZZ` from git history using date-based release groupings starting at the first point where Core, Marco, and Polo co-exist in the repository (2025-10-18).

## [Unreleased]

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
- **Windows installer.** Releases now ship a `_setup.exe` alongside the portable zip, built with Inno Setup. It installs per user by default, so it needs no administrator rights, and can be run elevated for an all-users install instead. It creates Start Menu entries for Marco and Polo, offers optional desktop shortcuts, and registers a proper uninstaller. The portable zip is unchanged and remains the right choice if you want everything to live in one folder you can carry around. _(2026-08-18)_
- **"System default" colour mode.** Settings → Appearance → Light/Dark Mode gains a third option that follows the operating system, and follows it *live* — flipping your desktop between light and dark reskins Marco as it happens, without a restart. On Linux this reads the XDG Desktop Portal's `color-scheme` preference (the same key GNOME, KDE and Flatpak sandboxes publish); on Windows it reads `AppsUseLightTheme` and watches the registry key for changes. A desktop that exposes neither still resolves correctly at startup, it just stops tracking afterwards. _(2026-08-17)_
- **Dead link and image detection.** Every local link, image and link-definition destination is resolved against the document's own directory and checked against the filesystem. A destination with nothing at it is reported as a warning — underlined in the editor, listed in the footer issues panel, and explained on hover — as `MD206` for a broken link and `MD404` for a broken image. Remote URLs are never fetched (that would mean a network request per keystroke), `#fragments` point inside the rendered document rather than at a file, and an unsaved document is not judged at all, because a relative path has no directory to resolve against until it is saved. _(2026-08-17)_
- Clicking a link whose target is missing now says so, naming the path it looked for, instead of opening a confirmation dialog whose only possible outcome was a blank page. _(2026-08-17)_
- **Save As rebases local paths.** Saving a document to a new location rewrites every local link, image and link-definition path so it still points at the same file: the absolute paths an unsaved document carries become relative ones, and a document moved elsewhere gets its relative paths recomputed. The editor is updated to match what was written, as a single undoable action. Remote URLs, data URIs, in-document anchors and paths that cannot be resolved are left exactly as you wrote them. _(2026-08-17)_
- Undo and redo buttons in the toolbar, between the line-numbers toggle and the text-formatting group. They drive the existing `app.undo` / `app.redo` actions, so they grey out on their own when there is nothing to undo and stay in step with the Edit menu and Ctrl+Z / Ctrl+Y. _(2026-08-16)_
- **Browse** in the Link, Reference link and Image popovers now works on a document that has never been saved. It was previously disabled, with no way to discover why — GTK4 delivers no pointer events to insensitive widgets, so the tooltip explaining it could never appear. With no document directory for a relative path to resolve against it inserts the file's absolute path; a saved document still gets a `./relative` one. _(2026-08-16)_
- In-page find for the preview, implemented in JavaScript (`find_engine`) and driven from the existing search window. Supports case-sensitive and whole-word matching, highlights every hit in the rendered document, and reports a live "K of N" count back to the search UI. _(2026-07-16, 2026-07-30)_
- Hovered links now show their full address in an in-page HTML/CSS tooltip instead of a native popover. Native tooltips are suppressed by moving `title` attributes onto data attributes, which also avoids a crash this triggered on Windows. _(2026-07-30)_
- Shared drag-and-drop overlay for webview pages, so dropping a file onto the preview presents the same themed drop target on both platforms. _(2026-07-30)_
- Toolbar colour-mode toggle now swaps its own icon to match the active light/dark theme. _(2026-07-18)_

### Changed
- Updated to `dark-light` 3.0.0, which drops the `ashpd` and `async-std` dependencies in favour of talking to the XDG Desktop Portal over D-Bus directly. This is the upstream release Marco was waiting for, so the `[patch.crates-io]` pin to a git revision — added because `ashpd 0.10` would not compile on Ubuntu 24.04 CI — has been removed. _(2026-08-17)_
- Link and image path handling — classification, relative/absolute conversion, percent-encoding, `#fragment` splitting and Windows drive-path detection — now lives in one shared module (`marco-shared::logic::link_path`) used by the toolbar popovers, Save As and the new diagnostics, replacing three separate partial implementations that disagreed on edge cases. _(2026-08-17)_
- Toolbar buttons now signal that they are unavailable by greying the icon itself, instead of filling in a background plate and dimming the whole button. _(2026-08-16)_
- The Link, Reference link and Image popovers no longer close when you click outside them; use Escape, Cancel or Ok. They are no longer autohide popovers, because an autohide popover holds an input grab that blocks the modal file chooser behind **Browse** — the old workaround hid the popover and re-showed it afterwards, which is what lost it. A side benefit is that a stray click can no longer discard a half-filled form. _(2026-08-16)_
- Updated to `marco-core` 1.3.2. _(2026-08-17)_
- Updated to `marco-core` 1.3.1, and to the `gtk4-webkit6` fork of `wry` 0.56.1 (from 0.55.1). _(2026-08-16)_
- **Unified webview backend.** Linux and Windows now share a single `PlatformWebView` built on the gtk4-webkit6 fork of `wry` — WebKitGTK as a native GTK4 widget on Linux, WebView2 as a child window on Windows. The separate Linux/Windows implementations of the preview, search window, detached preview window and print path have been collapsed into one code path, so behaviour no longer diverges between platforms. _(2026-07-16 – 2026-07-18)_
- Preview HTML and its local assets are now served over a `marco-preview://` custom protocol on both platforms. Relative image references (`![alt](./pic.png)`) resolve through ordinary URL resolution against the document's own URL, so no `<base>` tag or backend-specific base-URI call is involved. _(2026-07-16)_
- PDF export consolidated across platforms: the Windows path now goes through the same print driver structure as Linux, with paper size and margin handling aligned to the Linux implementation. _(2026-07-16)_
- Window-control and toolbar SVG icons are now inline string constants rendered through a single `render_svg_icon` entry point, replacing the `WindowIcon` enum. Icons rasterise at a fixed supersample factor so they render consistently across display backends. _(2026-07-18)_
- Updated to `marco-core` 1.3.0. _(2026-07-18)_
- Slide-deck IDs are handled more robustly when generating deck markup. _(2026-07-18)_
- Viewer modules renamed to say what they are rather than which backend they came from: `wry_platform_webview` → `platform_webview`, `wry_find` → `find_engine`, `wry.rs` → `preview_helpers`, `print_driver` → `print_driver_linux`, and the two detached-window modules to `detached_window_linux` / `detached_window_windows`. _(2026-07-16)_

### Fixed
- **Linux: a packaged install shows Marco's own icon again.** The window asked GTK for an icon named `marco`, but the rename above installs it as `markdowncomposer`, so the lookup failed — silently, as icon-theme lookups do — and the window, taskbar and alt-tab entry fell back to a generic icon. Verified against the built `.deb`: the new name resolves at all eleven packaged sizes, each to its own size directory. Wayland sessions never saw this, because the shell takes the icon from the `.desktop` file, which was already correct. _(2026-08-20)_
- The preview no longer risks a hard crash after an unrelated error. The rendered HTML is handed to the webview through a shared buffer behind a mutex, and both the write on every preview update and the read inside the custom-protocol handler unwrapped that lock. One panic while the lock was held poisoned it, so every later preview update panicked too — the second time from inside the protocol handler, unwinding through native WebKit/WebView2 frames. Both sides now recover from a poisoned lock and carry on. _(2026-08-20)_
- **Windows: an installed copy no longer keeps its settings inside the install directory.** Portable mode is detected by looking for a `config/` folder next to the executable, and failing that, by checking whether that directory is writable at all. A per-user install lands under your own profile, which is writable by definition, so every installed copy was being mistaken for a portable one — settings, themes and recent files went into the install folder and were deleted on uninstall. The check now recognises the uninstaller that an installed copy always has next to it, and treats such a copy as installed, storing settings in `%APPDATA%\marco` as intended. _(2026-08-18)_
- On Linux, starting Marco in dark mode left GTK's own rendering — file choosers, native menus, anything not covered by Marco's CSS — in light theme until you changed the theme once. The GTK dark preference was only ever set on a theme *change*, never at startup. _(2026-08-17)_
- Clicking a relative link in the preview loaded the raw Markdown as a blank page instead of opening the file in the editor. Preview documents are served over the `marco-preview://` protocol, so a relative link resolves against *that* origin, but only `file://` links were being recognised as local. _(2026-08-17)_
- Links to files with non-ASCII names (e.g. `unicode-✓.md`) could not be opened: the percent-escapes such a name arrives as were decoded one byte at a time rather than as UTF-8, producing a path that does not exist. _(2026-08-17)_
- On Windows, images in an unsaved document did not render in the preview. An unsaved document stores image paths as absolute (`C:/Users/…/pic.png`), and a URL parser reads the leading `C:` as a scheme. Applies to both first render and incremental updates. _(2026-08-17)_
- Choosing a file through **Browse** left the popover closed and the picked path nowhere to be seen, so the insert could never be completed. _(2026-08-16)_
- After **Browse** filled in a local path, **Ok** went insensitive with no visible explanation: a local target makes the alt/label field mandatory, and it was empty. The field is now seeded from the chosen file's name and stays editable. _(2026-08-16)_
- Using undo or redo from the toolbar left keyboard focus on the button, so the next keystroke went nowhere and you had to click back into the text. Focus now returns to the editor. _(2026-08-16)_
- Dismissing one of the insert popovers could leave keyboard focus stranded inside the hidden popover, leaving the editor unable to accept typing while pointer-driven toolbar buttons still worked. _(2026-08-16)_

### Removed
- The `wry_print_to_pdf` module; its behaviour now lives directly in the Windows print driver. _(2026-07-16)_
- `WEBVIEW_MIGRATION.md` and the accompanying gap-analysis document, which described work that is now finished. _(2026-07-18)_

## [0.24.2] - 2026-07-12

**Uses:** Core 1.2.0

### Added
- Locale files may now use a region-qualified code (`{lang}-{REGION}`, e.g. `zh-CN` / `zh-TW`) alongside the existing bare ISO 639-1 codes, so macrolanguages with major script/dialect splits — starting with Chinese — can ship distinct Simplified/Traditional translations instead of being forced into a single code.
- System-locale auto-detection ("System Default" in Settings → Language) now preserves the region subtag when the OS/environment reports one (e.g. `zh_CN` → `zh-CN`, including Windows' `zh-Hans-CN` / `zh-Hant-TW` forms), so it can auto-select a region-qualified locale on first launch instead of only ever resolving to the bare language.
- ~40 UI strings that were previously hardcoded in English — regardless of the selected language — are now translatable: the context menu's row/column insert items, the Modules and Format-table menu entries, the TOC panel header and empty state, footer issue count and diagnostics text, the TOC toggle tooltip, several View-menu items, export progress dialog text (including all pipeline phase labels), the "Select Local File" picker title, the export theme label and paper-size tooltip, the open-local-file confirmation dialog, print-preview's page-columns setting, the layout-switcher tooltip, and a few Settings/Advanced strings.

### Fixed
- The "Pages per Row" row in print-preview settings was reading the wrong translation keys and would silently show "Show Page Numbers" label/description text regardless of locale.
- The Modules menu's Table / Tab block / Slider deck / Mermaid / Admonition items were hardcoded in English even though matching, already-translated keys existed for them — the menu just never read them.

### Changed
- Loading a locale now cascades through fallbacks instead of jumping straight to English: a region-qualified code that has no matching file (e.g. a detected `zh-HK`) falls back to its bare language (`zh`) before falling back to English.

## [0.24.1] - 2026-07-08

**Uses:** Core 1.2.0

### Added
- Four new HTML preview themes: **Nord** (arctic/frost palette), **Gruvbox** (warm retro palette), **Solarized** (Ethan Schoonover's precision colour scheme), and **Sepia** (warm, eye-comfort reading theme). All appear automatically in Settings → Appearance → HTML Preview Theme and in the Export dialog's theme picker.
- Theme metadata support: preview theme CSS files can now declare `--theme-name`, `--theme-author`, `--theme-license`, `--theme-version`, and `--theme-description` in `:root`, parsed via `marco-core` 1.2.0's new `parse_theme_metadata`. The theme picker displays `--theme-name` when present, falling back to a filename-derived label otherwise.
- All preview themes are now sorted alphabetically by display name in the Settings and Export theme dropdowns, instead of following arbitrary filesystem read order.

### Changed
- Updated to `marco-core` 1.2.0.
- `github.css` recoloured to match GitHub's current Primer palette (verified against the live `sindresorhus/github-markdown-css` extraction): unified text colour, updated border and secondary-text tones in both light and dark mode.
- All HTML preview themes now use `--body-max-width: 100%` instead of a fixed pixel cap, so preview content is no longer artificially width-constrained.
- Consolidated the overlapping `neutral` / `minimal` / `academic` preview themes down to `neutral` and `academic`, since they had nearly identical colour tokens and differed mainly in font choice.

### Fixed
- Corrected a stale doc comment in every preview theme CSS file that pointed to a nonexistent local `core/src/render/base_css.rs` path; it now correctly references the external `marco-core` crate.
- Removed a dangling reference to the deleted `minimal.css` theme from the Export dialog's fallback theme list.
- Fixed unreadable code blocks in light mode for the `academic`, `gruvbox`, `nord`, and `sepia` themes: plain (unhighlighted) code fences rendered dark text on a near-identical dark background (contrast as low as 1.0:1) because `--bg-pre` was set to a dark colour while `--text-color` stayed dark too. Code block backgrounds in these themes are now light in light mode.
- Fixed the same class of contrast issue in `solarized.css`'s light-mode code blocks, and darkened its `--mark-color` and `--blockquote-text` tokens (custom additions not part of the original Solarized spec) to meet accessible contrast.
- Fixed `neutral.css`'s table-of-contents sidebar, which was still using leftover colours from an old GitHub palette (in both light and dark mode) instead of its own blue/grey palette.

### Removed
- Removed the `minimal` HTML preview theme (redundant with `academic` and the new `sepia` theme).

## [0.24.0] - 2026-06-04

**Uses:** Core 1.1.0

### Added
- New multi-layer parse and render cache (HTML, AST, section, TOC, and diagnostics layers) backed by Moka TinyLFU. Frequently edited sections are served from cache without re-parsing, reducing CPU usage on large documents.
- Section-based incremental preview rendering: only the document sections whose content changed since the last keystroke are re-rendered and patched into the DOM. Full-page rebuilds now occur only on the very first render of a document.
- Rendering progress overlay shown over the preview while large documents are loading. The overlay displays a framed "Rendering…" indicator with an indeterminate progress bar in the app's blue accent color and stays visible until the preview has fully painted, so long opens no longer look like a frozen window. The overlay follows the active light/dark theme.

### Changed
- Updated to `marco-core` 1.1.0. Internal JS bridge identifiers (`MarcoCorePreview`, `mc_paged_ready`, `mc-content-container`) were updated to match the new Core API; no user-visible behavior change.
- The `marco-core` crate now lives in its own repository (https://github.com/Ranrar/marco-core) and is consumed from crates.io. No user-visible behavior change; pinned via `[workspace.dependencies.marco-core]` in the root `Cargo.toml`.
- Localization coverage expanded across Marco dialogs: the Lists and Mention insert dialogs are now fully translatable, and the German (`de`) locale received the matching strings. Other locales fall back to English for any keys they do not yet provide.
- Language changes made from Settings now apply at runtime to every translated surface — menus, toolbar, footer, dialogs, the untitled-document title, and the custom titlebar tooltips (app icon, layout buttons, and window minimize / maximize / close controls) — without requiring an application restart.

### Fixed
- Dialog strings no longer stayed in English after switching the UI language. The shared dialog translation helper now reads the configured (or system-detected) locale instead of always loading English, so all dialogs render in the active language immediately after a language switch.
- Custom titlebar tooltips (app icon, the four layout-mode buttons, and the window minimize / maximize / close buttons) now update when the UI language is changed at runtime instead of remaining in the language that was active at startup.

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
