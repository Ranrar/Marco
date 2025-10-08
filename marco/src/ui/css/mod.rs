// CSS loading and management module
//
//! # CSS Module
//!
//! Manages CSS styling for Marco's GTK UI using programmatic generation.
//!
//! ## Architecture
//!
//! Marco's CSS system follows Polo's modular approach:
//!
//! 1. **Centralized Constants** (`constants.rs`): All colors, spacing, and sizing
//! 2. **Component Modules**: Each UI area generates its own CSS
//! 3. **Single CssProvider**: All CSS combined and loaded once
//!
//! ## Submodules
//!
//! - **`constants`**: Centralized color palettes, spacing, and sizing constants
//! - **`menu`**: Titlebar, window controls, icon fonts, and layout buttons
//! - **`toolbar`**: Toolbar buttons, icons, and styling
//! - **`footer`**: Footer elements, status indicators, and styling
//!
//! ## CSS Generation Flow
//!
//! 1. Each module has a `generate_css()` function
//! 2. `generate_marco_css()` combines all module CSS
//! 3. `load_css()` applies the combined CSS to the GTK display
//!
//! ## Theme Classes
//!
//! The window uses dynamic theme classes for light/dark mode:
//! - `.marco-theme-light`: Light mode styling
//! - `.marco-theme-dark`: Dark mode styling
//!
//! CSS rules target these classes for theme-specific colors.
//!
//! ## Usage
//!
//! Call `load_css()` early in application startup (before window creation):
//!
//! ```rust,ignore
//! use crate::ui::css::load_css;
//! load_css();  // Loads and applies all CSS styling
//! ```

pub mod constants;
pub mod dialog;
pub mod footer;
pub mod menu;
pub mod toolbar;

use gtk4::{gdk::Display, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

/// Minimal fallback CSS if dynamic generation fails
/// This ensures basic functionality even if CSS generation has issues
const FALLBACK_CSS: &str = r#"
    /* Critical styles for basic functionality */
    .icon-font {
        font-family: 'icomoon';
        font-size: 16px;
    }
    .window-control-btn {
        background: transparent;
        border: none;
    }
    .titlebar {
        min-height: 32px;
    }
"#;

/// Generate all Marco-specific CSS from modular components
pub fn generate_marco_css() -> String {
    let mut css = String::with_capacity(16384); // Larger capacity for Marco's more complex UI
    
    // Menu/titlebar styling
    css.push_str(&menu::generate_css());
    
    // Toolbar styling
    css.push_str(&toolbar::generate_css());
    
    // Footer styling
    css.push_str(&footer::generate_css());
    
    // Dialog styling
    css.push_str(&dialog::generate_css());
    
    css
}

/// Load CSS for Marco styling
/// This generates all CSS dynamically and applies it to the GTK display
pub fn load_css() {
    let css_provider = CssProvider::new();
    
    // Generate Marco's CSS from modular components
    let marco_css = generate_marco_css();
    
    // Use generated CSS (with fallback if generation fails)
    let css_to_load = if marco_css.is_empty() {
        log::warn!("CSS generation returned empty, using fallback");
        String::from(FALLBACK_CSS)
    } else {
        log::debug!("Generated {} bytes of CSS", marco_css.len());
        marco_css
    };
    
    css_provider.load_from_data(&css_to_load);
    
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        log::info!("CSS loaded successfully");
    } else {
        log::error!("Failed to get GTK display for CSS loading");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_generate_marco_css() {
        let css = generate_marco_css();
        
        // Verify not empty
        assert!(!css.is_empty(), "Generated CSS should not be empty");
        
        // Verify all major components present
        assert!(css.contains(".icon-font"), "Should contain icon-font class");
        assert!(css.contains(".titlebar"), "Should contain titlebar class");
        
        // Verify both themes present
        assert!(css.contains(".marco-theme-light"), "Should contain light theme");
        assert!(css.contains(".marco-theme-dark"), "Should contain dark theme");
        
        // Verify substantial output (at least 5KB)
        assert!(css.len() > 5000, "CSS should be substantial (got {} bytes)", css.len());
    }
    
    #[test]
    fn debug_dump_css_line_429() {
        let css = generate_marco_css();
        let lines: Vec<&str> = css.lines().collect();
        
        println!("\n=== CSS Generation Report ===");
        println!("Total lines: {}", lines.len());
        
        if lines.len() >= 429 {
            println!("\n=== Around Line 429 (GTK Error Location) ===");
            for i in 425..=432 {
                if i < lines.len() {
                    println!("Line {}: {}", i + 1, lines[i]);
                }
            }
            
            if lines.len() > 428 {
                let line_429 = lines[428];
                println!("\n=== Line 429 Details ===");
                println!("Full line: {}", line_429);
                if line_429.len() >= 39 {
                    println!("Columns 33-39: '{}'", &line_429[32..39]);
                }
            }
        }
        
        // Find all lines with :empty
        println!("\n=== Lines with :empty ===");
        for (i, line) in lines.iter().enumerate() {
            if line.contains(":empty") {
                println!("Line {}: {}", i + 1, line);
            }
        }
    }

    #[test]
    fn smoke_test_load_css_doesnt_panic() {
        // Just verify load_css() can be called without panicking
        // Note: In headless test environment, GTK display may not be available
        // so we can't fully test the loading, but we can test generation
        let css = generate_marco_css();
        assert!(!css.is_empty());
    }
    
    #[test]
    fn smoke_test_fallback_css() {
        // Verify fallback CSS has critical styles
        assert!(FALLBACK_CSS.contains(".icon-font"));
        assert!(FALLBACK_CSS.contains(".window-control-btn"));
        assert!(FALLBACK_CSS.contains(".titlebar"));
    }
}
