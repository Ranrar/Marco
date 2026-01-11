//! Toolbar CSS Generation
//!
//! Generates CSS for Marco's toolbar buttons, icons, and styling.
//! Converted from assets/themes/ui_elements/toolbar.css
//!
//! ## Components Styled
//!
//! - `.toolbar`: Toolbar container
//! - `.toolbar-btn`: Generic toolbar buttons
//! - `.toolbar-headings-btn`: Headings dropdown button
//! - `.toolbar-headings-popover-btn`: Buttons inside headings popover
//! - `.toolbar-btn-bold`, `.toolbar-btn-italic`, etc.: Specific toolbar buttons
//! - `.toolbar-separator`: Visual separator between button groups
//! - `.toolbar-popover`: Popover container styling
//! - `.toolbar-btn .icon`: Icon styling inside buttons
//!
//! ## Theme Support
//!
//! All components have light and dark theme variants using:
//! - `.marco-theme-light` for light mode
//! - `.marco-theme-dark` for dark mode

use super::constants::*;

/// Generate complete toolbar CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(8192);

    // Toolbar container (light theme)
    css.push_str(&generate_toolbar_container_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Toolbar container (dark theme)
    css.push_str(&generate_toolbar_container_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Unified toolbar button base styles (light theme)
    css.push_str(&generate_toolbar_buttons_base_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Unified toolbar button base styles (dark theme)
    css.push_str(&generate_toolbar_buttons_base_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Unified hover state (light theme)
    css.push_str(&generate_toolbar_buttons_hover_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Unified hover state (dark theme)
    css.push_str(&generate_toolbar_buttons_hover_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Unified active state (light theme)
    css.push_str(&generate_toolbar_buttons_active_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Unified active state (dark theme)
    css.push_str(&generate_toolbar_buttons_active_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Unified disabled state (light theme)
    css.push_str(&generate_toolbar_buttons_disabled_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Unified disabled state (dark theme)
    css.push_str(&generate_toolbar_buttons_disabled_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Icon styling (theme-independent)
    css.push_str(&generate_toolbar_icon_css());

    // Separator styling (light theme)
    css.push_str(&generate_toolbar_separator_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Separator styling (dark theme)
    css.push_str(&generate_toolbar_separator_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Popover styling (light theme)
    css.push_str(&generate_toolbar_popover_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Popover styling (dark theme)
    css.push_str(&generate_toolbar_popover_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    css
}

/// Generate toolbar container CSS for a specific theme
fn generate_toolbar_container_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Toolbar container - {theme} */
.{theme} .toolbar {{
    background-color: {bg};
    border-bottom: {border_width} {border_color};
    padding: {padding};
}}
"#,
        theme = theme_class,
        bg = palette.toolbar_bg,
        border_width = TOOLBAR_BORDER_WIDTH,
        border_color = palette.toolbar_border,
        padding = TOOLBAR_PADDING,
    )
}

/// List of all toolbar button class names
const TOOLBAR_BUTTON_CLASSES: &[&str] = &[
    "toolbar-btn",
    "toolbar-headings-btn",
    "toolbar-headings-popover-btn",
    "toolbar-btn-bold",
    "toolbar-btn-italic",
    "toolbar-btn-code",
    "toolbar-btn-strikethrough",
    "toolbar-btn-bullet",
    "toolbar-btn-number",
];

/// Generate unified base styles for all toolbar buttons
fn generate_toolbar_buttons_base_css(theme_class: &str, palette: &ColorPalette) -> String {
    let selectors = TOOLBAR_BUTTON_CLASSES
        .iter()
        .map(|class| format!(".{} .{}", theme_class, class))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        r#"
/* Unified toolbar button styling - {theme} */
{selectors} {{
    min-width: {min_width};
    min-height: {min_height};
    padding: {padding};
    margin: {margin};
    border-radius: {radius};
    background: transparent;
    border: {border_width} {border_color};
    color: {color};
    font-size: {font_size};
    font-family: {font_family};
    box-shadow: none;
    transition: {transition};
    opacity: {opacity};
}}
"#,
        theme = theme_class,
        selectors = selectors,
        min_width = TOOLBAR_BUTTON_MIN_WIDTH,
        min_height = TOOLBAR_BUTTON_MIN_HEIGHT,
        padding = TOOLBAR_BUTTON_PADDING,
        margin = TOOLBAR_BUTTON_MARGIN,
        radius = TOOLBAR_BORDER_RADIUS,
        border_width = TOOLBAR_BUTTON_BORDER_WIDTH,
        border_color = palette.toolbar_border,
        color = palette.toolbar_button,
        font_size = TOOLBAR_BUTTON_FONT_SIZE,
        font_family = UI_FONT_FAMILY_ALT,
        transition = STANDARD_TRANSITION,
        opacity = NORMAL_OPACITY,
    )
}

/// Generate unified hover state for all toolbar buttons
fn generate_toolbar_buttons_hover_css(theme_class: &str, palette: &ColorPalette) -> String {
    let selectors = TOOLBAR_BUTTON_CLASSES
        .iter()
        .map(|class| format!(".{} .{}:hover", theme_class, class))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        r#"
/* Unified hover state for all toolbar buttons - {theme} */
{selectors} {{
    background: transparent;
    color: {color_hover};
    border-color: {border_hover};
    opacity: {opacity};
}}
"#,
        theme = theme_class,
        selectors = selectors,
        color_hover = palette.toolbar_button_hover,
        border_hover = palette.toolbar_button_hover_border,
        opacity = NORMAL_OPACITY,
    )
}

/// Generate unified active state for all toolbar buttons
fn generate_toolbar_buttons_active_css(theme_class: &str, palette: &ColorPalette) -> String {
    let selectors = TOOLBAR_BUTTON_CLASSES
        .iter()
        .map(|class| format!(".{} .{}:active", theme_class, class))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        r#"
/* Unified active state for all toolbar buttons - {theme} */
{selectors} {{
    background: transparent;
    color: {color_active};
    border-color: {border_active};
    opacity: {opacity};
}}
"#,
        theme = theme_class,
        selectors = selectors,
        color_active = palette.toolbar_button_active,
        border_active = palette.toolbar_button_hover_border,
        opacity = NORMAL_OPACITY,
    )
}

/// Generate unified disabled state for all toolbar buttons
fn generate_toolbar_buttons_disabled_css(theme_class: &str, palette: &ColorPalette) -> String {
    let selectors = TOOLBAR_BUTTON_CLASSES
        .iter()
        .map(|class| format!(".{} .{}:disabled", theme_class, class))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        r#"
/* Unified disabled state for all toolbar buttons - {theme} */
{selectors} {{
    background: {bg_disabled};
    color: {color_disabled};
    border-color: {border_disabled};
    opacity: {opacity};
}}
"#,
        theme = theme_class,
        selectors = selectors,
        bg_disabled = palette.toolbar_button_disabled_bg,
        color_disabled = palette.toolbar_button_disabled,
        border_disabled = palette.toolbar_button_disabled_border,
        opacity = DISABLED_OPACITY,
    )
}

/// Generate icon styling inside toolbar buttons (theme-independent)
fn generate_toolbar_icon_css() -> String {
    format!(
        r#"
/* Icon styling for buttons (if using icons) */
.toolbar-btn .icon,
.toolbar-headings-btn .icon,
.toolbar-headings-popover-btn .icon {{
    margin-right: {margin};
    /* prefer min-size for GTK compatibility */
    min-width: {size};
    min-height: {size};
}}
"#,
        margin = TOOLBAR_ICON_MARGIN,
        size = TOOLBAR_ICON_SIZE,
    )
}

/// Generate toolbar separator CSS for a specific theme
fn generate_toolbar_separator_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Separator styling - {theme} */
.{theme} .toolbar-separator {{
    min-width: {width};
    background: {bg};
    margin: {margin};
}}
"#,
        theme = theme_class,
        width = TOOLBAR_SEPARATOR_WIDTH,
        bg = palette.toolbar_separator,
        margin = TOOLBAR_SEPARATOR_MARGIN,
    )
}

/// Generate toolbar popover CSS for a specific theme
fn generate_toolbar_popover_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Popover styling - {theme} */
.{theme} .toolbar-popover {{
    background: {bg};
    border: {border_width} {border_color};
    border-radius: {radius};
    padding: {padding};
}}
"#,
        theme = theme_class,
        bg = palette.toolbar_popover_bg,
        border_width = TOOLBAR_POPOVER_BORDER_WIDTH,
        border_color = palette.toolbar_popover_border,
        radius = TOOLBAR_BORDER_RADIUS,
        padding = TOOLBAR_POPOVER_PADDING,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_toolbar_css_generation() {
        let css = generate_css();

        // Verify not empty
        assert!(!css.is_empty(), "Toolbar CSS should not be empty");

        // Verify major components present
        assert!(css.contains(".toolbar"), "Should contain toolbar class");
        assert!(
            css.contains(".toolbar-btn"),
            "Should contain toolbar-btn class"
        );
        assert!(
            css.contains(".toolbar-separator"),
            "Should contain toolbar-separator class"
        );
        assert!(
            css.contains(".toolbar-popover"),
            "Should contain toolbar-popover class"
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

        // Verify specific button types
        assert!(css.contains(".toolbar-btn-bold"), "Should have bold button");
        assert!(
            css.contains(".toolbar-btn-italic"),
            "Should have italic button"
        );
        assert!(
            css.contains(".toolbar-headings-btn"),
            "Should have headings button"
        );

        // Verify states
        assert!(css.contains(":hover"), "Should have hover states");
        assert!(css.contains(":active"), "Should have active states");
        assert!(css.contains(":disabled"), "Should have disabled states");

        // Verify substantial output (at least 4KB)
        assert!(
            css.len() > 4000,
            "Toolbar CSS should be substantial (got {} bytes)",
            css.len()
        );
    }

    #[test]
    fn smoke_test_toolbar_container_generation() {
        let css = generate_toolbar_container_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".toolbar"));
        assert!(css.contains("background-color"));
        assert!(css.contains("border-bottom"));
    }

    #[test]
    fn smoke_test_toolbar_buttons_base_generation() {
        let css = generate_toolbar_buttons_base_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".toolbar-btn"));
        assert!(css.contains("min-width"));
        assert!(css.contains("border-radius"));
    }

    #[test]
    fn smoke_test_toolbar_separator_generation() {
        let css = generate_toolbar_separator_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".toolbar-separator"));
        assert!(css.contains("min-width"));
    }

    #[test]
    fn smoke_test_toolbar_popover_generation() {
        let css = generate_toolbar_popover_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".toolbar-popover"));
        assert!(css.contains("background"));
        assert!(css.contains("border"));
    }
}
