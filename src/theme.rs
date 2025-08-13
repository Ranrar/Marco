
thread_local! {
}

// Removed duplicate save_appearance_settings; use Swanson.rs only
use std::fs;
use std::path::{Path, PathBuf};
use crate::logic::swanson::{load_settings, save_settings};
use crate::logic::settings_struct::{Settings, AppearanceSettings};
use crate::logic::crossplatforms::is_dark_mode_supported;
use dark_light::Mode as SystemMode;

// All settings logic now uses robust struct from Swanson.rs

/// Color palette for SourceView
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub background: &'static str,
    pub text: &'static str,
}

/// Light palette: very light background, dark text
pub const LIGHT_PALETTE: Palette = Palette {
    background: "#FAFAFA",
    text: "#24292E",
};

/// Dark palette: very dark background, light text
pub const DARK_PALETTE: Palette = Palette {
    background: "#1E1E1E",
    text: "#D4D4D4",
};

/// List all available HTML preview themes (*.css) in /themes/
pub fn list_preview_themes(theme_dir: &Path) -> Vec<String> {
    fs::read_dir(theme_dir)
        .map(|entries| {
            entries.filter_map(|e| {
                let e = e.ok()?;
                let path = e.path();
                if path.extension().map_or(false, |ext| ext == "css") {
                    path.file_name()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            }).collect()
        })
        .unwrap_or_default()
}

/// Determines the effective color mode (light/dark) based on settings and system.
pub fn resolve_effective_mode(color_mode: &str) -> String {
    match color_mode.to_lowercase().as_str() {
        "system default" => {
            match dark_light::detect() {
                Ok(SystemMode::Dark) => "dark".to_string(),
                Ok(SystemMode::Light) => "light".to_string(),
                _ => "light".to_string(),
            }
        },
        other => other.to_lowercase(),
    }
}

/// Applies the HTML preview theme by loading the correct CSS file.
/// Returns the path to the CSS file to load.
pub fn select_preview_theme(settings: &AppearanceSettings, theme_dir: &Path) -> Option<PathBuf> {
    let theme_file = settings.preview_theme.as_deref().unwrap_or("standard.css");
    let path = theme_dir.join(theme_file);
    if path.exists() { Some(path) } else { None }
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
    pub settings: Settings,
    pub ui_theme_dir: PathBuf,
    pub preview_theme_dir: PathBuf,
}

impl ThemeManager {
    pub fn new(settings_path: &Path, ui_theme_dir: PathBuf, preview_theme_dir: PathBuf) -> Self {
        let mut settings = load_settings(settings_path).unwrap_or_default();
        // Print settings in a human-readable, multi-line format
        println!("Loaded settings:");
        if let Some(appearance) = &settings.appearance {
            println!("  color_mode: {}", appearance.editor_mode.as_deref().unwrap_or("None"));
            println!("  preview_theme: {}", appearance.preview_theme.as_deref().map(|s| format!("\"{}\"", s)).unwrap_or("None".to_string()));
            println!("  ui_font: {}", appearance.ui_font.as_deref().unwrap_or("None"));
            println!("  ui_font_size: {}", appearance.ui_font_size.map(|s| s.to_string()).unwrap_or("None".to_string()));
        }
        if let Some(editor) = &settings.editor {
            println!("  editor.font: {}", editor.font.as_deref().unwrap_or("None"));
            println!("  editor.font_size: {}", editor.font_size.map(|s| s.to_string()).unwrap_or("None".to_string()));
        }
        if let Some(layout) = &settings.layout {
            println!("  layout.view_mode: {}", layout.view_mode.as_deref().unwrap_or("None"));
        }
        if let Some(language) = &settings.language {
            println!("  language: {}", language.language.as_deref().unwrap_or("None"));
        }
        if let Some(window) = &settings.window {
            println!("  window.width: {}", window.width.map(|w| w.to_string()).unwrap_or("None".to_string()));
            println!("  window.height: {}", window.height.map(|h| h.to_string()).unwrap_or("None".to_string()));
        }
        if let Some(advanced) = &settings.advanced {
            println!("  advanced.enabled_variants: {}", advanced.enabled_variants.as_ref().map(|v| format!("{:?}", v)).unwrap_or("None".to_string()));
            println!("  advanced.plugins: {}", advanced.plugins.as_ref().map(|v| format!("{:?}", v)).unwrap_or("None".to_string()));
        }

        // If color_mode is System default, detect system theme and set effective mode
        if let Some(appearance) = settings.appearance.as_mut() {
            let color_mode = appearance.editor_mode.as_deref().unwrap_or("System default");
            if color_mode == "System default" && is_dark_mode_supported() {
                // Use dark_light crate for actual system theme detection
                let sys_mode = match dark_light::detect() {
                    Ok(SystemMode::Dark) => "dark",
                    Ok(SystemMode::Light) => "light",
                    _ => "light",
                };
                // Set the effective color mode for this session (do not persist)
                println!("System color_mode detected: {} mode", sys_mode);
                appearance.editor_mode = Some(sys_mode.to_string());
            }
        }
        let tm = ThemeManager { settings, ui_theme_dir, preview_theme_dir };
        tm
    }

    /// List available preview themes
    pub fn available_preview_themes(&self) -> Vec<String> {
        list_preview_themes(&self.preview_theme_dir)
    }

    /// Get the effective color mode (light/dark)
    pub fn effective_mode(&self) -> String {
        let color_mode = self.settings.appearance.as_ref().and_then(|a| a.editor_mode.as_ref()).map(|s| s.as_str()).unwrap_or("System default");
        resolve_effective_mode(color_mode)
    }

    /// Get the path to the current preview theme CSS file
    pub fn current_preview_theme_path(&self) -> Option<PathBuf> {
        let appearance = self.settings.appearance.as_ref()?;
        select_preview_theme(appearance, &self.preview_theme_dir)
    }

    /// Get the data-theme value for the HTML preview ("light" or "dark")
    pub fn preview_data_theme(&self) -> &'static str {
        let color_mode = self.settings.appearance.as_ref().and_then(|a| a.editor_mode.as_ref()).map(|s| s.as_str()).unwrap_or("System default");
        get_preview_data_theme(color_mode)
    }

    /// Change color mode (Light, Dark, System) and update themes
    pub fn set_color_mode(&mut self, mode: &str, settings_path: &Path) {
        let mut settings = load_settings(settings_path).unwrap_or_default();
        let mut appearance = settings.appearance.clone().unwrap_or_default();
        let old = appearance.editor_mode.clone().unwrap_or_else(|| "<unset>".to_string());
        println!("set_color_mode: {} => {}", old, mode);
        let mode_lc = mode.to_lowercase();
        appearance.editor_mode = Some(mode_lc.clone());
        settings.appearance = Some(appearance);
        // Set GTK global theme property using correct API
        use gtk4::Settings;
        let prefer_dark = mode_lc == "dark";
        if let Some(settings_obj) = Settings::default() {
            settings_obj.set_gtk_application_prefer_dark_theme(prefer_dark);
        }
        if let Err(e) = save_settings(settings_path, &settings) {
            eprintln!("[ERROR] Failed to save color_mode to {:?}: {}", settings_path, e);
        } else {
            self.settings = settings;
        }
    }

    /// Change preview theme (filename)
    pub fn set_preview_theme(&mut self, theme: String, settings_path: &Path) {
        let mut settings = load_settings(settings_path).unwrap_or_default();
        let mut appearance = settings.appearance.clone().unwrap_or_default();
        let old = appearance.preview_theme.clone().unwrap_or_else(|| "<unset>".to_string());
        println!("set_preview_theme: {} => {}", old, theme);
        appearance.preview_theme = Some(theme.clone());
        settings.appearance = Some(appearance);
        if let Err(e) = save_settings(settings_path, &settings) {
            eprintln!("[ERROR] Failed to save preview_theme to {:?}: {}", settings_path, e);
        } else {
            self.settings = settings;
        }
        // HTML preview reload should be triggered by the UI layer
    }
}