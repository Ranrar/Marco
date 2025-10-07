//! CSS Constants Module
//!
//! Centralized constants for Marco's CSS styling system.
//! All colors and values extracted from menu.css, toolbar.css, and footer.css.
//!
//! ## Color Palettes
//!
//! `ColorPalette` structs define theme-specific colors used throughout Marco's UI:
//! - `LIGHT_PALETTE`: Colors for light mode (extracted from .marco-theme-light rules)
//! - `DARK_PALETTE`: Colors for dark mode (extracted from .marco-theme-dark rules)
//!
//! ## Spacing & Sizing
//!
//! - `TITLEBAR_HEIGHT`: Standard titlebar height (32px)
//! - `TOOLBAR_BUTTON_PADDING`: Toolbar button padding (2px 6px)
//! - `MENU_ITEM_PADDING`: Menu item padding (0 12px)
//! - `FOOTER_PADDING`: Footer padding (2px 8px)
//! - `BORDER_RADIUS`: Standard corner radius (6px for toolbar, 4px for menu)
//! - `FOOTER_MIN_HEIGHT`: Minimum footer height (26px)
//!
//! ## Transitions
//!
//! - `STANDARD_TRANSITION`: Default transition timing for interactive elements
//! - `ICON_TRANSITION`: Transition for icon font elements
//!
//! ## Fonts
//!
//! - `UI_FONT_FAMILY`: Standard UI font family
//! - `ICON_FONT_FAMILY`: Icon font family (icomoon)

/// Color palette for a single theme (light or dark)
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette {
    // Titlebar/Menu colors
    /// Titlebar/menubar background color
    pub titlebar_bg: &'static str,
    /// Titlebar/menubar border color (bottom border)
    pub titlebar_border: &'static str,
    /// Primary text/foreground color for titlebar and menu items
    pub titlebar_foreground: &'static str,
    /// Menu item hover text color
    pub menu_hover: &'static str,
    /// Menu item active/pressed text color
    pub menu_active: &'static str,
    /// Menu item disabled text color
    pub menu_disabled: &'static str,
    /// Title label text color (document name in titlebar)
    pub title_label: &'static str,
    /// Window control button icon color (default state)
    pub window_control: &'static str,
    /// Window control button icon color (hover state)
    pub window_control_hover: &'static str,
    /// Window control button icon color (active/pressed state)
    pub window_control_active: &'static str,
    /// Layout state icon color (default)
    pub layout_icon: &'static str,
    /// Layout state icon color (hover)
    pub layout_icon_hover: &'static str,
    /// Layout state icon color (active)
    pub layout_icon_active: &'static str,
    
    // Toolbar colors
    /// Toolbar background color
    pub toolbar_bg: &'static str,
    /// Toolbar border color (bottom border)
    pub toolbar_border: &'static str,
    /// Toolbar button text/icon color (default state)
    pub toolbar_button: &'static str,
    /// Toolbar button hover text color
    pub toolbar_button_hover: &'static str,
    /// Toolbar button hover border color
    pub toolbar_button_hover_border: &'static str,
    /// Toolbar button active text color
    pub toolbar_button_active: &'static str,
    /// Toolbar button disabled background
    pub toolbar_button_disabled_bg: &'static str,
    /// Toolbar button disabled text color
    pub toolbar_button_disabled: &'static str,
    /// Toolbar button disabled border color
    pub toolbar_button_disabled_border: &'static str,
    /// Toolbar separator color
    pub toolbar_separator: &'static str,
    /// Toolbar popover background
    pub toolbar_popover_bg: &'static str,
    /// Toolbar popover border
    pub toolbar_popover_border: &'static str,
    
    // Footer colors
    /// Footer background color (matches toolbar)
    pub footer_bg: &'static str,
    /// Footer border color (top border)
    pub footer_border: &'static str,
    /// Footer text color
    pub footer_text: &'static str,
}

