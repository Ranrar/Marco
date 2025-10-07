//! Button CSS Generation
//!
//! Generates CSS for all Polo button types with consistent styling.
//!
//! ## Button Types
//!
//! - **Open File Button** (`.polo-open-file-btn`): Primary file selection button
//! - **Open Editor Button** (`.polo-open-editor-btn`): Launch Marco editor button
//! - **Mode Toggle Button** (`.polo-mode-toggle-btn`): Light/dark theme switcher with emoji
//!
//! ## Design Philosophy
//!
//! All buttons follow Marco's flat, minimal design:
//! - Transparent background with borders
//! - Smooth transitions on hover/active states
//! - Consistent padding and border radius
//! - Theme-aware colors

use super::constants::*;

/// Generate complete button CSS for all button types and both themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(4096);
    
    // Open File button
    css.push_str(&generate_open_file_button_css());
    
    // Open Editor button
    css.push_str(&generate_open_editor_button_css());
    
    // Mode Toggle button (with emoji filters)
    css.push_str(&generate_mode_toggle_button_css());
    
    css
}

/// Generate Open File button CSS for both themes
pub fn generate_open_file_button_css() -> String {
    let mut css = String::with_capacity(1024);
    
    css.push_str(&generate_standard_button_css(
        "marco-theme-light",
        "polo-open-file-btn",
        &LIGHT_PALETTE,
    ));
    
    css.push_str(&generate_standard_button_css(
        "marco-theme-dark",
        "polo-open-file-btn",
        &DARK_PALETTE,
    ));
    
    css
}

/// Generate Open Editor button CSS for both themes
pub fn generate_open_editor_button_css() -> String {
    let mut css = String::with_capacity(1024);
    
    css.push_str(&generate_standard_button_css(
        "marco-theme-light",
        "polo-open-editor-btn",
        &LIGHT_PALETTE,
    ));
    
    css.push_str(&generate_standard_button_css(
        "marco-theme-dark",
        "polo-open-editor-btn",
        &DARK_PALETTE,
    ));
    
    css
}

/// Generate Mode Toggle button CSS with emoji filters
pub fn generate_mode_toggle_button_css() -> String {
    let mut css = String::with_capacity(2048);
    
    // Light theme (darken emoji)
    css.push_str(&format!(
        r#"
    /* Dark mode toggle button - LIGHT MODE */
    .marco-theme-light .polo-mode-toggle-btn {{
        min-width: {min_width};
        min-height: {min_height};
        padding: {padding};
        border: 1px solid {border};
        border-radius: {radius};
        background: transparent;
        color: {foreground};
        font-size: {font_size};
        transition: {transition};
    }}
    
    .marco-theme-light .polo-mode-toggle-btn:hover {{
        background: transparent;
        color: {hover_accent};
        border-color: {border_hover};
    }}
    
    .marco-theme-light .polo-mode-toggle-btn:active {{
        background: transparent;
        color: {active_text};
        border-color: {border_hover};
    }}
    
    /* Make emoji dark for light mode */
    .marco-theme-light .polo-mode-toggle-btn label {{
        filter: grayscale(100%) brightness(0.3);
    }}
    
    .marco-theme-light .polo-mode-toggle-btn:hover label {{
        filter: grayscale(100%) brightness(0.2);
    }}
    
    .marco-theme-light .polo-mode-toggle-btn:active label {{
        filter: grayscale(100%) brightness(0);
    }}
"#,
        min_width = BUTTON_MIN_WIDTH,
        min_height = BUTTON_MIN_HEIGHT,
        padding = MODE_TOGGLE_PADDING,
        border = LIGHT_PALETTE.border,
        radius = BORDER_RADIUS,
        foreground = LIGHT_PALETTE.foreground,
        font_size = TITLE_FONT_SIZE,
        transition = STANDARD_TRANSITION,
        hover_accent = LIGHT_PALETTE.hover_accent,
        border_hover = LIGHT_PALETTE.border_hover,
        active_text = LIGHT_PALETTE.active_text,
    ));
    
    // Dark theme (brighten emoji)
    css.push_str(&format!(
        r#"
    /* Dark mode toggle button - DARK MODE */
    .marco-theme-dark .polo-mode-toggle-btn {{
        min-width: {min_width};
        min-height: {min_height};
        padding: {padding};
        border: 1px solid {border};
        border-radius: {radius};
        background: transparent;
        color: {foreground};
        font-size: {font_size};
        transition: {transition};
    }}
    
    .marco-theme-dark .polo-mode-toggle-btn:hover {{
        background: transparent;
        color: {hover_accent};
        border-color: {border_hover};
    }}
    
    .marco-theme-dark .polo-mode-toggle-btn:active {{
        background: transparent;
        color: {active_text};
        border-color: {border_hover};
    }}
    
    /* Make emoji bright for dark mode */
    .marco-theme-dark .polo-mode-toggle-btn label {{
        filter: grayscale(100%) brightness(2);
    }}
    
    .marco-theme-dark .polo-mode-toggle-btn:hover label {{
        filter: grayscale(100%) brightness(2.5);
    }}
    
    .marco-theme-dark .polo-mode-toggle-btn:active label {{
        filter: grayscale(100%) brightness(3);
    }}
"#,
        min_width = BUTTON_MIN_WIDTH,
        min_height = BUTTON_MIN_HEIGHT,
        padding = MODE_TOGGLE_PADDING,
        border = DARK_PALETTE.border,
        radius = BORDER_RADIUS,
        foreground = DARK_PALETTE.foreground,
        font_size = TITLE_FONT_SIZE,
        transition = STANDARD_TRANSITION,
        hover_accent = DARK_PALETTE.hover_accent,
        border_hover = DARK_PALETTE.border_hover,
        active_text = DARK_PALETTE.active_text,
    ));
    
    css
}

