
thread_local! {
}

impl ThemeManager {
    /// Apply the current GTK4 UI theme by loading the CSS file into the default screen's style context.
    pub fn apply_ui_theme(&self) {
        // Print the UI theme directory and list all files in it
        println!("[ThemeManager] ui_theme_dir: {:?}", self.ui_theme_dir);
        match std::fs::read_dir(&self.ui_theme_dir) {
            Ok(entries) => {
                println!("[ThemeManager] Files in ui_theme_dir:");
                for entry in entries.flatten() {
                    println!("  - {:?}", entry.file_name());
                }
            }
            Err(e) => {
                println!("[ThemeManager] Could not read ui_theme_dir: {}", e);
            }
        }

        // Print the full path being checked for the theme
        if let Some(css_path) = self.current_ui_theme_path() {
            println!("[ThemeManager] Applying GTK4 UI theme: {:?}", css_path);
            if let Err(_e) = std::fs::read_to_string(&css_path) {
                eprintln!("[ThemeManager] Failed to read CSS file: {:?}", css_path);
            }
        } else {
            eprintln!("[ThemeManager] No UI theme path found to apply.");
        }
    }
}
// Removed duplicate save_appearance_settings; use Swanson.rs only
use std::fs;
use std::path::{Path, PathBuf};
use crate::logic::swanson::{load_settings, save_settings};
use crate::logic::settings_struct::{Settings, AppearanceSettings};
use crate::logic::crossplatforms::is_dark_mode_supported;
use dark_light::Mode as SystemMode;

// All settings logic now uses robust struct from Swanson.rs

/// Color palette for SourceView and HTML Preview
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
                Ok(SystemMode::Dark) => "Dark".to_string(),
                Ok(SystemMode::Light) => "Light".to_string(),
                _ => "Light".to_string(),
            }
        },
        other => other.to_string(),
    }
}

/// Applies the application theme (GTK4 UI) by loading the correct CSS file.
/// Returns the path to the CSS file to load.
pub fn select_app_theme(settings: &AppearanceSettings, theme_dir: &Path) -> Option<PathBuf> {
    // Always use the chosen app_theme if set, else fallback to color mode default
    let theme_file = settings.app_theme.as_deref().or_else(|| {
        let color_mode = settings.color_mode.as_deref().unwrap_or("System default");
        let effective_mode = resolve_effective_mode(color_mode);
        match effective_mode.as_str() {
            "Dark" => Some("standard-dark.css"),
            "Light" => Some("standard-light.css"),
            _ => Some("standard-light.css"),
        }
    }).unwrap_or("standard-light.css");
    let path = theme_dir.join(theme_file);
    if path.exists() { Some(path) } else { None }
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
        "Dark" => "dark",
        "Light" => "light",
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
            println!("  color_mode: {}", appearance.color_mode.as_deref().unwrap_or("None"));
            println!("  app_theme: {}", appearance.app_theme.as_deref().map(|s| format!("\"{}\"", s)).unwrap_or("None".to_string()));
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
            let color_mode = appearance.color_mode.as_deref().unwrap_or("System default");
            if color_mode == "System default" && is_dark_mode_supported() {
                // Use dark_light crate for actual system theme detection
                let sys_mode = match dark_light::detect() {
                    Ok(SystemMode::Dark) => "Dark",
                    Ok(SystemMode::Light) => "Light",
                    _ => "Light",
                };
                // Set the effective color mode for this session (do not persist)
                println!("System color_mode detected: {} mode", sys_mode);
                appearance.color_mode = Some(sys_mode.to_string());
            }
        }
        let tm = ThemeManager { settings, ui_theme_dir, preview_theme_dir };
        tm.apply_ui_theme();
        tm
    }

    /// List available preview themes
    pub fn available_preview_themes(&self) -> Vec<String> {
        list_preview_themes(&self.preview_theme_dir)
    }

    /// Get the effective color mode (light/dark)
    pub fn effective_mode(&self) -> String {
        let color_mode = self.settings.appearance.as_ref().and_then(|a| a.color_mode.as_ref()).map(|s| s.as_str()).unwrap_or("System default");
        resolve_effective_mode(color_mode)
    }

    /// Get the path to the current UI theme CSS file
    pub fn current_ui_theme_path(&self) -> Option<PathBuf> {
        let appearance = self.settings.appearance.as_ref()?;
        select_app_theme(appearance, &self.ui_theme_dir)
    }

    /// Get the path to the current preview theme CSS file
    pub fn current_preview_theme_path(&self) -> Option<PathBuf> {
        let appearance = self.settings.appearance.as_ref()?;
        select_preview_theme(appearance, &self.preview_theme_dir)
    }

    /// Get the data-theme value for the HTML preview ("light" or "dark")
    pub fn preview_data_theme(&self) -> &'static str {
        let color_mode = self.settings.appearance.as_ref().and_then(|a| a.color_mode.as_ref()).map(|s| s.as_str()).unwrap_or("System default");
        get_preview_data_theme(color_mode)
    }

    /// Change color mode (Light, Dark, System) and update themes
    pub fn set_color_mode(&mut self, mode: &str, settings_path: &Path) {
        let mut settings = load_settings(settings_path).unwrap_or_default();
        let mut appearance = settings.appearance.clone().unwrap_or_default();
        let old = appearance.color_mode.clone().unwrap_or_else(|| "<unset>".to_string());
        println!("set_color_mode: {} => {}", old, mode);
        appearance.color_mode = Some(mode.to_string());
        settings.appearance = Some(appearance);
        if let Err(e) = save_settings(settings_path, &settings) {
            eprintln!("[ERROR] Failed to save color_mode to {:?}: {}", settings_path, e);
        } else {
            self.settings = settings;
        }
        self.apply_ui_theme();
    }

    /// Change UI theme (filename)
    pub fn set_app_theme(&mut self, theme: String, settings_path: &Path) {
        let mut settings = load_settings(settings_path).unwrap_or_default();
        let mut appearance = settings.appearance.clone().unwrap_or_default();
        let old = appearance.app_theme.clone().unwrap_or_else(|| "<unset>".to_string());
        println!("set_app_theme: {} => {}", old, theme);
        appearance.app_theme = Some(theme.clone());
        settings.appearance = Some(appearance);
        if let Err(e) = save_settings(settings_path, &settings) {
            eprintln!("[ERROR] Failed to save app_theme to {:?}: {}", settings_path, e);
        } else {
            self.settings = settings;
        }
        self.apply_ui_theme();
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