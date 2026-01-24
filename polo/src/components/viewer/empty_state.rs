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

use core::logic::swanson::SettingsManager;
use servo_runner::WebView;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static EMPTY_STATE_SEQ: AtomicU64 = AtomicU64::new(0);

/// Show empty state when no file is opened - theme-aware version matching markdown rendering
pub fn show_empty_state_with_theme(webview: &WebView, settings_manager: &Arc<SettingsManager>) {
    // Determine theme_mode from settings (same logic as markdown rendering)
    let theme_mode = crate::components::utils::get_theme_mode(settings_manager);

    show_empty_state_with_theme_mode(webview, &theme_mode);
}

/// Show empty state using an explicit theme mode.
///
/// This is intentionally decoupled from settings reads so callers that just toggled
/// theme can render a consistent empty state even if settings persistence is delayed.
pub fn show_empty_state_with_theme_mode(webview: &WebView, theme_mode: &str) {
    // Normalize inputs like "marco-dark" → "dark" (defensive; callers should pass "light"/"dark")
    let theme_mode = if theme_mode.contains("dark") {
        "dark"
    } else {
        "light"
    };

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

    // Servo loads via file:// URL. Use a unique temp filename per call to avoid
    // caching/stale loads when we re-render the empty state (e.g. during theme toggles).
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let seq = EMPTY_STATE_SEQ.fetch_add(1, Ordering::Relaxed);
    let temp_file = temp_dir.join(format!("polo_empty_{}_{}.html", std::process::id(), seq));

    if let Ok(mut file) = std::fs::File::create(&temp_file) {
        if file.write_all(html.as_bytes()).is_ok() {
            let temp_url = if cfg!(windows) {
                let path_str = temp_file.display().to_string().replace('\\', "/");
                format!("file:///{}", path_str)
            } else {
                format!("file://{}", temp_file.display())
            };
            webview.load_url(&temp_url);
        }
    }
}
