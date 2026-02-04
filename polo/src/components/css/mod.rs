// CSS loading and management module
//
//! # CSS Module
//!
//! Manages CSS styling for Polo's GTK UI and markdown preview themes.
//!
//! ## Submodules
//!
//! - **`constants`**: Centralized color palettes, spacing, and sizing constants
//! - **`titlebar`**: Titlebar and window background styling
//! - **`buttons`**: All button types (open-file, mode-toggle, open-editor)
//! - **`dropdown`**: Theme selection dropdown and popover styling
//! - **`dialog`**: Custom dialog windows matching app theme
//! - **`scrollbar`**: GTK scrollbar styling matching WebKit preview
//! - **`tooltips`**: GTK tooltip styling for both themes
//! - **`theme`**: Theme CSS loading and syntax highlighting generation
//!
//! ## CSS Architecture
//!
//! Polo's styling combines two sources:
//!
//! 1. **Marco's menu.css**: Shared UI styling for consistency
//!    - Loaded from `assets/themes/ui_elements/menu.css`
//!    - Provides base styles for titlebars, buttons, icons
//!
//! 2. **Polo-specific CSS**: Generated programmatically from modular components
//!    - `constants` module provides theme-aware color palettes
//!    - Each UI component module generates its own CSS rules
//!    - `generate_polo_css()` combines all component CSS
//!
//! ## Theme Classes
//!
//! The window has a dynamic theme class that controls UI appearance:
//! - `.marco-theme-light`: Light mode styling
//! - `.marco-theme-dark`: Dark mode styling
//!
//! CSS rules target these classes for theme-specific colors and borders.
//!
//! ## Usage
//!
//! Call `load_css()` early in application startup (before window creation):
//!
//! ```rust,ignore
//! load_css();  // Loads and applies all CSS styling
//! ```

pub mod buttons;
pub mod constants;
pub mod dialog;
pub mod dropdown;
pub mod scrollbar;
pub mod theme;
pub mod titlebar;
pub mod tooltips;

use gtk4::{gdk::Display, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

/// Minimal fallback CSS if menu.css cannot be loaded
/// Note: This should rarely be used - menu.css is the canonical source
const FALLBACK_MENU_CSS: &str = r#"
    /* Critical styles for basic functionality */
    .window-control-btn {
        background: transparent;
        border: none;
    }
    .titlebar {
        min-height: 32px;
    }
"#;

/// Generate all Polo-specific CSS from modular components
pub fn generate_polo_css() -> String {
    let mut css = String::with_capacity(8192);

    // Titlebar and window styling
    css.push_str(&titlebar::generate_css());

    // All button types
    css.push_str(&buttons::generate_css());

    // Theme dropdown
    css.push_str(&dropdown::generate_css());

    // Dialog windows
    css.push_str(&dialog::generate_css());

    // Scrollbar styling
    css.push_str(&scrollbar::generate_css());

    // Tooltips
    css.push_str(&tooltips::generate_css());

    css
}

/// Load CSS for Polo styling with asset root
/// This loads Marco's menu.css for consistent UI styling and adds Polo-specific styles
///
/// # Arguments
/// * `asset_root` - The asset root directory path
pub fn load_css_from_path(asset_root: &std::path::Path) {
    let css_provider = CssProvider::new();

    // Load Marco's menu.css for consistent UI styling
    let menu_css_path = asset_root.join("themes/ui_elements/menu.css");

    let menu_css = if let Ok(css) = std::fs::read_to_string(&menu_css_path) {
        log::debug!("Loaded menu.css from: {}", menu_css_path.display());
        css
    } else {
        log::warn!("Could not load menu.css, using fallback");
        String::from(FALLBACK_MENU_CSS)
    };

    // Generate Polo-specific styles from modular components
    let polo_css = generate_polo_css();

    // Combine Marco's menu.css with Polo-specific styles.
    let combined_css = format!("{}\n\n/* Polo-specific styles */\n{}", menu_css, polo_css);
    css_provider.load_from_data(&combined_css);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_generate_polo_css() {
        let css = generate_polo_css();

        // Verify all components present
        assert!(css.contains(".polo-titlebar"));
        assert!(css.contains(".polo-open-file-btn"));
        assert!(css.contains(".polo-mode-toggle-btn"));
        assert!(css.contains(".polo-theme-dropdown"));
        assert!(css.contains("tooltip"));

        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));

        // Verify substantial output
        assert!(css.len() > 5000);
    }

    #[test]
    fn smoke_test_load_css_doesnt_panic() {
        // Just verify load_css() can be called without panicking
        // Note: In headless test environment, GTK display may not be available
        // so we can't fully test the loading, but we can test generation
        let css = generate_polo_css();
        assert!(!css.is_empty());
    }
}
