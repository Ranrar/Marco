//! CSS Constants Module
//!
//! Centralized constants for Polo's CSS styling system.
//!
//! ## Color Palettes
//!
//! `ColorPalette` structs define theme-specific colors used throughout Polo's UI:
//! - `LIGHT_PALETTE`: Colors for light mode
//! - `DARK_PALETTE`: Colors for dark mode
//!
//! ## Spacing & Sizing
//!
//! - `TITLEBAR_HEIGHT`: Standard titlebar height (32px)
//! - `BUTTON_PADDING`: Standard button padding
//! - `BORDER_RADIUS`: Standard corner radius (6px)
//!
//! ## Transitions
//!
//! - `STANDARD_TRANSITION`: Default transition timing for interactive elements

/// Color palette for a single theme (light or dark)
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette {
    /// Window background color
    pub window_bg: &'static str,
    /// Titlebar background color (matches Marco's menu.css)
    pub titlebar_bg: &'static str,
    /// Primary text/foreground color
    pub foreground: &'static str,
    /// Default border color for buttons and controls
    pub border: &'static str,
    /// Border color on hover state
    pub border_hover: &'static str,
    /// Accent color for hover text
    pub hover_accent: &'static str,
    /// Active/pressed text color
    pub active_text: &'static str,
    /// Popover/dropdown background
    pub popover_bg: &'static str,
    /// Hover background for dropdown items
    pub item_hover_bg: &'static str,
    /// Tooltip background
    pub tooltip_bg: &'static str,
    /// Tooltip text color
    pub tooltip_fg: &'static str,
    /// Tooltip border color
    pub tooltip_border: &'static str,
}

/// Light theme color palette (matches Marco's menu.css exactly)
pub const LIGHT_PALETTE: ColorPalette = ColorPalette {
    window_bg: "#ffffff",
    titlebar_bg: "#e8ecef",
    foreground: "#2c3e50",
    border: "#d0d0d0",
    border_hover: "#0066cc",
    hover_accent: "#5a6c7d",
    active_text: "#000",
    popover_bg: "#ffffff",
    item_hover_bg: "#e8e8e8",
    tooltip_bg: "#2c3e50",
    tooltip_fg: "#ffffff",
    tooltip_border: "#5a6c7d",
};

/// Dark theme color palette (matches Marco's menu.css exactly)
pub const DARK_PALETTE: ColorPalette = ColorPalette {
    window_bg: "#1a1a1a",
    titlebar_bg: "#23272e",
    foreground: "#f0f5f1",
    border: "#505050",
    border_hover: "#4f8cff",
    hover_accent: "#9198a1",
    active_text: "#fff",
    popover_bg: "#2d2d2d",
    item_hover_bg: "#3d3d3d",
    tooltip_bg: "#3d3d3d",
    tooltip_fg: "#e0e0e0",
    tooltip_border: "#505050",
};

/// Standard titlebar height in pixels
pub const TITLEBAR_HEIGHT: &str = "32px";

/// Standard button padding
pub const BUTTON_PADDING: &str = "2px 8px";

/// Mode toggle button padding (slightly different)
pub const MODE_TOGGLE_PADDING: &str = "2px 6px";

/// Standard border radius for buttons and controls
pub const BORDER_RADIUS: &str = "6px";

/// Standard transition timing for interactive elements
pub const STANDARD_TRANSITION: &str = "background 0.15s, color 0.15s, border 0.15s";

/// Title label font size (matches Marco)
pub const TITLE_FONT_SIZE: &str = "14px";

/// Title label font weight (matches Marco)
pub const TITLE_FONT_WEIGHT: &str = "600";

/// Button font size
pub const BUTTON_FONT_SIZE: &str = "12px";

/// Button font weight
pub const BUTTON_FONT_WEIGHT: &str = "500";

/// Minimum button height
pub const BUTTON_MIN_HEIGHT: &str = "20px";

/// Minimum button width (for compact buttons like mode toggle)
pub const BUTTON_MIN_WIDTH: &str = "20px";

/// Dropdown minimum width
pub const DROPDOWN_MIN_WIDTH: &str = "150px";

/// Dropdown item padding
pub const DROPDOWN_ITEM_PADDING: &str = "4px 8px";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_light_palette_colors() {
        // Verify all colors are valid hex codes
        assert!(LIGHT_PALETTE.window_bg.starts_with('#'));
        assert!(LIGHT_PALETTE.titlebar_bg.starts_with('#'));
        assert!(LIGHT_PALETTE.foreground.starts_with('#'));
        assert!(LIGHT_PALETTE.border.starts_with('#'));
        assert!(LIGHT_PALETTE.border_hover.starts_with('#'));
        
        // Verify color format (# followed by 3 or 6 hex digits)
        assert!(LIGHT_PALETTE.window_bg.len() == 7); // #ffffff
        assert!(LIGHT_PALETTE.titlebar_bg.len() == 7); // #e8ecef
    }

    #[test]
    fn smoke_test_dark_palette_colors() {
        // Verify all colors are valid hex codes
        assert!(DARK_PALETTE.window_bg.starts_with('#'));
        assert!(DARK_PALETTE.titlebar_bg.starts_with('#'));
        assert!(DARK_PALETTE.foreground.starts_with('#'));
        assert!(DARK_PALETTE.border.starts_with('#'));
        assert!(DARK_PALETTE.border_hover.starts_with('#'));
        
        // Verify color format
        assert!(DARK_PALETTE.window_bg.len() == 7);
        assert!(DARK_PALETTE.titlebar_bg.len() == 7);
    }

    #[test]
    fn smoke_test_palettes_have_different_colors() {
        // Light and dark should have different values
        assert_ne!(LIGHT_PALETTE.window_bg, DARK_PALETTE.window_bg);
        assert_ne!(LIGHT_PALETTE.titlebar_bg, DARK_PALETTE.titlebar_bg);
        assert_ne!(LIGHT_PALETTE.foreground, DARK_PALETTE.foreground);
        assert_ne!(LIGHT_PALETTE.border, DARK_PALETTE.border);
    }

    #[test]
    fn smoke_test_spacing_constants() {
        // Verify spacing constants have proper CSS format
        assert!(TITLEBAR_HEIGHT.ends_with("px"));
        assert!(BUTTON_PADDING.contains("px"));
        assert!(BORDER_RADIUS.ends_with("px"));
        
        // Verify constants have expected values
        assert_eq!(TITLEBAR_HEIGHT, "32px");
        assert_eq!(BUTTON_PADDING, "2px 8px");
        assert_eq!(BORDER_RADIUS, "6px");
    }

    #[test]
    fn smoke_test_transition_format() {
        // Verify transition has proper CSS format
        assert!(STANDARD_TRANSITION.contains("0.15s"));
        assert!(STANDARD_TRANSITION.contains("background"));
        assert!(STANDARD_TRANSITION.contains("color"));
        assert!(STANDARD_TRANSITION.contains("border"));
    }
}
