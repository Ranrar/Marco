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

    // Diagnostics trigger state colors (theme-independent)
    css.push_str(&generate_footer_diagnostics_trigger_css());

    // Diagnostics popover controls/chips
    css.push_str(&generate_footer_diagnostics_popover_css());

    // Hovered-link slot styling
    css.push_str(&generate_footer_hovered_link_css());

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

/// Generate diagnostics trigger "button" state colors without changing layout metrics.
fn generate_footer_diagnostics_trigger_css() -> String {
    format!(
        r##"
/* Footer diagnostics/stub buttons - toolbar-like icon/text behavior */
.marco-theme-light .footer .footer-diagnostics-trigger,
.marco-theme-dark .footer .footer-diagnostics-trigger {{
    border-radius: 4px;
    padding: 0;
    line-height: 1.0;
    margin: 0;
    min-height: 0;
    min-width: 0;
    background: transparent;
    border: none;
}}

.marco-theme-light .footer .footer-diagnostics-trigger {{
    color: {light_normal};
}}

.marco-theme-dark .footer .footer-diagnostics-trigger {{
    color: {dark_normal};
}}

.marco-theme-light .footer .footer-diagnostics-trigger:hover {{
    background: transparent;
    color: {light_hover};
}}

.marco-theme-dark .footer .footer-diagnostics-trigger:hover {{
    background: transparent;
    color: {dark_hover};
}}

.marco-theme-light .footer .footer-diagnostics-trigger:active {{
    background: transparent;
    color: {light_active};
}}

.marco-theme-dark .footer .footer-diagnostics-trigger:active {{
    background: transparent;
    color: {dark_active};
}}

.marco-theme-light .footer .footer-diagnostics-trigger .footer-status-label,
.marco-theme-dark .footer .footer-diagnostics-trigger .footer-status-label {{
    color: inherit;
    padding: 0;
    margin: 0;
}}

.marco-theme-light .footer .footer-diagnostics-trigger .footer-status-label {{
    color: {light_normal};
}}

.marco-theme-dark .footer .footer-diagnostics-trigger .footer-status-label {{
    color: {dark_normal};
}}

.marco-theme-light .footer .footer-diagnostics-trigger:hover .footer-status-label {{
    color: {light_hover};
}}

.marco-theme-dark .footer .footer-diagnostics-trigger:hover .footer-status-label {{
    color: {dark_hover};
}}

.marco-theme-light .footer .footer-diagnostics-trigger:active .footer-status-label {{
    color: {light_active};
}}

.marco-theme-dark .footer .footer-diagnostics-trigger:active .footer-status-label {{
    color: {dark_active};
}}

.marco-theme-light .footer .footer-diagnostics-trigger box,
.marco-theme-dark .footer .footer-diagnostics-trigger box {{
    padding: 0;
    margin: 0;
}}

/* Neutralize severity classes so all 3 buttons share toolbar-like look */
.marco-theme-light .footer .footer-diagnostics-trigger.footer-diagnostics-ok,
.marco-theme-light .footer .footer-diagnostics-trigger.footer-diagnostics-warning,
.marco-theme-light .footer .footer-diagnostics-trigger.footer-diagnostics-error,
.marco-theme-dark .footer .footer-diagnostics-trigger.footer-diagnostics-ok,
.marco-theme-dark .footer .footer-diagnostics-trigger.footer-diagnostics-warning,
.marco-theme-dark .footer .footer-diagnostics-trigger.footer-diagnostics-error {{
    background: transparent;
    color: inherit;
}}
"##,
        light_normal = LIGHT_PALETTE.control_icon,
        light_hover = LIGHT_PALETTE.control_icon_hover,
        light_active = LIGHT_PALETTE.control_icon_active,
        dark_normal = DARK_PALETTE.control_icon,
        dark_hover = DARK_PALETTE.control_icon_hover,
        dark_active = DARK_PALETTE.control_icon_active,
    )
}

