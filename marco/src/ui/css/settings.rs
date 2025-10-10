//! Settings CSS Generation
//!
//! Generates CSS for Marco's settings dialog window and tab layouts.
//!
//! ## Components Styled
//!
//! - `.marco-settings-window`: Main settings window
//! - `.marco-settings-content`: Content container
//! - `.marco-settings-notebook`: Notebook widget with tabs
//! - `.marco-settings-tab`: Individual tab container
//! - `.marco-settings-section`: Logical section grouping
//! - `.marco-settings-row`: Individual setting row (title + control)
//! - `.marco-settings-header`: Section header text (bold)
//! - `.marco-settings-description`: Description text (dimmed)
//!
//! ## Theme Support
//!
//! Generates rules for both `.marco-theme-light` and `.marco-theme-dark` classes.

use super::constants::*;

/// Generate complete settings CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(4096);
    
    // Base settings styling (theme-independent)
    css.push_str(&generate_base_settings_css());
    
    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    
    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    
    css
}

/// Generate base settings CSS (theme-independent layout and sizing)
fn generate_base_settings_css() -> String {
    format!(
        r#"
    /* Settings window base styles */
    .marco-settings-window {{
        min-width: 600px;
        min-height: 500px;
    }}
    
    .marco-settings-content {{
        padding: 0;
    }}
    
    /* Notebook (tabs) styling */
    .marco-settings-notebook {{
        padding: 0;
        border: none;
        background: transparent;
    }}
    
    .marco-settings-notebook > header {{
        padding: 1px 10px 0px 10px;
        background: transparent;
    }}
    
    .marco-settings-notebook > header > tabs {{
        min-height: 16px;
        background: transparent;
        border: none;
        box-shadow: none;
    }}
    
    .marco-settings-notebook > header > tabs > tab {{
        min-width: 0px;
        min-height: 12px;
        padding: 5px 10px;
        border-radius: {tab_radius} {tab_radius} 0 0;
        font-size: 12px;
        font-weight: 500;
        transition: {transition};
        background: transparent;
        border: none;
        box-shadow: none;
    }}
    
    .marco-settings-notebook > header > tabs > tab label {{
        font-weight: 500;
    }}
    
    /* Tab content area */
    .marco-settings-tab {{
        padding: 12px 18px;
    }}
    
    /* Setting sections */
    .marco-settings-section {{
        margin-bottom: 10px;
    }}
    
    /* Setting rows (title + control) */
    .marco-settings-row {{
        min-height: 36px;
        padding: 3px 0;
        margin-bottom: 3px;
    }}
    
    /* Setting row frame (table-like borders) */
    .marco-settings-row-frame {{
        min-height: 56px;
        border-width: 1px;
        border-style: solid;
        border-radius: 4px;
        padding: 0;
        margin: 0;
        background: transparent;
    }}
    
    .marco-settings-row-frame > * {{
        background: transparent;
        border: none;
    }}
    
    /* Setting headers (bold titles) */
    .marco-settings-header {{
        font-size: 13px;
        font-weight: 600;
        margin: 0;
        padding: 0;
    }}
    
    /* Setting descriptions (dimmed text) */
    .marco-settings-description {{
        font-size: 12px;
        line-height: 1.3;
        margin-top: 2px;
        margin-bottom: 6px;
        opacity: 0.8;
    }}
    
    /* Close button frame (matches setting row frames) */
    .marco-settings-close-frame {{
        min-height: 56px;
        border-width: 0;
        border-top-width: 1px;
        border-style: solid;
        border-radius: 0px;
        padding: 0;
        margin: 0;
    }}
    
    /* Close button at bottom */
    .marco-settings-close-button {{
        min-width: 90px;
        min-height: {close_button_height};
        padding: {button_padding};
        border-radius: {button_radius};
        font-size: 12px;
        font-weight: 500;
        margin: 0;
        transition: {transition};
    }}
"#,
        tab_radius = TOOLBAR_BORDER_RADIUS,
        close_button_height = DIALOG_BUTTON_MIN_HEIGHT,
        button_padding = DIALOG_BUTTON_PADDING,
        button_radius = TOOLBAR_BORDER_RADIUS,
        transition = STANDARD_TRANSITION,
    )
}

