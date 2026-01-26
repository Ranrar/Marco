//! Display configuration for the editor
//!
//! This module manages editor display settings including:
//! - Font family selection and caching
//! - Font size configuration
//! - Line height and wrapping
//! - Visibility toggles (invisibles, line numbers)
//! - Tab behavior (tabs vs spaces)
//!
//! # Font Management
//!
//! Fonts are loaded from the system using cached lookups for performance.
//! The module prefers common monospace coding fonts (Hack, Fira Code, etc.)
//! and falls back to system defaults if unavailable.
//!
//! # Settings Storage
//!
//! Display settings are persisted via the `SettingsManager` and can be
//! updated at runtime through the editor settings dialog.

use core::logic::{
    loaders::font_loader::{AvailableFonts, FontLoader},
    swanson::{EditorSettings, SettingsManager},
};
use log::debug;
use std::sync::Arc;

/// Editor configuration manager for fonts and display settings
pub struct EditorConfiguration {
    fonts: AvailableFonts,
    settings_manager: Arc<SettingsManager>,
}

impl EditorConfiguration {
    /// Create a new editor configuration instance using cached fonts (fast)
    pub fn new(settings_manager: Arc<SettingsManager>) -> Result<Self, Box<dyn std::error::Error>> {
        // Use cached monospace fonts for fast loading
        let monospace_fonts = FontLoader::get_cached_monospace_fonts();

        let fonts = AvailableFonts {
            monospace: monospace_fonts.clone(),
        };

        Ok(Self {
            fonts,
            settings_manager,
        })
    }

    /// Get current editor settings from storage
    pub fn get_current_editor_settings(&self) -> EditorDisplaySettings {
        let settings = self.settings_manager.get_settings();
        let editor = settings.editor.unwrap_or_default();

        let font_family = editor
            .font
            .unwrap_or_else(|| self.get_default_editor_font());
        let font_size = editor.font_size.unwrap_or(14);
        let line_height = editor.line_height.unwrap_or(1.4);
        let line_wrapping = editor.line_wrapping.unwrap_or(false);
        let show_invisibles = editor.show_invisibles.unwrap_or(false);
        let tabs_to_spaces = editor.tabs_to_spaces.unwrap_or(false);

        debug!(
            "Loaded editor settings from SettingsManager: {} {}px line-height:{} wrap:{} show_invisibles:{} tabs_to_spaces:{}",
            font_family, font_size, line_height, line_wrapping, show_invisibles, tabs_to_spaces
        );

        let show_line_numbers = settings
            .layout
            .as_ref()
            .and_then(|l| l.show_line_numbers)
            .unwrap_or(true);

        EditorDisplaySettings {
            font_family,
            font_size,
            line_height,
            line_wrapping,
            show_invisibles,
            tabs_to_spaces,
            syntax_colors: editor.syntax_colors.unwrap_or(true),
            show_line_numbers,
        }
    }

    /// Save editor settings to storage
    pub fn save_editor_settings(&self, editor_settings: &EditorDisplaySettings) -> Result<(), Box<dyn std::error::Error>> {
        debug!(
            "Saving editor settings: {} {}px line-height:{} wrap:{} show_invisibles:{} tabs_to_spaces:{}",
            editor_settings.font_family,
            editor_settings.font_size,
            editor_settings.line_height,
            editor_settings.line_wrapping,
            editor_settings.show_invisibles,
            editor_settings.tabs_to_spaces
        );

        self.settings_manager
            .update_settings(|settings| {
                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }

                // Update editor settings
                if let Some(ref mut editor) = settings.editor {
                    editor.font = Some(editor_settings.font_family.clone());
                    editor.font_size = Some(editor_settings.font_size);
                    editor.line_height = Some(editor_settings.line_height);
                    editor.line_wrapping = Some(editor_settings.line_wrapping);
                    editor.show_invisibles = Some(editor_settings.show_invisibles);
                    editor.tabs_to_spaces = Some(editor_settings.tabs_to_spaces);
                    editor.syntax_colors = Some(editor_settings.syntax_colors);
                }
            })
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        debug!("Editor settings saved successfully");
        Ok(())
    }

    /// Get default editor font (prefers monospace)
    pub fn get_default_editor_font(&self) -> String {
        if !self.fonts.monospace.is_empty() {
            // Prefer common monospace fonts for coding
            let preferred = [
                "Hack",
                "Fira Code",
                "Source Code Pro",
                "JetBrains Mono",
                "Ubuntu Mono",
                "Consolas",
                "Monaco",
                "Inconsolata",
            ];

            for font_name in &preferred {
                if self
                    .fonts
                    .monospace
                    .iter()
                    .any(|f| f.name.contains(font_name))
                {
                    return font_name.to_string();
                }
            }

            // Fallback to first available monospace
            if let Some(first_mono) = self.fonts.monospace.first() {
                return first_mono.name.clone();
            }
        }

        // Ultimate fallback
        "Monospace".to_string()
    }
}

/// Editor display settings structure
#[derive(Debug, Clone, PartialEq)]
pub struct EditorDisplaySettings {
    pub font_family: String,
    pub font_size: u8,
    pub line_height: f32,
    pub line_wrapping: bool,
    pub show_invisibles: bool,
    pub tabs_to_spaces: bool,
    pub syntax_colors: bool,
    pub show_line_numbers: bool,
}

impl Default for EditorDisplaySettings {
    fn default() -> Self {
        Self {
            font_family: "Monospace".to_string(),
            font_size: 16,
            line_height: 1.0,
            line_wrapping: false,
            show_invisibles: false,
            tabs_to_spaces: false,
            syntax_colors: true,
            show_line_numbers: true,
        }
    }
}

/// Extract a color value from an editor theme XML file
///
/// Looks for XML color elements like:
/// ```xml
/// <color name="<key>" value="#RRGGBB"/>
/// ```
///
/// Returns the hex color string if found.
pub fn extract_xml_color_value(contents: &str, key: &str) -> Option<String> {
    let needle = format!("name=\"{}\"", key);
    if let Some(pos) = contents.find(&needle) {
        if let Some(val_pos) = contents[pos..].find("value=\"") {
            let start = pos + val_pos + "value=\"".len();
            if let Some(end_rel) = contents[start..].find('"') {
                let val = contents[start..start + end_rel].trim().to_string();
                return Some(val);
            }
        }
    }
    None
}
