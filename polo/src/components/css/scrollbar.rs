//! Scrollbar CSS Generation
//!
//! Generates CSS for GTK scrollbars matching Marco's editor theme.
//!
//! ## Components Styled
//!
//! - `scrollbar` - Main scrollbar widget (12px width/height)
//! - `scrollbar trough` - Scrollbar track/background
//! - `scrollbar slider` - Scrollbar thumb/handle
//!
//! ## Theme Support
//!
//! Generates rules for both `.marco-theme-light` and `.marco-theme-dark` classes.
//!
//! ## Scrollbar Colors
//!
//! Colors are extracted from Marco's editor theme XML files:
//! - **Light**: thumb `#D0D4D8`, track `#F0F0F0`
//! - **Dark**: thumb `#3A3F44`, track `#252526`
//!
//! These colors match the WebKit scrollbar styling used in the HTML preview.

/// Light theme scrollbar colors (from assets/themes/editor/light.xml)
const LIGHT_SCROLLBAR_THUMB: &str = "#D0D4D8";
const LIGHT_SCROLLBAR_TRACK: &str = "#F0F0F0";

/// Dark theme scrollbar colors (from assets/themes/editor/dark.xml)
const DARK_SCROLLBAR_THUMB: &str = "#3A3F44";
const DARK_SCROLLBAR_TRACK: &str = "#252526";

/// Generate complete scrollbar CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(2048);

    // Base scrollbar styling (theme-independent)
    css.push_str(
        r#"
    /* Base GTK scrollbar styling - 12px width matching WebKit */
    scrollbar {
        -gtk-icon-transform: none;
        min-width: 12px;
        min-height: 12px;
        background: transparent;
        border: none;
        box-shadow: none;
        padding: 0;
        margin: 0;
    }
    
    scrollbar trough {
        border: none;
        box-shadow: none;
        min-width: 12px;
        min-height: 12px;
        padding: 0;
        margin: 0;
    }
    
    scrollbar slider {
        border-radius: 0px;
        border: none;
        box-shadow: none;
        min-width: 12px;
        min-height: 12px;
        margin: 0;
        padding: 0;
    }
    
    scrollbar slider:hover {
        opacity: 0.9;
    }
"#,
    );

    // Light theme
    css.push_str(&generate_theme_css(
        "marco-theme-light",
        LIGHT_SCROLLBAR_THUMB,
        LIGHT_SCROLLBAR_TRACK,
    ));

    // Dark theme
    css.push_str(&generate_theme_css(
        "marco-theme-dark",
        DARK_SCROLLBAR_THUMB,
        DARK_SCROLLBAR_TRACK,
    ));

    css
}

/// Generate theme-specific scrollbar CSS
fn generate_theme_css(theme_class: &str, thumb: &str, track: &str) -> String {
    format!(
        r#"
    /* {theme} - Scrollbar colors matching editor theme */
    .{theme} scrollbar trough {{
        background-color: {track};
    }}
    
    .{theme} scrollbar slider {{
        background-color: {thumb};
    }}
    
    .{theme} scrollbar slider:hover {{
        background-color: {thumb};
    }}
"#,
        theme = theme_class,
        thumb = thumb,
        track = track,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_scrollbar_css_generation() {
        let css = generate_css();

        // Verify basic structure
        assert!(css.contains("scrollbar"));
        assert!(css.contains("scrollbar trough"));
        assert!(css.contains("scrollbar slider"));

        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));

        // Verify light theme colors
        assert!(css.contains(LIGHT_SCROLLBAR_THUMB));
        assert!(css.contains(LIGHT_SCROLLBAR_TRACK));

        // Verify dark theme colors
        assert!(css.contains(DARK_SCROLLBAR_THUMB));
        assert!(css.contains(DARK_SCROLLBAR_TRACK));

        // Verify essential properties
        assert!(css.contains("min-width: 12px"));
        assert!(css.contains("min-height: 12px"));
        assert!(css.contains("border-radius: 0px"));

        // Verify not empty
        assert!(!css.is_empty());
        assert!(css.len() > 500);
    }

    #[test]
    fn smoke_test_scrollbar_colors_match_webkit() {
        let css = generate_css();

        // Verify light theme colors match editor XML
        assert!(css.contains("#D0D4D8")); // light thumb
        assert!(css.contains("#F0F0F0")); // light track

        // Verify dark theme colors match editor XML
        assert!(css.contains("#3A3F44")); // dark thumb
        assert!(css.contains("#252526")); // dark track
    }

    #[test]
    fn smoke_test_scrollbar_12px_width() {
        let css = generate_css();

        // Verify 12px width/height matching WebKit
        assert!(css.matches("12px").count() >= 6);
    }
}
