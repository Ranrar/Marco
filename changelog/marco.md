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
