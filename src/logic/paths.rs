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
pub fn get_font_path(font_name: &str) -> Result<PathBuf, AssetError> {
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
pub fn get_ui_theme_path(theme_file: &str) -> Result<PathBuf, AssetError> {
    let asset_dir = get_asset_dir_checked()?;
    Ok(asset_dir.join("themes").join("ui_elements").join(theme_file))
}

/// Returns the path to settings.ron in the tests/settings directory.
/// This ensures all binaries (marco, marco-test, marco-parser-debug) use the same settings.
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
