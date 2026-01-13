//! Footer CSS Generation
//!
//! Generates CSS for Marco's footer elements, status indicators, and styling.
//! Converted from assets/themes/ui_elements/footer.css
//!
//! ## Components Styled
//!
//! - `.footer`: Footer container matching toolbar styling
//! - `.footer label`: Label elements within the footer
//! - `.footer label:empty`: Empty labels (spacers) with transparent background
//!
//! ## Theme Support
//!
//! All components have light and dark theme variants using:
//! - `.marco-theme-light` for light mode
//! - `.marco-theme-dark` for dark mode

use super::constants::*;

/// Generate complete footer CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(2048);

    // Footer container (light theme)
    css.push_str(&generate_footer_container_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Footer container (dark theme)
    css.push_str(&generate_footer_container_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Footer labels (light theme)
    css.push_str(&generate_footer_label_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Footer labels (dark theme)
    css.push_str(&generate_footer_label_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Footer spacer (empty labels) - theme-independent
    css.push_str(&generate_footer_spacer_css());

    css
}

/// Generate footer container CSS for a specific theme
fn generate_footer_container_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Footer container - {theme} (matches toolbar) */
.{theme} .footer {{
    background-color: {bg};
    border-top: {border_width} {border_color};
    color: {color};
    font-size: {font_size};
    font-family: {font_family};
    padding: {padding};
    min-height: {min_height};
}}
"#,
        theme = theme_class,
        bg = palette.footer_bg,
        border_width = FOOTER_BORDER_WIDTH,
        border_color = palette.footer_border,
        color = palette.footer_text,
        font_size = FOOTER_FONT_SIZE,
        font_family = UI_FONT_FAMILY_ALT,
        padding = FOOTER_PADDING,
        min_height = FOOTER_MIN_HEIGHT,
    )
}

/// Generate footer label CSS for a specific theme
fn generate_footer_label_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Footer labels - {theme} */
.{theme} .footer label {{
    color: {color};
    font-size: {font_size};
    font-weight: {font_weight};
    padding: {padding};
}}
"#,
        theme = theme_class,
        color = palette.footer_text,
        font_size = FOOTER_FONT_SIZE,
        font_weight = FOOTER_LABEL_FONT_WEIGHT,
        padding = FOOTER_LABEL_PADDING,
    )
}

/// Generate footer spacer CSS (theme-independent)
/// Note: Removed :empty pseudo-class as GTK CSS parser doesn't fully support it
/// Instead, use a .footer-spacer class in the code for invisible elements
fn generate_footer_spacer_css() -> String {
    r#"
/* Footer spacer (invisible elements should remain transparent) */
/* Note: Use class .footer-spacer for spacing elements instead of :empty */
.marco-theme-light .footer .footer-spacer,
.marco-theme-dark .footer .footer-spacer {
    background-color: transparent;
    padding: 0;
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_footer_css_generation() {
        let css = generate_css();

        // Verify not empty
        assert!(!css.is_empty(), "Footer CSS should not be empty");

        // Verify major components present
        assert!(css.contains(".footer"), "Should contain footer class");
        assert!(
            css.contains(".footer label"),
            "Should contain footer label selector"
        );
        assert!(
            css.contains(".footer-spacer"),
            "Should contain footer-spacer styling"
        );

        // Verify both themes present
        assert!(
            css.contains(".marco-theme-light"),
            "Should contain light theme"
        );
        assert!(
            css.contains(".marco-theme-dark"),
            "Should contain dark theme"
        );

        // Verify specific properties
        assert!(css.contains("min-height"), "Footer should have min-height");
        assert!(css.contains("border-top"), "Footer should have top border");
        assert!(
            css.contains("background-color"),
            "Footer should have background"
        );

        // Verify substantial output (at least 500 bytes)
        assert!(
            css.len() > 500,
            "Footer CSS should be substantial (got {} bytes)",
            css.len()
        );
    }

    #[test]
    fn smoke_test_footer_container_generation() {
        let css = generate_footer_container_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".footer"));
        assert!(css.contains("background-color"));
        assert!(css.contains("border-top"));
        assert!(css.contains(FOOTER_MIN_HEIGHT));
    }

    #[test]
    fn smoke_test_footer_label_generation() {
        let css = generate_footer_label_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".footer label"));
        assert!(css.contains("font-size"));
        assert!(css.contains("font-weight"));
    }

    #[test]
    fn smoke_test_footer_spacer_generation() {
        let css = generate_footer_spacer_css();
        assert!(css.contains(".footer-spacer"));
        assert!(css.contains("transparent"));
        // Verify label:empty is NOT used as a selector (GTK doesn't fully support it)
        assert!(
            !css.contains("label:empty"),
            "Should not use label:empty selector"
        );
    }
}
