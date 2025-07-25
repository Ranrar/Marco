/// Tracks current values and changes in the settings dialog
#[derive(Debug, Clone)]
pub struct SettingsChangeTracker {
    // Current values in the dialog (not yet saved)
    pub function_highlighting: bool,
    pub editor_color_syntax: bool,
    pub markdown_warnings: bool,
    pub editor_text_wrap: bool,
    pub ui_theme: String,
    pub css_theme: String,
    pub layout_mode: String,
    pub layout_ratio: i32,
    pub view_mode: String,
    pub language: String,
    pub custom_css_file: String,
    pub debounce_timeout_ms: i32,
}

impl SettingsChangeTracker {
    pub fn load_current() -> Self {
        let prefs = get_app_preferences();
        let layout_mode = prefs.get_layout_mode();
        let layout_ratio = prefs.get_layout_ratio();
        Self {
            function_highlighting: prefs.get_function_highlighting(),
            editor_color_syntax: prefs.get_editor_color_syntax(),
            markdown_warnings: prefs.get_markdown_warnings(),
            editor_text_wrap: prefs.get_editor_text_wrap(),
            ui_theme: prefs.get_ui_theme(),
            css_theme: prefs.get_css_theme(),
            layout_mode,
            layout_ratio,
            view_mode: prefs.get_view_mode(),
            language: prefs.get_language(),
            custom_css_file: prefs.get_custom_css_file(),
            debounce_timeout_ms: prefs.get_debounce_timeout_ms(),
        }
    }

    pub fn has_changes(&self, original: &OriginalSettings) -> bool {
        self.function_highlighting != original.function_highlighting
            || self.editor_color_syntax != original.editor_color_syntax
            || self.markdown_warnings != original.markdown_warnings
            || self.editor_text_wrap != original.editor_text_wrap
            || self.ui_theme != original.ui_theme
            || self.css_theme != original.css_theme
            || self.layout_mode != original.layout_mode
            || self.layout_ratio != original.layout_ratio
            || self.view_mode != original.view_mode
            || self.language != original.language
            || self.custom_css_file != original.custom_css_file
            || self.debounce_timeout_ms != original.debounce_timeout_ms
    }

    pub fn apply_changes(
        &self,
        editor: &crate::editor::MarkdownEditor,
        theme_manager: &crate::theme::ThemeManager,
    ) {
        let prefs = get_app_preferences();

        // Store current values for comparison
        let old_ui_theme = prefs.get_ui_theme();
        let old_css_theme = prefs.get_css_theme();
        let old_language = prefs.get_language();
        let _old_view_mode = prefs.get_view_mode();
        let _old_layout_mode = prefs.get_layout_mode();
        let old_function_highlighting = prefs.get_function_highlighting();
        let old_editor_color_syntax = prefs.get_editor_color_syntax();
        let old_markdown_warnings = prefs.get_markdown_warnings();

        let old_editor_text_wrap = prefs.get_editor_text_wrap();

        // Apply all changes to settings
        prefs.set_function_syntax_coloring(self.function_highlighting);
        prefs.set_editor_color_syntax(self.editor_color_syntax);
        prefs.set_markdown_warnings(self.markdown_warnings);
        prefs.set_editor_text_wrap(self.editor_text_wrap);
        prefs.set_ui_theme(&self.ui_theme);
        prefs.set_css_theme(&self.css_theme);
        prefs.set_layout_mode(&self.layout_mode);
        prefs.set_layout_ratio(self.layout_ratio);
        prefs.set_view_mode(&self.view_mode);
        prefs.set_language(&self.language);
        prefs.set_custom_css_file(&self.custom_css_file);
        prefs.set_debounce_timeout_ms(self.debounce_timeout_ms);
        // Apply text wrap setting immediately if changed
        if old_editor_text_wrap != self.editor_text_wrap {
            editor.set_text_wrap(self.editor_text_wrap);
        }

        // Update the editor/viewer split after saving
        // Get the current window width and set the paned position
        if let Some(window) = editor.widget.root().and_then(|w| w.downcast::<gtk4::Window>().ok()) {
            let total_width = window.allocated_width();
            let min = 200;
            let max = (total_width - 200).max(min);
            let mut pos = (total_width * self.layout_ratio / 100).clamp(min, max);
            if pos < min { pos = min; }
            if pos > max { pos = max; }
            editor.widget.set_position(pos);
        } else {
            // Fallback: use paned width if window not found
            let total_width = editor.widget.width();
            let min = 200;
            let max = (total_width - 200).max(min);
            let mut pos = (total_width * self.layout_ratio / 100).clamp(min, max);
            if pos < min { pos = min; }
            if pos > max { pos = max; }
            editor.widget.set_position(pos);
        }

        // Apply UI theme changes immediately
        if old_ui_theme != self.ui_theme {
            let new_theme = match self.ui_theme.as_str() {
                "light" => crate::theme::Theme::Light,
                "dark" => crate::theme::Theme::Dark,
                "system" => crate::theme::Theme::System,
                _ => crate::theme::Theme::System,
            };
            theme_manager.set_theme(new_theme);
            
            // When UI theme changes, also update the CSS theme to use the correct variant
            if let Ok(_css_content) = crate::ui::css_theme::CssTheme::set_css_theme(&self.css_theme) {
                // This will automatically apply the correct light/dark variant
                editor.set_css_theme(&self.css_theme);
            }
            
            // Also update the editor theme immediately
            editor.update_editor_theme();
        }

        // Apply CSS theme changes immediately  
        if old_css_theme != self.css_theme {
            // When CSS theme changes, check current UI theme and apply correct variant
            if let Ok(_css_content) = crate::ui::css_theme::CssTheme::set_css_theme(&self.css_theme) {
                editor.set_css_theme(&self.css_theme);
            }
        }

        // Apply language changes immediately
        if old_language != self.language {
            crate::language::set_locale(&self.language);
        }
        if old_function_highlighting != self.function_highlighting {
            editor.set_function_colloring(self.function_highlighting);
        }
        if old_editor_color_syntax != self.editor_color_syntax {
            editor.set_editor_color_syntax(self.editor_color_syntax);
        }
        if old_markdown_warnings != self.markdown_warnings {
            editor.set_markdown_warnings(self.markdown_warnings);
        }
    }
}

