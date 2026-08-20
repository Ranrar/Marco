//! Comprehensive path management system for Marco, Polo, and core.
//!
//! This module provides a structured approach to managing asset paths across:
//! - Different binaries (marco vs polo)
//! - Different modes (development vs installed)
//! - Different asset types (fonts, themes, config, etc.)
//!
//! # Architecture
//!
//! - **core.rs**: Binary detection, mode detection, asset root finding
//! - **shared.rs**: Assets shared between marco and polo (fonts, icons, language)
//! - **marco.rs**: Marco-specific paths (editor themes, UI CSS)
//! - **polo.rs**: Polo-specific paths
//! - **dev.rs**: Development mode helpers (test assets, workspace root)
//! - **install.rs**: Installation mode helpers (system paths)
//!
//! # Usage
//!
//! ```no_run
//! use marco_shared::paths::{MarcoPaths, PathProvider};
//!
//! // Get paths for the appropriate binary
//! let marco_paths = MarcoPaths::new().expect("Failed to initialize paths");
//! let font_path = marco_paths.shared().font("custom.ttf");
//! let theme_path = marco_paths.editor_theme("dark");
//! ```

pub mod core;
pub mod marco;
pub mod polo;
pub mod shared;

pub(crate) mod platform;

// Re-export main types and functions
pub use core::{find_asset_root, get_binary_name, is_dev_mode, AssetError};

/// Directory name used for this application's per-user config, data, cache and
/// installed assets.
///
/// Linux uses `markdowncomposer`. The obvious name, `marco`, is already taken
/// across the filesystem by the unrelated MATE window manager of that name, so
/// `/usr/bin/marco`, `/usr/share/marco/` and friends collide with a package
/// shipped in the Debian/Ubuntu archives — dpkg matches on file path, not
/// package name, which made the .deb impossible to install on any MATE system
/// (<https://github.com/Ranrar/Marco/issues/41>). `marco-suite` was an interim
/// name that only moved the problem.
///
/// **This is a deliberate breaking change.** No legacy directory names are
/// consulted: an upgrade starts from a fresh `markdowncomposer` directory and
/// existing settings under `~/.config/marco/` are not read or migrated. Users
/// who want their old configuration back must copy it across by hand.
///
/// Other platforms keep `marco` — no such collision exists there, and renaming
/// would strand existing users' settings for no benefit.
#[cfg(target_os = "linux")]
pub const APP_DIR_NAME: &str = "markdowncomposer";

/// Directory name used for this application's per-user config, data and cache.
/// See the Linux variant for why the two differ.
#[cfg(not(target_os = "linux"))]
pub const APP_DIR_NAME: &str = "marco";

/// Directory name used for the viewer's per-user config, data and cache.
///
/// Linux uses `markdownviewer`, for the same reason the composer uses
/// `markdowncomposer` — see [`APP_DIR_NAME`]. Naming the directories after
/// what the programs are, rather than after the Marco/Polo pair, keeps them
/// clear of names owned by unrelated packages in the distro archives.
///
/// Like the composer's, this is a breaking change with no migration.
#[cfg(target_os = "linux")]
pub const VIEWER_DIR_NAME: &str = "markdownviewer";

/// Directory name used for the viewer's per-user config, data and cache.
#[cfg(not(target_os = "linux"))]
pub const VIEWER_DIR_NAME: &str = "polo";

/// File name of the composer executable as installed.
///
/// Matches the binary the .deb ships to `/usr/bin`. Kept separate from
/// [`APP_DIR_NAME`] even though the two currently agree on Linux — one names a
/// directory, the other an executable, and conflating them would silently
/// break whichever changed first.
#[cfg(target_os = "linux")]
pub const COMPOSER_EXE_NAME: &str = "markdowncomposer";

/// File name of the composer executable as installed.
#[cfg(not(target_os = "linux"))]
pub const COMPOSER_EXE_NAME: &str = "marco";

