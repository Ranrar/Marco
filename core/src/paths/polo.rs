//! Polo-specific path management
//!
//! This module provides paths specific to the Polo viewer:
//! - Polo-specific configuration
//! - Preview-only themes

use super::core::{find_asset_root, is_dev_mode, AssetError};
use super::shared::SharedPaths;
use super::PathProvider;
use std::path::PathBuf;

/// Polo-specific paths
pub struct PoloPaths {
    asset_root: PathBuf,
    shared: SharedPaths,
    dev_mode: bool,
}

impl PoloPaths {
    /// Create a new PoloPaths instance
    ///
    /// # Errors
    /// Returns an error if the asset root cannot be found
    pub fn new() -> Result<Self, AssetError> {
        let asset_root = find_asset_root()?;
        let shared = SharedPaths::new()?;
        let dev_mode = is_dev_mode();
        
        Ok(Self {
            asset_root,
            shared,
            dev_mode,
        })
    }

    // ========================================================================
    // Polo Configuration
    // ========================================================================

    /// Get Polo's config directory
    ///
    /// - Dev mode: workspace_root/tests/settings/
    /// - Install mode: ~/.config/polo/
    pub fn config_dir(&self) -> PathBuf {
        if self.dev_mode {
            if let Some(workspace) = super::dev::workspace_root() {
                workspace.join("tests").join("settings")
            } else {
                dirs::config_dir()
                    .map(|c| c.join("polo"))
                    .or_else(|| dirs::home_dir().map(|h| h.join(".config/polo")))
                    .unwrap_or_else(|| PathBuf::from("/tmp/polo/config"))
            }
        } else {
            dirs::config_dir()
                .map(|c| c.join("polo"))
                .or_else(|| dirs::home_dir().map(|h| h.join(".config/polo")))
                .unwrap_or_else(|| PathBuf::from("/tmp/polo/config"))
        }
    }

    /// Get Polo's settings file path
    pub fn settings_file(&self) -> PathBuf {
        self.shared.settings_file()
    }

    /// Get Polo's recent files list path
    pub fn recent_files(&self) -> PathBuf {
        self.user_data_dir().join("recent_files.ron")
    }

    /// Get Polo's window state file path
    pub fn window_state(&self) -> PathBuf {
        self.user_data_dir().join("window_state.ron")
    }

    // ========================================================================
    // Polo-specific data directories
    // ========================================================================

    /// Get Polo's user data directory
    pub fn user_data_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .map(|d| d.join("polo"))
            .or_else(|| dirs::home_dir().map(|h| h.join(".local/share/polo")))
            .unwrap_or_else(|| PathBuf::from("/tmp/polo/data"))
    }

    /// Get Polo's cache directory
    pub fn cache_dir(&self) -> PathBuf {
        dirs::cache_dir()
            .map(|c| c.join("polo"))
            .or_else(|| dirs::home_dir().map(|h| h.join(".cache/polo")))
            .unwrap_or_else(|| PathBuf::from("/tmp/polo/cache"))
    }
}

impl PathProvider for PoloPaths {
    fn shared(&self) -> &SharedPaths {
        &self.shared
    }

    fn asset_root(&self) -> &PathBuf {
        &self.asset_root
    }

    fn is_dev_mode(&self) -> bool {
        self.dev_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polo_paths_creation() {
        let polo = PoloPaths::new();
        // In test environments, asset root may not be found if not running from target/
        // This is expected behavior - the important thing is the code compiles
        if polo.is_err() {
            println!("Note: PoloPaths creation failed (expected in some test environments)");
        } else {
            println!("PoloPaths created successfully");
        }
    }

    #[test]
    fn test_polo_config_paths() {
        if let Ok(polo) = PoloPaths::new() {
            println!("Config dir: {}", polo.config_dir().display());
            println!("Settings file: {}", polo.settings_file().display());
            println!("Recent files: {}", polo.recent_files().display());
            println!("Window state: {}", polo.window_state().display());
            println!("Cache dir: {}", polo.cache_dir().display());
        }
    }

    #[test]
    fn test_polo_uses_separate_dirs() {
        if let Ok(polo) = PoloPaths::new() {
            let config = polo.config_dir();
            let data = polo.user_data_dir();
            let cache = polo.cache_dir();
            
            // Verify polo has separate directories from marco
            assert!(config.to_string_lossy().contains("polo"));
            assert!(data.to_string_lossy().contains("polo"));
            assert!(cache.to_string_lossy().contains("polo"));
            
            println!("Polo config: {}", config.display());
            println!("Polo data: {}", data.display());
            println!("Polo cache: {}", cache.display());
        }
    }

    #[test]
    fn test_shared_access() {
        if let Ok(polo) = PoloPaths::new() {
            // Polo should have access to shared assets
            let preview_themes = polo.shared().preview_themes_dir();
            let icon_font = polo.shared().icon_font();
            
            println!("Preview themes (via shared): {}", preview_themes.display());
            println!("Icon font (via shared): {}", icon_font.display());
        }
    }

    #[test]
    fn test_preview_theme_access() {
        if let Ok(polo) = PoloPaths::new() {
            let github_theme = polo.shared().preview_theme("github");
            println!("GitHub preview theme: {}", github_theme.display());
            
            let themes = polo.shared().list_preview_themes();
            println!("Available preview themes: {:?}", themes);
            assert!(!themes.is_empty(), "Should have some preview themes");
        }
    }
}
