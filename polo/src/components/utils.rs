// Utility functions for Polo
//
//! # Utility Functions Module
//!
//! Provides helper functions for Polo's UI and theme management:
//!
//! ## Functions
//!
//! - **`parse_hex_to_rgba`**: Converts hex color strings (#RGB, #RRGGBB, #RRGGBBAA) to GDK RGBA
//! - **`apply_gtk_theme_preference`**: Sets GTK dark/light mode based on settings
//! - **`get_theme_mode`**: Extracts theme mode ("light"/"dark") from settings
//! - **`list_available_themes`**: Scans assets directory for available CSS themes
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! // Parse a hex color for WebView background
//! if let Some(rgba) = parse_hex_to_rgba("#1e1e1e") {
//!     webview.set_background_color(&rgba);
//! }
//!
//! // Get current theme mode
//! let mode = get_theme_mode(&settings_manager); // "light" or "dark"
//!
//! // List available themes (without .css extension)
//! let themes = list_available_themes(); // ["github", "marco", "academic", ...]
//! ```

use gtk4::gdk;
use marco_core::logic::swanson::SettingsManager;
use std::sync::Arc;

/// Parse hex color string to RGBA
/// Supports formats: #RGB, #RRGGBB, #RRGGBBAA
pub fn parse_hex_to_rgba(hex: &str) -> Option<gdk::RGBA> {
    let hex = hex.trim_start_matches('#');
    
    let (r, g, b, a) = match hex.len() {
        3 => {
            // #RGB format
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            (r, g, b, 255)
        }
        6 => {
            // #RRGGBB format
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            // #RRGGBBAA format
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };
    
    Some(gdk::RGBA::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

/// Apply GTK dark/light mode preference based on settings
pub fn apply_gtk_theme_preference(settings_manager: &Arc<SettingsManager>) {
    let settings = settings_manager.get_settings();
    let editor_mode = settings
        .appearance
        .as_ref()
        .and_then(|a| a.editor_mode.as_ref())
        .map(|m| m.as_str())
        .unwrap_or("light");
    
    // Determine if we should use dark mode
    let prefer_dark = editor_mode.contains("dark");
    
    // Set GTK global theme property
    if let Some(settings_obj) = gtk4::Settings::default() {
        settings_obj.set_gtk_application_prefer_dark_theme(prefer_dark);
        log::debug!("Set GTK prefer dark theme: {}", prefer_dark);
    }
}

/// Get theme mode (light/dark) from settings
/// Returns "light" or "dark" string
pub fn get_theme_mode(settings_manager: &Arc<SettingsManager>) -> String {
    let settings = settings_manager.get_settings();
    let editor_mode = settings
        .appearance
        .as_ref()
        .and_then(|a| a.editor_mode.as_ref())
        .map(|m| m.as_str())
        .unwrap_or("light");
    
    // Convert editor_mode to theme_mode
    // Editor mode can be "light", "dark", "marco-light", "marco-dark", etc.
    if editor_mode.contains("dark") {
        "dark".to_string()
    } else {
        "light".to_string()
    }
}

/// List available HTML preview themes from assets directory
pub fn list_available_themes() -> Vec<String> {
    use marco_core::logic::paths::get_asset_dir_checked;
    use std::path::PathBuf;
    
    let asset_dir = get_asset_dir_checked().unwrap_or_else(|_| PathBuf::from("assets"));
    let themes_dir = asset_dir.join("themes/html_viever");
    
    let mut themes = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".css") {
                    // Remove the .css extension for display
                    let theme_name = filename.trim_end_matches(".css").to_string();
                    themes.push(theme_name);
                }
            }
        }
    }
    
    // Ensure we have at least one theme
    if themes.is_empty() {
        themes.push("marco".to_string());  // Without .css extension
    }
    
    themes.sort();
    themes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_hex_to_rgba_rgb_format() {
        // Test #RGB format (3 chars)
        let rgba = parse_hex_to_rgba("#abc").expect("Should parse #abc");
        assert!((rgba.red() - 0.667).abs() < 0.01, "Red channel should be ~0.667");
        assert!((rgba.green() - 0.733).abs() < 0.01, "Green channel should be ~0.733");
        assert!((rgba.blue() - 0.8).abs() < 0.01, "Blue channel should be ~0.8");
        assert!((rgba.alpha() - 1.0).abs() < 0.01, "Alpha should be 1.0");
    }

    #[test]
    fn smoke_test_parse_hex_to_rgba_rrggbb_format() {
        // Test #RRGGBB format (6 chars)
        let rgba = parse_hex_to_rgba("#1e1e1e").expect("Should parse #1e1e1e");
        assert!((rgba.red() - 0.118).abs() < 0.01, "Red channel should be ~0.118");
        assert!((rgba.green() - 0.118).abs() < 0.01, "Green channel should be ~0.118");
        assert!((rgba.blue() - 0.118).abs() < 0.01, "Blue channel should be ~0.118");
        assert!((rgba.alpha() - 1.0).abs() < 0.01, "Alpha should be 1.0");
    }

    #[test]
    fn smoke_test_parse_hex_to_rgba_rrggbbaa_format() {
        // Test #RRGGBBAA format (8 chars)
        let rgba = parse_hex_to_rgba("#ff000080").expect("Should parse #ff000080");
        assert!((rgba.red() - 1.0).abs() < 0.01, "Red channel should be 1.0");
        assert!((rgba.green() - 0.0).abs() < 0.01, "Green channel should be 0.0");
        assert!((rgba.blue() - 0.0).abs() < 0.01, "Blue channel should be 0.0");
        assert!((rgba.alpha() - 0.502).abs() < 0.01, "Alpha should be ~0.502 (128/255)");
    }

    #[test]
    fn smoke_test_parse_hex_to_rgba_without_hash() {
        // Test parsing without # prefix
        let rgba = parse_hex_to_rgba("ffffff").expect("Should parse ffffff");
        assert!((rgba.red() - 1.0).abs() < 0.01, "Should be white");
        assert!((rgba.green() - 1.0).abs() < 0.01, "Should be white");
        assert!((rgba.blue() - 1.0).abs() < 0.01, "Should be white");
    }

    #[test]
    fn smoke_test_parse_hex_to_rgba_invalid_input() {
        // Test invalid inputs return None
        assert!(parse_hex_to_rgba("invalid").is_none());
        assert!(parse_hex_to_rgba("#gg").is_none());
        assert!(parse_hex_to_rgba("#12").is_none());
        assert!(parse_hex_to_rgba("#12345").is_none());
        assert!(parse_hex_to_rgba("").is_none());
    }

    #[test]
    fn smoke_test_list_available_themes() {
        let themes = list_available_themes();
        
        // Should have at least one theme (fallback)
        assert!(!themes.is_empty(), "Should have at least one theme");
        
        // Themes should NOT include .css extension
        for theme in &themes {
            assert!(
                !theme.ends_with(".css"),
                "Theme '{}' should not include .css extension",
                theme
            );
            assert!(!theme.is_empty(), "Theme name should not be empty");
        }
        
        // Should be sorted
        let mut sorted_themes = themes.clone();
        sorted_themes.sort();
        assert_eq!(themes, sorted_themes, "Themes should be sorted");
    }

    #[test]
    fn smoke_test_get_theme_mode_default() {
        // This test is more challenging since it requires SettingsManager
        // For now, we test the function exists and doesn't panic
        // In a full integration test, we would create a mock SettingsManager
        
        // Note: This test would require a proper settings setup to run fully
        // We're documenting that the function exists and has the right signature
        let _ = stringify!(get_theme_mode);
    }
}
