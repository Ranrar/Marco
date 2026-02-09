//! Centralized Settings Manager using RON and Serde
//!
//! This module provides thread-safe, centralized settings management for Marco.
//! SettingsManager is the single authority for all settings operations.

use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Type alias for settings change listener callbacks
type SettingsListener = Arc<dyn Fn(&Settings) + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    // Marco-specific settings
    pub editor: Option<EditorSettings>,
    pub layout: Option<LayoutSettings>,
    pub window: Option<WindowSettings>, // Marco window settings
    pub files: Option<FileSettings>,
    pub active_schema: Option<String>,
    pub schema_disabled: Option<bool>,

    // Polo-specific settings
    pub polo: Option<PoloSettings>,

    // Common settings (shared between Marco and Polo)
    pub appearance: Option<AppearanceSettings>,
    pub language: Option<LanguageSettings>,
    pub telemetry: Option<TelemetrySettings>,
    pub debug: Option<bool>,
    pub log_to_file: Option<bool>,
    pub engine: Option<EngineSettings>,
}

impl Settings {
    /// Load settings from a RON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
        let settings: Self = ron::de::from_str(&content)?;
        Ok(settings)
    }

    /// Save settings to a RON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
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
    pub fn update_window_settings<F>(
        &mut self,
        updater: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut WindowSettings),
    {
        let window_settings = self.get_or_create_window_settings();
        updater(window_settings);
        Ok(())
    }

    /// Create default settings for the current system
    pub fn create_default_for_system() -> Self {
        Settings {
            // Marco-specific settings
            editor: Some(EditorSettings {
                font_size: Some(12),
                line_wrapping: Some(true),
                auto_pairing: Some(true),
                show_invisibles: Some(false),
                tabs_to_spaces: Some(true),
                syntax_colors: Some(true),
                linting: Some(true),
                ..Default::default()
            }),
            layout: Some(LayoutSettings {
                view_mode: Some("HTML Preview".to_string()),
                sync_scrolling: Some(true),
                editor_view_split: Some(60),
                show_line_numbers: Some(true),
                text_direction: Some("ltr".to_string()),
            }),
            window: Some(WindowSettings {
                width: Some(1200),
                height: Some(800),
                maximized: Some(false),
                split_ratio: Some(60),
                ..Default::default()
            }),
            files: Some(FileSettings {
                recent_files: Some(Vec::new()),
                max_recent_files: Some(5),
            }),
            active_schema: None,
            schema_disabled: None,

            // Polo-specific settings
            polo: Some(PoloSettings {
                window: Some(PoloWindowSettings {
                    width: Some(1000),
                    height: Some(800),
                    maximized: Some(false),
                    ..Default::default()
                }),
                last_opened_file: None,
                auto_refresh: Some(false),
                refresh_interval_ms: Some(1000),
            }),

            // Common settings (shared between Marco and Polo)
            appearance: Some(AppearanceSettings {
                editor_mode: Some("marco-light".to_string()),
                preview_theme: Some("marco".to_string()),
                ui_font_size: Some(11),
                ..Default::default()
            }),
            language: Some(LanguageSettings {
                language: Some("en".to_string()),
            }),
            telemetry: Some(TelemetrySettings {
                enabled: Some(false),
                first_run_dialog_shown: Some(false),
            }),
            debug: Some(false),
            log_to_file: Some(false),
            engine: None,
        }
    }
}

