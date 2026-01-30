//! Menu/Titlebar CSS Generation
//!
//! Generates CSS for Marco's titlebar, window controls, icon fonts, and layout buttons.
//! Converted from assets/themes/ui_elements/menu.css
//!
//! ## Components Styled
//!
//! - `.layout-state`: Layout buttons inside the layout popover
//! - `.layout-btn`: Transparent buttons for layout popover
//! - `.layout-btn`: Transparent buttons for layout popover
//! - `.menubar`, `.titlebar`: Menu bar and titlebar container
//! - `.menuitem`: Menu item buttons
//! - `.window-control-btn`: Window control buttons (minimize/maximize/close)
//! - `.topright-btn`: Spacing helper for top-right controls
//! - `.title-label`: Centered document title in titlebar
//!
//! ## Theme Support
//!
//! All components have light and dark theme variants using:
//! - `.marco-theme-light` for light mode
//! - `.marco-theme-dark` for dark mode

use super::constants::*;

/// Generate complete menu/titlebar CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(8192);

    // Base icon font styling (theme-independent)
    css.push_str(&generate_base_styles());

    // Layout state icons (light theme)
    css.push_str(&generate_layout_state_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Layout state icons (dark theme)
    css.push_str(&generate_layout_state_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Layout buttons (theme-independent)
    css.push_str(&generate_layout_button_css());

    // Menubar/Titlebar container (light theme)
    css.push_str(&generate_menubar_css("marco-theme-light", &LIGHT_PALETTE));

    // Menubar/Titlebar container (dark theme)
    css.push_str(&generate_menubar_css("marco-theme-dark", &DARK_PALETTE));

    // PopoverMenuBar and popover styling (light theme)
    css.push_str(&generate_popover_menu_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // PopoverMenuBar and popover styling (dark theme)
    css.push_str(&generate_popover_menu_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Universal popover styling for SourceView5/WebKit6 context menus (light theme)
    css.push_str(&generate_universal_popover_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Universal popover styling for SourceView5/WebKit6 context menus (dark theme)
    css.push_str(&generate_universal_popover_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Menu buttons (light theme)
    css.push_str(&generate_menu_button_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Menu buttons (dark theme)
    css.push_str(&generate_menu_button_css("marco-theme-dark", &DARK_PALETTE));

    // Menu items (light theme)
    css.push_str(&generate_menuitem_css("marco-theme-light", &LIGHT_PALETTE));

    // Menu items (dark theme)
    css.push_str(&generate_menuitem_css("marco-theme-dark", &DARK_PALETTE));

    // Menu items disabled state (theme-independent base + theme-specific colors)
    css.push_str(&generate_menuitem_disabled_css());

    // Window control buttons (light theme)
    css.push_str(&generate_window_controls_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Window control buttons (dark theme)
    css.push_str(&generate_window_controls_css(
        "marco-theme-dark",
        &DARK_PALETTE,
    ));

    // Window control button base styles (theme-independent)
    css.push_str(&generate_window_control_base_css());

    // Top right button spacing
    css.push_str(&generate_topright_btn_css());

    // Title label (light theme)
    css.push_str(&generate_title_label_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
    ));

    // Title label (dark theme)
    css.push_str(&generate_title_label_css("marco-theme-dark", &DARK_PALETTE));

    css
}

/// Generate base icon font styling (theme-independent)
fn generate_base_styles() -> String {
    format!(
        r#"
/*
 * Styles for titlebar/menu icons and controls.
 *
 * Icons are SVG-based and rendered at high DPI for crispness.
 * Runtime Theme Switching:
 * - Uses .marco-theme-light and .marco-theme-dark classes on window
 * - Classes toggled dynamically when theme changes without app restart
 */
"#,
    )
}

/// Generate layout state icon CSS for a specific theme
fn generate_layout_state_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Layout-state icons shown inside the layout popover - {theme} */
.{theme} .layout-state {{
    font-size: {layout_size};
    color: {color};
    padding: {padding};
    background: transparent;
}}

.{theme} .layout-state:hover {{
    font-size: {layout_size};
    color: {color_hover};
    padding: {padding};
    background: transparent;
}}

.{theme} .layout-state:active {{
    font-size: {layout_size};
    color: {color_active};
    padding: {padding};
    background: transparent;
}}
"#,
        theme = theme_class,
        layout_size = LAYOUT_ICON_SIZE,
        color = palette.layout_icon,
        color_hover = palette.layout_icon_hover,
        color_active = palette.layout_icon_active,
        padding = LAYOUT_STATE_PADDING,
    )
}

/// Generate layout button CSS (theme-independent - always transparent)
fn generate_layout_button_css() -> String {
    r#"
/* Buttons inside the layout popover - keep transparent background on hover
   but allow the glyph color to change */
.layout-btn {
    background: transparent;
    border: none;
    padding: 0px;
}
.layout-btn:hover, .layout-btn:active, .layout-btn:focus { background: transparent; }
"#
    .to_string()
}

/// Generate menubar/titlebar container CSS for a specific theme
fn generate_menubar_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Menu bar container - {theme} */
.{theme} .menubar, 
.{theme} .titlebar {{
    min-height: {height};
    background: {bg};
    border-bottom: {border_width} {border_color};
    font-family: {font_family};
    font-size: {font_size};
    color: {color};
}}
"#,
        theme = theme_class,
        height = TITLEBAR_HEIGHT,
        bg = palette.titlebar_bg,
        border_width = TITLEBAR_BORDER_WIDTH,
        border_color = palette.titlebar_border,
        font_family = UI_FONT_FAMILY,
        font_size = MENU_FONT_SIZE,
        color = palette.titlebar_foreground,
    )
}

/// Generate popover menu CSS following Polo's simpler approach
fn generate_popover_menu_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* PopoverMenuBar - remove all decorations from menu items - {theme} */
.{theme} menubar,
.{theme} menubar > item,
.{theme} menubar > item label,
.{theme} menubar > item button,
.{theme} menubar > item box {{
    border: 0px solid transparent;
    border-top: 0px solid transparent;
    border-bottom: 0px solid transparent;
    border-left: 0px solid transparent;
    border-right: 0px solid transparent;
    outline: 0px solid transparent;
    outline-width: 0px;
    border-width: 0px;
    box-shadow: none;
    text-decoration: none;
}}

.{theme} menubar > item {{
    background: transparent;
    color: {color};
    padding: 4px 8px;
    font-size: {font_size};
}}

.{theme} menubar > item:hover {{
    background: transparent;
    color: {color_hover};
}}

.{theme} menubar > item:active,
.{theme} menubar > item.active {{
    background: transparent;
    color: {color_active};
}}

.{theme} menubar > item:focus {{
    background: transparent;
}}

.{theme} menubar > item label {{
    color: inherit;
}}

/* Popover menu styling - {theme} - Marco standard design matching Polo */
.{theme} popover.menu,
.{theme} popover.background {{
    background: transparent;
    border: none;
    box-shadow: none;
}}

/* Style the popover arrow - subtle and matching the background */
.{theme} popover.menu > arrow {{
    background: {popover_bg};
    border: none;
    min-height: {arrow_size};
    min-width: {arrow_size};
    -gtk-icon-shadow: {shadow};
}}

/* Arrow background shape - blends seamlessly with contents */
.{theme} popover.menu > arrow.top {{
    -gtk-icon-source: -gtk-recolor(url("arrow-up-symbolic"));
}}

.{theme} popover.menu > arrow.bottom {{
    -gtk-icon-source: -gtk-recolor(url("arrow-down-symbolic"));
}}

.{theme} popover.menu > arrow.left {{
    -gtk-icon-source: -gtk-recolor(url("arrow-left-symbolic"));
}}

.{theme} popover.menu > arrow.right {{
    -gtk-icon-source: -gtk-recolor(url("arrow-right-symbolic"));
}}

/* Style the contents node - Marco standard with proper padding */
.{theme} popover.menu > contents {{
    background: {popover_bg};
    color: {color};
    border: none;
    box-shadow: {shadow};
    border-radius: {popover_radius};
    padding: {contents_padding};
}}

/* Menu items inside popover - Marco standard with smooth transitions */
.{theme} popover.menu modelbutton {{
    background: transparent;
    color: {color};
    padding: {item_padding};
    border-radius: {popover_radius};
    min-height: {item_min_height};
    font-size: {font_size};
    font-weight: {font_weight};
    margin: {item_margin};
    transition: background 0.15s, color 0.15s;
}}

.{theme} popover.menu modelbutton:hover {{
    background: {item_hover_bg};
    color: {color};
}}

.{theme} popover.menu modelbutton:active {{
    background: {item_hover_bg};
    color: {color};
}}

.{theme} popover.menu modelbutton:disabled {{
    color: {disabled};
    opacity: 0.5;
}}

/* Labels inside menu items - inherit font properties */
.{theme} popover.menu modelbutton label {{
    color: inherit;
    font-weight: inherit;
}}

.{theme} popover.menu separator {{
    background: {border};
    min-height: {separator_height};
    margin: {separator_margin};
}}"#,
        theme = theme_class,
        color = palette.titlebar_foreground,
        color_hover = palette.menu_hover,
        color_active = palette.menu_active,
        popover_bg = if theme_class.contains("light") {
            "#ffffff"
        } else {
            "#2d2d2d"
        },
        border = palette.titlebar_border,
        disabled = palette.menu_disabled,
        item_hover_bg = if theme_class.contains("light") {
            "#e8e8e8"
        } else {
            "#3d3d3d"
        },
        shadow = if theme_class.contains("light") {
            "0 2px 6px rgba(0, 0, 0, 0.15)"
        } else {
            "0 2px 6px rgba(0, 0, 0, 0.4)"
        },
        font_size = MENU_FONT_SIZE,
        font_weight = MENU_ITEM_FONT_WEIGHT,
        popover_radius = POPOVER_BORDER_RADIUS,
        contents_padding = POPOVER_CONTENTS_PADDING,
        item_padding = POPOVER_ITEM_PADDING,
        item_margin = POPOVER_ITEM_MARGIN,
        item_min_height = POPOVER_ITEM_MIN_HEIGHT,
        arrow_size = POPOVER_ARROW_SIZE,
        separator_margin = POPOVER_SEPARATOR_MARGIN,
        separator_height = POPOVER_SEPARATOR_HEIGHT,
    )
}

