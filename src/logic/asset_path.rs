//! Helper for detecting the asset directory ("marco/") at runtime.
//!
//! Usage:
//! ```rust
//! use crate::logic::asset_path::get_asset_dir;
//!
//! let asset_dir = get_asset_dir();
//! let font_path = asset_dir.join("fonts/ui_menu.ttf");
//! let settings_path = asset_dir.join("settings.ron");
//! ```
//!
//! This works regardless of where the binary is run from, as long as "marco/" is next to the binary.

use std::env;
use std::path::{Path, PathBuf};
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

/// Returns the path to the asset directory ("marco") next to the binary, or an error if not found.
pub fn get_asset_dir_checked() -> Result<PathBuf, AssetError> {
    let exe_path = env::current_exe().map_err(AssetError::ExePathError)?;
    let parent = exe_path.parent().ok_or(AssetError::ParentMissing)?;
    let asset_dir = parent.join("marco_assets");
    if asset_dir.exists() && asset_dir.is_dir() {
        Ok(asset_dir)
    } else {
        Err(AssetError::AssetDirMissing(asset_dir))
    }
}

/// Returns the asset directory path ("marco") next to the binary, panicking if not found.
pub fn get_asset_dir() -> PathBuf {
    get_asset_dir_checked().expect("Asset directory not found next to binary")
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

/// Returns the path to a documentation file in the asset directory.
pub fn get_doc_path(doc_name: &str) -> Result<PathBuf, AssetError> {
    let asset_dir = get_asset_dir_checked()?;
    Ok(asset_dir.join("documentation").join(doc_name))
}

