// ...existing code...
use crate::utils::cache::Cache;
use crate::ui::ui_theme::UiThemeProvider;
use crate::ui::css_theme::CssTheme;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    /// Convert theme to string representation for settings/serialization
    ///
    /// Used for converting theme enum values to strings for storage in
    /// configuration files and settings persistence.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    /// Convert string to Theme enum for loading from settings
    ///
    /// Used for parsing theme strings from configuration files back
    /// into Theme enum values. Defaults to System theme for unknown values.
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Theme {
        match s {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            _ => Theme::System,
        }
    }
}

pub struct ThemeManager {
    current_theme: Rc<RefCell<Theme>>,
    callbacks: Rc<RefCell<Vec<Box<dyn Fn(Theme)>>>>,
    current_css_theme: Rc<RefCell<String>>,
    ui_theme_provider: UiThemeProvider,
    css_cache: Rc<Cache<String, String>>,
}

impl ThemeManager {
    /// Get the current CssTheme for the HTML/markdown view
    pub fn get_current_css_theme_for_view(&self) -> Result<CssTheme, String> {
        let theme_name = crate::ui::css_theme::CssTheme::get_current_css_theme();
        CssTheme::load(&theme_name)
    }
    pub fn new() -> Self {
        let detected_theme = Self::detect_system_theme();
        Self {
            current_theme: Rc::new(RefCell::new(detected_theme)),
            callbacks: Rc::new(RefCell::new(Vec::new())),
            current_css_theme: Rc::new(RefCell::new("standard".to_string())),
            ui_theme_provider: UiThemeProvider::new(),
            css_cache: Rc::new(Cache::new()),
        }
    }

    /// Detect the system theme preference
    pub fn detect_system_theme() -> Theme {
        // Try to detect system theme using GTK settings
        if let Some(settings) = gtk4::Settings::default() {
            // Use property instead of boolean method
            let prefer_dark: bool = settings.property("gtk-application-prefer-dark-theme");
            if prefer_dark {
                return Theme::Dark;
            }
        }

        // Fallback: try environment variables (works on Linux)
        if let Ok(theme) = std::env::var("GTK_THEME") {
            if theme.to_lowercase().contains("dark") {
                return Theme::Dark;
            }
        }

        // Check for GNOME dark theme preference
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            if let Ok(theme_name) = String::from_utf8(output.stdout) {
                if theme_name.to_lowercase().contains("dark") {
                    return Theme::Dark;
                }
            }
        }

        // Default to light theme
        Theme::Light
    }

    pub fn get_current_theme(&self) -> Theme {
        *self.current_theme.borrow()
    }

    pub fn set_theme(&self, theme: Theme) {
        let actual_theme = match theme {
            Theme::System => Self::detect_system_theme(),
            other => other,
        };

        *self.current_theme.borrow_mut() = theme;

        // Reload GTK CSS for the editor with the new theme
        let css_theme_name = crate::ui::css_theme::CssTheme::get_current_css_theme();
        if let Ok(css_content) = crate::ui::css_theme::CssTheme::load(&css_theme_name) {
            self.ui_theme_provider.reload_gtk_css(&css_content.css_content, actual_theme);
        } else {
            eprintln!("WARNING: Could not load CSS theme '{}' for GTK reload", css_theme_name);
        }

        // Notify all callbacks
        for callback in self.callbacks.borrow().iter() {
            callback(actual_theme);
        }
    }

    pub fn get_effective_theme(&self) -> Theme {
        match self.get_current_theme() {
            Theme::System => Self::detect_system_theme(),
            other => other,
        }
    }

    /// Register a callback for theme change notifications
    ///
    /// Allows components to register functions that will be called whenever
    /// the theme changes. Useful for updating UI elements that need to
    /// respond to theme switches.
    ///
    /// # Arguments
    /// * `callback` - Function to call when theme changes, receives the new effective theme
    #[allow(dead_code)]
    pub fn add_theme_change_callback<F>(&self, callback: F)
    where
        F: Fn(Theme) + 'static,
    {
        self.callbacks.borrow_mut().push(Box::new(callback));
    }

    /// Get CSS class name for the current effective theme
    ///
    /// Returns appropriate CSS class names that can be applied to UI elements
    /// for consistent theming. Useful for components that need to apply
    /// theme-specific styling beyond the automatic theme detection.
    ///
    /// # Returns
    /// * "dark-theme" for dark mode
    /// * "light-theme" for light mode or system fallback
    #[allow(dead_code)]
    pub fn get_css_class(&self) -> &'static str {
        match self.get_effective_theme() {
            Theme::Dark => "dark-theme",
            Theme::Light => "light-theme",
            Theme::System => "light-theme", // fallback
        }
    }




    /// Generate GTK-compatible CSS from theme variables
    // GTK CSS generation now handled by UiTheme


    /// Initialize the theme manager with default CSS theme
    /// This should be called at startup to ensure the CSS is loaded
    pub fn initialize(&self) -> Result<(), String> {
        // Initialize the GTK CSS provider
        self.ui_theme_provider.initialize_gtk_css_provider()?;
        
        // Load the default CSS theme
        crate::ui::css_theme::CssTheme::set_css_theme("standard")?;
        Ok(())
    }

    /// Get the syntax theme name based on current theme
    pub fn get_syntax_theme_name(&self) -> String {
        match self.get_effective_theme() {
            Theme::Dark => "dark".to_string(),
            Theme::Light => "light".to_string(),
            Theme::System => {
                match Self::detect_system_theme() {
                    Theme::Dark => "dark".to_string(),
                    Theme::Light => "light".to_string(),
                    Theme::System => "light".to_string(), // fallback
                }
            }
        }
    }


    /// Create a weak reference to this theme manager
    pub fn downgrade(&self) -> std::rc::Weak<RefCell<ThemeManager>> {
        // This is a placeholder - the actual implementation would depend on how the theme manager is stored
        // For now, we'll use a different approach in the preferences module
        std::rc::Weak::new()
    }
}

impl Clone for ThemeManager {
    fn clone(&self) -> Self {
        Self {
            current_theme: self.current_theme.clone(),
            callbacks: self.callbacks.clone(), // Share the same callbacks vector
            current_css_theme: self.current_css_theme.clone(),
            ui_theme_provider: UiThemeProvider::new(),
            css_cache: self.css_cache.clone(),
        }
    }
}
