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
//! - **Open File button**: The primary way to open a file from this screen
//!   (the toolbar's old "Open file" button was removed — see `toolbar.rs`).
//!   Since this is plain HTML rendered inside the webview, a native GTK
//!   button can't be embedded here; instead the button posts a
//!   `polo_open_file:` IPC message (`window.ipc.postMessage`) which
//!   `main.rs` wires to the same file-picker dialog every other "Open" entry
//!   point uses.
//!
//! ## Implementation
//!
//! Uses inline HTML with embedded CSS that responds to `.theme-light` and
//! `.theme-dark` classes on the `<html>` element, ensuring visual consistency
//! with markdown rendering.

use crate::components::toolbar::SVG_OPEN_FILE;
use crate::components::utils::get_theme_mode;
use crate::components::viewer::drop_overlay;
use crate::components::viewer::platform_webview::PlatformWebView;
use marco_shared::logic::swanson::SettingsManager;
use std::sync::Arc;

/// Default-state icon (Tabler `icon-tabler-markdown`), replacing the earlier
/// 📄 emoji glyph so it renders the same way the `viewer::drop_overlay` icon
/// does — an inline `stroke="currentColor"` SVG that inherits the page's own
/// text color — instead of an emoji glyph whose color comes from the
/// platform's emoji font and can't be made to match at all.
const SVG_MARKDOWN_FILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none" /><path d="M3 7a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-10" /><path d="M7 15v-6l2 2l2 -2v6" /><path d="M14 13l2 2l2 -2m-2 2v-6" /></svg>"#;

/// Show empty state when no file is opened - theme-aware version matching markdown rendering
pub fn show_empty_state_with_theme(
    webview: &PlatformWebView,
    settings_manager: &Arc<SettingsManager>,
) {
    // Determine theme_mode from settings (same logic as markdown rendering)
    let theme_mode = get_theme_mode(settings_manager);

    // Create theme class for HTML element (theme-light or theme-dark)
    let theme_class = format!("theme-{}", theme_mode);
    log::debug!("Empty state using theme class: {}", theme_class);

    let html = format!(
        r#"<!DOCTYPE html>
<html class="{theme}">
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
        .theme-light .empty-state p {{
            font-size: 1.2rem;
            margin: 0.5rem 0;
            color: #5a6c7d;
        }}
        .theme-light .empty-state-open-btn {{
            color: #2c3e50;
            border-color: #ccc;
        }}
        .theme-light .empty-state-open-btn:hover {{
            color: #0066cc;
            border-color: #0066cc;
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
        .theme-dark .empty-state p {{
            font-size: 1.2rem;
            margin: 0.5rem 0;
            color: #9198a1;
        }}
        .theme-dark .empty-state-open-btn {{
            color: #e0e0e0;
            border-color: #444;
        }}
        .theme-dark .empty-state-open-btn:hover {{
            color: #4f8cff;
            border-color: #4f8cff;
        }}

        /* Theme-independent button layout */
        .empty-state-open-btn {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            margin-top: 1.25rem;
            padding: 0.5rem 1.25rem;
            font: inherit;
            font-size: 1rem;
            background: transparent;
            border: 1px solid;
            border-radius: 6px;
            cursor: pointer;
            opacity: 1;
            transition: color 0.15s, border-color 0.15s, opacity 0.15s ease;
        }}
        .empty-state-open-btn svg {{
            width: 20px;
            height: 20px;
            stroke: currentColor;
            flex-shrink: 0;
        }}
        .empty-state-icon {{
            margin: 0 0 1rem 0;
        }}
        .empty-state-icon svg {{
            width: 3rem;
            height: 3rem;
            stroke: currentColor;
        }}

        /* The "Open File" button fades out while a file drag hovers the
           webview -- it can't usefully be clicked mid-drag, and leaving it
           up made the drop overlay below look like it was showing two
           competing calls to action at once. The overlay itself (icon +
           "Drop file to open" text) is `viewer::drop_overlay` -- the same
           shared markup/CSS a loaded document's page also carries, so
           dragging a file looks identical whether this welcome screen or a
           rendered document is showing. */
        html.dragging .empty-state-open-btn {{
            opacity: 0;
            pointer-events: none;
        }}
        {drop_overlay_css}
    </style>
</head>
<body>
    <div class="empty-state">
        <div class="empty-state-icon">{file_icon}</div>
        <p>Welcome to Polo</p>
        <p>Drag in or open a markdown file to get started</p>
        <button type="button" id="polo-empty-open-btn" class="empty-state-open-btn">
            {open_icon}
            <span>Open File</span>
        </button>
    </div>
    {drop_overlay_html}
    <script>
        (function() {{
            var btn = document.getElementById('polo-empty-open-btn');
            if (!btn) return;
            btn.addEventListener('click', function() {{
                try {{
                    if (window.ipc && typeof window.ipc.postMessage === 'function') {{
                        window.ipc.postMessage('polo_open_file:');
                    }}
                }} catch (e) {{}}
            }});
        }})();
    </script>
</body>
</html>"#,
        theme = theme_class,
        open_icon = SVG_OPEN_FILE,
        file_icon = SVG_MARKDOWN_FILE,
        drop_overlay_css = drop_overlay::CSS,
        drop_overlay_html = drop_overlay::html(),
    );

    webview.load_html_with_base(&html, None);
}
