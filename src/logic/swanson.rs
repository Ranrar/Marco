//! Robust settings loader/saver using RON and Serde
use log::trace;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub editor: Option<EditorSettings>,
    pub appearance: Option<AppearanceSettings>,
    pub layout: Option<LayoutSettings>,
    pub language: Option<LanguageSettings>,
    pub window: Option<WindowSettings>,
    pub files: Option<FileSettings>,
    pub debug: Option<bool>,
    pub log_to_file: Option<bool>,
    pub active_schema: Option<String>,
    pub schema_disabled: Option<bool>,
    pub engine: Option<EngineSettings>,
}

impl Settings {
    /// Load settings from a RON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(&path)?;
        let settings: Self = ron::de::from_str(&content)?;
        Ok(settings)
    }

    /// Save settings to a RON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        // Use a pretty RON serializer to make the settings file human-readable.
        let pretty = ron::ser::PrettyConfig::new();
        let ron = ron::ser::to_string_pretty(self, pretty)?;
        // Resolve path as string for later audit message without moving `path`
        let path_ref = path.as_ref().to_path_buf();
        fs::write(&path_ref, ron)?;
        // Audit: record that settings were saved (don't log sensitive content)
        // We log the path and a Debug representation of the settings for auditing
        // purposes. Use trace level so it's filtered unless enabled.
        if let Some(p) = path_ref.to_str() {
            trace!("audit: settings saved to {} -> {:?}", p, self);
        } else {
            trace!("audit: settings saved -> {:?}", self);
        }
        Ok(())
    }

    /// Get recent files list, validating that files still exist
    pub fn get_recent_files(&self) -> Vec<PathBuf> {
        if let Some(files_settings) = &self.files {
            if let Some(recent_files) = &files_settings.recent_files {
                // Filter out files that no longer exist
                return recent_files
                    .iter()
                    .filter(|path| path.exists())
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }

    /// Add a file to recent files list
    pub fn add_recent_file<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref().to_path_buf();

        // Get max files before borrowing mutably
        let max_files = self.get_max_recent_files();

        // Ensure files settings exists
        if self.files.is_none() {
            self.files = Some(FileSettings::default());
        }

        let files_settings = self.files.as_mut().unwrap();

        // Ensure recent_files vec exists
        if files_settings.recent_files.is_none() {
            files_settings.recent_files = Some(Vec::new());
        }

        let recent_files = files_settings.recent_files.as_mut().unwrap();

        // Remove if already exists (to move to front)
        recent_files.retain(|p| p != &path);

        // Add to front
        recent_files.insert(0, path);

        // Limit to max files
        if recent_files.len() > max_files {
            recent_files.truncate(max_files);
        }
    }

    /// Clear all recent files
    pub fn clear_recent_files(&mut self) {
        if let Some(files_settings) = &mut self.files {
            files_settings.recent_files = Some(Vec::new());
        }
    }

    /// Get maximum number of recent files to store
    pub fn get_max_recent_files(&self) -> usize {
        if let Some(files_settings) = &self.files {
            if let Some(max_files) = files_settings.max_recent_files {
                return max_files as usize;
            }
        }
        5 // Default to 5 recent files
    }

    /// Clean up recent files list by removing non-existent files
    pub fn clean_recent_files(&mut self) -> bool {
        if let Some(files_settings) = &mut self.files {
            if let Some(recent_files) = &mut files_settings.recent_files {
                let original_len = recent_files.len();
                recent_files.retain(|path| path.exists());
                return recent_files.len() != original_len;
            }
        }
        false
    }

    /// Get the line break mode for HTML rendering
    pub fn get_line_break_mode(&self) -> String {
        self.engine
            .as_ref()
            .and_then(|e| e.render.as_ref())
            .and_then(|r| r.html.as_ref())
            .and_then(|h| h.line_break_mode.clone())
            .unwrap_or_else(|| "normal".to_string())
    }

    /// Get window settings, creating default if none exist
    pub fn get_window_settings(&self) -> WindowSettings {
        self.window.clone().unwrap_or_default()
    }

    /// Get mutable reference to window settings, creating if none exist
    pub fn get_or_create_window_settings(&mut self) -> &mut WindowSettings {
        if self.window.is_none() {
            self.window = Some(WindowSettings::default());
        }
        self.window.as_mut().unwrap()
    }

    /// Update window settings
    pub fn update_window_settings<F>(&mut self, updater: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut WindowSettings),
    {
        let window_settings = self.get_or_create_window_settings();
        updater(window_settings);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorSettings {
    pub font: Option<String>,
    pub font_size: Option<u8>,
    pub line_height: Option<f32>,
    pub line_wrapping: Option<bool>,
    pub auto_pairing: Option<bool>,
    pub show_invisibles: Option<bool>,
    pub tabs_to_spaces: Option<bool>,
    pub syntax_colors: Option<bool>,
    pub linting: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppearanceSettings {
    pub editor_mode: Option<String>,
    pub preview_theme: Option<String>,
    pub ui_font: Option<String>,
    pub ui_font_size: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutSettings {
    pub view_mode: Option<String>,
    pub sync_scrolling: Option<bool>,
    pub editor_view_split: Option<u8>,
    pub show_line_numbers: Option<bool>,
    pub text_direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LanguageSettings {
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: Option<bool>,
    pub split_ratio: Option<i32>, // between 10% and 90%
}

impl WindowSettings {
    /// Get the split ratio or return default (60%)
    pub fn get_split_ratio(&self) -> i32 {
        self.split_ratio.unwrap_or(60)
    }

    /// Set the split ratio, ensuring it's within valid bounds (10-90%)
    pub fn set_split_ratio(&mut self, ratio: i32) {
        self.split_ratio = Some(ratio.clamp(10, 90));
    }

    /// Get window dimensions or return defaults
    pub fn get_window_size(&self) -> (u32, u32) {
        (self.width.unwrap_or(1200), self.height.unwrap_or(800))
    }

    /// Get window position or return None (let window manager decide)
    pub fn get_window_position(&self) -> Option<(i32, i32)> {
        if let (Some(x), Some(y)) = (self.x, self.y) {
            Some((x, y))
        } else {
            None
        }
    }

    /// Check if window should be maximized
    pub fn is_maximized(&self) -> bool {
        self.maximized.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileSettings {
    pub recent_files: Option<Vec<PathBuf>>,
    pub max_recent_files: Option<u8>,
}

/// Marco Engine specific settings integrated with main application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineSettings {
    /// Rendering configuration
    pub render: Option<EngineRenderSettings>,
}

/// Rendering-specific engine settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineRenderSettings {
    /// HTML rendering options
    pub html: Option<EngineHtmlSettings>,
}

/// HTML rendering specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineHtmlSettings {
    /// Generate table of contents (UI setting - not consumed by current Marco engine)
    pub generate_toc: Option<bool>,
    /// Include metadata in HTML head (UI setting - not consumed by current Marco engine)
    pub include_metadata: Option<bool>,
    /// Line break behavior: "normal" (CommonMark) or "reversed" (Marco)
    /// Normal: Single Enter = soft break (no <br>), Double space/backslash + Enter = hard break (<br>)
    /// Reversed: Single Enter = hard break (<br>), Double space/backslash + Enter = soft break (no <br>)
    pub line_break_mode: Option<String>,
}