/// Generate universal popover CSS for all popovers (including SourceView5 and WebKit6 context menus)
fn generate_universal_popover_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Universal popover styling - {theme} - Applies to ALL popovers including SourceView/WebKit context menus */
.{theme} popover:not(.menu) {{
    background: transparent;
    border: none;
    box-shadow: none;
}}

/* Universal popover arrow styling */
.{theme} popover:not(.menu) > arrow {{
    background: {popover_bg};
    border: none;
    min-height: {arrow_size};
    min-width: {arrow_size};
    -gtk-icon-shadow: {shadow};
}}

/* Arrow directional styling */
.{theme} popover:not(.menu) > arrow.top {{
    -gtk-icon-source: -gtk-recolor(url("arrow-up-symbolic"));
}}

.{theme} popover:not(.menu) > arrow.bottom {{
    -gtk-icon-source: -gtk-recolor(url("arrow-down-symbolic"));
}}

.{theme} popover:not(.menu) > arrow.left {{
    -gtk-icon-source: -gtk-recolor(url("arrow-left-symbolic"));
}}

.{theme} popover:not(.menu) > arrow.right {{
    -gtk-icon-source: -gtk-recolor(url("arrow-right-symbolic"));
}}

/* Universal popover contents styling */
.{theme} popover:not(.menu) > contents {{
    background: {popover_bg};
    color: {color};
    border: none;
    box-shadow: {shadow};
    border-radius: {popover_radius};
    padding: {contents_padding};
}}

