thread_local! {}

// Removed duplicate save_appearance_settings; use Swanson.rs only
use crate::logic::crossplatforms::is_dark_mode_supported;
use crate::logic::swanson::{AppearanceSettings, SettingsManager};
use dark_light::Mode as SystemMode;
use gtk4::Settings as GtkSettings;
use sourceview5::{StyleScheme, StyleSchemeManager};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// List all available HTML preview themes (*.css) in /themes/
pub fn list_preview_themes(theme_dir: &Path) -> Vec<String> {
    fs::read_dir(theme_dir)
        .map(|entries| {
            entries
                .filter_map(|e| {
                    let e = e.ok()?;
                    let path = e.path();
                    if path.extension().is_some_and(|ext| ext == "css") {
                        path.file_name()?.to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Determines the effective color mode (light/dark) based on settings and system.
pub fn resolve_effective_mode(color_mode: &str) -> String {
    match color_mode.to_lowercase().as_str() {
        "system default" => match dark_light::detect() {
            Ok(SystemMode::Dark) => "dark".to_string(),
            Ok(SystemMode::Light) => "light".to_string(),
            _ => "light".to_string(),
        },
        other => other.to_lowercase(),
    }
}

/// Applies the HTML preview theme by loading the correct CSS file.
/// Returns the path to the CSS file to load.
pub fn select_preview_theme(settings: &AppearanceSettings, theme_dir: &Path) -> Option<PathBuf> {
    let theme_file = settings.preview_theme.as_deref().unwrap_or("standard.css");
    let path = theme_dir.join(theme_file);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Synchronizes the HTML preview theme context (e.g., sets data-theme attribute)
pub fn get_preview_data_theme(color_mode: &str) -> &'static str {
    match resolve_effective_mode(color_mode).as_str() {
        "dark" => "dark",
        "light" => "light",
        _ => "light",
    }
}

/// ThemeManager: manages current theme state and applies/synchronizes themes
pub struct ThemeManager {
    pub ui_theme_dir: PathBuf,
    pub preview_theme_dir: PathBuf,
    pub editor_theme_dir: PathBuf,
    style_scheme_manager: StyleSchemeManager,
    settings_manager: Arc<SettingsManager>,
}

impl ThemeManager {
    pub fn new(
        settings_manager: Arc<SettingsManager>,
        ui_theme_dir: PathBuf,
        preview_theme_dir: PathBuf,
        editor_theme_dir: PathBuf,
    ) -> Self {
        // Initialize StyleSchemeManager and add our custom themes directory
        let style_scheme_manager = StyleSchemeManager::new();

        // Add our custom editor themes directory to the search path
        let current_paths = style_scheme_manager.search_path();
        let editor_path_str = editor_theme_dir.to_string_lossy();
        let mut paths: Vec<&str> = current_paths.iter().map(|s| s.as_str()).collect();
        paths.push(&editor_path_str);
        style_scheme_manager.set_search_path(&paths);

        // Loaded settings (verbose output suppressed in normal startup).
        // If you need to debug settings locally, temporarily enable the prints below.
        // e.g. println!("Loaded settings: {:?}", settings);

        // Convert legacy editor_mode to style scheme if needed
        let _ = settings_manager.update_settings(|settings| {
            if let Some(appearance) = settings.appearance.as_mut() {
                // If we have an old editor_mode setting but no style scheme, convert it
                if let Some(editor_mode) = &appearance.editor_mode {
                    match editor_mode.as_str() {
                        "light" => {
                            appearance.editor_mode = Some("marco-light".to_string());
                            // legacy conversion applied (silent)
                        }
                        "dark" => {
                            appearance.editor_mode = Some("marco-dark".to_string());
                            // legacy conversion applied (silent)
                        }
                        "System default" => {
                            // Detect system theme and set appropriate scheme
                            if is_dark_mode_supported() {
                                let sys_mode = match dark_light::detect() {
                                    Ok(SystemMode::Dark) => "marco-dark",
                                    Ok(SystemMode::Light) => "marco-light",
                                    _ => "marco-light",
                                };
                                appearance.editor_mode = Some(sys_mode.to_string());
                                // System theme detected; applied silently
                            } else {
                                appearance.editor_mode = Some("marco-light".to_string());
                            }
                        }
                        _ => {
                            // Keep as-is if it's already a style scheme ID
                            // Debug output suppressed.
                        }
                    }
                } else {
                    // Default to light theme if nothing is set (silent)
                    appearance.editor_mode = Some("marco-light".to_string());
                }
            }
        });

        ThemeManager {
            ui_theme_dir,
            preview_theme_dir,
            editor_theme_dir,
            style_scheme_manager,
            settings_manager: settings_manager.clone(),
        }
    }

    /// List available preview themes
    pub fn available_preview_themes(&self) -> Vec<String> {
        list_preview_themes(&self.preview_theme_dir)
    }

    /// List available editor style schemes
    pub fn available_editor_schemes(&self) -> Vec<crate::logic::loaders::theme_loader::ThemeEntry> {
        crate::logic::loaders::theme_loader::list_editor_style_schemes(&self.editor_theme_dir)
    }

    /// Get the current editor style scheme ID
    pub fn current_editor_scheme_id(&self) -> String {
        let settings = self.settings_manager.get_settings();
        settings
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .cloned()
            .unwrap_or_else(|| "marco-light".to_string())
    }

    /// Get the current editor style scheme object
    pub fn current_editor_scheme(&self) -> Option<StyleScheme> {
        let scheme_id = self.current_editor_scheme_id();
        self.style_scheme_manager.scheme(&scheme_id)
    }

    /// Get editor style scheme by ID
    pub fn get_editor_scheme(&self, scheme_id: &str) -> Option<StyleScheme> {
        self.style_scheme_manager.scheme(scheme_id)
    }

    /// Get the effective color mode (light/dark) - deprecated but kept for HTML preview
    pub fn effective_mode(&self) -> String {
        let scheme_id = self.current_editor_scheme_id();
        if scheme_id.contains("dark") {
            "dark".to_string()
        } else {
            "light".to_string()
        }
    }

    /// Get the path to the current preview theme CSS file
    pub fn current_preview_theme_path(&self) -> Option<PathBuf> {
        let settings = self.settings_manager.get_settings();
        let appearance = settings.appearance.as_ref()?;
        select_preview_theme(appearance, &self.preview_theme_dir)
    }

    /// Get the data-theme value for the HTML preview ("light" or "dark")
    pub fn preview_data_theme(&self) -> &'static str {
        let settings = self.settings_manager.get_settings();
        let color_mode = settings
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("System default");
        get_preview_data_theme(color_mode)
    }

    /// Get preview theme mode from scheme ID
    pub fn preview_theme_mode_from_scheme(&self, scheme_id: &str) -> String {
        let theme_mode = if scheme_id.contains("dark") {
            "dark"
        } else {
            "light"
        };
        format!("theme-{}", theme_mode)
    }

    /// Change editor style scheme and update themes
    pub fn set_editor_scheme(&mut self, scheme_id: &str, _settings_path: &Path) {
        if let Err(e) = self.settings_manager.update_settings(|settings| {
            let mut appearance = settings.appearance.clone().unwrap_or_default();
            // Debug: set_editor_scheme changed - terminal output suppressed.
            appearance.editor_mode = Some(scheme_id.to_string());
            settings.appearance = Some(appearance);
        }) {
            eprintln!("[ERROR] Failed to save editor_scheme: {}", e);
        }

        // Settings are now always fresh from SettingsManager

        // Set GTK global theme property based on scheme
        let prefer_dark = scheme_id.contains("dark");
        if let Some(settings_obj) = GtkSettings::default() {
            settings_obj.set_gtk_application_prefer_dark_theme(prefer_dark);
        }
    }

    /// Change color mode (Light, Dark, System) and update themes - deprecated but kept for compatibility
    pub fn set_color_mode(&mut self, mode: &str, settings_path: &Path) {
        // Convert legacy color mode to style scheme
        let scheme_id = match mode.to_lowercase().as_str() {
            "light" => "marco-light",
            "dark" => "marco-dark",
            "system default" | "system" => {
                if is_dark_mode_supported() {
                    match dark_light::detect() {
                        Ok(SystemMode::Dark) => "marco-dark",
                        Ok(SystemMode::Light) => "marco-light",
                        _ => "marco-light",
                    }
                } else {
                    "marco-light"
                }
            }
            _ => mode, // Assume it's already a scheme ID
        };
        self.set_editor_scheme(scheme_id, settings_path);
    }

    /// Change preview theme (filename)
    pub fn set_preview_theme(&mut self, theme: String, _settings_path: &Path) {
        if let Err(e) = self.settings_manager.update_settings(|settings| {
            let mut appearance = settings.appearance.clone().unwrap_or_default();
            // Debug: set_preview_theme changed - terminal output suppressed.
            appearance.preview_theme = Some(theme.clone());
            settings.appearance = Some(appearance);
        }) {
            eprintln!("[ERROR] Failed to save preview_theme: {}", e);
        }

        // Settings are now always fresh from SettingsManager
        // HTML preview reload should be triggered by the UI layer
    }

    /// Get current settings from SettingsManager
    pub fn get_settings(&self) -> crate::logic::swanson::Settings {
        self.settings_manager.get_settings()
    }
}
