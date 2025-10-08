//! Dialog CSS Generation
//!
//! Generates CSS for custom dialog windows that match Marco's theme.
//!
//! ## Components Styled
//!
//! - `.marco-dialog`: Main dialog window styling
//! - `.marco-dialog-content`: Content area with proper spacing
//! - `.marco-dialog-title`: Dialog title text
//! - `.marco-dialog-message`: Dialog message/description text
//! - `.marco-dialog-button-box`: Action button container
//! - `.marco-dialog-button`: Action buttons (destructive, suggested, cancel)
//!
//! ## Theme Support
//!
//! Generates rules for both `.marco-theme-light` and `.marco-theme-dark` classes.
//!
//! ## Usage
//!
//! Dialog windows should:
//! 1. Use `gtk4::Window` (not deprecated `Dialog`)
//! 2. Add CSS class: `.add_css_class("marco-dialog")`
//! 3. Add theme class: `.add_css_class("marco-theme-light")` or `"marco-theme-dark"`
//! 4. Use `.transient_for(parent)` for modal behavior
//! 5. Use `.set_modal(true)` for modal dialogs

use super::constants::*;

/// Generate complete dialog CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(2048);
    
    // Base dialog styling (theme-independent)
    css.push_str(&format!(
        r#"
    /* Base dialog window styling */
    .marco-dialog {{
        border-radius: {border_radius};
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    }}
    
    .marco-dialog-content {{
        padding: 24px;
        min-width: 400px;
    }}
    
    .marco-dialog-title {{
        font-size: 16px;
        font-weight: 600;
        margin-bottom: 12px;
    }}
    
    .marco-dialog-message {{
        font-size: 14px;
        line-height: 1.5;
        margin-bottom: 20px;
    }}
    
    .marco-dialog-button-box {{
        margin-top: 16px;
        padding: 0;
    }}
    
    .marco-dialog-button {{
        min-width: 100px;
        min-height: 32px;
        padding: {button_padding};
        border-radius: {border_radius};
        font-size: 14px;
        font-weight: 500;
        margin: 0 4px;
        transition: {transition};
    }}
    
    .marco-dialog-button:first-child {{
        margin-left: 0;
    }}
    
    .marco-dialog-button:last-child {{
        margin-right: 0;
    }}
"#,
        border_radius = TOOLBAR_BORDER_RADIUS,
        button_padding = TOOLBAR_BUTTON_PADDING,
        transition = STANDARD_TRANSITION,
    ));
    
    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    
    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    
    css
}

/// Generate theme-specific dialog CSS
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* {theme} - Dialog styles */
    .{theme} .marco-dialog {{
        background: {titlebar_bg};
        border: 1px solid {border};
    }}
    
    .{theme} .marco-dialog-content {{
        background: {titlebar_bg};
    }}
    
    .{theme} .marco-dialog-title {{
        color: {titlebar_foreground};
    }}
    
    .{theme} .marco-dialog-message {{
        color: {titlebar_foreground};
        opacity: 0.9;
    }}
    
    .{theme} .marco-dialog-button {{
        background: transparent;
        color: {titlebar_foreground};
        border: 1px solid {toolbar_border};
    }}
    
    .{theme} .marco-dialog-button:hover {{
        background: {toolbar_popover_bg};
        border-color: {toolbar_button_hover_border};
        color: {toolbar_button_hover};
    }}
    
    .{theme} .marco-dialog-button:active {{
        background: {toolbar_popover_bg};
        color: {toolbar_button_active};
    }}
    
    /* Destructive action button (Close without Saving) */
    .{theme} .marco-dialog-button.destructive-action {{
        background: #d9534f;
        color: #ffffff;
        border-color: #d9534f;
    }}
    
    .{theme} .marco-dialog-button.destructive-action:hover {{
        background: #c9302c;
        border-color: #c9302c;
        color: #ffffff;
    }}
    
    .{theme} .marco-dialog-button.destructive-action:active {{
        background: #ac2925;
        border-color: #ac2925;
        color: #ffffff;
    }}
    
    /* Suggested action button (Save As...) */
    .{theme} .marco-dialog-button.suggested-action {{
        background: {toolbar_button_hover_border};
        color: #ffffff;
        border-color: {toolbar_button_hover_border};
    }}
    
    .{theme} .marco-dialog-button.suggested-action:hover {{
        background: {toolbar_button_hover};
        border-color: {toolbar_button_hover};
        color: #ffffff;
    }}
    
    .{theme} .marco-dialog-button.suggested-action:active {{
        background: {toolbar_button_active};
        border-color: {toolbar_button_active};
        color: #ffffff;
    }}
"#,
        theme = theme_class,
        titlebar_bg = palette.titlebar_bg,
        titlebar_foreground = palette.titlebar_foreground,
        border = palette.titlebar_border,
        toolbar_border = palette.toolbar_border,
        toolbar_popover_bg = palette.toolbar_popover_bg,
        toolbar_button_hover_border = palette.toolbar_button_hover_border,
        toolbar_button_hover = palette.toolbar_button_hover,
        toolbar_button_active = palette.toolbar_button_active,
    )
}