/* Universal menu items (for context menus) */
.{theme} popover:not(.menu) modelbutton,
.{theme} popover:not(.menu) menuitem {{
    background: transparent;
    color: {color};
    padding: {item_padding};
    border-radius: {popover_radius};
    min-height: {item_min_height};
    font-size: {font_size};
    font-weight: {font_weight};
    margin: {item_margin};
    transition: background 0.15s, color 0.15s;
}}

.{theme} popover:not(.menu) modelbutton:hover,
.{theme} popover:not(.menu) menuitem:hover {{
    background: {item_hover_bg};
    color: {color};
}}

.{theme} popover:not(.menu) modelbutton:active,
.{theme} popover:not(.menu) menuitem:active {{
    background: {item_hover_bg};
    color: {color};
}}

.{theme} popover:not(.menu) modelbutton:disabled,
.{theme} popover:not(.menu) menuitem:disabled {{
    color: {disabled};
    opacity: 0.5;
}}

/* Labels inside universal popover items */
.{theme} popover:not(.menu) modelbutton label,
.{theme} popover:not(.menu) menuitem label {{
    color: inherit;
    font-weight: inherit;
}}

/* Universal popover separators */
.{theme} popover:not(.menu) separator {{
    background: {border};
    min-height: {separator_height};
    margin: {separator_margin};
}}
"#,
        theme = theme_class,
        color = palette.titlebar_foreground,
        popover_bg = if theme_class.contains("light") {
            "#ffffff"
        } else {
            "#2d2d2d"
        },
        border = palette.titlebar_border,
        disabled = palette.menu_disabled,
        item_hover_bg = if theme_class.contains("light") {
            "#e8e8e8"
        } else {
            "#3d3d3d"
        },
        shadow = if theme_class.contains("light") {
            "0 2px 6px rgba(0, 0, 0, 0.15)"
        } else {
            "0 2px 6px rgba(0, 0, 0, 0.4)"
        },
        font_size = MENU_FONT_SIZE,
        font_weight = MENU_ITEM_FONT_WEIGHT,
        popover_radius = POPOVER_BORDER_RADIUS,
        contents_padding = POPOVER_CONTENTS_PADDING,
        item_padding = POPOVER_ITEM_PADDING,
        item_margin = POPOVER_ITEM_MARGIN,
        item_min_height = POPOVER_ITEM_MIN_HEIGHT,
        arrow_size = POPOVER_ARROW_SIZE,
        separator_margin = POPOVER_SEPARATOR_MARGIN,
        separator_height = POPOVER_SEPARATOR_HEIGHT,
    )
}

