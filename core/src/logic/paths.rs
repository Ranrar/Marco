//! Helper for detecting the asset directory ("marco/") at runtime.
//!
//! **DEPRECATED**: This module is deprecated in favor of `core::components::paths`.
//! 
//! ## Migration Guide
//! 
//! **Old API** (deprecated):
//! ```no_run
//! use core::logic::paths::{get_asset_dir_checked, get_font_path, get_settings_path};
//!
//! let asset_dir = get_asset_dir_checked()?;
//! let font_path = get_font_path("ui_menu.ttf")?;
//! let settings_path = get_settings_path()?;
//! ```
//!
//! **New API** (recommended):
//! ```no_run
//! use core::components::paths::{MarcoPaths, PoloPaths, PathProvider};
//!
//! // For Marco binary:
//! let marco_paths = MarcoPaths::new()?;
//! let asset_root = marco_paths.asset_root();
//! let font_path = marco_paths.shared().font("ui_menu.ttf");
//! let settings_path = marco_paths.settings_file();
//!
//! // For Polo binary:
//! let polo_paths = PoloPaths::new()?;
//! let asset_root = polo_paths.asset_root();
//! let font_path = polo_paths.shared().font("ui_menu.ttf");
//! let settings_path = polo_paths.settings_file();
//! ```
//!
//! ## Benefits of New API
//! - Binary-specific paths (marco vs polo have separate config dirs)
//! - Mode-aware (dev vs install detection)
//! - Cached lookups (OnceLock for performance)
//! - Type-safe methods for all asset types
//! - Better error messages

use std::env;
use std::fmt;
use std::path::PathBuf;

/// Custom error type for asset path detection
#[derive(Debug)]
pub enum AssetError {
    ExePathError(std::io::Error),
    ParentMissing,
    AssetDirMissing(PathBuf),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::ExePathError(e) => write!(f, "Failed to get current exe path: {}", e),
            AssetError::ParentMissing => write!(f, "Executable has no parent directory"),
            AssetError::AssetDirMissing(p) => {
                write!(f, "Asset directory not found: {}", p.display())
            }
        }
    }
}

impl std::error::Error for AssetError {}

/// Returns the path to the asset directory, checking multiple locations in order:
/// 1. "marco_assets" next to the binary (development/portable)
/// 2. "~/.local/share/marco" (user installation)
/// 3. "/usr/local/share/marco" (system installation)
/// 4. "/usr/share/marco" (package installation)
///
/// **DEPRECATED**: Use `MarcoPaths::new()` or `PoloPaths::new()` from `core::components::paths` instead.
#[deprecated(
    since = "0.2.0",
    note = "Use MarcoPaths::new() or PoloPaths::new() from core::components::paths"
)]
pub fn get_asset_dir_checked() -> Result<PathBuf, AssetError> {
    let exe_path = env::current_exe().map_err(AssetError::ExePathError)?;
    let parent = exe_path.parent().ok_or(AssetError::ParentMissing)?;

    // Try locations in order of preference
    let candidate_paths = [
        // 1. Next to binary (development/portable)
        parent.join("marco_assets"),
        // 2. User local share directory
        dirs::home_dir()
            .map(|h| h.join(".local/share/marco"))
            .unwrap_or_else(|| PathBuf::from("/tmp")),
        // 3. System local share directory
        PathBuf::from("/usr/local/share/marco"),
        // 4. System share directory
        PathBuf::from("/usr/share/marco"),
    ];

    for asset_dir in candidate_paths.iter() {
        if asset_dir.exists() && asset_dir.is_dir() {
            return Ok(asset_dir.clone());
        }
    }

    // If none found, return error with the first (preferred) location
    Err(AssetError::AssetDirMissing(candidate_paths[0].clone()))
}

/// Returns the path to a font file in the asset directory.
///
/// **DEPRECATED**: Use `SharedPaths::font()` from the new paths API instead.
#[deprecated(
    since = "0.2.0",
    note = "Use marco_paths.shared().font(name) or polo_paths.shared().font(name)"
)]
pub fn get_font_path(font_name: &str) -> Result<PathBuf, AssetError> {
    #[allow(deprecated)]
    let asset_dir = get_asset_dir_checked()?;
    Ok(asset_dir.join("fonts").join(font_name))
}

/// Returns the path to a UI theme CSS file in the asset directory.
/// 
/// # Arguments
/// * `theme_file` - The CSS filename (e.g., "menu.css", "toolbar.css")
/// 
/// # Examples
/// ```no_run
/// use marco::logic::paths::get_ui_theme_path;
/// let menu_css = get_ui_theme_path("menu.css")?;
/// let toolbar_css = get_ui_theme_path("toolbar.css")?;
/// ```
///
/// **DEPRECATED**: Use appropriate methods from the new paths API instead.
#[deprecated(
    since = "0.2.0",
    note = "Use marco_paths.asset_root().join(...) for custom paths"
)]
pub fn get_ui_theme_path(theme_file: &str) -> Result<PathBuf, AssetError> {
    #[allow(deprecated)]
    let asset_dir = get_asset_dir_checked()?;
    Ok(asset_dir.join("themes").join("ui_elements").join(theme_file))
}

/// Returns the path to settings.ron in the tests/settings directory.
/// This ensures all binaries (marco, marco-test, marco-parser-debug) use the same settings.
///
/// **DEPRECATED**: Use `MarcoPaths::settings_file()` or `PoloPaths::settings_file()` instead.
#[deprecated(
    since = "0.2.0",
    note = "Use marco_paths.settings_file() or polo_paths.settings_file()"
)]
pub fn get_settings_path() -> Result<PathBuf, AssetError> {
    // Always use tests/settings/settings.ron relative to the project root
    let exe_path = env::current_exe().map_err(AssetError::ExePathError)?;
    let parent = exe_path.parent().ok_or(AssetError::ParentMissing)?;
    
    // Go up from target/debug (or wherever binary is) to find project root
    let mut project_root = parent.to_path_buf();
    
    // Search upward for Cargo.toml to find project root
    while !project_root.join("Cargo.toml").exists() {
        project_root = match project_root.parent() {
            Some(parent) => parent.to_path_buf(),
            None => return Err(AssetError::AssetDirMissing(project_root)),
        };
    }
    
    let settings_dir = project_root.join("tests").join("settings");
    let settings_path = settings_dir.join("settings.ron");
    
    // Create the settings directory if it doesn't exist
    if !settings_dir.exists() {
        std::fs::create_dir_all(&settings_dir).map_err(|_| {
            AssetError::AssetDirMissing(settings_dir.clone())
        })?;
    }
    
    Ok(settings_path)
}
