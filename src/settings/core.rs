use gio::prelude::*;
use gio::Settings;

/// Application settings using GSettings
pub struct AppPreferences {
    settings: Settings,
}

impl AppPreferences {
    /// Create a new AppPreferences instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let settings = Settings::new("com.example.Marco");
        Ok(Self { settings })
    }
    
    /// Function highlighting toggle
    pub fn get_function_highlighting(&self) -> bool {
        self.settings.boolean("function-highlighting")
    }
    
    pub fn set_function_highlighting(&self, enabled: bool) {
        let _ = self.settings.set_boolean("function-highlighting", enabled);
    }
    
    /// Markdown format detection
    pub fn get_markdown_warnings(&self) -> bool {
        self.settings.boolean("markdown-warnings")
    }
    
    pub fn set_markdown_warnings(&self, enabled: bool) {
        let _ = self.settings.set_boolean("markdown-warnings", enabled);
    }
    
    /// Individual markdown warning categories
    pub fn get_markdown_syntax_errors(&self) -> bool {
        self.settings.boolean("markdown-syntax-errors")
    }
    
    pub fn set_markdown_syntax_errors(&self, enabled: bool) {
        let _ = self.settings.set_boolean("markdown-syntax-errors", enabled);
    }
    
    pub fn get_markdown_formatting_issues(&self) -> bool {
        self.settings.boolean("markdown-formatting-issues")
    }
    
    pub fn set_markdown_formatting_issues(&self, enabled: bool) {
        let _ = self.settings.set_boolean("markdown-formatting-issues", enabled);
    }
    
    pub fn get_markdown_style_warnings(&self) -> bool {
        self.settings.boolean("markdown-style-warnings")
    }
    
    pub fn set_markdown_style_warnings(&self, enabled: bool) {
        let _ = self.settings.set_boolean("markdown-style-warnings", enabled);
    }
    
    pub fn get_markdown_structure_issues(&self) -> bool {
        self.settings.boolean("markdown-structure-issues")
    }
    
    pub fn set_markdown_structure_issues(&self, enabled: bool) {
        let _ = self.settings.set_boolean("markdown-structure-issues", enabled);
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
    
    pub fn get_window_position(&self) -> (i32, i32) {
        let x = self.settings.int("window-x");
        let y = self.settings.int("window-y");
        (x, y)
    }
    
    pub fn set_window_position(&self, x: i32, y: i32) {
        let _ = self.settings.set_int("window-x", x);
        let _ = self.settings.set_int("window-y", y);
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
    
    /// Bind a widget property to a settings key
    pub fn bind_property<T>(&self, key: &str, object: &T, property: &str) 
    where 
        T: glib::object::IsA<glib::Object>,
    {
        self.settings.bind(key, object, property).build();
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

/// Global settings instance - using a simpler approach without thread safety
/// This should be accessed only from the main thread
static mut APP_PREFERENCES: Option<AppPreferences> = None;

/// Initialize the global settings instance
pub fn initialize_global_settings() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        APP_PREFERENCES = Some(AppPreferences::new()?);
    }
    Ok(())
}

/// Get a reference to the global settings instance
/// This should only be called from the main thread
pub fn get_app_preferences() -> &'static AppPreferences {
    unsafe {
        APP_PREFERENCES.as_ref().expect("Settings not initialized. Call initialize_global_settings() first.")
    }
}

/// Get a mutable reference to the global settings instance
/// This should only be called from the main thread
pub fn get_app_preferences_mut() -> &'static mut AppPreferences {
    unsafe {
        APP_PREFERENCES.as_mut().expect("Settings not initialized. Call initialize_global_settings() first.")
    }
}

/// Initialize settings system
pub fn initialize_settings() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the global settings instance
    initialize_global_settings()?;
    
    println!("GSettings initialized successfully");
    Ok(())
}