/// File name the composer's binary has in a development build.
///
/// Cargo names the artifact after the crate, so `cargo build` always produces
/// `marco` regardless of what the package installs it as.
pub const COMPOSER_DEV_EXE_NAME: &str = "marco";

pub use marco::MarcoPaths;
pub use polo::PoloPaths;
pub use shared::SharedPaths;

use std::path::PathBuf;

/// Installation location type.
///
/// Note: this reflects where the *asset bundle* is loaded from (not where config is stored).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallLocation {
    /// User local installation (e.g. Linux: `~/.local/share/markdowncomposer/`)
    UserLocal,
    /// System local installation (e.g. Linux: `/usr/local/share/markdowncomposer/`)
    SystemLocal,
    /// System global installation (e.g. Linux: `/usr/share/markdowncomposer/`)
    SystemGlobal,
    /// Development mode (not installed)
    Development,
    /// Portable mode: running from a user-writable directory next to the executable
    Portable,
}

/// Return the user configuration directory.
///
/// This directory must be writable for the current user.
pub fn config_dir() -> PathBuf {
    platform::config_dir()
}

/// Return the user data directory.
///
/// This directory is used for user-specific data like recent files, cached webview data, etc.
pub fn user_data_dir() -> PathBuf {
    platform::user_data_dir()
}

/// Return the user telemetry directory.
///
/// This is a cross-platform, user-writable location where queued telemetry
/// events can be stored locally (e.g. `queue.jsonl`).
pub fn telemetry_dir() -> PathBuf {
    user_data_dir().join("telemetry")
}

/// Detect portable mode (returns the portable root directory if detected).
///
/// Portable mode is detected when the application is running from a writable directory
/// (or from a layout that has a writable `config/` directory next to the executable).
pub fn detect_portable_mode() -> Option<PathBuf> {
    platform::detect_portable_mode()
}

/// Detect the system locale as a BCP-47-ish tag: a bare ISO 639-1 code
/// (`en`), or that code plus an ISO 3166-1 region subtag (`zh-CN`) when the
/// OS/environment reports one.
///
/// Marco translation files are stored as `assets/language/{code}.toml`,
/// where `{code}` may itself be region-qualified (e.g. `zh-CN.toml`).
///
/// Returns `None` if no useful locale can be detected.
pub fn detect_system_locale_bcp47() -> Option<String> {
    platform::detect_system_locale_bcp47()
}

/// Detect the current installation location for the *asset bundle*.
pub fn detect_install_location() -> InstallLocation {
    if is_dev_mode() {
        return InstallLocation::Development;
    }

    if let Ok(asset_root) = find_asset_root() {
        return platform::detect_install_location_from_asset_root(&asset_root);
    }

    // If assets can't be found, default to the most permissive assumption.
    InstallLocation::UserLocal
}

// --------------------------------------------------------------------------
// Development/workspace helpers (formerly in dev.rs)
// --------------------------------------------------------------------------

/// Get the workspace root directory.
///
/// Only works in development mode. Returns `None` if not in a workspace.
pub fn workspace_root() -> Option<PathBuf> {
    core::find_workspace_root()
}

/// Get the test assets directory (tests/markdown_showcase/)
pub fn test_assets_dir() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("tests").join("markdown_showcase"))
}

/// Get the test specs directory (tests/spec/)
pub fn test_specs_dir() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("tests").join("spec"))
}

/// Get the source assets directory (marco-shared/src/assets/ in the workspace).
pub fn source_assets_dir() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("marco-shared").join("src").join("assets"))
}

/// Get the test settings file (tests/settings/settings.ron)
pub fn test_settings_file() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("tests").join("settings").join("settings.ron"))
}

/// Trait for path providers - allows polymorphic path access
pub trait PathProvider {
    /// Get the shared paths accessor
    fn shared(&self) -> &SharedPaths;

    /// Get the asset root directory
    fn asset_root(&self) -> &std::path::PathBuf;

    /// Check if running in development mode
    fn is_dev_mode(&self) -> bool;
}