/// Light theme color palette
/// Extracted from .marco-theme-light rules in menu.css, toolbar.css, and footer.css
pub const LIGHT_PALETTE: ColorPalette = ColorPalette {
    // Titlebar/Menu (from menu.css)
    titlebar_bg: "#e8ecef",
    titlebar_border: "#ccc",
    titlebar_foreground: "#2c3e50",
    menu_hover: "#000000",
    menu_active: "#0066cc",
    menu_disabled: "#999",
    title_label: "#2c3e50",
    window_control: "#2c3e50",
    window_control_hover: "#5a6c7d",
    window_control_active: "#000",
    layout_icon: "#2c3e50",
    layout_icon_hover: "#5a6c7d",
    layout_icon_active: "#000",
    
    // Toolbar (from toolbar.css)
    toolbar_bg: "#f5f5f5",
    toolbar_border: "#ddd",
    toolbar_button: "#2c3e50",
    toolbar_button_hover: "#5a6c7d",
    toolbar_button_hover_border: "#0066cc",
    toolbar_button_active: "#000",
    toolbar_button_disabled_bg: "#ddd",
    toolbar_button_disabled: "#999",
    toolbar_button_disabled_border: "#ccc",
    toolbar_separator: "#ccc",
    toolbar_popover_bg: "#f5f5f5",
    toolbar_popover_border: "#ccc",
    
    // Footer (from footer.css)
    footer_bg: "#f5f5f5",
    footer_border: "#ddd",
    footer_text: "#2c3e50",
};

/// Dark theme color palette
/// Extracted from .marco-theme-dark rules in menu.css, toolbar.css, and footer.css
pub const DARK_PALETTE: ColorPalette = ColorPalette {
    // Titlebar/Menu (from menu.css)
    titlebar_bg: "#23272e",
    titlebar_border: "#444",
    titlebar_foreground: "#e0e0e0",
    menu_hover: "#ffffff",
    menu_active: "#ffd700",
    menu_disabled: "#888",
    title_label: "#e0e0e0",
    window_control: "#f0f5f1",
    window_control_hover: "#9198a1",
    window_control_active: "#fff",
    layout_icon: "#f0f5f1",
    layout_icon_hover: "#9198a1",
    layout_icon_active: "#fff",
    
    // Toolbar (from toolbar.css)
    toolbar_bg: "#252526",
    toolbar_border: "#3c3c3c",
    toolbar_button: "#f0f5f1",
    toolbar_button_hover: "#9198a1",
    toolbar_button_hover_border: "#4f8cff",
    toolbar_button_active: "#fff",
    toolbar_button_disabled_bg: "#555",
    toolbar_button_disabled: "#aaa",
    toolbar_button_disabled_border: "#555",
    toolbar_separator: "#444",
    toolbar_popover_bg: "#23272e",
    toolbar_popover_border: "#444",
    
    // Footer (from footer.css)
    footer_bg: "#252526",
    footer_border: "#3c3c3c",
    footer_text: "#cccccc",
};

// ============================================================================
// Spacing & Sizing Constants
// ============================================================================

/// Standard titlebar/menubar height in pixels
pub const TITLEBAR_HEIGHT: &str = "32px";

/// Toolbar button padding (2px vertical, 6px horizontal)
pub const TOOLBAR_BUTTON_PADDING: &str = "2px 6px";

/// Menu item padding (0 vertical, 12px horizontal)
pub const MENU_ITEM_PADDING: &str = "0 12px";

/// Footer padding (2px vertical, 8px horizontal)
pub const FOOTER_PADDING: &str = "2px 8px";

/// Footer label padding (2px vertical, 4px horizontal)
pub const FOOTER_LABEL_PADDING: &str = "2px 4px";

/// Toolbar border radius for buttons and controls
pub const TOOLBAR_BORDER_RADIUS: &str = "6px";

/// Menu border radius for menu items
pub const MENU_BORDER_RADIUS: &str = "4px";

/// Toolbar button margins
pub const TOOLBAR_BUTTON_MARGIN: &str = "2px";

/// Minimum footer height
pub const FOOTER_MIN_HEIGHT: &str = "26px";

/// Toolbar padding
pub const TOOLBAR_PADDING: &str = "2px 5px";

/// Toolbar popover padding
pub const TOOLBAR_POPOVER_PADDING: &str = "6px";

/// Toolbar separator width
pub const TOOLBAR_SEPARATOR_WIDTH: &str = "2px";

/// Toolbar separator margin
pub const TOOLBAR_SEPARATOR_MARGIN: &str = "0 4px";

/// Window control button padding
pub const WINDOW_CONTROL_PADDING: &str = "0 2px";

/// Icon font padding
pub const ICON_FONT_PADDING: &str = "0 2px";

/// Layout state padding
pub const LAYOUT_STATE_PADDING: &str = "0px";

/// Top right button margins
pub const TOPRIGHT_BTN_MARGIN: &str = "0px";

/// Title label padding
pub const TITLE_LABEL_PADDING: &str = "0 0px";

/// Title label margin
pub const TITLE_LABEL_MARGIN: &str = "0 0px";

// ============================================================================
// Font Constants
// ============================================================================

/// Standard UI font family
pub const UI_FONT_FAMILY: &str = "'Segoe UI', 'Roboto', 'Arial', sans-serif";

