// Empty state (welcome screen) for when no file is opened
//
//! # Empty State Module
//!
//! Displays a theme-aware welcome screen when Polo starts without a file.
//!
//! ## Features
//!
//! - **Theme Awareness**: Matches current light/dark theme mode
//! - **Centered Layout**: Clean, minimalist design
//! - **Helpful Message**: Guides user to open a file
//!
//! ## Implementation
//!
//! Uses inline HTML with embedded CSS that responds to `.theme-light` and
//! `.theme-dark` classes on the `<html>` element, ensuring visual consistency
//! with markdown rendering.

use crate::components::utils::get_theme_mode;
use marco_core::logic::swanson::SettingsManager;
use std::sync::Arc;
use webkit6::prelude::WebViewExt;

/// Show empty state when no file is opened - theme-aware version matching markdown rendering
pub fn show_empty_state_with_theme(
    webview: &webkit6::WebView,
    settings_manager: &Arc<SettingsManager>,
) {
    // Determine theme_mode from settings (same logic as markdown rendering)
    let theme_mode = get_theme_mode(settings_manager);
    
    // Create theme class for HTML element (theme-light or theme-dark)
    let theme_class = format!("theme-{}", theme_mode);
    log::debug!("Empty state using theme class: {}", theme_class);
    
    let html = format!(
        r#"<!DOCTYPE html>
<html class="{}">
<head>
    <meta charset="UTF-8">
    <style>
        /* Light theme (default) */
        .theme-light body {{
            font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background: #ffffff;
            color: #2c3e50;
        }}
        .theme-light .empty-state {{
            text-align: center;
            padding: 2rem;
            opacity: 0.7;
        }}
        .theme-light .empty-state h1 {{
            font-size: 3rem;
            margin: 0 0 1rem 0;
        }}
        .theme-light .empty-state p {{
            font-size: 1.2rem;
            margin: 0.5rem 0;
            color: #5a6c7d;
        }}
        
        /* Dark theme */
        .theme-dark body {{
            font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background: #1e1e1e;
            color: #e0e0e0;
        }}
        .theme-dark .empty-state {{
            text-align: center;
            padding: 2rem;
            opacity: 0.7;
        }}
        .theme-dark .empty-state h1 {{
            font-size: 3rem;
            margin: 0 0 1rem 0;
        }}
        .theme-dark .empty-state p {{
            font-size: 1.2rem;
            margin: 0.5rem 0;
            color: #9198a1;
        }}
    </style>
</head>
<body>
    <div class="empty-state">
        <h1>📄</h1>
        <p>Welcome to Polo</p>
        <p>Open a markdown file to get started</p>
    </div>
</body>
</html>"#,
        theme_class
    );
    
    let webview_clone = webview.clone();
    let html_string = html.to_string();
    gtk4::glib::idle_add_local_once(move || {
        webview_clone.load_html(&html_string, None);
    });
}
