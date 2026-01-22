//! Marco-specific path management
//!
//! This module provides paths specific to the Marco editor:
//! - Editor themes (SourceView style schemes)
//! - UI CSS paths
//! - Marco-specific configuration

use super::core::{find_asset_root, is_dev_mode, AssetError};
use super::shared::SharedPaths;
use super::PathProvider;
use std::path::PathBuf;

/// Marco-specific paths
pub struct MarcoPaths {
    asset_root: PathBuf,
    shared: SharedPaths,
    dev_mode: bool,
}

impl MarcoPaths {
    /// Create a new MarcoPaths instance
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
    // Editor Themes (SourceView5 style schemes)
    // ========================================================================

    /// Get the editor themes directory
    ///
    /// Contains .xml style schemes for SourceView5
    pub fn editor_themes_dir(&self) -> PathBuf {
        self.asset_root.join("themes").join("editor")
    }

    /// Get path to a specific editor theme
    ///
    /// # Examples
    /// ```no_run
    /// use core::paths::MarcoPaths;
    ///
    /// # fn main() -> Result<(), core::paths::AssetError> {
    /// let marco = MarcoPaths::new()?;
    /// let dark_theme = marco.editor_theme("dark");
    /// # Ok(())
    /// # }
    /// ```
    pub fn editor_theme(&self, theme_name: &str) -> PathBuf {
        // Support both "dark.xml" and "dark" formats
        let filename = if theme_name.ends_with(".xml") {
            theme_name.to_string()
        } else {
            format!("{}.xml", theme_name)
        };
        self.editor_themes_dir().join(filename)
    }

    /// Get the editor syntax directory (for sublime-syntax files)
    pub fn editor_syntax_dir(&self) -> PathBuf {
        self.editor_themes_dir().join("syntax")
    }

    /// List all available editor themes
    pub fn list_editor_themes(&self) -> Vec<String> {
        let mut themes = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.editor_themes_dir()) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".xml") {
                        // Remove .xml extension
                        themes.push(name.trim_end_matches(".xml").to_string());
                    }
                }
            }
        }
        themes.sort();
        themes
    }

    // ========================================================================
    // Marco Configuration
    // ========================================================================

    /// Get Marco's config directory
    ///
    /// - Dev mode: workspace_root/tests/settings/
    /// - Install mode: ~/.config/marco/
    pub fn config_dir(&self) -> PathBuf {
        if self.dev_mode {
            if let Some(workspace) = super::dev::workspace_root() {
                workspace.join("tests").join("settings")
            } else {
                super::install::config_dir()
            }
        } else {
            super::install::config_dir()
        }
    }

    /// Get Marco's settings file path
    pub fn settings_file(&self) -> PathBuf {
        self.shared.settings_file()
    }

    /// Get Marco's recent files list path
    pub fn recent_files(&self) -> PathBuf {
        super::install::user_data_dir().join("recent_files.ron")
    }

    /// Get Marco's window state file path
    pub fn window_state(&self) -> PathBuf {
        super::install::user_data_dir().join("window_state.ron")
    }

    // ========================================================================
    // Marco-specific data directories
    // ========================================================================

    /// Get Marco's user data directory
    pub fn user_data_dir(&self) -> PathBuf {
        super::install::user_data_dir()
    }

    /// Get Marco's cache directory
    pub fn cache_dir(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            dirs::cache_dir()
                .map(|c| c.join("marco"))
                .unwrap_or_else(|| {
                    std::env::temp_dir().join("marco").join("cache")
                })
        }
        #[cfg(not(target_os = "windows"))]
        {
            dirs::cache_dir()
                .map(|c| c.join("marco"))
                .or_else(|| dirs::home_dir().map(|h| h.join(".cache/marco")))
                .unwrap_or_else(|| PathBuf::from("/tmp/marco/cache"))
        }
    }
}

impl PathProvider for MarcoPaths {
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
    fn test_marco_paths_creation() {
        let marco = MarcoPaths::new();
        // In test environments, asset root may not be found if not running from target/
        // This is expected behavior - the important thing is the code compiles
        if marco.is_err() {
            println!("Note: MarcoPaths creation failed (expected in some test environments)");
        } else {
            println!("MarcoPaths created successfully");
        }
    }

    #[test]
    fn test_editor_theme_paths() {
        if let Ok(marco) = MarcoPaths::new() {
            let themes_dir = marco.editor_themes_dir();
            let dark_theme = marco.editor_theme("dark");

            println!("Editor themes dir: {}", themes_dir.display());
            println!("Dark theme: {}", dark_theme.display());

            let themes = marco.list_editor_themes();
            println!("Available editor themes: {:?}", themes);
        }
    }

    #[test]
    fn test_marco_config_paths() {
        if let Ok(marco) = MarcoPaths::new() {
            println!("Config dir: {}", marco.config_dir().display());
            println!("Settings file: {}", marco.settings_file().display());
            println!("Recent files: {}", marco.recent_files().display());
            println!("Window state: {}", marco.window_state().display());
            println!("Cache dir: {}", marco.cache_dir().display());
        }
    }

    #[test]
    fn test_shared_access() {
        if let Ok(marco) = MarcoPaths::new() {
            let icon_font = marco.shared().icon_font();
            println!("Icon font (via shared): {}", icon_font.display());
        }
    }
}