#[derive(Debug)]
pub enum SettingsError {
    Io(std::io::Error),
    Parse(ron::error::SpannedError),
    Validation(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::Io(e) => write!(f, "IO error: {}", e),
            SettingsError::Parse(e) => write!(f, "Parse error: {}", e),
            SettingsError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<std::io::Error> for SettingsError {
    fn from(error: std::io::Error) -> Self {
        SettingsError::Io(error)
    }
}

impl From<ron::error::SpannedError> for SettingsError {
    fn from(error: ron::error::SpannedError) -> Self {
        SettingsError::Parse(error)
    }
}

impl From<ron::Error> for SettingsError {
    fn from(error: ron::Error) -> Self {
        SettingsError::Validation(format!("RON serialization error: {}", error))
    }
}

/// Centralized settings manager providing thread-safe access and change notifications
pub struct SettingsManager {
    settings: Arc<RwLock<Settings>>,
    settings_path: PathBuf,
    change_listeners: Arc<RwLock<HashMap<String, SettingsListener>>>,
    last_modified: Arc<RwLock<Option<SystemTime>>>,
}

impl SettingsManager {
    /// Initialize the settings manager with robust file handling
    pub fn initialize(settings_path: PathBuf) -> Result<Arc<Self>, SettingsError> {
        let manager = Arc::new(SettingsManager {
            settings: Arc::new(RwLock::new(Settings::default())),
            settings_path: settings_path.clone(),
            change_listeners: Arc::new(RwLock::new(HashMap::new())),
            last_modified: Arc::new(RwLock::new(None)),
        });

        // Ensure settings file exists and load settings
        manager.ensure_settings_file_exists()?;
        manager.reload_settings()?;

        Ok(manager)
    }

    /// Get current settings (read-only clone)
    pub fn get_settings(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    /// Update settings using a closure and notify listeners
    pub fn update_settings<F>(&self, updater: F) -> Result<(), SettingsError>
    where
        F: FnOnce(&mut Settings),
    {
        {
            let mut settings = self.settings.write().unwrap();
            updater(&mut settings);

            // Validate settings after update
            if let Err(validation_errors) = self.validate_settings(&settings) {
                return Err(SettingsError::Validation(validation_errors));
            }
        }

        // Save to file
        self.save_settings()?;

        // Notify listeners
        self.notify_listeners();

        Ok(())
    }

    /// Register a change listener
    pub fn register_change_listener<F>(&self, id: String, callback: F)
    where
        F: Fn(&Settings) + Send + Sync + 'static,
    {
        let mut listeners = self.change_listeners.write().unwrap();
        listeners.insert(id, Arc::new(callback));
    }

    /// Remove a change listener
    pub fn remove_change_listener(&self, id: &str) {
        let mut listeners = self.change_listeners.write().unwrap();
        listeners.remove(id);
    }

    /// Register a listener specifically for theme/appearance changes
    pub fn register_theme_listener<F>(&self, id: String, callback: F)
    where
        F: Fn(&AppearanceSettings) + Send + Sync + 'static,
    {
        self.register_change_listener(id, move |settings| {
            if let Some(appearance) = &settings.appearance {
                callback(appearance);
            }
        });
    }

    /// Register a listener specifically for editor settings changes
    pub fn register_editor_listener<F>(&self, id: String, callback: F)
    where
        F: Fn(&EditorSettings) + Send + Sync + 'static,
    {
        self.register_change_listener(id, move |settings| {
            if let Some(editor) = &settings.editor {
                callback(editor);
            }
        });
    }

    /// Register a listener specifically for window settings changes
    pub fn register_window_listener<F>(&self, id: String, callback: F)
    where
        F: Fn(&WindowSettings) + Send + Sync + 'static,
    {
        self.register_change_listener(id, move |settings| {
            if let Some(window) = &settings.window {
                callback(window);
            }
        });
    }

    /// Register a listener specifically for layout settings changes
    pub fn register_layout_listener<F>(&self, id: String, callback: F)
    where
        F: Fn(&LayoutSettings) + Send + Sync + 'static,
    {
        self.register_change_listener(id, move |settings| {
            if let Some(layout) = &settings.layout {
                callback(layout);
            }
        });
    }

    /// Ensure settings file exists, create with defaults if missing
    pub fn ensure_settings_file_exists(&self) -> Result<(), SettingsError> {
        if !self.settings_path.exists() {
            // Ensure parent directory exists
            if let Some(parent) = self.settings_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Create default settings
            let default_settings = Settings::create_default_for_system();

            // Save with pretty formatting
            let pretty_config = ron::ser::PrettyConfig::new()
                .enumerate_arrays(true)
                .indentor("  ".to_string());
            let ron_content = ron::ser::to_string_pretty(&default_settings, pretty_config)?;

            fs::write(&self.settings_path, ron_content)?;

            trace!("Created default settings file at {:?}", self.settings_path);
        }

        Ok(())
    }

    /// Reload settings from file
    fn reload_settings(&self) -> Result<(), SettingsError> {
        let content = fs::read_to_string(&self.settings_path)?;
        let parsed_settings: Settings = ron::de::from_str(&content)?;

        // Validate loaded settings
        if let Err(validation_error) = self.validate_settings(&parsed_settings) {
            warn!(
                "Settings validation failed, using repaired settings: {}",
                validation_error
            );
            let mut repaired_settings = parsed_settings;
            self.repair_invalid_settings(&mut repaired_settings);
            *self.settings.write().unwrap() = repaired_settings;
        } else {
            *self.settings.write().unwrap() = parsed_settings;
        }

        // Update last modified time
        if let Ok(metadata) = fs::metadata(&self.settings_path) {
            if let Ok(modified) = metadata.modified() {
                *self.last_modified.write().unwrap() = Some(modified);
            }
        }

        Ok(())
    }

    /// Save current settings to file
    fn save_settings(&self) -> Result<(), SettingsError> {
        let settings = self.settings.read().unwrap();
        let pretty_config = ron::ser::PrettyConfig::new()
            .enumerate_arrays(true)
            .indentor("  ".to_string());
        let ron_content = ron::ser::to_string_pretty(&*settings, pretty_config)?;

        fs::write(&self.settings_path, ron_content)?;

        // Update last modified time
        if let Ok(metadata) = fs::metadata(&self.settings_path) {
            if let Ok(modified) = metadata.modified() {
                *self.last_modified.write().unwrap() = Some(modified);
            }
        }

        trace!("Settings saved to {:?}", self.settings_path);
        Ok(())
    }

    /// Validate settings and return error message if invalid
    fn validate_settings(&self, settings: &Settings) -> Result<(), String> {
        let mut errors = Vec::new();

        // Validate editor settings
        if let Some(editor) = &settings.editor {
            if let Some(font_size) = editor.font_size {
                if !(8..=72).contains(&font_size) {
                    errors.push(format!(
                        "Font size {} is out of valid range (8-72)",
                        font_size
                    ));
                }
            }
        }

        // Validate window settings
        if let Some(window) = &settings.window {
            if let Some(width) = window.width {
                if !(400..=5000).contains(&width) {
                    errors.push(format!(
                        "Window width {} is out of valid range (400-5000)",
                        width
                    ));
                }
            }
            if let Some(height) = window.height {
                if !(300..=4000).contains(&height) {
                    errors.push(format!(
                        "Window height {} is out of valid range (300-4000)",
                        height
                    ));
                }
            }
            if let Some(split_ratio) = window.split_ratio {
                if !(10..=90).contains(&split_ratio) {
                    errors.push(format!(
                        "Split ratio {} is out of valid range (10-90)",
                        split_ratio
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Repair invalid settings by clamping to valid ranges and removing invalid entries
    fn repair_invalid_settings(&self, settings: &mut Settings) {
        // Repair editor settings
        if let Some(editor) = &mut settings.editor {
            if let Some(font_size) = &mut editor.font_size {
                *font_size = (*font_size).clamp(8, 72);
            }
        }

        // Repair window settings
        if let Some(window) = &mut settings.window {
            if let Some(width) = &mut window.width {
                *width = (*width).clamp(400, 5000);
            }
            if let Some(height) = &mut window.height {
                *height = (*height).clamp(300, 4000);
            }
            if let Some(split_ratio) = &mut window.split_ratio {
                *split_ratio = (*split_ratio).clamp(10, 90);
            }
        }

        // Remove non-existent recent files
        if let Some(files) = &mut settings.files {
            if let Some(recent_files) = &mut files.recent_files {
                recent_files.retain(|path| path.exists());
            }
        }
    }

    /// Notify all registered listeners of settings changes
    fn notify_listeners(&self) {
        let settings = self.get_settings();
        let listeners_to_notify: Vec<(String, SettingsListener)> = {
            let listeners = self.change_listeners.read().unwrap();
            listeners
                .iter()
                .map(|(id, listener)| (id.clone(), Arc::clone(listener)))
                .collect()
        };

        for (id, listener) in listeners_to_notify {
            // Use trace level to avoid spamming logs
            trace!("Notifying settings listener: {}", id);
            (listener)(&settings);
        }
    }

    /// Get the settings file path
    pub fn get_settings_path(&self) -> &Path {
        &self.settings_path
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
pub struct TelemetrySettings {
    pub enabled: Option<bool>,
    pub first_run_dialog_shown: Option<bool>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoloSettings {
    pub window: Option<PoloWindowSettings>,
    pub last_opened_file: Option<PathBuf>,
    pub auto_refresh: Option<bool>, // Future: watch file for changes
    pub refresh_interval_ms: Option<u32>, // Future: how often to check for changes
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoloWindowSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: Option<bool>,
}

impl PoloWindowSettings {
    /// Get window dimensions or return defaults (optimized for reading)
    pub fn get_window_size(&self) -> (u32, u32) {
        (self.width.unwrap_or(1000), self.height.unwrap_or(800))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_ron_0_11_compatibility() {
        // Test that RON 0.11 can parse the actual settings file
        let settings_path = "src/assets/settings.ron";

        // Skip if settings file doesn't exist (CI environment)
        if !std::path::Path::new(settings_path).exists() {
            return;
        }

        // Test loading settings
        let result = Settings::load_from_file(settings_path);
        assert!(
            result.is_ok(),
            "Failed to load settings with RON 0.11: {:?}",
            result.err()
        );

        let settings = result.unwrap();

        // Verify some expected fields exist
        assert!(
            settings.editor.is_some(),
            "Editor settings should be present"
        );
        assert!(
            settings.appearance.is_some(),
            "Appearance settings should be present"
        );

        // Test that we can serialize it back
        let pretty = ron::ser::PrettyConfig::new();
        let serialized = ron::ser::to_string_pretty(&settings, pretty);
        assert!(
            serialized.is_ok(),
            "Failed to serialize settings with RON 0.11: {:?}",
            serialized.err()
        );

        // Test that the serialized version can be parsed again
        let reparsed: Result<Settings, _> = ron::de::from_str(&serialized.unwrap());
        assert!(
            reparsed.is_ok(),
            "Failed to reparse serialized settings: {:?}",
            reparsed.err()
        );
    }

    #[test]
    fn smoke_test_ron_error_types() {
        // Test that RON 0.11 error types work correctly with our error handling
        let bad_ron = "( invalid: Some( }";
        let result: Result<Settings, _> = ron::de::from_str(bad_ron);
        assert!(result.is_err(), "Should fail to parse invalid RON");

        // Test that error can be converted to SettingsError
        let settings_error: SettingsError = result.unwrap_err().into();
        assert!(matches!(settings_error, SettingsError::Parse(_)));
    }
}
