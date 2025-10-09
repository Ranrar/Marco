//! Controls CSS Generation
//!
//! Generates CSS for GTK4 control widgets used throughout Marco's UI.
//!
//! ## Components Styled
//!
//! - `dropdown.marco-dropdown`: DropDown widget with nested selectors
//! - `switch.marco-switch`: Switch toggle widget
//! - `scale.marco-scale`: Scale slider widget
//! - `spinbutton.marco-spinbutton`: SpinButton numeric input
//! - `entry.marco-entry`: Entry text input widget
//!
//! ## Theme Support
//!
//! Generates rules for both `.marco-theme-light` and `.marco-theme-dark` classes.
//!
//! ## GTK4 Widget Complexity
//!
//! GTK4 widgets have complex nested structures requiring specific CSS selectors:
//! - DropDown: `dropdown > button`, `dropdown > popover > contents`, `dropdown > popover listview > row`
//! - Switch: `switch > slider`
//! - Scale: `scale > trough`, `scale > highlight`, `scale > slider`

use super::constants::*;

/// Generate complete controls CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(8192);
    
    // DropDown styling (ported from Polo)
    css.push_str(&generate_dropdown_css());
    
    // Switch styling
    css.push_str(&generate_switch_css());
    
    // Scale (slider) styling
    css.push_str(&generate_scale_css());
    
    // SpinButton styling
    css.push_str(&generate_spinbutton_css());
    
    // Entry styling
    css.push_str(&generate_entry_css());
    
    // Button styling
    css.push_str(&generate_button_css());
    
    // CheckButton styling
    css.push_str(&generate_checkbutton_css());
    
    css
}

