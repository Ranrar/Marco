//! Titlebar CSS Generation
//!
//! Generates CSS for Polo's custom titlebar and window styling.
//!
//! ## Components Styled
//!
//! - `.polo-window`: Main window background
//! - `.polo-titlebar`: Custom HeaderBar with 32px height
//! - `.polo-title-label`: Title text styling
//!
//! ## Theme Support
//!
//! Generates rules for both `.marco-theme-light` and `.marco-theme-dark` classes.

use super::constants::*;

/// Generate complete titlebar CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(1024);

    // Force HeaderBar to 32px height - override GTK defaults (theme-independent)
    css.push_str(&format!(
        r#"
    /* Force HeaderBar to 32px height - override GTK defaults */
    .polo-titlebar, headerbar.polo-titlebar {{
        min-height: {height};
        padding-top: 0;
        padding-bottom: 0;
    }}
    
    .polo-window {{
        background: {window_bg};
    }}
"#,
        height = TITLEBAR_HEIGHT,
        window_bg = LIGHT_PALETTE.window_bg,
    ));

    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));

    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));

    css
}

/// Generate theme-specific titlebar CSS
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* {theme} - Titlebar styles */
    .{theme} .polo-window {{
        background: {window_bg};
    }}
    
    .{theme} .polo-titlebar {{
        min-height: {height};
        background: {titlebar_bg};
        border-bottom: 0px solid {border};
    }}
    
    .{theme} .polo-title-label {{
        font-size: {font_size};
        font-weight: {font_weight};
        color: {foreground};
    }}
"#,
        theme = theme_class,
        window_bg = palette.window_bg,
        height = TITLEBAR_HEIGHT,
        titlebar_bg = palette.titlebar_bg,
        border = palette.border,
        font_size = TITLE_FONT_SIZE,
        font_weight = TITLE_FONT_WEIGHT,
        foreground = palette.foreground,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_titlebar_css_generation() {
        let css = generate_css();

        // Verify basic structure
        assert!(css.contains(".polo-titlebar"));
        assert!(css.contains(".polo-window"));
        assert!(css.contains(".polo-title-label"));

        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));

        // Verify essential properties
        assert!(css.contains("min-height: 32px"));
        assert!(css.contains("font-size: 14px"));
        assert!(css.contains("font-weight: 600"));

        // Verify not empty
        assert!(css.len() > 200);
    }

    #[test]
    fn smoke_test_light_dark_have_different_colors() {
        let css = generate_css();

        // Light should use #e8ecef, dark should use #23272e
        assert!(css.contains("#e8ecef")); // Light titlebar bg
        assert!(css.contains("#23272e")); // Dark titlebar bg
        assert!(css.contains("#2c3e50")); // Light foreground
        assert!(css.contains("#f0f5f1")); // Dark foreground
    }

    #[test]
    fn smoke_test_theme_css_structure() {
        let light_css = generate_theme_css("marco-theme-light", &LIGHT_PALETTE);

        // Verify CSS selector structure
        assert!(light_css.contains(".marco-theme-light .polo-window"));
        assert!(light_css.contains(".marco-theme-light .polo-titlebar"));
        assert!(light_css.contains(".marco-theme-light .polo-title-label"));

        // Verify color from palette
        assert!(light_css.contains(LIGHT_PALETTE.titlebar_bg));
        assert!(light_css.contains(LIGHT_PALETTE.foreground));
    }
}