/// Generate menu button CSS for custom menu buttons
fn generate_menu_button_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Menu buttons - custom styled buttons for menubar - {theme} */
.{theme} .menu-button {{
    background: transparent;
    color: {color};
    border: none;
    border-radius: 5px;
    padding: 2px 8px;
    margin: 4px 1px;
    font-size: {font_size};
    font-weight: 400;
    min-height: 16px;
    transition: background 80ms ease, color 80ms ease;
    box-shadow: none;
    outline: none;
}}

.{theme} .menu-button:hover {{
    background: {hover_bg};
    color: {color};
}}

.{theme} .menu-button:active {{
    background: {active_bg};
    color: {color};
}}

.{theme} .menu-button:focus {{
    outline: none;
    box-shadow: none;
    background: transparent;
}}
"#,
        theme = theme_class,
        color = palette.titlebar_foreground,
        hover_bg = "rgba(90, 93, 94, 0.31)",
        active_bg = "rgba(90, 93, 94, 0.45)",
        font_size = MENU_FONT_SIZE,
    )
}

/// Generate menu item CSS for a specific theme
fn generate_menuitem_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Menu item base style - {theme} */
.{theme} .menuitem {{
    padding: {padding};
    border-radius: {radius};
    background: transparent;
    color: {color};
    font-weight: {weight};
}}

/* Menu item hover - {theme} */
.{theme} .menuitem:hover {{
    color: {color_hover};
}}

/* Menu item active (pressed) - {theme} */
.{theme} .menuitem:active {{
    color: {color_active};
}}
"#,
        theme = theme_class,
        padding = MENU_ITEM_PADDING,
        radius = MENU_BORDER_RADIUS,
        color = palette.titlebar_foreground,
        weight = MENU_ITEM_FONT_WEIGHT,
        color_hover = palette.menu_hover,
        color_active = palette.menu_active,
    )
}

/// Generate menu item disabled state CSS
fn generate_menuitem_disabled_css() -> String {
    format!(
        r#"
/* Menu item disabled - both modes */
.menuitem:disabled {{
    background: transparent;
    opacity: {opacity};
}}

.marco-theme-light .menuitem:disabled {{
    color: {light_disabled};
}}

.marco-theme-dark .menuitem:disabled {{
    color: {dark_disabled};
}}
"#,
        opacity = DISABLED_OPACITY,
        light_disabled = LIGHT_PALETTE.menu_disabled,
        dark_disabled = DARK_PALETTE.menu_disabled,
    )
}

