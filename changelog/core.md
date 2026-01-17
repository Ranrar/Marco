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
