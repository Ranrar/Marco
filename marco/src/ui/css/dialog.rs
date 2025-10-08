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
    let mut css = String::with_capacity(4096); // Increased for search dialog styles
    
    // Base dialog styling (theme-independent)
    css.push_str(&generate_base_dialog_css());
    
    // Search window specific styles
    css.push_str(&generate_base_search_css());
    
    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    
    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    
    css
}

/// Generate base dialog CSS (theme-independent)
fn generate_base_dialog_css() -> String {
    format!(
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
    )
}

/// Generate base search window CSS (theme-independent)
fn generate_base_search_css() -> String {
    format!(
        r#"
    /* Search window styling */
    .marco-search-window {{
        min-width: 500px;
        min-height: 300px;
    }}
    
    .marco-search-content {{
        padding: 16px;
    }}
    
    .marco-search-row {{
        margin-bottom: 12px;
    }}
    
    .marco-search-entry {{
        min-height: 32px;
        padding: 6px 12px;
        font-size: 14px;
        border-radius: {border_radius};
    }}
    
    .marco-search-button {{
        min-height: 32px;
        padding: {button_padding};
        border-radius: {border_radius};
        font-size: 14px;
        margin: 0 4px;
        transition: {transition};
    }}
    
    .marco-search-checkbox {{
        margin: 4px 8px;
        font-size: 14px;
    }}
    
    .marco-search-checkbox check {{
        min-width: 16px;
        min-height: 16px;
    }}
    
    .marco-search-label {{
        font-size: 14px;
    }}
    
    .marco-search-separator {{
        margin-top: 8px;
        margin-bottom: 8px;
        min-height: 1px;
    }}
    
    .marco-search-match-label {{
        font-size: 13px;
        margin-right: 45px;
    }}
"#,
        border_radius = TOOLBAR_BORDER_RADIUS,
        button_padding = TOOLBAR_BUTTON_PADDING,
        transition = STANDARD_TRANSITION,
    )
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
    
    /* Search window theme styles */
    .{theme} .marco-search-window {{
        background: {titlebar_bg};
    }}
    
    .{theme} .marco-search-content {{
        background: {titlebar_bg};
    }}
    
    .{theme} .marco-search-entry {{
        background: {toolbar_popover_bg};
        color: {titlebar_foreground};
        border: 1px solid {toolbar_border};
        border-radius: 4px;
        outline: none;
        caret-color: {titlebar_foreground};
        box-shadow: none;
    }}
    
    .{theme} .marco-search-entry:hover {{
        border-color: {toolbar_button_hover_border};
    }}
    
    .{theme} .marco-search-entry:focus {{
        background: {toolbar_popover_bg};
        border-color: {toolbar_button_hover_border};
        outline: none;
        caret-color: {titlebar_foreground};
        box-shadow: none;
    }}
    
    .{theme} entry.marco-search-entry {{
        background: {toolbar_popover_bg};
        color: {titlebar_foreground};
        border: 1px solid {toolbar_border};
        border-radius: 4px;
        outline: none;
        caret-color: {titlebar_foreground};
        box-shadow: none;
    }}
    
    .{theme} entry.marco-search-entry:hover {{
        border-color: {toolbar_button_hover_border};
    }}
    
    .{theme} entry.marco-search-entry:focus {{
        background: {toolbar_popover_bg};
        border-color: {toolbar_button_hover_border};
        outline: none;
        caret-color: {titlebar_foreground};
        box-shadow: none;
    }}
    
    .{theme} .marco-search-button {{
        background: transparent;
        color: {titlebar_foreground};
        border: 1px solid {toolbar_border};
    }}
    
    .{theme} .marco-search-button:hover {{
        background: {toolbar_popover_bg};
        border-color: {toolbar_button_hover_border};
        color: {toolbar_button_hover};
    }}
    
    .{theme} .marco-search-button:active {{
        background: {toolbar_popover_bg};
        color: {toolbar_button_active};
    }}
    
    .{theme} .marco-search-button:disabled {{
        background: transparent;
        color: {toolbar_border};
        border-color: {toolbar_border};
        opacity: 0.5;
    }}
    
    .{theme} .marco-search-checkbox {{
        color: {titlebar_foreground};
    }}
    
    .{theme} .marco-search-checkbox:hover {{
        color: {toolbar_button_hover};
    }}
    
    .{theme} .marco-search-checkbox check {{
        background: {toolbar_popover_bg};
        border: 1px solid {toolbar_border};
        border-radius: 3px;
    }}
    
    .{theme} .marco-search-checkbox check:checked {{
        background: {toolbar_button_hover_border};
        border-color: {toolbar_button_hover_border};
        color: #ffffff;
    }}
    
    .{theme} .marco-search-checkbox check:hover {{
        border-color: {toolbar_button_hover_border};
    }}
    
    .{theme} .marco-search-label {{
        color: {titlebar_foreground};
    }}
    
    .{theme} .marco-search-separator {{
        background: {toolbar_border};
    }}
    
    .{theme} .marco-search-match-label {{
        color: {titlebar_foreground};
        opacity: 0.7;
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
