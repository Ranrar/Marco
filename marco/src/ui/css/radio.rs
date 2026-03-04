//! Radio (grouped CheckButton) CSS Generation
//!
//! GTK4 uses `CheckButton` for both checkboxes and radio buttons.
//! When `CheckButton`s are grouped via `set_group()`, the indicator node
//! may use `radio` instead of `check` depending on GTK theme/structure.
//!
//! This module provides accent-colored styling for radio indicators used
//! in compact dialogs (e.g. Insert List).

use super::constants::{DARK_PALETTE, LIGHT_PALETTE};

/// Generate radio-indicator CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(1024);

    css.push_str(&generate_base_css());
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));

    css
}

fn generate_base_css() -> String {
    // Only touches the indicator node (radio/check) so it won't override
    // existing `.marco-checkbutton` typography/padding.
    r#"
    /* Radio indicator base styles (applies to grouped CheckButtons) */
    checkbutton.marco-radio > radio,
    checkbutton.marco-radio > check {
        min-width: 14px;
        min-height: 14px;
        margin-right: 8px;
        border-radius: 999px;
        outline: none;
        box-shadow: none;

        /* Some GTK themes draw the inner radio dot via an icon source.
         * We control the checked state ourselves; disable the icon so
         * `:checked` can be a solid filled circle.
         */
        -gtk-icon-source: none;
        -gtk-icon-shadow: none;
        background-image: none;
    }
"#
    .to_string()
}

fn generate_theme_css(theme_class: &str, palette: &super::constants::ColorPalette) -> String {
    let accent = palette.toolbar_button_hover_border;

    format!(
        r#"
    /* {theme} - Radio indicator theme styles */

    .{theme} checkbutton.marco-radio > radio,
    .{theme} checkbutton.marco-radio > check {{
        background: transparent;
        border: 1px solid {border};
    }}

    .{theme} checkbutton.marco-radio,
    .{theme} checkbutton.marco-radio > label {{
        color: {text_color};
    }}

    .{theme} checkbutton.marco-radio > radio:checked,
    .{theme} checkbutton.marco-radio > check:checked {{
        background: {accent};
        border-color: {accent};

        /* Ensure checked is solid (no inner dot glyph) */
        -gtk-icon-source: none;
        -gtk-icon-shadow: none;
        background-image: none;
    }}

    .{theme} checkbutton.marco-radio:hover > radio,
    .{theme} checkbutton.marco-radio:hover > check {{
        border-color: {accent};
    }}

    .{theme} checkbutton.marco-radio:focus > radio,
    .{theme} checkbutton.marco-radio:focus > check,
    .{theme} checkbutton.marco-radio:focus-within > radio,
    .{theme} checkbutton.marco-radio:focus-within > check {{
        border-color: {accent};
        box-shadow: 0 0 0 1px {accent};
    }}
"#,
        theme = theme_class,
        accent = accent,
        border = palette.toolbar_border,
        text_color = palette.titlebar_foreground,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_radio_css_generation() {
        let css = generate_css();
        assert!(!css.is_empty());
        assert!(css.contains("checkbutton.marco-radio"));
        assert!(css.contains("marco-theme-light"));
        assert!(css.contains("marco-theme-dark"));
    }
}
