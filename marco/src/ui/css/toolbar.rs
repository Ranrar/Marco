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

    // Table picker styling (light theme)
    css.push_str(&generate_table_picker_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Table picker styling (dark theme)
    css.push_str(&generate_table_picker_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Toolbar popover entry styling (menu-like) - light theme
    css.push_str(&generate_toolbar_popover_entry_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Toolbar popover entry styling (menu-like) - dark theme
    css.push_str(&generate_toolbar_popover_entry_css(
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
    "toolbar-btn-text-inline",
    "toolbar-btn-lists",
    "toolbar-btn-inline-items",
    "toolbar-btn-block-items",
    "toolbar-btn-container-items",
    "toolbar-btn-strikethrough",
    "toolbar-btn-bullet",
    "toolbar-btn-number",
    "toolbar-btn-link",
    "toolbar-btn-link-reference",
    "toolbar-btn-blockquote",
    "toolbar-btn-tasklist",
    "toolbar-btn-definition-list",
    "toolbar-btn-image",
    "toolbar-btn-table",
    "toolbar-btn-hr",
    "toolbar-btn-fenced-code-block",
    "toolbar-btn-undo",
    "toolbar-btn-redo",
    "toolbar-btn-functions",
    "toolbar-functions-popover-btn",
    "toolbar-btn-gutter-on",
    "toolbar-btn-gutter-off",
    "toolbar-btn-heading-id",
    "toolbar-btn-admonition",
    "toolbar-btn-footnote",
    "toolbar-btn-inline-footnote",
    "toolbar-btn-inline-math",
    "toolbar-btn-inline-checkbox",
    "toolbar-btn-superscript",
    "toolbar-btn-subscript",
    "toolbar-btn-emoji",
    "toolbar-btn-mention",
    "toolbar-btn-tab-block",
    "toolbar-btn-slideshow",
    "toolbar-btn-math",
    "toolbar-btn-mermaid",
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
    border: none;
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
    opacity: {opacity};
}}
"#,
        theme = theme_class,
        selectors = selectors,
        color_hover = palette.toolbar_button_hover,
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
    opacity: {opacity};
}}
"#,
        theme = theme_class,
        selectors = selectors,
        color_active = palette.toolbar_button_active,
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
    border: none;
    opacity: {opacity};
}}
"#,
        theme = theme_class,
        selectors = selectors,
        bg_disabled = palette.toolbar_button_disabled_bg,
        color_disabled = palette.toolbar_button_disabled,
        opacity = DISABLED_OPACITY,
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
    opacity: 0.65;
    margin: {margin};
}}
"#,
        theme = theme_class,
        width = TOOLBAR_SEPARATOR_WIDTH,
        bg = palette.toolbar_separator,
        margin = TOOLBAR_SEPARATOR_MARGIN,
    )
}

/// Generate table picker cell styling for a specific theme
fn generate_table_picker_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Table picker styling - {theme} */
.{theme} .toolbar-table-picker-cell {{
    min-width: 12px;
    min-height: 12px;
    padding: 0;
    margin: 0;
    border-radius: 2px;
    border: 1px solid {border};
    background: transparent;
}}

.{theme} .toolbar-table-picker-cell:hover,
.{theme} .toolbar-table-picker-cell-active {{
    border: 1px solid {active};
    background: {active_bg};
}}

.{theme} .toolbar-dropdown-btn {{
    padding-right: 2px;
}}
"#,
        theme = theme_class,
        border = palette.toolbar_separator,
        active = palette.toolbar_button_hover,
        active_bg = palette.toolbar_button_disabled_bg,
    )
}

/// Generate menu-like styling for toolbar popover entry buttons.
///
/// This aligns toolbar popover entries with menubar popover items:
/// same text style, row height, padding, colors, and hover/active backgrounds.
fn generate_toolbar_popover_entry_css(theme_class: &str, palette: &ColorPalette) -> String {
    let item_hover_bg = if theme_class.contains("light") {
        "#e8e8e8"
    } else {
        "#3d3d3d"
    };

    format!(
        r#"
/* Toolbar popover entries (menu-style) - {theme} */
.{theme} popover .toolbar-headings-popover-btn,
.{theme} popover .toolbar-functions-popover-btn {{
    background: transparent;
    color: {color};
    padding: {item_padding};
    border-radius: {radius};
    min-height: {item_min_height};
    margin: {item_margin};
    font-size: {font_size};
    font-weight: {font_weight};
    border: none;
    box-shadow: none;
    transition: background 0.15s, color 0.15s;
}}

.{theme} popover .toolbar-headings-popover-btn:hover,
.{theme} popover .toolbar-functions-popover-btn:hover {{
    background: {hover_bg};
    color: {color};
}}

.{theme} popover .toolbar-headings-popover-btn:active,
.{theme} popover .toolbar-functions-popover-btn:active {{
    background: {hover_bg};
    color: {color};
}}

.{theme} popover .toolbar-headings-popover-btn label,
.{theme} popover .toolbar-functions-popover-btn label {{
    color: inherit;
    font-weight: inherit;
}}
"#,
        theme = theme_class,
        color = palette.titlebar_foreground,
        item_padding = POPOVER_ITEM_PADDING,
        radius = POPOVER_BORDER_RADIUS,
        item_min_height = POPOVER_ITEM_MIN_HEIGHT,
        item_margin = POPOVER_ITEM_MARGIN,
        font_size = MENU_FONT_SIZE,
        font_weight = MENU_ITEM_FONT_WEIGHT,
        hover_bg = item_hover_bg,
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
    fn smoke_test_toolbar_popover_entry_generation() {
        let css = generate_toolbar_popover_entry_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".toolbar-headings-popover-btn"));
        assert!(css.contains(".toolbar-functions-popover-btn"));
        assert!(css.contains(POPOVER_ITEM_PADDING));
        assert!(css.contains(POPOVER_ITEM_MIN_HEIGHT));
    }
}
