use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Application settings that control View menu states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub view_mode: String,
    pub css_theme: String,
    pub ui_theme: String,
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            view_mode: "html".to_string(),
            css_theme: "standard".to_string(),
            ui_theme: "system".to_string(),
            language: "en".to_string(),
        }
    }
}

/// Settings manager that handles loading, saving, and updating app settings
pub struct SettingsManager {
    settings: AppSettings,
    settings_file: String,
    callbacks: Vec<Box<dyn Fn(&AppSettings) + Send + Sync>>,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new() -> Self {
        let settings_file = Self::get_settings_path();
        let settings = Self::load_from_file(&settings_file).unwrap_or_default();
        
        Self {
            settings,
            settings_file,
            callbacks: Vec::new(),
        }
    }
    
    /// Get the path to the settings file (hidden from user)
    fn get_settings_path() -> String {
        // Use a hidden file in the current directory for now
        // In production, this would typically go in a user config directory
        ".marco_settings.json".to_string()
    }
    
    /// Load settings from file
    fn load_from_file(path: &str) -> Result<AppSettings, Box<dyn std::error::Error>> {
        if !Path::new(path).exists() {
            return Ok(AppSettings::default());
        }
        
        let content = fs::read_to_string(path)?;
        let settings: AppSettings = serde_json::from_str(&content)?;
        Ok(settings)
    }
    
    /// Save settings to file
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.settings)?;
        fs::write(&self.settings_file, content)?;
        Ok(())
    }
    
    /// Get current settings
    pub fn get_settings(&self) -> &AppSettings {
        &self.settings
    }
    
    /// Update view mode setting
    pub fn set_view_mode(&mut self, mode: &str) {
        if self.settings.view_mode != mode {
            self.settings.view_mode = mode.to_string();
            self.save_and_notify();
        }
    }
    
    /// Update CSS theme setting
    pub fn set_css_theme(&mut self, theme: &str) {
        if self.settings.css_theme != theme {
            self.settings.css_theme = theme.to_string();
            self.save_and_notify();
        }
    }
    
    /// Update UI theme setting
    pub fn set_ui_theme(&mut self, theme: &str) {
        if self.settings.ui_theme != theme {
            self.settings.ui_theme = theme.to_string();
            self.save_and_notify();
        }
    }
    
    /// Update language setting
    pub fn set_language(&mut self, language: &str) {
        if self.settings.language != language {
            self.settings.language = language.to_string();
            self.save_and_notify();
        }
    }
    
    /// Save settings and notify all callbacks
    fn save_and_notify(&self) {
        if let Err(e) = self.save_to_file() {
            eprintln!("Failed to save settings: {}", e);
        }
        
        // Notify all callbacks that settings have changed
        for callback in &self.callbacks {
            callback(&self.settings);
        }
    }
    
    /// Add a callback to be called when settings change
    pub fn add_change_callback<F>(&mut self, callback: F)
    where
        F: Fn(&AppSettings) + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }
    
    /// Get all view settings as a map for easy iteration
    pub fn get_view_settings_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("view_mode".to_string(), self.settings.view_mode.clone());
        map.insert("css_theme".to_string(), self.settings.css_theme.clone());
        map.insert("ui_theme".to_string(), self.settings.ui_theme.clone());
        map.insert("language".to_string(), self.settings.language.clone());
        map
    }
    
    /// Initialize settings from current application state
    pub fn initialize_from_app_state(&mut self, view_mode: &str, css_theme: &str, ui_theme: &str, language: &str) {
        self.settings.view_mode = view_mode.to_string();
        self.settings.css_theme = css_theme.to_string();
        self.settings.ui_theme = ui_theme.to_string();
        self.settings.language = language.to_string();
        
        if let Err(e) = self.save_to_file() {
            eprintln!("Failed to save initial settings: {}", e);
        }
    }
}

/// Global settings instance (will be properly initialized in main)
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    pub static ref SETTINGS: Arc<Mutex<SettingsManager>> = Arc::new(Mutex::new(SettingsManager::new()));
}

/// Convenience functions for accessing global settings
pub fn get_current_settings() -> AppSettings {
    SETTINGS.lock().unwrap().get_settings().clone()
}

pub fn update_view_mode(mode: &str) {
    SETTINGS.lock().unwrap().set_view_mode(mode);
}

pub fn update_css_theme(theme: &str) {
    SETTINGS.lock().unwrap().set_css_theme(theme);
}

pub fn update_ui_theme(theme: &str) {
    SETTINGS.lock().unwrap().set_ui_theme(theme);
}

pub fn update_language(language: &str) {
    SETTINGS.lock().unwrap().set_language(language);
}

pub fn add_settings_change_callback<F>(callback: F)
where
    F: Fn(&AppSettings) + Send + Sync + 'static,
{
    SETTINGS.lock().unwrap().add_change_callback(callback);
}

pub fn initialize_settings_from_app(view_mode: &str, css_theme: &str, ui_theme: &str, language: &str) {
    SETTINGS.lock().unwrap().initialize_from_app_state(view_mode, css_theme, ui_theme, language);
}