/// Stores original values to allow reverting changes
#[derive(Debug, Clone)]
pub struct OriginalSettings {
    pub function_highlighting: bool,
    pub editor_color_syntax: bool,
    pub markdown_warnings: bool,
    pub editor_text_wrap: bool,
    pub ui_theme: String,
    pub css_theme: String,
    pub layout_mode: String,
    pub layout_ratio: i32,
    pub view_mode: String,
    pub language: String,
    pub custom_css_file: String,
    pub debounce_timeout_ms: i32,
}

impl OriginalSettings {
    pub fn load_current() -> Self {
        let prefs = get_app_preferences();
        Self {
            function_highlighting: prefs.get_function_highlighting(),
            editor_color_syntax: prefs.get_editor_color_syntax(),
            markdown_warnings: prefs.get_markdown_warnings(),
            editor_text_wrap: prefs.get_editor_text_wrap(),
            ui_theme: prefs.get_ui_theme(),
            css_theme: prefs.get_css_theme(),
            layout_mode: prefs.get_layout_mode(),
            layout_ratio: prefs.get_layout_ratio(),
            view_mode: prefs.get_view_mode(),
            language: prefs.get_language(),
            custom_css_file: prefs.get_custom_css_file(),
            debounce_timeout_ms: prefs.get_debounce_timeout_ms(),
        }
    }



}
use gio::prelude::*;
use gio::Settings;
use gtk4::prelude::WidgetExt;
use std::cell::RefCell;

/// Application settings using GSettings
pub struct AppPreferences {
    settings: Settings,
}

impl AppPreferences {
    /// Editor text wrap toggle
    pub fn get_editor_text_wrap(&self) -> bool {
        self.settings.boolean("editor-text-wrap")
    }

    pub fn set_editor_text_wrap(&self, enabled: bool) {
        let _ = self.settings.set_boolean("editor-text-wrap", enabled);
    }
    /// Debounce timeout (ms)
    pub fn get_debounce_timeout_ms(&self) -> i32 {
        self.settings.int("debounce-timeout-ms")
    }

