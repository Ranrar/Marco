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
//! use core::paths::{MarcoPaths, PathProvider};
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
pub use marco::MarcoPaths;
pub use polo::PoloPaths;
pub use shared::SharedPaths;

use std::path::PathBuf;

/// Installation location type.
///
/// Note: this reflects where the *asset bundle* is loaded from (not where config is stored).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallLocation {
    /// User local installation (e.g. Linux: `~/.local/share/marco/`)
    UserLocal,
    /// System local installation (e.g. Linux: `/usr/local/share/marco/`)
    SystemLocal,
    /// System global installation (e.g. Linux: `/usr/share/marco/`)
    SystemGlobal,
    /// Development mode (not installed)
    Development,
    /// Portable mode (Windows): running from writable directory
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

/// Detect Windows portable mode (returns the portable root directory if detected).
///
/// On non-Windows targets this always returns `None`.
pub fn detect_portable_mode() -> Option<PathBuf> {
    platform::detect_portable_mode()
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

/// Get the source assets directory (workspace assets/).
pub fn source_assets_dir() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("assets"))
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
