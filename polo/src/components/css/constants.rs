//! CSS Constants Module
//!
//! Centralized constants for Polo's CSS styling system.
//! Aligned with Marco's compact sizing standards for visual consistency.
//!
//! ## Color Palettes
//!
//! `ColorPalette` structs define theme-specific colors used throughout Polo's UI:
//! - `LIGHT_PALETTE`: Colors for light mode (matches Marco exactly)
//! - `DARK_PALETTE`: Colors for dark mode (matches Marco exactly)
//!
//! ## Spacing & Sizing
//!
//! ### UI Elements
//! - `TITLEBAR_HEIGHT`: Standard titlebar height (32px)
//! - `BUTTON_PADDING`: Standard button padding (2px 8px)
//! - `BORDER_RADIUS`: Standard corner radius (6px)
//!
//! ### Dialog Elements (Aligned with Marco's Compact Sizing)
//! - `DIALOG_BUTTON_MIN_HEIGHT`: Dialog button height (24px - matches Marco)
//! - `DIALOG_BUTTON_PADDING`: Dialog button padding (2px 8px)
//! - `DIALOG_CONTENT_PADDING`: Dialog content padding (20px all sides)
//! - `DIALOG_MIN_CONTENT_WIDTH`: Dialog minimum width (340px - compact)
//!
//! ## Transitions
//!
//! - `STANDARD_TRANSITION`: Default transition timing for interactive elements
//!
//! ## Design Philosophy
//!
//! Polo follows Marco's compact, minimal design:
//! - Small button sizes (24px height) for efficient space usage
//! - Consistent padding across all UI elements
//! - Smooth transitions for polished feel
//! - Visual consistency with Marco editor

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
    /// Disabled button background (matches Marco)
    pub disabled_bg: &'static str,
    /// Disabled button text color (matches Marco)
    pub disabled_fg: &'static str,
    /// Disabled button border color (matches Marco)
    pub disabled_border: &'static str,
}

/// Light theme color palette (matches Marco exactly)
pub const LIGHT_PALETTE: ColorPalette = ColorPalette {
    window_bg: "#ffffff",
    titlebar_bg: "#e8ecef",           // Marco's titlebar_bg
    foreground: "#2c3e50",            // Marco's titlebar_foreground
    border: "#ccc",                   // Marco's titlebar_border (was #d0d0d0)
    border_hover: "#0066cc",          // Marco's menu_active
    hover_accent: "#5a6c7d",          // Marco's menu_hover / window_control_hover
    active_text: "#000",              // Marco's menu_active / window_control_active
    popover_bg: "#f5f5f5",            // Marco's toolbar_popover_bg (was #ffffff)
    item_hover_bg: "#e8e8e8",         // Slightly darker for hover
    tooltip_bg: "#2c3e50",            // Matches foreground for contrast
    tooltip_fg: "#ffffff",            // White text on dark tooltip
    tooltip_border: "#5a6c7d",        // Subtle border
    disabled_bg: "#ddd",              // Marco's disabled button background
    disabled_fg: "#999",              // Marco's disabled button text
    disabled_border: "#ccc",          // Marco's disabled button border
};

/// Dark theme color palette (matches Marco exactly)
pub const DARK_PALETTE: ColorPalette = ColorPalette {
    window_bg: "#252526",             // Marco's toolbar_bg (was #1a1a1a)
    titlebar_bg: "#23272e",           // Marco's titlebar_bg
    foreground: "#f0f5f1",            // Marco's window_control (was #f0f5f1, correct)
    border: "#444",                   // Marco's titlebar_border (was #505050)
    border_hover: "#4f8cff",          // Marco's toolbar_button_hover_border
    hover_accent: "#9198a1",          // Marco's window_control_hover
    active_text: "#fff",              // Marco's window_control_active
    popover_bg: "#23272e",            // Marco's toolbar_popover_bg (was #2d2d2d)
    item_hover_bg: "#3d3d3d",         // Slightly lighter for hover visibility
    tooltip_bg: "#3d3d3d",            // Dark tooltip background
    tooltip_fg: "#e0e0e0",            // Marco's title_label dark
    tooltip_border: "#444",           // Marco's titlebar_border (was #505050)
    disabled_bg: "#555",              // Marco's disabled button background
    disabled_fg: "#aaa",              // Marco's disabled button text
    disabled_border: "#555",          // Marco's disabled button border
};

/// Standard titlebar height in pixels
pub const TITLEBAR_HEIGHT: &str = "32px";

/// Standard button padding (matches Marco: 2px 8px)
pub const BUTTON_PADDING: &str = "2px 8px";

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

/// Minimum button height (matches Marco's standard 24px)
pub const BUTTON_MIN_HEIGHT: &str = "24px";

/// Minimum button width (for compact buttons like mode toggle)
pub const BUTTON_MIN_WIDTH: &str = "20px";

/// Dropdown minimum width
pub const DROPDOWN_MIN_WIDTH: &str = "150px";

/// Dropdown item padding
pub const DROPDOWN_ITEM_PADDING: &str = "4px 8px";

// ============================================================================
// Dialog & Button Size Constants (Matches Marco's Compact Sizing)
// ============================================================================

/// Dialog button minimum height (matches Marco's compact sizing)
pub const DIALOG_BUTTON_MIN_HEIGHT: &str = "24px";

/// Dialog button padding
pub const DIALOG_BUTTON_PADDING: &str = "2px 8px";

/// Dialog button minimum width
pub const DIALOG_BUTTON_MIN_WIDTH: &str = "80px";

/// Dialog content padding (all sides)
pub const DIALOG_CONTENT_PADDING: &str = "20px";

/// Dialog minimum content width (reduced from 400px)
pub const DIALOG_MIN_CONTENT_WIDTH: &str = "340px";

/// Dialog title font size
pub const DIALOG_TITLE_FONT_SIZE: &str = "15px";

/// Dialog title font weight
pub const DIALOG_TITLE_FONT_WEIGHT: &str = "600";

/// Dialog message font size
pub const DIALOG_MESSAGE_FONT_SIZE: &str = "13px";

/// Dialog button font size
pub const DIALOG_BUTTON_FONT_SIZE: &str = "12px";

/// Dialog button font weight
pub const DIALOG_BUTTON_FONT_WEIGHT: &str = "500";

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