/// Cantarell UI font family (alternative)
pub const UI_FONT_FAMILY_ALT: &str = r#""Segoe UI", "Cantarell", "Arial", sans-serif"#;

/// Icon font family (icomoon)
pub const ICON_FONT_FAMILY: &str = "icomoon";

/// Icon font size
pub const ICON_FONT_SIZE: &str = "16px";

/// Layout state icon font size
pub const LAYOUT_ICON_SIZE: &str = "24px";

/// Menu/titlebar font size
pub const MENU_FONT_SIZE: &str = "12px";

/// Toolbar button font size
pub const TOOLBAR_BUTTON_FONT_SIZE: &str = "12px";

/// Footer font size
pub const FOOTER_FONT_SIZE: &str = "12px";

/// Title label font size
pub const TITLE_LABEL_FONT_SIZE: &str = "14px";

/// Title label font weight
pub const TITLE_LABEL_FONT_WEIGHT: &str = "600";

/// Menu item font weight
pub const MENU_ITEM_FONT_WEIGHT: &str = "500";

/// Footer label font weight
pub const FOOTER_LABEL_FONT_WEIGHT: &str = "400";

// ============================================================================
// Transition Constants
// ============================================================================

/// Standard transition timing for interactive elements (background, color, border)
pub const STANDARD_TRANSITION: &str = "background 0.15s, color 0.15s, border 0.15s";

/// Icon transition timing (color and transform)
pub const ICON_TRANSITION: &str = "color 0.12s, transform 0.08s";

// ============================================================================
// Dimension Constants
// ============================================================================

/// Minimum width for toolbar buttons
pub const TOOLBAR_BUTTON_MIN_WIDTH: &str = "16px";

/// Minimum height for toolbar buttons
pub const TOOLBAR_BUTTON_MIN_HEIGHT: &str = "16px";

/// Toolbar icon min dimensions
pub const TOOLBAR_ICON_SIZE: &str = "16px";

/// Toolbar icon margin (right side)
pub const TOOLBAR_ICON_MARGIN: &str = "4px";

// ============================================================================
// Border Constants
// ============================================================================

/// Toolbar border width (bottom border)
pub const TOOLBAR_BORDER_WIDTH: &str = "1px solid";

/// Footer border width (top border)
pub const FOOTER_BORDER_WIDTH: &str = "1px solid";

/// Titlebar border width (bottom border) - currently 0px in original CSS
pub const TITLEBAR_BORDER_WIDTH: &str = "0px solid";

/// Toolbar popover border width
pub const TOOLBAR_POPOVER_BORDER_WIDTH: &str = "1px solid";

/// Toolbar button border width (for disabled state)
pub const TOOLBAR_BUTTON_BORDER_WIDTH: &str = "0px solid";

/// Window control button border radius
pub const WINDOW_CONTROL_BORDER_RADIUS: &str = "0px";

// ============================================================================
// Opacity Constants
// ============================================================================

/// Disabled element opacity
pub const DISABLED_OPACITY: &str = "0.6";

/// Normal element opacity
pub const NORMAL_OPACITY: &str = "1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_color_palettes() {
        // Verify light palette has all colors defined
        assert!(!LIGHT_PALETTE.titlebar_bg.is_empty());
        assert!(!LIGHT_PALETTE.toolbar_bg.is_empty());
        assert!(!LIGHT_PALETTE.footer_bg.is_empty());
        
        // Verify dark palette has all colors defined
        assert!(!DARK_PALETTE.titlebar_bg.is_empty());
        assert!(!DARK_PALETTE.toolbar_bg.is_empty());
        assert!(!DARK_PALETTE.footer_bg.is_empty());
        
        // Verify palettes are different
        assert_ne!(LIGHT_PALETTE.titlebar_bg, DARK_PALETTE.titlebar_bg);
        assert_ne!(LIGHT_PALETTE.toolbar_bg, DARK_PALETTE.toolbar_bg);
    }

    #[test]
    fn smoke_test_constants() {
        // Verify sizing constants
        assert_eq!(TITLEBAR_HEIGHT, "32px");
        assert_eq!(FOOTER_MIN_HEIGHT, "26px");
        assert_eq!(TOOLBAR_BORDER_RADIUS, "6px");
        assert_eq!(MENU_BORDER_RADIUS, "4px");
        
        // Verify font constants
        assert_eq!(ICON_FONT_FAMILY, "icomoon");
        assert_eq!(ICON_FONT_SIZE, "16px");
        
        // Verify transition constants
        assert!(STANDARD_TRANSITION.contains("0.15s"));
        assert!(ICON_TRANSITION.contains("0.12s"));
    }
}
