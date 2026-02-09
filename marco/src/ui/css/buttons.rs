//! Button CSS Generation
//!
//! Provides standard colored buttons for dialogs and UI actions.
//!
//! Goals:
//! - Match the Settings close button look/feel (blue)
//! - Provide consistent Yellow and Red variants
//! - Keep layout/sizing controlled by the caller (or other CSS classes)

use super::constants::*;

/// Generate button CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(2048);

    css.push_str(&generate_base_css());

    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));

    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));

    css
}

fn generate_base_css() -> String {
    format!(
        r#"
    /* Base button sizing for dialogs (opt-in via .marco-btn) */
    .marco-btn {{
        min-width: 90px;
        min-height: {button_height};
        padding: {button_padding};
        border-radius: {border_radius};
        font-size: 12px;
        font-weight: 500;
        margin: 0 3px;
        transition: {transition};
    }}

    .marco-btn:first-child {{
        margin-left: 0;
    }}

    .marco-btn:last-child {{
        margin-right: 0;
    }}
"#,
        button_height = DIALOG_BUTTON_MIN_HEIGHT,
        button_padding = DIALOG_BUTTON_PADDING,
        border_radius = TOOLBAR_BORDER_RADIUS,
        transition = STANDARD_TRANSITION,
    )
}

fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    // Blue is sourced from the existing Settings close button style.
    let blue = palette.toolbar_button_hover_border;

    // Yellow/Red are shared across themes; they remain readable on both.
    // (These were previously used in dialog.rs as warning/destructive colors.)
    let yellow = "#f0ad4e";
    let red = "#d9534f";

    format!(
        r#"
    /* {theme} - Colored buttons */

    /* Disabled (shared): keep deactivated buttons visibly greyed out */
    .{theme} .marco-btn:disabled,
    .{theme}.marco-btn:disabled {{
        background: {disabled_bg};
        color: {disabled_fg};
        border: 1px solid {disabled_border};
        opacity: 0.6;
    }}

    /* Also cover Search buttons that combine sizing + color classes */
    .{theme} .marco-search-button:disabled,
    .{theme}.marco-search-window .marco-search-button:disabled {{
        background: {disabled_bg};
        color: {disabled_fg};
        border: 1px solid {disabled_border};
        opacity: 0.6;
    }}

    /* Blue (default) */
    .{theme} .marco-btn-blue,
    .{theme} .marco-search-button.marco-btn-blue,
    .{theme}.marco-btn-blue {{
        background: {blue};
        color: #ffffff;
        border: 1px solid {blue};
    }}

    .{theme} .marco-btn-blue:disabled,
    .{theme} .marco-search-button.marco-btn-blue:disabled,
    .{theme}.marco-btn-blue:disabled {{
        background: {disabled_bg};
        color: {disabled_fg};
        border: 1px solid {disabled_border};
        opacity: 0.6;
    }}

    .{theme} .marco-btn-blue:hover,
    .{theme} .marco-search-button.marco-btn-blue:hover,
    .{theme}.marco-btn-blue:hover,
    .{theme}.marco-search-window .marco-search-button.marco-btn-blue:hover {{
        /* Keep the same variant color on hover (do not drift to generic search hover colors) */
        background: {blue};
        border-color: {blue};
        opacity: 0.9;
    }}

    .{theme} .marco-btn-blue:active,
    .{theme} .marco-search-button.marco-btn-blue:active,
    .{theme}.marco-btn-blue:active,
    .{theme}.marco-search-window .marco-search-button.marco-btn-blue:active {{
        background: {blue};
        border-color: {blue};
        opacity: 0.8;
    }}

    /* Yellow */
    .{theme} .marco-btn-yellow,
    .{theme} .marco-search-button.marco-btn-yellow,
    .{theme}.marco-btn-yellow {{
        background: {yellow};
        color: #ffffff;
        border: 1px solid {yellow};
    }}

    .{theme} .marco-btn-yellow:disabled,
    .{theme} .marco-search-button.marco-btn-yellow:disabled,
    .{theme}.marco-btn-yellow:disabled {{
        background: {disabled_bg};
        color: {disabled_fg};
        border: 1px solid {disabled_border};
        opacity: 0.6;
    }}

    .{theme} .marco-btn-yellow:hover,
    .{theme} .marco-search-button.marco-btn-yellow:hover,
    .{theme}.marco-btn-yellow:hover,
    .{theme}.marco-search-window .marco-search-button.marco-btn-yellow:hover {{
        background: {yellow};
        border-color: {yellow};
        opacity: 0.9;
    }}

    .{theme} .marco-btn-yellow:active,
    .{theme} .marco-search-button.marco-btn-yellow:active,
    .{theme}.marco-btn-yellow:active,
    .{theme}.marco-search-window .marco-search-button.marco-btn-yellow:active {{
        background: {yellow};
        border-color: {yellow};
        opacity: 0.8;
    }}

    /* Red */
    .{theme} .marco-btn-red,
    .{theme} .marco-search-button.marco-btn-red,
    .{theme}.marco-btn-red {{
        background: {red};
        color: #ffffff;
        border: 1px solid {red};
    }}

    .{theme} .marco-btn-red:disabled,
    .{theme} .marco-search-button.marco-btn-red:disabled,
    .{theme}.marco-btn-red:disabled {{
        background: {disabled_bg};
        color: {disabled_fg};
        border: 1px solid {disabled_border};
        opacity: 0.6;
    }}

    .{theme} .marco-btn-red:hover,
    .{theme} .marco-search-button.marco-btn-red:hover,
    .{theme}.marco-btn-red:hover,
    .{theme}.marco-search-window .marco-search-button.marco-btn-red:hover {{
        background: {red};
        border-color: {red};
        opacity: 0.9;
    }}

    .{theme} .marco-btn-red:active,
    .{theme} .marco-search-button.marco-btn-red:active,
    .{theme}.marco-btn-red:active,
    .{theme}.marco-search-window .marco-search-button.marco-btn-red:active {{
        background: {red};
        border-color: {red};
        opacity: 0.8;
    }}
"#,
        theme = theme_class,
        blue = blue,
        yellow = yellow,
        red = red,
        disabled_bg = palette.toolbar_button_disabled_bg,
        disabled_fg = palette.toolbar_button_disabled,
        disabled_border = palette.toolbar_button_disabled_border,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_buttons_css_generation() {
        let css = generate_css();
        assert!(!css.is_empty());

        // Base
        assert!(css.contains(".marco-btn"));

        // Variants
        assert!(css.contains(".marco-btn-blue"));
        assert!(css.contains(".marco-btn-yellow"));
        assert!(css.contains(".marco-btn-red"));

        // Themes
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
    }
}
