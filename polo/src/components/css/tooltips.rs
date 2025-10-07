//! Tooltip CSS Generation
//!
//! Generates CSS for GTK tooltips in both light and dark themes.
//!
//! ## Components Styled
//!
//! - `tooltip`: Base tooltip container
//! - `tooltip > contents`: Tooltip content wrapper
//!
//! ## Design
//!
//! Tooltips use inverted colors from the UI theme for contrast:
//! - Light theme: Dark tooltip background with light text
//! - Dark theme: Slightly lighter background with light text

use super::constants::*;

/// Generate complete tooltip CSS for both themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(512);
    
    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    
    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    
    css
}

/// Generate theme-specific tooltip CSS
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* Tooltip - {theme} */
    .{theme} tooltip {{
        background: {bg};
        color: {fg};
        border: 1px solid {border};
    }}
    
    .{theme} tooltip > contents {{
        background: {bg};
        color: {fg};
    }}
"#,
        theme = theme_class,
        bg = palette.tooltip_bg,
        fg = palette.tooltip_fg,
        border = palette.tooltip_border,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_tooltip_css_generation() {
        let css = generate_css();
        
        // Verify tooltip classes present
        assert!(css.contains("tooltip"));
        assert!(css.contains("tooltip > contents"));
        
        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify essential properties
        assert!(css.contains("background:"));
        assert!(css.contains("color:"));
        assert!(css.contains("border:"));
        
        // Verify not empty
        assert!(css.len() > 100);
    }

    #[test]
    fn smoke_test_theme_colors_differ() {
        let css = generate_css();
        
        // Light should use #2c3e50, dark should use #3d3d3d for tooltip bg
        assert!(css.contains("#2c3e50"));  // Light tooltip bg
        assert!(css.contains("#3d3d3d"));  // Dark tooltip bg
        
        // Both use light text
        assert!(css.contains("#ffffff"));  // Light tooltip text
        assert!(css.contains("#e0e0e0"));  // Dark tooltip text
    }

    #[test]
    fn smoke_test_theme_css_structure() {
        let light_css = generate_theme_css("marco-theme-light", &LIGHT_PALETTE);
        
        // Verify selector structure
        assert!(light_css.contains(".marco-theme-light tooltip"));
        assert!(light_css.contains(".marco-theme-light tooltip > contents"));
        
        // Verify uses palette colors
        assert!(light_css.contains(LIGHT_PALETTE.tooltip_bg));
        assert!(light_css.contains(LIGHT_PALETTE.tooltip_fg));
        assert!(light_css.contains(LIGHT_PALETTE.tooltip_border));
    }
}