    pub fn set_debounce_timeout_ms(&self, value: i32) {
        let _ = self.settings.set_int("debounce-timeout-ms", value);
    }
    /// Create a new AppPreferences instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let settings = Settings::new("org.marco.editor");
        Ok(Self { settings })
    }

    /// Function highlighting toggle
    pub fn get_function_highlighting(&self) -> bool {
        self.settings.boolean("function-highlighting")
    }

    pub fn set_function_syntax_coloring(&self, enabled: bool) {
        let _ = self.settings.set_boolean("function-highlighting", enabled);
    }

    /// Editor color syntax highlighting toggle
    pub fn get_editor_color_syntax(&self) -> bool {
        self.settings.boolean("syntax-color-enabled")
    }

    pub fn set_editor_color_syntax(&self, enabled: bool) {
        let _ = self.settings.set_boolean("syntax-color-enabled", enabled);
    }

    /// Markdown format detection
    pub fn get_markdown_warnings(&self) -> bool {
        self.settings.boolean("markdown-warnings")
    }

    pub fn set_markdown_warnings(&self, enabled: bool) {
        let _ = self.settings.set_boolean("markdown-warnings", enabled);
    }


    /// Window size and position
    pub fn get_window_size(&self) -> (i32, i32) {
        let width = self.settings.int("window-width");
        let height = self.settings.int("window-height");
        (width, height)
    }

    pub fn set_window_size(&self, width: i32, height: i32) {
        let _ = self.settings.set_int("window-width", width);
        let _ = self.settings.set_int("window-height", height);
    }


    pub fn get_window_maximized(&self) -> bool {
        self.settings.boolean("window-maximized")
    }

    pub fn set_window_maximized(&self, maximized: bool) {
        let _ = self.settings.set_boolean("window-maximized", maximized);
    }

    /// Layout preferences
    pub fn get_layout_mode(&self) -> String {
        self.settings.string("layout-mode").to_string()
    }

    pub fn set_layout_mode(&self, mode: &str) {
        let _ = self.settings.set_string("layout-mode", mode);
    }

    /// Editor/Viewer split ratio (percentage of editor width)
    pub fn get_layout_ratio(&self) -> i32 {
        self.settings.int("layout-ratio")
    }

    pub fn set_layout_ratio(&self, ratio: i32) {
        let _ = self.settings.set_int("layout-ratio", ratio);
    }

    /// Theme settings
    pub fn get_ui_theme(&self) -> String {
        self.settings.string("ui-theme").to_string()
    }

    pub fn set_ui_theme(&self, theme: &str) {
        let _ = self.settings.set_string("ui-theme", theme);
    }

    /// CSS theme settings
    pub fn get_css_theme(&self) -> String {
        self.settings.string("css-theme").to_string()
    }

    pub fn set_css_theme(&self, theme: &str) {
        let _ = self.settings.set_string("css-theme", theme);
    }

    /// Custom CSS file path
    pub fn get_custom_css_file(&self) -> String {
        self.settings.string("custom-css-file").to_string()
    }

    pub fn set_custom_css_file(&self, path: &str) {
        let _ = self.settings.set_string("custom-css-file", path);
    }

    /// Language settings
    pub fn get_language(&self) -> String {
        self.settings.string("language").to_string()
    }

    pub fn set_language(&self, language: &str) {
        let _ = self.settings.set_string("language", language);
    }

    /// View mode settings
    pub fn get_view_mode(&self) -> String {
        self.settings.string("view-mode").to_string()
    }

    pub fn set_view_mode(&self, mode: &str) {
        let _ = self.settings.set_string("view-mode", mode);
    }


    /// Connect to settings changes
    pub fn connect_changed<F>(&self, key: Option<&str>, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Settings, &str) + 'static,
    {
        self.settings.connect_changed(key, callback)
    }

    /// Reset all settings to default values
    pub fn reset_to_defaults(&self) {
        // Reset all keys to their default values
        let _ = self.settings.reset("function-highlighting");
        let _ = self.settings.reset("syntax-color-enabled");
        let _ = self.settings.reset("markdown-warnings");
        let _ = self.settings.reset("ui-theme");
        let _ = self.settings.reset("css-theme");
        let _ = self.settings.reset("custom-css-file");
        let _ = self.settings.reset("layout-mode");
        let _ = self.settings.reset("window-width");
        let _ = self.settings.reset("window-height");
        let _ = self.settings.reset("window-x");
        let _ = self.settings.reset("window-y");
        let _ = self.settings.reset("window-maximized");
        let _ = self.settings.reset("language");
        let _ = self.settings.reset("view-mode");
    }
}

thread_local! {
    static APP_PREFERENCES: RefCell<Option<std::rc::Rc<AppPreferences>>> = RefCell::new(None);
}

pub fn initialize_global_settings() -> Result<(), Box<dyn std::error::Error>> {
    let prefs = std::rc::Rc::new(AppPreferences::new()?);
    APP_PREFERENCES.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_some() {
            return Err("Settings already initialized".to_string());
        }
        *opt = Some(prefs);
        Ok(())
    })?;
    Ok(())
}

/// Get a reference to the global settings instance (main thread only)
pub fn get_app_preferences() -> std::rc::Rc<AppPreferences> {
    APP_PREFERENCES.with(|cell| {
        cell.borrow().as_ref().expect("Settings not initialized. Call initialize_global_settings() first.").clone()
    })
}


/// Initialize settings system
pub fn initialize_settings() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the global settings instance
    initialize_global_settings()?;

    println!("GSettings initialized successfully");
    Ok(())
}
