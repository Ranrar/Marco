// Polo-specific CSS styles
// These styles define the appearance of the Polo UI components
//
//! # Polo Styles Module
//!
//! Defines CSS styling constants for Polo's GTK UI components.
//!
//! ## Main Constant: `POLO_CSS`
//!
//! A large string constant containing all Polo-specific CSS rules:
//!
//! - **Titlebar**: Height override (32px), background colors for light/dark modes
//! - **Buttons**: Flat, transparent design with hover effects
//! - **Dropdown**: Theme selector styling with proper popover colors
//! - **Window Controls**: Minimize, maximize, close button appearance
//! - **Mode Toggle**: Light/dark mode switch button with emoji styling
//!
//! ## Theme Structure
//!
//! CSS rules are organized by theme:
//!
//! ```css
//! /* Light mode */
//! .marco-theme-light .polo-titlebar { ... }
//! .marco-theme-light .polo-open-file-btn { ... }
//!
//! /* Dark mode */
//! .marco-theme-dark .polo-titlebar { ... }
//! .marco-theme-dark .polo-open-file-btn { ... }
//! ```
//!
//! ## Fallback CSS
//!
//! `FALLBACK_MENU_CSS` provides minimal styling if Marco's menu.css cannot be loaded.
//! This ensures basic functionality even in degraded environments.