/// Generate standard button CSS (used by open-file and open-editor buttons)
fn generate_standard_button_css(theme_class: &str, button_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* {button} button - {theme} */
    .{theme} .{button} {{
        background: transparent;
        color: {foreground};
        border: 1px solid {border};
        border-radius: {radius};
        padding: {padding};
        min-height: {min_height};
        font-weight: {font_weight};
        font-size: {font_size};
        transition: {transition};
    }}
    
    .{theme} .{button}:hover {{
        background: transparent;
        color: {hover_accent};
        border-color: {border_hover};
    }}
    
    .{theme} .{button}:active {{
        background: transparent;
        color: {active_text};
        border-color: {border_hover};
    }}
"#,
        button = button_class,
        theme = theme_class,
        foreground = palette.foreground,
        border = palette.border,
        radius = BORDER_RADIUS,
        padding = BUTTON_PADDING,
        min_height = BUTTON_MIN_HEIGHT,
        font_weight = BUTTON_FONT_WEIGHT,
        font_size = BUTTON_FONT_SIZE,
        transition = STANDARD_TRANSITION,
        hover_accent = palette.hover_accent,
        border_hover = palette.border_hover,
        active_text = palette.active_text,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_all_buttons_css() {
        let css = generate_css();
        
        // Verify all button types present
        assert!(css.contains(".polo-open-file-btn"));
        assert!(css.contains(".polo-open-editor-btn"));
        assert!(css.contains(".polo-mode-toggle-btn"));
        
        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify essential properties
        assert!(css.contains("border-radius: 6px"));
        assert!(css.contains("transition:"));
        assert!(css.contains(":hover"));
        assert!(css.contains(":active"));
        
        // Verify substantial output
        assert!(css.len() > 1000);
    }

    #[test]
    fn smoke_test_open_file_button() {
        let css = generate_open_file_button_css();
        
        // Verify structure
        assert!(css.contains(".polo-open-file-btn"));
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify colors differ between themes
        assert!(css.contains("#2c3e50"));  // Light foreground
        assert!(css.contains("#f0f5f1"));  // Dark foreground
    }

    #[test]
    fn smoke_test_mode_toggle_has_emoji_filters() {
        let css = generate_mode_toggle_button_css();
        
        // Verify emoji filter rules present
        assert!(css.contains("filter: grayscale(100%)"));
        assert!(css.contains("brightness(0.3)"));  // Light mode darken
        assert!(css.contains("brightness(2)"));    // Dark mode brighten
        
        // Verify hover and active filters
        assert!(css.contains(":hover label"));
        assert!(css.contains(":active label"));
    }

    #[test]
    fn smoke_test_standard_button_structure() {
        let css = generate_standard_button_css("marco-theme-light", "test-btn", &LIGHT_PALETTE);
        
        // Verify selector structure
        assert!(css.contains(".marco-theme-light .test-btn"));
        assert!(css.contains(".marco-theme-light .test-btn:hover"));
        assert!(css.contains(".marco-theme-light .test-btn:active"));
        
        // Verify uses palette colors
        assert!(css.contains(LIGHT_PALETTE.foreground));
        assert!(css.contains(LIGHT_PALETTE.border));
        assert!(css.contains(LIGHT_PALETTE.border_hover));
    }
}
