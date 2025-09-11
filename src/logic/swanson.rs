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
    pub advanced: Option<AdvancedSettings>,
    pub files: Option<FileSettings>,
    // Toggle to enable debug features in the UI
    pub debug: Option<bool>,
    // Logging to file configuration
    pub log_to_file: Option<bool>,
    // --- Markdown schema selection ---
    pub active_schema: Option<String>,
    pub schema_disabled: Option<bool>,
    // --- Marco Engine Settings ---
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

/// Marco Engine specific settings integrated with main application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineSettings {
    /// Parser configuration
    pub parser: Option<EngineParserSettings>,
    /// Rendering configuration  
    pub render: Option<EngineRenderSettings>,
    /// Performance and processing configuration
    pub performance: Option<EnginePerformanceSettings>,
    /// Output format preferences
    pub output: Option<EngineOutputSettings>,
}

/// Parser-specific engine settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineParserSettings {
    /// Enable position tracking for better error reporting
    pub track_positions: Option<bool>,
    /// Enable parse result caching for performance
    pub enable_cache: Option<bool>,
    /// Maximum number of cached parse results
    pub max_cache_size: Option<usize>,
    /// Enable detailed error reporting
    pub detailed_errors: Option<bool>,
    /// Collect parsing statistics for debugging
    pub collect_stats: Option<bool>,
}

/// Rendering-specific engine settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineRenderSettings {
    /// HTML rendering options
    pub html: Option<EngineHtmlSettings>,
    /// Text rendering options
    pub text: Option<EngineTextSettings>,
    /// Default output format ("html", "text", "json", "json_pretty")
    pub default_format: Option<String>,
}

/// HTML rendering specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineHtmlSettings {
    /// Include CSS styling in output
    pub include_styles: Option<bool>,
    /// Generate table of contents
    pub generate_toc: Option<bool>,
    /// Enable syntax highlighting for code blocks (syncs with editor.syntax_colors)
    pub syntax_highlighting: Option<bool>,
    /// Use semantic HTML5 elements
    pub semantic_html: Option<bool>,
    /// Include metadata in HTML head
    pub include_metadata: Option<bool>,
    /// Line break behavior: "normal" (CommonMark) or "reversed" (Marco)
    /// Normal: Single Enter = soft break (no <br>), Double space/backslash + Enter = hard break (<br>)
    /// Reversed: Single Enter = hard break (<br>), Double space/backslash + Enter = soft break (no <br>)
    pub line_break_mode: Option<String>,
}

/// Text rendering specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineTextSettings {
    /// Width for text wrapping (0 = no wrapping)
    pub wrap_width: Option<usize>,
    /// Include formatting markers in output
    pub preserve_formatting: Option<bool>,
    /// Convert links to footnotes
    pub links_as_footnotes: Option<bool>,
}

/// Performance and processing settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnginePerformanceSettings {
    /// Number of worker threads for parallel processing (always enabled)
    pub worker_threads: Option<usize>,
    /// Enable AST caching for repeated operations
    pub cache_ast: Option<bool>,
    /// Maximum memory usage for caching (in MB)
    pub max_cache_memory_mb: Option<usize>,
    /// Enable debug mode for verbose output (syncs with main debug setting)
    pub debug_mode: Option<bool>,
}

/// Output format and behavior settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineOutputSettings {
    /// Pretty print JSON output
    pub pretty_json: Option<bool>,
    /// Include source position information in output
    pub include_positions: Option<bool>,
    /// Include parsing statistics in output
    pub include_stats: Option<bool>,
}
