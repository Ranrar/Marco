// CSS loading and management module
//
//! # CSS Module
//!
//! Manages CSS styling for Polo's GTK UI and markdown preview themes.
//!
//! ## Submodules
//!
//! - **`polo_styles`**: Polo-specific CSS constants (titlebar, buttons, dropdowns)
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
//! 2. **Polo-specific CSS**: Overrides and additions
//!    - Defined in `polo_styles::POLO_CSS`
//!    - Customizes titlebar height, button styles, dropdown appearance
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

pub mod polo_styles;
pub mod theme;

use gtk4::{gdk::Display, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use marco_core::logic::paths::get_asset_dir_checked;
use std::path::PathBuf;

pub use polo_styles::{FALLBACK_MENU_CSS, POLO_CSS};
// Note: theme functions (generate_syntax_highlighting_css, load_theme_css) are accessed
// directly via theme:: by components that need them, so no re-export needed

/// Load CSS for Polo styling
/// This loads Marco's menu.css for consistent UI styling and adds Polo-specific styles
pub fn load_css() {
    let css_provider = CssProvider::new();
    
    // Load Marco's menu.css for consistent UI styling
    let asset_dir = get_asset_dir_checked().unwrap_or_else(|_| PathBuf::from("assets"));
    let menu_css_path = asset_dir.join("themes/ui_elements/menu.css");
    
    let menu_css = if let Ok(css) = std::fs::read_to_string(&menu_css_path) {
        log::debug!("Loaded menu.css from: {}", menu_css_path.display());
        css
    } else {
        log::warn!("Could not load menu.css, using fallback");
        String::from(FALLBACK_MENU_CSS)
    };
    
    // Combine Marco's menu.css with Polo-specific styles
    let combined_css = format!("{}\n\n/* Polo-specific styles */\n{}", menu_css, POLO_CSS);
    css_provider.load_from_data(&combined_css);
    
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
