# Changelog
All notable user-visible changes to **Polo** are documented here.

This project follows **Semantic Versioning** and uses the **Keep a Changelog** format.

**Dependency note:** Polo uses **Core** for parsing and rendering. Polo releases reference the Core version they ship with.

Version scheme note: versions are reconstructed as `0.YY.ZZ` from git history using date-based release groupings starting at the first point where Core, Marco, and Polo co-exist in the repository (2025-10-18).

## [Unreleased]

### Added
- Servo web engine integration replacing WebKit6 for HTML preview rendering (Linux)
- Explicit subprocess cleanup mechanism via `WebView::cleanup()` method (Unix only)
- Icon loading from system icon theme (`Image::from_icon_name`) for production installs
- Windows build support with proper file:// URL handling
- SIGINT/SIGTERM signal handlers for graceful shutdown on Unix
- Cross-platform file URL generation (Windows: file:///C:/, Unix: file:///)

### Changed
- Web rendering engine changed from WebKit6 to Servo (via servo-gtk bindings, Linux only)
- Icon installation now uses ImageMagick to resize icons to all standard sizes (16x16 through 512x512, Linux)
- Window close handling changed from `connect_destroy` to `connect_close_request` for reliable cleanup
- servo-runner subprocess now properly terminated on window close using `force_exit()` (Unix only)
- File URL handling improved with platform-specific path conversion (backslash to forward slash on Windows)
- Empty state and error rendering now use platform-appropriate file URLs
- servo-gtk dependency path changed from third_party/servo-gtk to /servo-gtk/ (external repository)

### Fixed
- Window titlebar icon missing after .deb installation (now uses system icon theme, Linux)
- servo-runner subprocess orphaned when polo closes (now explicitly killed via force_exit, Unix)
- Icon file naming mismatch in build script (now correctly uses icon_64x64_polo.png and icon_662x662_polo.png, Linux)
- File URL format on Windows (now uses file:/// with forward slashes)

### Removed
- WebKit6 dependency removed in favor of Servo (Linux only, Marco still uses WebKit6)
- third_party directory reference (servo-gtk moved to external repository)

### Security
- Nothing yet.

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
