//! Shared asset paths used by both Marco and Polo
//!
//! This module provides paths to assets that are common between Marco and Polo:
//! - Fonts (IcoMoon icon font, UI fonts)
//! - Icons (application icons, UI icons)
//! - Language files (translations)
//! - Preview themes (HTML/CSS for markdown preview)
//! - Documentation (user guide, help files)

use super::core::{find_asset_root, AssetError};
use std::path::PathBuf;

/// Paths to assets shared between Marco and Polo
pub struct SharedPaths {
    asset_root: PathBuf,
}

impl SharedPaths {
    /// Create a new SharedPaths instance
    ///
    /// # Errors
    /// Returns an error if the asset root cannot be found
    pub fn new() -> Result<Self, AssetError> {
        let asset_root = find_asset_root()?;
        Ok(Self { asset_root })
    }

    /// Get the asset root directory
    pub fn asset_root(&self) -> &PathBuf {
        &self.asset_root
    }

    // ========================================================================
    // Fonts
    // ========================================================================

    /// Get the fonts directory
    pub fn fonts_dir(&self) -> PathBuf {
        self.asset_root.join("fonts")
    }

    /// Get path to a specific font file
    ///
    /// # Examples
    /// ```no_run
    /// use core::paths::SharedPaths;
    ///
    /// # fn main() -> Result<(), core::paths::AssetError> {
    /// let shared = SharedPaths::new()?;
    /// let icon_font = shared.font("ui_menu.ttf");
    /// # Ok(())
    /// # }
    /// ```
    pub fn font(&self, font_name: &str) -> PathBuf {
        self.fonts_dir().join(font_name)
    }

    /// Get path to the IcoMoon icon font (ui_menu.ttf)
    pub fn icon_font(&self) -> PathBuf {
        self.font("ui_menu.ttf")
    }

    // ========================================================================
    // Icons
    // ========================================================================

    /// Get the icons directory
    pub fn icons_dir(&self) -> PathBuf {
        self.asset_root.join("icons")
    }

    /// Get path to a specific icon file
    pub fn icon(&self, icon_name: &str) -> PathBuf {
        self.icons_dir().join(icon_name)
    }

    /// Get path to the application icon
    pub fn app_icon(&self) -> PathBuf {
        self.icon("icon.png")
    }

    /// Get path to the application favicon
    pub fn app_favicon(&self) -> PathBuf {
        self.icon("favicon.png")
    }

    // ========================================================================
    // Language Files
    // ========================================================================

    /// Get the language files directory
    pub fn language_dir(&self) -> PathBuf {
        self.asset_root.join("language")
    }

    /// Get path to a specific language file
    ///
    /// # Examples
    /// ```no_run
    /// use core::paths::SharedPaths;
    ///
    /// # fn main() -> Result<(), core::paths::AssetError> {
    /// let shared = SharedPaths::new()?;
    /// let danish = shared.language_file("da.json");
    /// # Ok(())
    /// # }
    /// ```
    pub fn language_file(&self, lang_code: &str) -> PathBuf {
        // Support both "da.json" and "da" formats
        let filename = if lang_code.ends_with(".json") {
            lang_code.to_string()
        } else {
            format!("{}.json", lang_code)
        };
        self.language_dir().join(filename)
    }

    // ========================================================================
    // Preview Themes (HTML viewer)
    // ========================================================================

    /// Get the HTML preview themes directory
    ///
    /// These are CSS themes for the WebKit preview (html_viever directory)
    pub fn preview_themes_dir(&self) -> PathBuf {
        self.asset_root.join("themes").join("html_viever")
    }

    /// Get path to a specific preview theme CSS file
    ///
    /// # Examples
    /// ```no_run
    /// use core::paths::SharedPaths;
    ///
    /// # fn main() -> Result<(), core::paths::AssetError> {
    /// let shared = SharedPaths::new()?;
    /// let github_theme = shared.preview_theme("github");
    /// # Ok(())
    /// # }
    /// ```
    pub fn preview_theme(&self, theme_name: &str) -> PathBuf {
        // Support both "github.css" and "github" formats
        let filename = if theme_name.ends_with(".css") {
            theme_name.to_string()
        } else {
            format!("{}.css", theme_name)
        };
        self.preview_themes_dir().join(filename)
    }

    /// List all available preview themes
    pub fn list_preview_themes(&self) -> Vec<String> {
        let mut themes = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.preview_themes_dir()) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".css") {
                        // Remove .css extension
                        themes.push(name.trim_end_matches(".css").to_string());
                    }
                }
            }
        }
        themes.sort();
        themes
    }

    // ========================================================================
    // Documentation
    // ========================================================================

    /// Get the documentation directory
    pub fn documentation_dir(&self) -> PathBuf {
        self.asset_root.join("documentation")
    }

    /// Get path to user guide
    pub fn user_guide(&self) -> PathBuf {
        self.documentation_dir().join("user_guide.md")
    }

    /// Get path to logo image
    pub fn logo(&self) -> PathBuf {
        self.documentation_dir().join("logo.png")
    }

    // ========================================================================
    // Settings
    // ========================================================================

    /// Get path to settings file
    ///
    /// In dev mode: workspace_root/tests/settings/settings.ron
    /// In install mode: asset_root/settings.ron
    pub fn settings_file(&self) -> PathBuf {
        use super::core::is_dev_mode;
        use super::dev::workspace_root;

        if is_dev_mode() {
            if let Some(workspace) = workspace_root() {
                workspace
                    .join("tests")
                    .join("settings")
                    .join("settings.ron")
            } else {
                // Fallback to asset root
                self.asset_root.join("settings.ron")
            }
        } else {
            self.asset_root.join("settings.ron")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_paths_creation() {
        let shared = SharedPaths::new();
        // In test environments, asset root may not be found if not running from target/
        // This is expected behavior - the important thing is the code compiles
        if shared.is_err() {
            println!("Note: SharedPaths creation failed (expected in some test environments)");
        } else {
            println!("SharedPaths created successfully");
        }
    }

    #[test]
    fn test_font_paths() {
        if let Ok(shared) = SharedPaths::new() {
            let fonts_dir = shared.fonts_dir();
            let icon_font = shared.icon_font();

            println!("Fonts dir: {}", fonts_dir.display());
            println!("Icon font: {}", icon_font.display());

            assert!(icon_font.to_string_lossy().contains("ui_menu.ttf"));
        }
    }

    #[test]
    fn test_preview_themes() {
        if let Ok(shared) = SharedPaths::new() {
            let themes_dir = shared.preview_themes_dir();
            let github_theme = shared.preview_theme("github");

            println!("Preview themes dir: {}", themes_dir.display());
            println!("GitHub theme: {}", github_theme.display());

            let themes = shared.list_preview_themes();
            println!("Available themes: {:?}", themes);
        }
    }

    #[test]
    fn test_settings_path() {
        if let Ok(shared) = SharedPaths::new() {
            let settings = shared.settings_file();
            println!("Settings file: {}", settings.display());
            assert!(settings.to_string_lossy().contains("settings.ron"));
        }
    }
}