/// Generate theme-specific settings CSS
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* Settings - {theme} */
    .{theme} .marco-settings-window {{
        background: {window_bg};
    }}
    
    .{theme} .marco-settings-content {{
        background: {window_bg};
    }}
    
    /* Notebook tabs - {theme} */
    .{theme} .marco-settings-notebook {{
        background: {window_bg};
    }}
    
    .{theme} .marco-settings-notebook > header {{
        background: {titlebar_bg};
        border-bottom: 1px solid {border};
    }}
    
    .{theme} .marco-settings-notebook > header > tabs {{
        background: transparent;
        border: none;
        box-shadow: none;
    }}
    
    .{theme} .marco-settings-notebook > header > tabs > tab {{
        background: transparent;
        color: {foreground};
        border-left: 1px solid transparent;
        border-top: 1px solid transparent;
        border-right: 1px solid transparent;
        border-bottom: none;
        box-shadow: none;
        outline: none;
    }}
    
    .{theme} .marco-settings-notebook > header > tabs > tab:hover {{
        background: {tab_hover};
        color: {hover};
        border-left: 1px solid {border};
        border-top: 1px solid {border};
        border-right: 1px solid {border};
        border-bottom: none;
        outline: none;
    }}
    
    .{theme} .marco-settings-notebook > header > tabs > tab:focus {{
        outline: none;
        box-shadow: inset 0 0 0 2px {accent};
    }}
    
    .{theme} .marco-settings-notebook > header > tabs > tab:checked {{
        background: {window_bg};
        color: {active};
        border-left: 1px solid {border};
        border-top: 1px solid {border};
        border-right: 1px solid {border};
        border-bottom: none;
        outline: none;
    }}
    
    .{theme} .marco-settings-notebook > header > tabs > tab:checked:focus {{
        outline: none;
        box-shadow: inset 0 0 0 2px {accent};
    }}
    
    .{theme} .marco-settings-notebook > header > tabs > tab label {{
        color: inherit;
    }}
    
    /* Tab content - {theme} */
    .{theme} .marco-settings-tab {{
        background: {window_bg};
        color: {foreground};
    }}
    
    /* Setting sections - {theme} */
    .{theme} .marco-settings-section {{
        color: {foreground};
    }}
    
    .{theme} .marco-settings-row {{
        color: {foreground};
    }}
    
    /* Setting row frames - {theme} - TRANSPARENT */
    .{theme} .marco-settings-row-frame {{
        background: transparent;
        border-color: {border};
        border-width: 0px;
    }}
    
    /* Remove hover effect for setting rows */
    .{theme} .marco-settings-row-frame:hover {{
        border-color: {border};
        background: transparent;
        border-width: 0px;
    }}
    
    .{theme} .marco-settings-header {{
        color: {foreground};
    }}
    
    .{theme} .marco-settings-description {{
        color: {foreground};
    }}
    
    /* Close button frame - {theme} - USE FOOTER BACKGROUND */
    .{theme} .marco-settings-close-frame {{
        background: {footer_bg};
        border-color: {border};
    }}
    
    /* Remove hover effect for close button frame */
    .{theme} .marco-settings-close-frame:hover {{
        background: {footer_bg};
        border-color: {border};
    }}
    
    /* Close button - {theme} - Suggested action style */
    .{theme} .marco-settings-close-button {{
        background: {accent};
        color: #ffffff;
        border-color: {accent};
    }}
    
    .{theme} .marco-settings-close-button:hover {{
        background: {accent};
        color: #ffffff;
        border-color: {accent};
        opacity: 0.9;
    }}
    
    .{theme} .marco-settings-close-button:active {{
        background: {accent};
        color: #ffffff;
        border-color: {accent};
        opacity: 0.8;
    }}
"#,
        theme = theme_class,
        window_bg = if theme_class.contains("light") { "#FAFAFA" } else { "#1E1E1E" },
        titlebar_bg = palette.titlebar_bg,
        footer_bg = palette.footer_bg,
        foreground = palette.titlebar_foreground,
        border = palette.titlebar_border,
        tab_hover = if theme_class.contains("light") { "#f5f5f5" } else { "#2a2a2a" },
        hover = palette.toolbar_button_hover,
        active = palette.toolbar_button_active,
        accent = palette.toolbar_button_hover_border,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_settings_css_generation() {
        let css = generate_css();
        
        // Verify settings classes present
        assert!(css.contains(".marco-settings-window"));
        assert!(css.contains(".marco-settings-content"));
        assert!(css.contains(".marco-settings-notebook"));
        assert!(css.contains(".marco-settings-tab"));
        assert!(css.contains(".marco-settings-section"));
        assert!(css.contains(".marco-settings-row"));
        assert!(css.contains(".marco-settings-header"));
        assert!(css.contains(".marco-settings-description"));
        
        // Verify theme variants
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify not empty
        assert!(!css.is_empty());
        
        println!("Settings CSS generation smoke test passed - {} bytes", css.len());
    }
    
    #[test]
    fn test_notebook_tab_selectors() {
        let css = generate_base_settings_css();
        
        // Verify GTK4 Notebook nested structure is handled
        assert!(css.contains(".marco-settings-notebook > header"));
        assert!(css.contains(".marco-settings-notebook > header > tabs"));
        assert!(css.contains(".marco-settings-notebook > header > tabs > tab"));
    }
}