/// Generate DropDown widget CSS (ported from Polo with Marco constants)
fn generate_dropdown_css() -> String {
    let mut css = String::with_capacity(3072);
    
    // Base dropdown sizing (theme-independent)
    css.push_str(
        r#"
    /* DropDown base styles */
    dropdown.marco-dropdown {
        min-width: 150px;
        min-height: 32px;
        font-size: 14px;
    }
"#
    );
    
    // Light theme dropdown
    css.push_str(&format!(
        r#"
    /* DropDown - Light Theme */
    .marco-theme-light dropdown.marco-dropdown > button {{
        background: transparent;
        color: {foreground};
        border: 1px solid {border};
        border-radius: {radius};
        padding: {padding};
        transition: {transition};
        outline: none;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > button:hover {{
        background: transparent;
        color: {hover};
        border-color: {border_hover};
        outline: none;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > button:active {{
        background: transparent;
        color: {active};
        border-color: {border_hover};
        outline: none;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > button:focus {{
        border-color: {border_hover};
        outline: none;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > button label {{
        color: inherit;
    }}
    
    /* DropDown popover - Light Theme */
    .marco-theme-light dropdown.marco-dropdown > popover.background,
    .marco-theme-light dropdown.marco-dropdown > popover {{
        background: transparent;
        border: none;
        box-shadow: none;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover > contents {{
        background: {popover_bg};
        color: {foreground};
        border: none;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
        border-radius: {radius};
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover listview {{
        background: {popover_bg};
        color: {foreground};
        border: none;
        border-radius: {radius};
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover listview > row {{
        background: transparent;
        color: {foreground};
        border: none;
        padding: 6px 12px;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover listview > row:hover {{
        background: {item_hover};
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover listview > row label {{
        color: {foreground};
    }}
    
    /* DropDown search entry - Light Theme */
    .marco-theme-light dropdown.marco-dropdown > popover > contents entry {{
        background: #ffffff;
        color: {foreground};
        caret-color: {foreground};
        border: 1px solid {border};
        border-radius: {radius};
        padding: 6px;
        outline: none;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover > contents entry:focus {{
        border-color: {border_hover};
        outline: none;
        box-shadow: inset 0 0 0 1px {border_hover};
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover > contents entry > text {{
        background: transparent;
        color: {foreground};
        caret-color: {foreground};
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover > contents entry > text > selection {{
        background-color: {border_hover};
        color: #ffffff;
    }}
    
    /* DropDown scrollbar - Light Theme */
    .marco-theme-light dropdown.marco-dropdown > popover scrolledwindow scrollbar {{
        background: transparent;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover scrolledwindow scrollbar slider {{
        background: {border};
        border-radius: 3px;
        min-width: 6px;
        min-height: 40px;
    }}
    
    .marco-theme-light dropdown.marco-dropdown > popover scrolledwindow scrollbar slider:hover {{
        background: {foreground};
    }}
"#,
        foreground = LIGHT_PALETTE.titlebar_foreground,
        border = LIGHT_PALETTE.titlebar_border,
        radius = TOOLBAR_BORDER_RADIUS,
        padding = TOOLBAR_BUTTON_PADDING,
        transition = STANDARD_TRANSITION,
        hover = LIGHT_PALETTE.toolbar_button_hover,
        border_hover = LIGHT_PALETTE.toolbar_button_hover_border,
        active = LIGHT_PALETTE.toolbar_button_active,
        popover_bg = LIGHT_PALETTE.toolbar_popover_bg,
        item_hover = "#e8e8e8",
    ));
    
    // Dark theme dropdown
    css.push_str(&format!(
        r#"
    /* DropDown - Dark Theme */
    .marco-theme-dark dropdown.marco-dropdown > button {{
        background: transparent;
        color: {foreground};
        border: 1px solid {border};
        border-radius: {radius};
        padding: {padding};
        transition: {transition};
        outline: none;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > button:hover {{
        background: transparent;
        color: {hover};
        border-color: {border_hover};
        outline: none;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > button:active {{
        background: transparent;
        color: {active};
        border-color: {border_hover};
        outline: none;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > button:focus {{
        border-color: {border_hover};
        outline: none;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > button label {{
        color: inherit;
    }}
    
    /* DropDown popover - Dark Theme */
    .marco-theme-dark dropdown.marco-dropdown > popover.background,
    .marco-theme-dark dropdown.marco-dropdown > popover {{
        background: transparent;
        border: none;
        box-shadow: none;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover > contents {{
        background: {popover_bg};
        color: {foreground};
        border: none;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        border-radius: {radius};
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover listview {{
        background: {popover_bg};
        color: {foreground};
        border: none;
        border-radius: {radius};
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover listview > row {{
        background: transparent;
        color: {foreground};
        border: none;
        padding: 6px 12px;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover listview > row:hover {{
        background: {item_hover};
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover listview > row label {{
        color: {foreground};
    }}
    
    /* DropDown search entry - Dark Theme */
    .marco-theme-dark dropdown.marco-dropdown > popover > contents entry {{
        background: #2d2d2d;
        color: {foreground};
        caret-color: {foreground};
        border: 1px solid {border};
        border-radius: {radius};
        padding: 6px;
        outline: none;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover > contents entry:focus {{
        border-color: {border_hover};
        outline: none;
        box-shadow: inset 0 0 0 1px {border_hover};
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover > contents entry > text {{
        background: transparent;
        color: {foreground};
        caret-color: {foreground};
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover > contents entry > text > selection {{
        background-color: {border_hover};
        color: #ffffff;
    }}
    
    /* DropDown scrollbar - Dark Theme */
    .marco-theme-dark dropdown.marco-dropdown > popover scrolledwindow scrollbar {{
        background: transparent;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover scrolledwindow scrollbar slider {{
        background: {border};
        border-radius: 3px;
        min-width: 6px;
        min-height: 40px;
    }}
    
    .marco-theme-dark dropdown.marco-dropdown > popover scrolledwindow scrollbar slider:hover {{
        background: {foreground};
    }}
"#,
        foreground = DARK_PALETTE.titlebar_foreground,
        border = DARK_PALETTE.titlebar_border,
        radius = TOOLBAR_BORDER_RADIUS,
        padding = TOOLBAR_BUTTON_PADDING,
        transition = STANDARD_TRANSITION,
        hover = DARK_PALETTE.toolbar_button_hover,
        border_hover = DARK_PALETTE.toolbar_button_hover_border,
        active = DARK_PALETTE.toolbar_button_active,
        popover_bg = DARK_PALETTE.toolbar_popover_bg,
        item_hover = "#3d3d3d",
    ));
    
    css
}

/// Generate Switch widget CSS
fn generate_switch_css() -> String {
    format!(
        r#"
    /* Switch base styles */
    switch.marco-switch {{
        min-width: 52px;
        min-height: 26px;
        border-radius: 13px;
    }}
    
    switch.marco-switch > slider {{
        min-width: 22px;
        min-height: 22px;
        border-radius: 11px;
    }}
    
    /* Switch - Light Theme */
    .marco-theme-light switch.marco-switch {{
        background: {light_border};
        border: 1px solid {light_border};
        outline: none;
    }}
    
    .marco-theme-light switch.marco-switch:checked {{
        background: {light_accent};
        border-color: {light_accent};
        outline: none;
    }}
    
    .marco-theme-light switch.marco-switch:focus {{
        outline: none;
        box-shadow: 0 0 0 2px rgba(0, 102, 204, 0.3);
    }}
    
    .marco-theme-light switch.marco-switch > slider {{
        background: #ffffff;
        border: 1px solid {light_border};
        outline: none;
    }}
    
    .marco-theme-light switch.marco-switch:checked > slider {{
        background: #ffffff;
        outline: none;
    }}
    
    /* Switch - Dark Theme */
    .marco-theme-dark switch.marco-switch {{
        background: {dark_border};
        border: 1px solid {dark_border};
        outline: none;
    }}
    
    .marco-theme-dark switch.marco-switch:checked {{
        background: {dark_accent};
        border-color: {dark_accent};
        outline: none;
    }}
    
    .marco-theme-dark switch.marco-switch:focus {{
        outline: none;
        box-shadow: 0 0 0 2px rgba(79, 140, 255, 0.4);
    }}
    
    .marco-theme-dark switch.marco-switch > slider {{
        background: #f0f0f0;
        border: 1px solid {dark_border};
        outline: none;
    }}
    
    .marco-theme-dark switch.marco-switch:checked > slider {{
        background: #f0f0f0;
        outline: none;
    }}
"#,
        light_border = LIGHT_PALETTE.titlebar_border,
        light_accent = LIGHT_PALETTE.toolbar_button_hover_border,
        dark_border = DARK_PALETTE.titlebar_border,
        dark_accent = DARK_PALETTE.toolbar_button_hover_border,
    )
}

/// Generate Scale (slider) widget CSS
fn generate_scale_css() -> String {
    format!(
        r#"
    /* Scale base styles */
    scale {{
        min-height: 40px;
    }}
    
    scale trough {{
        min-height: 6px;
        border-radius: 3px;
    }}
    
    scale fill {{
        min-width: 0px;
        min-height: 6px;
        border-radius: 3px;
    }}
    
    /* Marco Scale specific styles */
    scale.marco-scale {{
        min-height: 40px;
        padding: 4px 0;
    }}
    
    scale.marco-scale trough {{
        min-height: 6px;
        border-radius: 3px;
    }}
    
    scale.marco-scale fill {{
        min-width: 0px;
        min-height: 6px;
        border-radius: 3px;
    }}
    
    scale.marco-scale slider {{
        min-width: 18px;
        min-height: 18px;
        border-radius: 9px;
        margin: -6px;
    }}
    
    /* Scale - Light Theme */
    .marco-theme-light scale.marco-scale trough {{
        background: {light_trough_bg};
        border: 1px solid {light_border};
        outline: none;
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
    }}
    
    .marco-theme-light scale.marco-scale fill {{
        background: {light_accent};
        outline: none;
        border: none;
        min-width: 0px;
        min-height: 6px;
    }}
    
    .marco-theme-light scale.marco-scale slider {{
        background: linear-gradient(to bottom, #ffffff, #f8f8f8);
        border: 1px solid {light_border};
        outline: none;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    }}
    
    .marco-theme-light scale.marco-scale slider:hover {{
        background: linear-gradient(to bottom, #ffffff, #fafafa);
        border-color: {light_accent};
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
        outline: none;
    }}
    
    .marco-theme-light scale.marco-scale slider:active {{
        background: linear-gradient(to bottom, #f5f5f5, #ececec);
        border-color: {light_accent};
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
        outline: none;
    }}
    
    .marco-theme-light scale.marco-scale:focus {{
        outline: none;
    }}
    
    /* Scale - Dark Theme */
    .marco-theme-dark scale.marco-scale trough {{
        background: {dark_trough_bg};
        border: 1px solid {dark_border};
        outline: none;
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
    }}
    
    .marco-theme-dark scale.marco-scale fill {{
        background: {dark_accent};
        outline: none;
        border: none;
        min-width: 0px;
        min-height: 6px;
    }}
    
    .marco-theme-dark scale.marco-scale slider {{
        background: linear-gradient(to bottom, #f5f5f5, #e8e8e8);
        border: 1px solid {dark_slider_border};
        outline: none;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    }}
    
    .marco-theme-dark scale.marco-scale slider:hover {{
        background: linear-gradient(to bottom, #fafafa, #ececec);
        border-color: {dark_accent};
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
        outline: none;
    }}
    
    .marco-theme-dark scale.marco-scale slider:active {{
        background: linear-gradient(to bottom, #e0e0e0, #d5d5d5);
        border-color: {dark_accent};
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
        outline: none;
    }}
    
    .marco-theme-dark scale.marco-scale:focus {{
        outline: none;
    }}
"#,
        light_border = LIGHT_PALETTE.titlebar_border,
        light_trough_bg = "#e5e5e5",
        light_accent = LIGHT_PALETTE.toolbar_button_hover_border,
        dark_border = DARK_PALETTE.titlebar_border,
        dark_trough_bg = "#3a3a3a",
        dark_accent = DARK_PALETTE.toolbar_button_hover_border,
        dark_slider_border = "#888888",
    )
}

/// Generate SpinButton widget CSS
/// Generate SpinButton widget CSS
fn generate_spinbutton_css() -> String {
    format!(
        r#"
    /* SpinButton base styles */
    spinbutton.marco-spinbutton {{
        min-height: 32px;
        border-radius: {radius};
    }}
    
    spinbutton.marco-spinbutton > text {{
        min-height: 30px;
        padding: 0 8px;
    }}
    
    spinbutton.marco-spinbutton > text > undershoot {{
        background: none;
    }}
    
    spinbutton.marco-spinbutton > button {{
        min-width: 24px;
        border-radius: 0;
    }}
    
    /* SpinButton - Light Theme */
    .marco-theme-light spinbutton.marco-spinbutton {{
        background: #ffffff;
        color: {light_fg};
        border: 1px solid {light_border};
        border-radius: {radius};
    }}
    
    .marco-theme-light spinbutton.marco-spinbutton > text {{
        background: transparent;
        color: {light_fg};
        caret-color: {light_fg};
    }}
    
    .marco-theme-light spinbutton.marco-spinbutton > text > selection {{
        background-color: {light_accent};
        color: #ffffff;
    }}
    
    .marco-theme-light spinbutton.marco-spinbutton > button {{
        background: transparent;
        color: {light_fg};
        border-left: 1px solid {light_border};
    }}
    
    .marco-theme-light spinbutton.marco-spinbutton > button:hover {{
        background: {light_hover};
        color: {light_accent};
    }}
    
    .marco-theme-light spinbutton.marco-spinbutton > button:active {{
        background: {light_hover};
    }}
    
    .marco-theme-light spinbutton.marco-spinbutton:focus-within {{
        border-color: {light_accent};
        outline: none;
    }}
    
    .marco-theme-light spinbutton.marco-spinbutton:focus {{
        border-color: {light_accent};
        outline: none;
    }}
    
    /* SpinButton - Dark Theme */
    .marco-theme-dark spinbutton.marco-spinbutton {{
        background: #2d2d2d;
        color: {dark_fg};
        border: 1px solid {dark_border};
        border-radius: {radius};
    }}
    
    .marco-theme-dark spinbutton.marco-spinbutton > text {{
        background: transparent;
        color: {dark_fg};
        caret-color: {dark_fg};
    }}
    
    .marco-theme-dark spinbutton.marco-spinbutton > text > selection {{
        background-color: {dark_accent};
        color: #ffffff;
    }}
    
    .marco-theme-dark spinbutton.marco-spinbutton > button {{
        background: transparent;
        color: {dark_fg};
        border-left: 1px solid {dark_border};
    }}
    
    .marco-theme-dark spinbutton.marco-spinbutton > button:hover {{
        background: {dark_hover};
        color: {dark_accent};
    }}
    
    .marco-theme-dark spinbutton.marco-spinbutton > button:active {{
        background: {dark_hover};
    }}
    
    .marco-theme-dark spinbutton.marco-spinbutton:focus-within {{
        border-color: {dark_accent};
        outline: none;
    }}
    
    .marco-theme-dark spinbutton.marco-spinbutton:focus {{
        border-color: {dark_accent};
        outline: none;
    }}
"#,
        radius = TOOLBAR_BORDER_RADIUS,
        light_border = LIGHT_PALETTE.titlebar_border,
        light_fg = LIGHT_PALETTE.titlebar_foreground,
        light_hover = "#e8e8e8",
        light_accent = LIGHT_PALETTE.toolbar_button_hover_border,
        dark_border = DARK_PALETTE.titlebar_border,
        dark_fg = DARK_PALETTE.titlebar_foreground,
        dark_hover = "#3d3d3d",
        dark_accent = DARK_PALETTE.toolbar_button_hover_border,
    )
}

/// Generate Entry widget CSS
fn generate_entry_css() -> String {
    format!(
        r#"
    /* Entry base styles */
    entry.marco-entry {{
        min-height: 32px;
        padding: 6px 12px;
        border-radius: {radius};
        font-size: 14px;
    }}
    
    /* Entry - Light Theme */
    .marco-theme-light entry.marco-entry {{
        background: transparent;
        color: {light_fg};
        border: 1px solid {light_border};
        outline: none;
    }}
    
    .marco-theme-light entry.marco-entry:focus {{
        border-color: {light_accent};
        box-shadow: 0 0 0 1px {light_accent};
        outline: none;
    }}
    
    /* Entry - Dark Theme */
    .marco-theme-dark entry.marco-entry {{
        background: transparent;
        color: {dark_fg};
        border: 1px solid {dark_border};
        outline: none;
    }}
    
    .marco-theme-dark entry.marco-entry:focus {{
        border-color: {dark_accent};
        box-shadow: 0 0 0 1px {dark_accent};
        outline: none;
    }}
"#,
        radius = TOOLBAR_BORDER_RADIUS,
        light_fg = LIGHT_PALETTE.titlebar_foreground,
        light_border = LIGHT_PALETTE.titlebar_border,
        light_accent = LIGHT_PALETTE.toolbar_button_hover_border,
        dark_fg = DARK_PALETTE.titlebar_foreground,
        dark_border = DARK_PALETTE.titlebar_border,
        dark_accent = DARK_PALETTE.toolbar_button_hover_border,
    )
}

/// Generate Button widget CSS
fn generate_button_css() -> String {
    format!(
        r#"
    /* Button base styles */
    button.marco-button {{
        min-height: 32px;
        padding: 4px 12px;
        border-radius: {radius};
        font-size: 14px;
        font-weight: 500;
    }}
    
    /* Button - Light Theme */
    .marco-theme-light button.marco-button {{
        background: {light_bg};
        color: {light_fg};
        border: 1px solid {light_border};
        outline: none;
    }}
    
    .marco-theme-light button.marco-button:hover {{
        background: {light_hover};
        border-color: {light_hover_border};
        outline: none;
    }}
    
    .marco-theme-light button.marco-button:active {{
        background: {light_active};
        outline: none;
    }}
    
    .marco-theme-light button.marco-button:focus {{
        border-color: {light_hover_border};
        box-shadow: 0 0 0 1px {light_hover_border};
        outline: none;
    }}
    
    .marco-theme-light button.marco-button:disabled {{
        background: {light_disabled_bg};
        color: {light_disabled_fg};
        border-color: {light_disabled_border};
        outline: none;
    }}
    
    /* Button - Dark Theme */
    .marco-theme-dark button.marco-button {{
        background: {dark_bg};
        color: {dark_fg};
        border: 1px solid {dark_border};
        outline: none;
    }}
    
    .marco-theme-dark button.marco-button:hover {{
        background: {dark_hover};
        border-color: {dark_hover_border};
        outline: none;
    }}
    
    .marco-theme-dark button.marco-button:active {{
        background: {dark_active};
        outline: none;
    }}
    
    .marco-theme-dark button.marco-button:focus {{
        border-color: {dark_hover_border};
        box-shadow: 0 0 0 1px {dark_hover_border};
        outline: none;
    }}
    
    .marco-theme-dark button.marco-button:disabled {{
        background: {dark_disabled_bg};
        color: {dark_disabled_fg};
        border-color: {dark_disabled_border};
        outline: none;
    }}
"#,
        radius = TOOLBAR_BORDER_RADIUS,
        light_bg = LIGHT_PALETTE.toolbar_bg,
        light_fg = LIGHT_PALETTE.toolbar_button,
        light_border = LIGHT_PALETTE.titlebar_border,
        light_hover = LIGHT_PALETTE.toolbar_button_hover,
        light_hover_border = LIGHT_PALETTE.toolbar_button_hover_border,
        light_active = LIGHT_PALETTE.toolbar_button_active,
        light_disabled_bg = LIGHT_PALETTE.toolbar_button_disabled_bg,
        light_disabled_fg = LIGHT_PALETTE.toolbar_button_disabled,
        light_disabled_border = LIGHT_PALETTE.toolbar_button_disabled_border,
        dark_bg = DARK_PALETTE.toolbar_bg,
        dark_fg = DARK_PALETTE.toolbar_button,
        dark_border = DARK_PALETTE.titlebar_border,
        dark_hover = DARK_PALETTE.toolbar_button_hover,
        dark_hover_border = DARK_PALETTE.toolbar_button_hover_border,
        dark_active = DARK_PALETTE.toolbar_button_active,
        dark_disabled_bg = DARK_PALETTE.toolbar_button_disabled_bg,
        dark_disabled_fg = DARK_PALETTE.toolbar_button_disabled,
        dark_disabled_border = DARK_PALETTE.toolbar_button_disabled_border,
    )
}

/// Generate CheckButton widget CSS
fn generate_checkbutton_css() -> String {
    format!(
        r#"
    /* CheckButton base styles */
    checkbutton.marco-checkbutton {{
        min-height: 24px;
        padding: 4px 8px;
        border-radius: {radius};
        font-size: 14px;
    }}
    
    checkbutton.marco-checkbutton > check {{
        min-width: 18px;
        min-height: 18px;
        margin-right: 8px;
        border-radius: 3px;
    }}
    
    /* CheckButton - Light Theme */
    .marco-theme-light checkbutton.marco-checkbutton {{
        color: {light_fg};
        outline: none;
    }}
    
    .marco-theme-light checkbutton.marco-checkbutton > check {{
        background: transparent;
        border: 1px solid {light_border};
        outline: none;
    }}
    
    .marco-theme-light checkbutton.marco-checkbutton > check:checked {{
        background: {light_accent};
        border-color: {light_accent};
        outline: none;
    }}
    
    .marco-theme-light checkbutton.marco-checkbutton:hover > check {{
        border-color: {light_accent};
        outline: none;
    }}
    
    .marco-theme-light checkbutton.marco-checkbutton:focus > check {{
        border-color: {light_accent};
        box-shadow: 0 0 0 1px {light_accent};
        outline: none;
    }}
    
    .marco-theme-light checkbutton.marco-checkbutton:disabled {{
        color: {light_disabled_fg};
        outline: none;
    }}
    
    .marco-theme-light checkbutton.marco-checkbutton:disabled > check {{
        background: {light_disabled_bg};
        border-color: {light_disabled_border};
        outline: none;
    }}
    
    /* CheckButton - Dark Theme */
    .marco-theme-dark checkbutton.marco-checkbutton {{
        color: {dark_fg};
        outline: none;
    }}
    
    .marco-theme-dark checkbutton.marco-checkbutton > check {{
        background: transparent;
        border: 1px solid {dark_border};
        outline: none;
    }}
    
    .marco-theme-dark checkbutton.marco-checkbutton > check:checked {{
        background: {dark_accent};
        border-color: {dark_accent};
        outline: none;
    }}
    
    .marco-theme-dark checkbutton.marco-checkbutton:hover > check {{
        border-color: {dark_accent};
        outline: none;
    }}
    
    .marco-theme-dark checkbutton.marco-checkbutton:focus > check {{
        border-color: {dark_accent};
        box-shadow: 0 0 0 1px {dark_accent};
        outline: none;
    }}
    
    .marco-theme-dark checkbutton.marco-checkbutton:disabled {{
        color: {dark_disabled_fg};
        outline: none;
    }}
    
    .marco-theme-dark checkbutton.marco-checkbutton:disabled > check {{
        background: {dark_disabled_bg};
        border-color: {dark_disabled_border};
        outline: none;
    }}
"#,
        radius = TOOLBAR_BORDER_RADIUS,
        light_fg = LIGHT_PALETTE.titlebar_foreground,
        light_border = LIGHT_PALETTE.titlebar_border,
        light_accent = LIGHT_PALETTE.toolbar_button_hover_border,
        light_disabled_bg = LIGHT_PALETTE.toolbar_button_disabled_bg,
        light_disabled_fg = LIGHT_PALETTE.toolbar_button_disabled,
        light_disabled_border = LIGHT_PALETTE.toolbar_button_disabled_border,
        dark_fg = DARK_PALETTE.titlebar_foreground,
        dark_border = DARK_PALETTE.titlebar_border,
        dark_accent = DARK_PALETTE.toolbar_button_hover_border,
        dark_disabled_bg = DARK_PALETTE.toolbar_button_disabled_bg,
        dark_disabled_fg = DARK_PALETTE.toolbar_button_disabled,
        dark_disabled_border = DARK_PALETTE.toolbar_button_disabled_border,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_controls_css_generation() {
        let css = generate_css();
        
        // Verify all control types are present
        assert!(css.contains("dropdown.marco-dropdown"));
        assert!(css.contains("switch.marco-switch"));
        assert!(css.contains("scale.marco-scale"));
        assert!(css.contains("spinbutton.marco-spinbutton"));
        assert!(css.contains("entry.marco-entry"));
        
        // Verify theme variants
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify not empty
        assert!(!css.is_empty());
        
        println!("Controls CSS generation smoke test passed - {} bytes", css.len());
    }
    
    #[test]
    fn test_dropdown_nested_selectors() {
        let css = generate_dropdown_css();
        
        // Verify GTK4 nested structure is handled
        assert!(css.contains("dropdown.marco-dropdown > button"));
        assert!(css.contains("dropdown.marco-dropdown > popover"));
        assert!(css.contains("dropdown.marco-dropdown > popover > contents"));
        assert!(css.contains("dropdown.marco-dropdown > popover listview > row"));
    }
    
    #[test]
    fn test_scale_highlight_min_sizes() {
        let css = generate_scale_css();
        
        // Verify base highlight has min-sizes
        assert!(css.contains("scale.marco-scale highlight"));
        assert!(css.contains("min-width: 0"));
        assert!(css.contains("min-height: 0") || css.contains("min-height: 6px"));
        
        // Verify theme-specific highlight has min-height to prevent GTK warnings
        assert!(css.contains(".marco-theme-light scale.marco-scale highlight"));
        assert!(css.contains(".marco-theme-dark scale.marco-scale highlight"));
        
        // Count occurrences of min-height: 6px (should be in both theme-specific rules)
        let count = css.matches("min-height: 6px").count();
        assert!(count >= 2, "Expected at least 2 occurrences of 'min-height: 6px', found {}", count);
    }
}
