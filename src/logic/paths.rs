//! Helper for detecting the asset directory ("marco/") at runtime.
//!
//! Usage:
//! ```no_run
//! use marco::logic::paths::{get_asset_dir_checked, get_font_path, get_settings_path};
//!
//! let asset_dir = get_asset_dir_checked()?;
//! let font_path = get_font_path("ui_menu.ttf")?;
//! let settings_path = get_settings_path()?;
//! ```
//!
//! This works regardless of where the binary is run from, as long as "marco/" is next to the binary.

use std::env;
use std::path::PathBuf;
use std::fmt;

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
            AssetError::AssetDirMissing(p) => write!(f, "Asset directory not found: {}", p.display()),
        }
    }
}

impl std::error::Error for AssetError {}

/// Returns the path to the asset directory, checking multiple locations in order:
/// 1. "marco_assets" next to the binary (development/portable)
/// 2. "~/.local/share/marco" (user installation)
/// 3. "/usr/local/share/marco" (system installation)
/// 4. "/usr/share/marco" (package installation)
pub fn get_asset_dir_checked() -> Result<PathBuf, AssetError> {
    let exe_path = env::current_exe().map_err(AssetError::ExePathError)?;
    let parent = exe_path.parent().ok_or(AssetError::ParentMissing)?;
    
    // Try locations in order of preference
    let candidate_paths = [
        // 1. Next to binary (development/portable)
        parent.join("marco_assets"),
        // 2. User local share directory
        dirs::home_dir().map(|h| h.join(".local/share/marco")).unwrap_or_else(|| PathBuf::from("/tmp")),
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
pub fn get_font_path(font_name: &str) -> Result<PathBuf, AssetError> {
    let asset_dir = get_asset_dir_checked()?;
    Ok(asset_dir.join("fonts").join(font_name))
}

/// Returns the path to settings.ron in the asset directory.
pub fn get_settings_path() -> Result<PathBuf, AssetError> {
    let asset_dir = get_asset_dir_checked()?;
    Ok(asset_dir.join("settings.ron"))
}