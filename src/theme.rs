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
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark", 
            Theme::System => "system",
        }
    }
    
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
}

impl ThemeManager {
    pub fn new() -> Self {
        let detected_theme = Self::detect_system_theme();
        Self {
            current_theme: Rc::new(RefCell::new(detected_theme)),
            callbacks: Rc::new(RefCell::new(Vec::new())),
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
    
    pub fn add_theme_change_callback<F>(&self, callback: F)
    where
        F: Fn(Theme) + 'static,
    {
        self.callbacks.borrow_mut().push(Box::new(callback));
    }
    
    /// Get CSS class name for the current theme
    pub fn get_css_class(&self) -> &'static str {
        match self.get_effective_theme() {
            Theme::Dark => "dark-theme",
            Theme::Light => "light-theme",
            Theme::System => "light-theme", // fallback
        }
    }
}

impl Clone for ThemeManager {
    fn clone(&self) -> Self {
        Self {
            current_theme: self.current_theme.clone(),
            callbacks: Rc::new(RefCell::new(Vec::new())), // New callbacks vector for clone
        }
    }
}
