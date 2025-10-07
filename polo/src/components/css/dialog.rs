//! Dialog CSS Generation
//!
//! Generates CSS for custom dialog windows that match Polo's theme.
//!
//! ## Components Styled
//!
//! - `.polo-dialog`: Main dialog window styling
//! - `.polo-dialog-content`: Content area with proper spacing
//! - `.polo-dialog-title`: Dialog title text
//! - `.polo-dialog-message`: Dialog message/description text
//! - `.polo-dialog-button-box`: Action button container
//! - `.polo-dialog-button`: Action buttons (primary, secondary, cancel)
//!
//! ## Theme Support
//!
//! Generates rules for both `.marco-theme-light` and `.marco-theme-dark` classes.
//!
//! ## Usage
//!
//! Dialog windows should:
//! 1. Use `gtk4::Window` (not deprecated `Dialog`)
//! 2. Add CSS class: `.add_css_class("polo-dialog")`
//! 3. Add theme class: `.add_css_class("marco-theme-light")` or `"marco-theme-dark"`
//! 4. Use `.transient_for(parent)` for modal behavior
//! 5. Use `.set_modal(true)` for modal dialogs

use super::constants::*;

/// Generate complete dialog CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(2048);
    
    // Base dialog styling (theme-independent)
    css.push_str(&format!(
        r#"
    /* Base dialog window styling */
    .polo-dialog {{
        border-radius: {border_radius};
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    }}
    
    .polo-dialog-content {{
        padding: 24px;
        min-width: 400px;
    }}
    
    .polo-dialog-title {{
        font-size: 16px;
        font-weight: 600;
        margin-bottom: 12px;
    }}
    
    .polo-dialog-message {{
        font-size: 14px;
        line-height: 1.5;
        margin-bottom: 20px;
    }}
    
    .polo-dialog-button-box {{
        margin-top: 16px;
        padding: 0;
    }}
    
    .polo-dialog-button {{
        min-width: 80px;
        min-height: 32px;
        padding: {button_padding};
        border-radius: {border_radius};
        font-size: 14px;
        font-weight: 500;
        margin: 0 4px;
        transition: {transition};
    }}
    
    .polo-dialog-button:first-child {{
        margin-left: 0;
    }}
    
    .polo-dialog-button:last-child {{
        margin-right: 0;
    }}
"#,
        border_radius = BORDER_RADIUS,
        button_padding = BUTTON_PADDING,
        transition = STANDARD_TRANSITION,
    ));
    
    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    
    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    
    css
}

/// Generate theme-specific dialog CSS
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* {theme} - Dialog styles */
    .{theme} .polo-dialog {{
        background: {window_bg};
        border: 1px solid {border};
    }}
    
    .{theme} .polo-dialog-content {{
        background: {window_bg};
    }}
    
    .{theme} .polo-dialog-title {{
        color: {foreground};
    }}
    
    .{theme} .polo-dialog-message {{
        color: {foreground};
        opacity: 0.9;
    }}
    
    .{theme} .polo-dialog-button {{
        background: transparent;
        color: {foreground};
        border: 1px solid {border};
    }}
    
    .{theme} .polo-dialog-button:hover {{
        background: {item_hover_bg};
        border-color: {border_hover};
        color: {hover_accent};
    }}
    
    .{theme} .polo-dialog-button:active {{
        background: {item_hover_bg};
        color: {active_text};
    }}
    
    .{theme} .polo-dialog-button.primary {{
        background: {border_hover};
        color: #ffffff;
        border-color: {border_hover};
    }}
    
    .{theme} .polo-dialog-button.primary:hover {{
        background: {hover_accent};
        border-color: {hover_accent};
        color: #ffffff;
    }}
    
    .{theme} .polo-dialog-button.primary:active {{
        background: {active_text};
        border-color: {active_text};
        color: #ffffff;
    }}
    
    .{theme} .polo-dialog-button.destructive {{
        color: #dc3545;
        border-color: #dc3545;
    }}
    
    .{theme} .polo-dialog-button.destructive:hover {{
        background: #dc3545;
        color: #ffffff;
    }}
"#,
        theme = theme_class,
        window_bg = palette.window_bg,
        foreground = palette.foreground,
        border = palette.border,
        border_hover = palette.border_hover,
        hover_accent = palette.hover_accent,
        active_text = palette.active_text,
        item_hover_bg = palette.item_hover_bg,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_dialog_css_generation() {
        let css = generate_css();
        
        // Verify basic structure
        assert!(css.contains(".polo-dialog"));
        assert!(css.contains(".polo-dialog-content"));
        assert!(css.contains(".polo-dialog-title"));
        assert!(css.contains(".polo-dialog-message"));
        assert!(css.contains(".polo-dialog-button"));
        assert!(css.contains(".polo-dialog-button-box"));
        
        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify button variants
        assert!(css.contains(".primary"));
        assert!(css.contains(".destructive"));
        
        // Verify essential properties
        assert!(css.contains("border-radius: 6px"));
        assert!(css.contains("min-width: 400px"));
        assert!(css.contains("transition:"));
        
        // Verify not empty
        assert!(!css.is_empty());
        assert!(css.len() > 500);
    }
    
    #[test]
    fn smoke_test_dialog_has_proper_spacing() {
        let css = generate_css();
        
        // Verify spacing properties
        assert!(css.contains("padding: 24px"));
        assert!(css.contains("margin-bottom:"));
        assert!(css.contains("margin-top:"));
    }
    
    #[test]
    fn smoke_test_dialog_theme_specificity() {
        let css = generate_css();
        
        // Each theme should have background, color, and border rules
        assert!(css.matches(".marco-theme-light").count() >= 5);
        assert!(css.matches(".marco-theme-dark").count() >= 5);
    }
}