/// Polo-specific CSS additions for window, titlebar, and controls
pub const POLO_CSS: &str = r#"
    /* Force HeaderBar to 32px height - override GTK defaults */
    .polo-titlebar, headerbar.polo-titlebar {
        min-height: 32px;
        padding-top: 0;
        padding-bottom: 0;
    }
    
    .polo-window {
        background: #ffffff;
    }
    
    /* Light theme styles - Match Marco's menu.css exactly */
    .marco-theme-light .polo-titlebar {
        min-height: 32px;
        background: #e8ecef;  /* Match Marco's exact color */
        border-bottom: 0px solid #ccc;
    }
    
    .marco-theme-light .polo-title-label {
        font-size: 14px;  /* Match Marco */
        font-weight: 600; /* Match Marco */
        color: #2c3e50;
    }
    
    /* Dark mode toggle button - LIGHT MODE - Match other buttons exactly */
    .marco-theme-light .polo-mode-toggle-btn {
        min-width: 20px;
        min-height: 20px;
        padding: 2px 6px;
        border: 1px solid #d0d0d0;
        border-radius: 6px;
        background: transparent;
        color: #2c3e50;
        font-size: 14px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-light .polo-mode-toggle-btn:hover {
        background: transparent;
        color: #5a6c7d;
        border-color: #0066cc;
    }
    
    .marco-theme-light .polo-mode-toggle-btn:active {
        background: transparent;
        color: #000;
        border-color: #0066cc;
    }
    
    /* Make emoji dark for light mode */
    .marco-theme-light .polo-mode-toggle-btn label {
        filter: grayscale(100%) brightness(0.3);
    }
    
    .marco-theme-light .polo-mode-toggle-btn:hover label {
        filter: grayscale(100%) brightness(0.2);
    }
    
    .marco-theme-light .polo-mode-toggle-btn:active label {
        filter: grayscale(100%) brightness(0);
    }
    
    /* Dark theme styles - Match Marco's menu.css exactly */
    .marco-theme-dark .polo-window {
        background: #1a1a1a;
    }
    
    .marco-theme-dark .polo-titlebar {
        min-height: 32px;
        background: #23272e;  /* Match Marco's exact color */
        border-bottom: 0px solid #444;
    }
    
    .marco-theme-dark .polo-title-label {
        font-size: 14px;  /* Match Marco */
        font-weight: 600; /* Match Marco */
        color: #e0e0e0;
    }
    
    /* Dark mode toggle button - DARK MODE - Match other buttons exactly */
    .marco-theme-dark .polo-mode-toggle-btn {
        min-width: 20px;
        min-height: 20px;
        padding: 2px 6px;
        border: 1px solid #505050;
        border-radius: 6px;
        background: transparent;
        color: #f0f5f1;
        font-size: 14px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-dark .polo-mode-toggle-btn:hover {
        background: transparent;
        color: #9198a1;
        border-color: #4f8cff;
    }
    
    .marco-theme-dark .polo-mode-toggle-btn:active {
        background: transparent;
        color: #fff;
        border-color: #4f8cff;
    }
    
    /* Make emoji bright for dark mode */
    .marco-theme-dark .polo-mode-toggle-btn label {
        filter: grayscale(100%) brightness(2);
    }
    
    .marco-theme-dark .polo-mode-toggle-btn:hover label {
        filter: grayscale(100%) brightness(2.5);
    }
    
    .marco-theme-dark .polo-mode-toggle-btn:active label {
        filter: grayscale(100%) brightness(3);
    }
    
    /* Dropdown styles - Match Marco's flat design */
    dropdown.polo-theme-dropdown {
        min-width: 150px;
        min-height: 20px;
        font-size: 12px;
    }
    
    /* Dropdown button - LIGHT MODE */
    .marco-theme-light dropdown.polo-theme-dropdown > button {
        background: transparent;
        color: #2c3e50;
        border: 1px solid #d0d0d0;
        border-radius: 6px;
        padding: 2px 6px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-light dropdown.polo-theme-dropdown > button:hover {
        background: transparent;
        color: #5a6c7d;
        border-color: #0066cc;
    }
    
    .marco-theme-light dropdown.polo-theme-dropdown > button:active {
        background: transparent;
        color: #000;
        border-color: #0066cc;
    }
    
    .marco-theme-light dropdown.polo-theme-dropdown > button label {
        color: inherit;
    }
    
    /* Dropdown popover - LIGHT MODE - Target the popover.background class */
    .marco-theme-light dropdown.polo-theme-dropdown > popover.background,
    .marco-theme-light dropdown.polo-theme-dropdown > popover {
        background: transparent;
        border: none;
        box-shadow: none;
    }
    
    /* Style the contents node - this is the visible part */
    .marco-theme-light dropdown.polo-theme-dropdown > popover > contents {
        background: #ffffff;
        color: #2c3e50;
        border: none;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
        border-radius: 6px;
    }
    
    /* Target the listview inside the popover */
    .marco-theme-light dropdown.polo-theme-dropdown > popover listview {
        background: #ffffff;
        color: #2c3e50;
        border: none;
        border-radius: 6px;
    }
    
    /* Target individual rows (list items) */
    .marco-theme-light dropdown.polo-theme-dropdown > popover listview > row {
        background: transparent;
        color: #2c3e50;
        border: none;
        padding: 4px 8px;
    }
    
    .marco-theme-light dropdown.polo-theme-dropdown > popover listview > row:hover {
        background: #e8e8e8;
    }
    
    /* Target labels inside rows */
    .marco-theme-light dropdown.polo-theme-dropdown > popover listview > row label {
        color: #2c3e50;
    }
    
    /* Tooltip - LIGHT MODE */
    .marco-theme-light tooltip {
        background: #2c3e50;
        color: #ffffff;
        border: 1px solid #5a6c7d;
    }
    
    .marco-theme-light tooltip > contents {
        background: #2c3e50;
        color: #ffffff;
    }
    
    /* Dropdown button - DARK MODE */
    .marco-theme-dark dropdown.polo-theme-dropdown > button {
        background: transparent;
        color: #f0f5f1;
        border: 1px solid #505050;
        border-radius: 6px;
        padding: 2px 6px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-dark dropdown.polo-theme-dropdown > button:hover {
        background: transparent;
        color: #9198a1;
        border-color: #4f8cff;
    }
    
    .marco-theme-dark dropdown.polo-theme-dropdown > button:active {
        background: transparent;
        color: #fff;
        border-color: #4f8cff;
    }
    
    .marco-theme-dark dropdown.polo-theme-dropdown > button label {
        color: inherit;
    }
    
    /* Dropdown popover - DARK MODE - Target the popover.background class */
    .marco-theme-dark dropdown.polo-theme-dropdown > popover.background,
    .marco-theme-dark dropdown.polo-theme-dropdown > popover {
        background: transparent;
        border: none;
        box-shadow: none;
    }
    
    /* Style the contents node - this is the visible part */
    .marco-theme-dark dropdown.polo-theme-dropdown > popover > contents {
        background: #2d2d2d;
        color: #e0e0e0;
        border: none;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        border-radius: 6px;
    }
    
    /* Target the listview inside the popover */
    .marco-theme-dark dropdown.polo-theme-dropdown > popover listview {
        background: #2d2d2d;
        color: #e0e0e0;
        border: none;
        border-radius: 6px;
    }
    
    /* Target individual rows (list items) */
    .marco-theme-dark dropdown.polo-theme-dropdown > popover listview > row {
        background: transparent;
        color: #e0e0e0;
        border: none;
        padding: 4px 8px;
    }
    
    .marco-theme-dark dropdown.polo-theme-dropdown > popover listview > row:hover {
        background: #3d3d3d;
    }
    
    /* Target labels inside rows */
    .marco-theme-dark dropdown.polo-theme-dropdown > popover listview > row label {
        color: #e0e0e0;
    }
    
    /* Tooltip - DARK MODE */
    .marco-theme-dark tooltip {
        background: #3d3d3d;
        color: #e0e0e0;
        border: 1px solid #505050;
    }
    
    .marco-theme-dark tooltip > contents {
        background: #3d3d3d;
        color: #e0e0e0;
    }
    
    /* Action buttons - Match Marco's flat, minimal style */
    
    /* Open File button - LIGHT MODE */
    .marco-theme-light .polo-open-file-btn {
        background: transparent;
        color: #2c3e50;
        border: 1px solid #d0d0d0;
        border-radius: 6px;
        padding: 2px 8px;
        min-height: 20px;
        font-weight: 500;
        font-size: 12px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-light .polo-open-file-btn:hover {
        background: transparent;
        color: #5a6c7d;
        border-color: #0066cc;
    }
    
    .marco-theme-light .polo-open-file-btn:active {
        background: transparent;
        color: #000;
        border-color: #0066cc;
    }
    
    /* Open File button - DARK MODE */
    .marco-theme-dark .polo-open-file-btn {
        background: transparent;
        color: #f0f5f1;
        border: 1px solid #505050;
        border-radius: 6px;
        padding: 2px 8px;
        min-height: 20px;
        font-weight: 500;
        font-size: 12px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-dark .polo-open-file-btn:hover {
        background: transparent;
        color: #9198a1;
        border-color: #4f8cff;
    }
    
    .marco-theme-dark .polo-open-file-btn:active {
        background: transparent;
        color: #fff;
        border-color: #4f8cff;
    }
    
    /* Open Editor button - LIGHT MODE */
    .marco-theme-light .polo-open-editor-btn {
        background: transparent;
        color: #2c3e50;
        border: 1px solid #d0d0d0;
        border-radius: 6px;
        padding: 2px 8px;
        min-height: 20px;
        font-weight: 500;
        font-size: 12px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-light .polo-open-editor-btn:hover {
        background: transparent;
        color: #5a6c7d;
        border-color: #0066cc;
    }
    
    .marco-theme-light .polo-open-editor-btn:active {
        background: transparent;
        color: #000;
        border-color: #0066cc;
    }
    
    /* Open Editor button - DARK MODE */
    .marco-theme-dark .polo-open-editor-btn {
        background: transparent;
        color: #f0f5f1;
        border: 1px solid #505050;
        border-radius: 6px;
        padding: 2px 8px;
        min-height: 20px;
        font-weight: 500;
        font-size: 12px;
        transition: background 0.15s, color 0.15s, border 0.15s;
    }
    
    .marco-theme-dark .polo-open-editor-btn:hover {
        background: transparent;
        color: #9198a1;
        border-color: #4f8cff;
    }
    
    .marco-theme-dark .polo-open-editor-btn:active {
        background: transparent;
        color: #fff;
        border-color: #4f8cff;
    }
"#;

/// Minimal fallback CSS if menu.css cannot be loaded
/// Note: This should rarely be used - menu.css is the canonical source
pub const FALLBACK_MENU_CSS: &str = r#"
    /* Critical styles for basic functionality */
    .icon-font {
        font-family: 'icomoon';
        font-size: 16px;
    }
    .window-control-btn {
        background: transparent;
        border: none;
    }
    .titlebar {
        min-height: 32px;
    }
"#;