/// Generate window control button CSS for a specific theme
fn generate_window_controls_css(theme_class: &str, palette: &ColorPalette) -> String {
    // Add both icon color rules and background hover/active background rules to match Polo
    let mut css = String::new();
    css.push_str(&format!(
        r#"/* Window controls - {theme} */
.{theme} .window-control-btn {{ color: {color}; }}
.{theme} .window-control-btn:hover {{ color: {color_hover}; transform: translateY(0); }}
.{theme} .window-control-btn:active {{ color: {color_active}; transform: translateY(0); }}
"#,
        theme = theme_class,
        color = palette.control_icon,
        color_hover = palette.control_icon_hover,
        color_active = palette.control_icon_active,
    ));

    // Add background hover/active colors per theme (use same RGBA values as Polo)
    if theme_class == "marco-theme-light" {
        css.push_str(r#"
.marco-theme-light .window-control-btn:hover {
    background: rgba(37, 99, 235, 0.08);
}
.marco-theme-light .window-control-btn:active {
    background: rgba(30, 64, 175, 0.12);
}
"#);
    } else if theme_class == "marco-theme-dark" {
        css.push_str(r#"
.marco-theme-dark .window-control-btn:hover {
    background: rgba(37, 99, 235, 0.12);
}
.marco-theme-dark .window-control-btn:active {
    background: rgba(30, 64, 175, 0.16);
}
"#);
    }

    css
}

/// Generate window control button base CSS (theme-independent)
fn generate_window_control_base_css() -> String {
    format!(
        r#"/* Window control hover/active states for SVG icons and buttons */
.window-control-btn, .topright-btn {{
    transition: background 0.15s ease, color 0.12s, transform 0.08s;
}}

/* Window control button base */
.window-control-btn {{ background: transparent; border: none; padding: {padding}; border-radius: {radius}; }}
"#,
        padding = WINDOW_CONTROL_PADDING,
        radius = WINDOW_CONTROL_BORDER_RADIUS,
    )
}

/// Generate top right button spacing CSS
fn generate_topright_btn_css() -> String {
    format!(
        r#"
/* Top right button style */
.topright-btn {{
    margin-left: {margin};
    margin-right: {margin};
}}
"#,
        margin = TOPRIGHT_BTN_MARGIN,
    )
}

/// Generate title label CSS for a specific theme
fn generate_title_label_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
/* Centered document title shown in the custom titlebar - {theme} */
.{theme} .title-label {{
    font-family: {font_family};
    font-size: {font_size};
    color: {color};
    font-weight: {weight};
    padding: {padding};
    margin: {margin};
}}
"#,
        theme = theme_class,
        font_family = UI_FONT_FAMILY,
        font_size = TITLE_LABEL_FONT_SIZE,
        color = palette.title_label,
        weight = TITLE_LABEL_FONT_WEIGHT,
        padding = TITLE_LABEL_PADDING,
        margin = TITLE_LABEL_MARGIN,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_menu_css_generation() {
        let css = generate_css();

        // Verify not empty
        assert!(!css.is_empty(), "Menu CSS should not be empty");

        // Verify major components present
        assert!(
            css.contains(".layout-state"),
            "Should contain layout-state class"
        );
        assert!(css.contains(".menubar"), "Should contain menubar class");
        assert!(css.contains(".titlebar"), "Should contain titlebar class");
        assert!(css.contains(".menuitem"), "Should contain menuitem class");
        assert!(
            css.contains(".window-control-btn"),
            "Should contain window-control-btn class"
        );
        assert!(
            css.contains(".title-label"),
            "Should contain title-label class"
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
        assert!(css.contains("32px"), "Should have 32px titlebar height");

        // Verify substantial output (at least 3KB)
        assert!(
            css.len() > 3000,
            "Menu CSS should be substantial (got {} bytes)",
            css.len()
        );
    }

    #[test]
    fn smoke_test_layout_state_generation() {
        let css = generate_layout_state_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".layout-state"));
        assert!(css.contains(":hover"));
        assert!(css.contains(":active"));
    }

    #[test]
    fn smoke_test_menubar_generation() {
        let css = generate_menubar_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".menubar"));
        assert!(css.contains(".titlebar"));
        assert!(css.contains(TITLEBAR_HEIGHT));
    }

    #[test]
    fn smoke_test_window_controls_generation() {
        let css = generate_window_controls_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".window-control-btn"));
    }

    #[test]
    fn smoke_test_title_label_generation() {
        let css = generate_title_label_css("marco-theme-light", &LIGHT_PALETTE);
        assert!(css.contains(".title-label"));
        assert!(css.contains(TITLE_LABEL_FONT_SIZE));
        assert!(css.contains(TITLE_LABEL_FONT_WEIGHT));
    }
}
