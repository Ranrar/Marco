# Changelog
All notable user-visible changes to **Polo** are documented here.

This project follows **Semantic Versioning** and uses the **Keep a Changelog** format.

**Dependency note:** Polo uses **Core** for parsing and rendering. Polo releases reference the Core version they ship with.

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