fn generate_footer_hovered_link_css() -> String {
    format!(
        r#"
/* Footer hovered-link slot — matches status-button label style */
.marco-theme-light .footer .footer-hovered-link .footer-status-label,
.marco-theme-dark .footer .footer-hovered-link .footer-status-label {{
    padding: 0;
    margin: 0;
    font-size: {font_size};
    font-weight: {font_weight};
}}
.marco-theme-light .footer .footer-hovered-link .footer-status-label {{
    color: {light_color};
}}
.marco-theme-dark .footer .footer-hovered-link .footer-status-label {{
    color: {dark_color};
}}
"#,
        font_size = FOOTER_FONT_SIZE,
        font_weight = FOOTER_LABEL_FONT_WEIGHT,
        light_color = LIGHT_PALETTE.control_icon,
        dark_color = DARK_PALETTE.control_icon,
    )
}

fn generate_footer_diagnostics_popover_css() -> String {
    r##"
/* Diagnostics popover controls */
.marco-theme-light .footer-diag-filter-check,
.marco-theme-dark .footer-diag-filter-check {
    border-radius: 4px;
    padding: 1px 6px;
    min-height: 0;
}

.marco-theme-light .footer-diag-filter-check check,
.marco-theme-dark .footer-diag-filter-check check {
    min-width: 12px;
    min-height: 12px;
}

.marco-theme-light .footer-diag-filter-check label,
.marco-theme-dark .footer-diag-filter-check label {
    font-weight: 700;
}

/* Diagnostics list rows */
.marco-theme-light .footer-diag-row,
.marco-theme-dark .footer-diag-row {
    padding: 2px 0;
}

/* Tiny severity chips */
.marco-theme-light .footer-diag-chip,
.marco-theme-dark .footer-diag-chip {
    border-radius: 999px;
    min-width: 12px;
    padding: 0 5px;
    font-size: 9px;
    font-weight: 700;
}

.marco-theme-light .footer-diag-chip.footer-diag-chip-error,
.marco-theme-dark .footer-diag-chip.footer-diag-chip-error {
    background-color: #d32f2f;
    color: #ffffff;
}

.marco-theme-light .footer-diag-chip.footer-diag-chip-warning,
.marco-theme-dark .footer-diag-chip.footer-diag-chip-warning {
    background-color: #f9a825;
    color: #1f1f1f;
}

.marco-theme-light .footer-diag-chip.footer-diag-chip-info,
.marco-theme-dark .footer-diag-chip.footer-diag-chip-info {
    background-color: #1976d2;
    color: #ffffff;
}

.marco-theme-light .footer-diag-chip.footer-diag-chip-hint,
.marco-theme-dark .footer-diag-chip.footer-diag-chip-hint {
    background-color: #607d8b;
    color: #ffffff;
}
"##
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
        assert!(
            css.contains(".footer-diagnostics-trigger"),
            "Should contain diagnostics trigger styling"
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

    #[test]
    fn smoke_test_footer_diagnostics_trigger_colors() {
        let css = generate_footer_diagnostics_trigger_css();
        assert!(css.contains("footer-diagnostics-ok"));
        assert!(css.contains("footer-diagnostics-warning"));
        assert!(css.contains("footer-diagnostics-error"));
    }

    #[test]
    fn smoke_test_footer_diagnostics_popover_styles() {
        let css = generate_footer_diagnostics_popover_css();
        assert!(css.contains("footer-diag-filter-check"));
        assert!(css.contains("footer-diag-chip"));
        assert!(css.contains("footer-diag-chip-error"));
        assert!(css.contains("footer-diag-chip-warning"));
        assert!(css.contains("footer-diag-chip-info"));
        assert!(css.contains("footer-diag-chip-hint"));
        assert!(css.contains("#d32f2f"));
        assert!(css.contains("#f9a825"));
        assert!(css.contains("#1976d2"));
    }
}
