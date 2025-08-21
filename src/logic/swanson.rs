//! Robust settings loader/saver using RON and Serde
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
    pub advanced: Option<AdvancedSettings>,
    pub files: Option<FileSettings>,
    // --- Markdown schema selection ---
    pub active_schema: Option<String>,
    pub schema_disabled: Option<bool>,
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
        let ron = ron::ser::to_string(self)?;
        fs::write(path, ron)?;
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvancedSettings {
    pub enabled_variants: Option<Vec<String>>,
    pub plugins: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileSettings {
    pub recent_files: Option<Vec<PathBuf>>,
    pub max_recent_files: Option<u8>,
}
