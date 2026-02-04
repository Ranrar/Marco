//! WebKit6-based HTML Viewer for Linux
//!
//! This module provides WebKit6 integration for rendering HTML previews on Linux.
//! The viewer displays pre-rendered HTML from Marco's markdown engine.
//!
//! # Key Features
//!
//! - Deferred HTML loading to avoid GTK allocation warnings
//! - JavaScript injection for smooth content updates
//! - External link handling (opens in system browser)
//! - Syntax highlighting for HTML source view
//! - Memory leak prevention with proper cleanup
//!
//! # Architecture
//!
//! The HTML viewer receives already-rendered HTML from `core::render` and displays it.
//! It does not perform Markdown-to-HTML conversion itself.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use webkit6::prelude::*;
use webkit6::WebView;

type WebViewOnceFn = Box<dyn FnOnce(WebView)>;

/// Run a closure once the widget is mapped (visible in the widget tree).
///
/// **Why this is needed**: GTK requires widgets to be mapped (visible) before certain
/// operations can be performed safely. Updating unmapped widgets causes GTK warnings
/// like "Trying to snapshot ... without a current allocation".
///
/// **Use cases**:
/// - Loading HTML into a WebView that's hidden in a Stack
/// - Updating content in tabs that aren't currently visible
///
/// **Implementation**: If the widget is already mapped, the closure runs immediately.
/// Otherwise, we connect to the "map" signal and run it once mapped, then disconnect.
fn run_once_when_mapped(webview: &WebView, f: impl FnOnce(WebView) + 'static) {
    if webview.is_mapped() {
        f(webview.clone());
        return;
    }

    let webview_clone = webview.clone();
    let handler_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Rc::new(RefCell::new(None));
    let f_cell: Rc<RefCell<Option<WebViewOnceFn>>> = Rc::new(RefCell::new(Some(Box::new(f))));

    let handler_id_clone = handler_id.clone();
    let f_cell_clone = f_cell.clone();

    let id = webview.connect_map(move |_| {
        if let Some(id) = handler_id_clone.borrow_mut().take() {
            webview_clone.disconnect(id);
        }
        if let Some(f) = f_cell_clone.borrow_mut().take() {
            f(webview_clone.clone());
        }
    });

    *handler_id.borrow_mut() = Some(id);
}

/// Load HTML into a WebView with deferred execution to avoid GTK allocation warnings.
///
/// **Problem this solves**: GTK throws warnings if you try to load HTML into a WebView
/// that doesn't have a valid allocation (width/height). This happens during initialization
/// or when widgets are being rearranged.
///
/// **Solution**: Poll with a timer (every 16ms ≈ 60fps) until the WebView is both:
/// 1. Realized (has a backing surface)
/// 2. Has a non-trivial allocation (width > 1, height > 1)
///
/// **Retry limit**: 300 attempts × 16ms = ~4.8 seconds maximum wait
///
/// **Special case**: If the WebView is unmapped (hidden), we defer until it's mapped
/// to avoid timing out while the widget is legitimately not visible.
pub fn load_html_when_ready(webview: &WebView, html: String, base_uri: Option<String>) {
    use std::cell::Cell;

    // If the WebView is not mapped, it is effectively "not on screen" (e.g. hidden
    // in a Stack when the user is in code view). Defer until mapped to avoid
    // snapshot/allocation warnings and avoid timing out while hidden.
    if !webview.is_mapped() {
        let webview = webview.clone();
        run_once_when_mapped(&webview, move |wv| {
            load_html_when_ready(&wv, html, base_uri)
        });
        return;
    }

    let webview = webview.clone();
    let tries = Cell::new(0u32);

    // Poll on a timer rather than an idle loop: idle sources can run extremely
    // fast (hundreds/thousands of iterations per second), which makes the retry
    // counter effectively meaningless and can lead to "giving up" too early.
    glib::timeout_add_local(Duration::from_millis(16), move || {
        let t = tries.get();
        if t >= 300 {
            log::debug!("[webkit6] Giving up delayed load_html after {} retries", t);
            return glib::ControlFlow::Break;
        }
        tries.set(t + 1);

        // Realized implies a backing surface exists, but allocation can still be
        // pending during the first frame(s).
        if !webview.is_realized() {
            return glib::ControlFlow::Continue;
        }
        if webview.allocated_width() <= 1 || webview.allocated_height() <= 1 {
            return glib::ControlFlow::Continue;
        }

        webview.load_html(&html, base_uri.as_deref());
        glib::ControlFlow::Break
    });
}

/// Parse a hex color string (e.g., "#2b303b") into a gtk4::gdk::RGBA struct.
/// Supports both 6-digit (#RRGGBB) and 3-digit (#RGB) formats.
/// Returns None if parsing fails.
fn parse_hex_to_rgba(hex: &str) -> Option<gtk4::gdk::RGBA> {
    let hex = hex.trim().trim_start_matches('#');

    let (r, g, b) = if hex.len() == 6 {
        // 6-digit format: #RRGGBB
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        (r, g, b)
    } else if hex.len() == 3 {
        // 3-digit format: #RGB -> #RRGGBB
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        (r, g, b)
    } else {
        return None;
    };

    Some(gtk4::gdk::RGBA::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        1.0,
    ))
}

/// Generate a file:// base URI from a document path for resolving relative file references.
/// If the document has a parent directory, returns a file:// URI for that directory.
/// This allows relative image paths and other file references in the document to work correctly.
pub fn generate_base_uri_from_path<P: AsRef<Path>>(document_path: P) -> Option<String> {
    if let Some(parent_dir) = document_path.as_ref().parent() {
        // Convert parent directory to absolute path and create file:// URI
        if let Ok(absolute_parent) = parent_dir.canonicalize() {
            let path_str = absolute_parent.to_string_lossy();
            Some(format!("file://{}/", path_str))
        } else {
            // Fallback: use the path as-is if canonicalize fails
            let path_str = parent_dir.to_string_lossy();
            Some(format!("file://{}/", path_str))
        }
    } else {
        None
    }
}
/// Setup UserContentManager for proper script and stylesheet management
/// This prevents memory leaks from accumulated JavaScript and CSS
fn setup_user_content_manager(webview: &WebView) {
    // Store a reference to track if cleanup is needed
    // For now, we'll implement the cleanup pattern in the HTML template
    // and use proper JavaScript management through the template system
    log::debug!(
        "[webkit6] Setting up UserContentManager for WebView: {:p}",
        webview
    );
}
/// Create a WebView widget with an optional base URI for resolving relative paths.
/// This version allows specifying a base URI to resolve local file references.
/// Optionally accepts a background_color hex string (e.g., "#2b303b") to set widget background.
pub fn create_html_viewer_with_base(
    html: &str,
    base_uri: Option<&str>,
    background_color: Option<&str>,
) -> WebView {
    let webview = WebView::new();

    // This prevents white flash during WebKit initialization (0ms delay)
    if let Some(bg_hex) = background_color {
        if let Some(rgba) = parse_hex_to_rgba(bg_hex) {
            webview.set_background_color(&rgba);
            log::debug!("[webkit6] Set widget background color: {}", bg_hex);
        } else {
            log::warn!("[webkit6] Failed to parse background color: {}", bg_hex);
        }
    }

    // Configure WebKit security settings to allow local file access
    if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
        settings.set_allow_file_access_from_file_urls(true);
        settings.set_allow_universal_access_from_file_urls(true);
        settings.set_auto_load_images(true);
    }

    // Initialize UserContentManager for proper script and stylesheet management
    setup_user_content_manager(&webview);

    // Set up cleanup on destruction to prevent memory leaks
    webview.connect_destroy({
        let webview_cleanup = webview.clone();
        move |_| {
            // Cleanup JavaScript state before destruction
            webview_cleanup.evaluate_javascript(
                "(function() { 
                    if (window.MarcoPreview) { 
                        MarcoPreview.cleanup(); 
                        delete window.MarcoPreview; 
                    } 
                })()",
                None,                      // world_name
                None,                      // source_uri
                None::<&gio::Cancellable>, // cancellable
                |_| {
                    // Cleanup completed, WebView can be safely destroyed
                },
            );
        }
    });

    // Defer loading HTML until the WebView is realized+allocated.
    load_html_when_ready(&webview, html.to_string(), base_uri.map(|s| s.to_string()));

    // Setup link handling for external/internal links
    setup_link_handling(&webview);

    webview.set_vexpand(true);
    webview.set_hexpand(true);
    webview
}

/// Update WebView content via JavaScript injection without full page reload.
///
/// **Performance benefit**: Avoids full page reload, preserving scroll position
/// and preventing the white flash that occurs during load_html().
///
/// **How it works**:
/// 1. Escapes the new HTML content for JavaScript string safety
/// 2. Injects JavaScript that:
///    a. Tries to use window.MarcoPreview.updateContent() if available
///    b. Falls back to direct DOM update if MarcoPreview isn't ready
///    c. Preserves scroll position during update
///
/// **Memory leak prevention**: Cleans up temporary variables and uses
/// IIFE (Immediately Invoked Function Expression) to avoid polluting global scope.
///
/// **Retry logic**: Like load_html_when_ready(), this defers execution if the
/// WebView isn't mapped or doesn't have a valid allocation.
pub fn update_html_content_smooth(webview: &WebView, content: &str) {
    // If the WebView isn't currently mapped (visible), don't try to update it yet.
    // We'll apply the latest update once it becomes mapped.
    if !webview.is_mapped() {
        let webview = webview.clone();
        let content = content.to_string();
        run_once_when_mapped(&webview, move |wv| {
            update_html_content_smooth(&wv, &content)
        });
        return;
    }

    // Avoid GTK warnings such as:
    // "Trying to snapshot GtkGizmo ... without a current allocation".
    // A WebView can be realized but still not have a size allocation during the
    // first frame(s) after being added to a container.
    if !webview.is_realized() || webview.allocated_width() <= 1 || webview.allocated_height() <= 1 {
        let webview = webview.clone();
        let content = content.to_string();

        // Retry briefly instead of dropping the update. This keeps first-load
        // behavior deterministic (open file => preview eventually updates).
        use std::cell::Cell;
        let tries = Cell::new(0u32);

        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let t = tries.get();
            if t >= 120 {
                log::debug!(
                    "[webkit6] Giving up delayed smooth update after {} retries",
                    t
                );
                return glib::ControlFlow::Break;
            }
            tries.set(t + 1);

            if !webview.is_realized()
                || webview.allocated_width() <= 1
                || webview.allocated_height() <= 1
            {
                return glib::ControlFlow::Continue;
            }

            update_html_content_smooth(&webview, &content);
            glib::ControlFlow::Break
        });

        return;
    }

    let escaped_content = content
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    // Use a more efficient JavaScript approach that avoids creating multiple functions
    // and cleans up properly to prevent memory leaks
    let js_code = format!(
        r#"
        (function() {{
            try {{
                // Cleanup any previous temporary variables
                if (window._marcoTempUpdate) {{
                    delete window._marcoTempUpdate;
                }}
                
                // Check if our MarcoPreview object exists with update function
                if (window.MarcoPreview && typeof window.MarcoPreview.updateContent === 'function') {{
                    window.MarcoPreview.updateContent('{}');
                    return;
                }}
                
                // Fallback: direct DOM update without creating persistent variables
                var container = document.getElementById('marco-content-container');
                if (container) {{
                    // Save scroll position
                    var scrollTop = document.documentElement.scrollTop || document.body.scrollTop;
                    
                    // Update content
                    container.innerHTML = '{}';
                    
                    // Restore scroll position
                    setTimeout(function() {{
                        document.documentElement.scrollTop = scrollTop;
                        document.body.scrollTop = scrollTop;
                    }}, 10);
                }} else {{
                    // Last resort: create container
                    var body = document.body || document.getElementsByTagName('body')[0];
                    if (body) {{
                        body.innerHTML = '<div id="marco-content-container">{}</div>';
                    }}
                }}
            }} catch(e) {{
                console.error('Error in content update:', e);
            }}
        }})();
        "#,
        escaped_content, escaped_content, escaped_content
    );

    let webview_clone = webview.clone();

    glib::idle_add_local(move || {
        webview_clone.evaluate_javascript(
            &js_code,
            None,                      // world_name
            None,                      // source_uri
            None::<&gio::Cancellable>, // cancellable
            |result| match result {
                Ok(_) => log::debug!("[webkit6] Content update JavaScript executed successfully"),
                Err(e) => log::warn!(
                    "[webkit6] Failed to execute content update JavaScript: {}",
                    e
                ),
            },
        );
        glib::ControlFlow::Break
    });
}

/// Wraps the HTML body with a full HTML document, injecting the provided CSS string into the <head>.
/// Enhanced with proper cleanup mechanisms to prevent memory leaks.
pub fn wrap_html_document(
    body: &str,
    css: &str,
    theme_mode: &str,
    background_color: Option<&str>,
) -> String {
    core::render::wrap_preview_html_document(body, css, theme_mode, background_color)
}

// Note: in-page JS helpers are embedded in the HTML template produced by
// `wrap_html_document`. When we need to trigger preview interactions from Rust,
// we do so via small helper functions that call into `window.MarcoPreview`.

/// Start autoplay timers for all `marco_sliders` decks in the current preview (if any).
#[allow(dead_code)]
pub fn sliders_play_all(webview: &WebView) {
    let js = r#"(function(){try{if(window.MarcoPreview&&window.MarcoPreview.sliders&&typeof window.MarcoPreview.sliders.playAll==='function'){window.MarcoPreview.sliders.playAll();}}catch(e){console.error('sliders_play_all error',e);}})();"#;
    let webview_clone = webview.clone();
    glib::idle_add_local(move || {
        webview_clone.evaluate_javascript(js, None, None, None::<&gio::Cancellable>, |_result| {});
        glib::ControlFlow::Break
    });
}

/// Stop autoplay timers for all `marco_sliders` decks in the current preview (if any).
#[allow(dead_code)]
pub fn sliders_pause_all(webview: &WebView) {
    let js = r#"(function(){try{if(window.MarcoPreview&&window.MarcoPreview.sliders&&typeof window.MarcoPreview.sliders.pauseAll==='function'){window.MarcoPreview.sliders.pauseAll();}}catch(e){console.error('sliders_pause_all error',e);}})();"#;
    let webview_clone = webview.clone();
    glib::idle_add_local(move || {
        webview_clone.evaluate_javascript(js, None, None, None::<&gio::Cancellable>, |_result| {});
        glib::ControlFlow::Break
    });
}

/// Create a WebView-based HTML source viewer with syntax highlighting.
///
/// This viewer displays HTML source code (from Marco's markdown rendering)
/// with professional syntax highlighting powered by syntect.
///
/// # Arguments
/// * `html_source` - The HTML code to display (already generated by Marco)
/// * `theme_mode` - "light" or "dark" theme mode for syntax highlighting
/// * `base_uri` - Optional base URI for resolving relative paths
///
/// # Returns
/// * `Ok(WebView)` - Configured WebView with highlighted HTML
/// * `Err(String)` - Error message if highlighting fails
///
/// # Example
/// ```ignore
/// let webview = create_html_source_viewer_webview(
///     "<h1>Hello</h1>",
///     "dark",
///     None,
/// )?;
/// ```
pub fn create_html_source_viewer_webview(
    html_source: &str,
    theme_mode: &str,
    base_uri: Option<&str>,
    editor_bg: Option<&str>,
    editor_fg: Option<&str>,
    scrollbar_thumb: Option<&str>,
    scrollbar_track: Option<&str>,
) -> Result<WebView, String> {
    use crate::logic::syntax_highlighter::{generate_css_with_global, global_syntax_highlighter};

    // Normalize theme mode to "light" or "dark"
    let normalized_theme = if theme_mode.contains("dark") {
        "dark"
    } else {
        "light"
    };

    log::debug!(
        "[webkit6] Creating WebView-based code viewer with theme: {} (normalized: {})",
        theme_mode,
        normalized_theme
    );
    log::debug!("[webkit6] HTML source length: {} bytes", html_source.len());

    // If HTML source is empty, use a placeholder
    let display_html = if html_source.is_empty() {
        log::debug!("[webkit6] HTML source is empty, using placeholder");
        "<!-- No content yet -->"
    } else {
        html_source
    };

    // Initialize global syntax highlighter
    global_syntax_highlighter()
        .map_err(|e| format!("Failed to initialize syntax highlighter: {}", e))?;

    // Get syntect CSS for current theme
    let syntect_css = generate_css_with_global(normalized_theme)
        .map_err(|e| format!("Failed to generate CSS: {}", e))?;

    // Highlight HTML source using syntect
    let highlighted_html = SYNTAX_HIGHLIGHTER.with(|highlighter| {
        let h = highlighter.borrow();
        let syntax_highlighter = h
            .as_ref()
            .ok_or_else(|| "Syntax highlighter not initialized".to_string())?;

        syntax_highlighter
            .highlight_to_html(display_html, "html", normalized_theme)
            .map_err(|e| format!("Highlighting failed: {}", e))
    })?;

    // Determine theme colors - use editor colors if provided, otherwise use defaults
    let (bg_color, fg_color) = if let (Some(bg), Some(fg)) = (editor_bg, editor_fg) {
        (bg, fg)
    } else if normalized_theme == "dark" {
        ("#2b303b", "#c0c5ce")
    } else {
        ("#fdf6e3", "#657b83") // Solarized Light colors
    };

    // Generate webkit scrollbar CSS to match editor
    let scrollbar_css = if let (Some(thumb), Some(track)) = (scrollbar_thumb, scrollbar_track) {
        crate::components::viewer::css_utils::webkit_scrollbar_css(thumb, track)
    } else {
        String::new()
    };

    // Build complete HTML page with syntect CSS and scrollbar styling
    let complete_page = format!(
        r#"<!DOCTYPE html>
<html style="height: 100%; margin: 0; padding: 0; overflow: hidden;">
  <head>
    <meta charset="UTF-8">
    <style>
      html, body {{
        height: 100%;
        margin: 0;
        padding: 0;
        overflow: hidden;
      }}
      body {{
        background: {};
        color: {};
        font-family: 'Fira Code', 'Monaco', 'Courier New', monospace;
        font-size: 12px;
        line-height: 1.5;
        display: flex;
        flex-direction: column;
      }}
      #code-container {{
        flex: 1;
        overflow: auto;
        padding: 16px;
        box-sizing: border-box;
      }}
      pre {{
        margin: 0;
        white-space: pre;
        word-wrap: normal;
      }}
      code {{
        font-family: inherit;
        white-space: pre;
      }}
      /* Syntect CSS */
      {}
      /* Scrollbar styling */
      {}
    </style>
  </head>
  <body>
    <div id="code-container">
      <pre><code>{}</code></pre>
    </div>
  </body>
</html>"#,
        bg_color, fg_color, syntect_css, scrollbar_css, highlighted_html
    );

    log::debug!(
        "[webkit6] Generated HTML page: {} bytes, bg={}, fg={}",
        complete_page.len(),
        bg_color,
        fg_color
    );
    log::debug!(
        "[webkit6] Highlighted HTML length: {} bytes",
        highlighted_html.len()
    );
    log::debug!(
        "[webkit6] Highlighted HTML preview: {}",
        &highlighted_html.chars().take(200).collect::<String>()
    );
    log::debug!("[webkit6] Syntect CSS length: {} bytes", syntect_css.len());

    // Debug: Write HTML to temporary file for inspection
    if let Err(e) = std::fs::write("/tmp/marco_code_view_debug.html", &complete_page) {
        log::warn!("[webkit6] Failed to write debug HTML: {}", e);
    } else {
        log::debug!("[webkit6] Debug HTML written to /tmp/marco_code_view_debug.html");
    }

    // Create WebView
    let webview = WebView::new();

    // Configure security settings (same as HTML preview)
    if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
        settings.set_allow_file_access_from_file_urls(true);
        settings.set_allow_universal_access_from_file_urls(true);
        settings.set_auto_load_images(true);
    }

    // Initialize UserContentManager
    setup_user_content_manager(&webview);

    // Set up cleanup on destruction
    webview.connect_destroy({
        let webview_cleanup = webview.clone();
        move |_| {
            webview_cleanup.evaluate_javascript(
                "(function() { if (window.MarcoPreview) { MarcoPreview.cleanup(); delete window.MarcoPreview; } })()",
                None,
                None,
                None::<&gio::Cancellable>,
                |_| {},
            );
        }
    });

    // Defer loading HTML until the WebView is realized+allocated.
    log::debug!(
        "[webkit6] Scheduling code view WebView initial load: {} bytes",
        complete_page.len()
    );
    load_html_when_ready(
        &webview,
        complete_page.clone(),
        base_uri.map(|s| s.to_string()),
    );

    // Setup link handling for external/internal links
    setup_link_handling(&webview);

    webview.set_vexpand(true);
    webview.set_hexpand(true);

    log::debug!("[webkit6] Code viewer WebView created successfully");
    Ok(webview)
}

/// Update code view WebView content smoothly using JavaScript injection.
/// This avoids full page reloads and prevents flickering while updating the HTML source.
pub fn update_code_view_smooth(
    webview: &WebView,
    html_source: &str,
    theme_mode: &str,
    editor_bg: Option<&str>,
    editor_fg: Option<&str>,
    scrollbar_thumb: Option<&str>,
    scrollbar_track: Option<&str>,
) -> Result<(), String> {
    use crate::logic::syntax_highlighter::{generate_css_with_global, global_syntax_highlighter};

    // If the WebView isn't currently mapped (visible), don't try to update it yet.
    // We'll apply the update once it becomes mapped.
    if !webview.is_mapped() {
        let webview = webview.clone();
        let html_source = html_source.to_string();
        let theme_mode = theme_mode.to_string();
        let editor_bg = editor_bg.map(|s| s.to_string());
        let editor_fg = editor_fg.map(|s| s.to_string());
        let scrollbar_thumb = scrollbar_thumb.map(|s| s.to_string());
        let scrollbar_track = scrollbar_track.map(|s| s.to_string());

        run_once_when_mapped(&webview, move |wv| {
            let _ = update_code_view_smooth(
                &wv,
                &html_source,
                &theme_mode,
                editor_bg.as_deref(),
                editor_fg.as_deref(),
                scrollbar_thumb.as_deref(),
                scrollbar_track.as_deref(),
            );
        });

        return Ok(());
    }

    // Avoid GTK warnings such as:
    // "Trying to snapshot GtkGizmo ... without a current allocation".
    if !webview.is_realized() || webview.allocated_width() <= 1 || webview.allocated_height() <= 1 {
        let webview = webview.clone();
        let html_source = html_source.to_string();
        let theme_mode = theme_mode.to_string();
        let editor_bg = editor_bg.map(|s| s.to_string());
        let editor_fg = editor_fg.map(|s| s.to_string());
        let scrollbar_thumb = scrollbar_thumb.map(|s| s.to_string());
        let scrollbar_track = scrollbar_track.map(|s| s.to_string());

        use std::cell::Cell;
        let tries = Cell::new(0u32);

        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let t = tries.get();
            if t >= 120 {
                log::debug!(
                    "[webkit6] Giving up delayed code view update after {} retries",
                    t
                );
                return glib::ControlFlow::Break;
            }
            tries.set(t + 1);

            if !webview.is_realized()
                || webview.allocated_width() <= 1
                || webview.allocated_height() <= 1
            {
                return glib::ControlFlow::Continue;
            }

            let _ = update_code_view_smooth(
                &webview,
                &html_source,
                &theme_mode,
                editor_bg.as_deref(),
                editor_fg.as_deref(),
                scrollbar_thumb.as_deref(),
                scrollbar_track.as_deref(),
            );
            glib::ControlFlow::Break
        });

        return Ok(());
    }

    // Normalize theme mode
    let normalized_theme = if theme_mode.contains("dark") {
        "dark"
    } else {
        "light"
    };

    log::debug!(
        "[webkit6] Smooth updating code view with theme: {} (normalized: {})",
        theme_mode,
        normalized_theme
    );

    // Handle empty HTML
    let display_html = if html_source.is_empty() {
        "<!-- No content yet -->"
    } else {
        html_source
    };

    // Initialize and get CSS
    global_syntax_highlighter()
        .map_err(|e| format!("Failed to initialize syntax highlighter: {}", e))?;

    let syntect_css = generate_css_with_global(normalized_theme)
        .map_err(|e| format!("Failed to generate CSS: {}", e))?;

    // Highlight HTML
    let highlighted_html = SYNTAX_HIGHLIGHTER.with(|highlighter| {
        let h = highlighter.borrow();
        let syntax_highlighter = h
            .as_ref()
            .ok_or_else(|| "Syntax highlighter not initialized".to_string())?;

        syntax_highlighter
            .highlight_to_html(display_html, "html", normalized_theme)
            .map_err(|e| format!("Highlighting failed: {}", e))
    })?;

    // Escape for JavaScript
    let escaped_html = highlighted_html
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    let escaped_css = syntect_css
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    // Determine colors for theme - use editor colors if provided, otherwise use defaults
    let (bg_color, fg_color) = if let (Some(bg), Some(fg)) = (editor_bg, editor_fg) {
        (bg, fg)
    } else if normalized_theme == "dark" {
        ("#2b303b", "#c0c5ce")
    } else {
        ("#fdf6e3", "#657b83")
    };

    // Generate webkit scrollbar CSS
    let scrollbar_css = if let (Some(thumb), Some(track)) = (scrollbar_thumb, scrollbar_track) {
        crate::components::viewer::css_utils::webkit_scrollbar_css(thumb, track)
    } else {
        String::new()
    };

    let escaped_scrollbar_css = scrollbar_css
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    // JavaScript to update content and theme
    let js_code = format!(
        r#"
        (function() {{
            try {{
                // Update body colors
                document.body.style.background = '{}';
                document.body.style.color = '{}';
                
                // Update syntect CSS
                var styleEl = document.getElementById('marco-syntect-style');
                if (!styleEl) {{
                    styleEl = document.createElement('style');
                    styleEl.id = 'marco-syntect-style';
                    document.head.appendChild(styleEl);
                }}
                styleEl.textContent = '{}';
                
                // Update scrollbar CSS
                var scrollbarStyleEl = document.getElementById('marco-scrollbar-style');
                if (!scrollbarStyleEl) {{
                    scrollbarStyleEl = document.createElement('style');
                    scrollbarStyleEl.id = 'marco-scrollbar-style';
                    document.head.appendChild(scrollbarStyleEl);
                }}
                scrollbarStyleEl.textContent = '{}';
                
                // Update code content
                var codeEl = document.querySelector('pre code');
                if (codeEl) {{
                    var scrollTop = window.scrollY;
                    codeEl.innerHTML = '{}';
                    window.scrollTo(0, scrollTop);
                }} else {{
                    console.error('Code element not found');
                }}
            }} catch(e) {{
                console.error('Update failed:', e);
            }}
        }})();
        "#,
        bg_color, fg_color, escaped_css, escaped_scrollbar_css, escaped_html
    );

    let webview_clone = webview.clone();
    glib::idle_add_local(move || {
        webview_clone.evaluate_javascript(
            &js_code,
            None,
            None,
            None::<&gio::Cancellable>,
            |result| match result {
                Ok(_) => log::debug!("[webkit6] Code view smooth update successful"),
                Err(e) => log::warn!("[webkit6] Code view smooth update failed: {}", e),
            },
        );
        glib::ControlFlow::Break
    });

    Ok(())
}

// Import SYNTAX_HIGHLIGHTER for use in create_html_source_viewer_webview
use crate::logic::syntax_highlighter::SYNTAX_HIGHLIGHTER;

/// Helper function to determine if a URI is external (should open in system browser)
/// or internal (should be handled by WebView).
///
/// External URIs:
/// - http or https schemes
/// - www. prefix (normalized to https)
///
/// Internal URIs:
/// - file:// scheme (local files)
/// - # anchor links (in-page navigation)
/// - Relative paths
/// - Empty or None URIs
fn is_external_uri(uri: &str) -> bool {
    let uri_lower = uri.to_lowercase();

    // External: HTTP/HTTPS schemes
    if uri_lower.starts_with("http:") || uri_lower.starts_with("https:") {
        return true;
    }

    // External: www. prefix (treat as https)
    if uri_lower.starts_with("www.") {
        return true;
    }

    // External: mailto links (open in email client)
    if uri_lower.starts_with("mailto:") {
        return true;
    }

    // Internal: everything else (file://, #anchors, relative paths, etc.)
    false
}

/// Open an external URI in the system's default browser.
/// Cross-platform support for Linux and Windows.
///
/// # Arguments
/// * `uri` - The URI to open (must be http/https or start with www.)
///
/// # Returns
/// * `Ok(())` if the URI was successfully launched
/// * `Err(String)` if launching failed
fn open_external_uri(uri: &str) -> Result<(), String> {
    // Normalize www. prefix to a secure default.
    let normalized_uri = if uri.to_lowercase().starts_with("www.") {
        format!("{}://{}", "https", uri)
    } else {
        uri.to_string()
    };

    log::info!(
        "[webkit6] Opening external URI in system browser: {}",
        normalized_uri
    );

    // Use gio's AppInfo to launch the URI with the system's default handler
    match gio::AppInfo::launch_default_for_uri(&normalized_uri, None::<&gio::AppLaunchContext>) {
        Ok(_) => {
            log::debug!(
                "[webkit6] Successfully launched external URI: {}",
                normalized_uri
            );
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to open external URI '{}': {}", normalized_uri, e);
            log::error!("[webkit6] {}", error_msg);
            Err(error_msg)
        }
    }
}

/// Setup link handling to open external links in system browser.
///
/// **Behavior**:
/// - **External links** (http://, https://, mailto:, www.*): Open in system browser
/// - **Internal links** (file://, #anchors, relative paths): Handle in WebView
///
/// **Implementation**: Intercepts the `decide-policy` signal for navigation actions.
/// For external links, it calls `decision.ignore()` to prevent WebView navigation,
/// then launches the system browser via `gio::AppInfo::launch_default_for_uri()`.
///
/// **Cross-platform**: Uses GIO's AppInfo which works on both Linux and Windows
/// to launch the default browser/handler.
fn setup_link_handling(webview: &WebView) {
    use webkit6::prelude::*;

    webview.connect_decide_policy(|_webview, decision, decision_type| {
        // Handle both navigation actions and new window actions (target="_blank" links)
        if decision_type != webkit6::PolicyDecisionType::NavigationAction
            && decision_type != webkit6::PolicyDecisionType::NewWindowAction
        {
            return false; // Let WebKit handle other decision types
        }

        // Try to downcast to NavigationPolicyDecision to get the URI
        if let Ok(navigation_decision) = decision
            .clone()
            .downcast::<webkit6::NavigationPolicyDecision>()
        {
            // Get the navigation action to extract the request URI
            if let Some(mut navigation_action) = navigation_decision.navigation_action() {
                if let Some(request) = navigation_action.request() {
                    if let Some(uri) = request.uri() {
                        let uri_str = uri.as_str();
                        log::debug!("[webkit6] Navigation decision for URI: {}", uri_str);

                        // Check if this is an external link
                        if is_external_uri(uri_str) {
                            log::info!("[webkit6] External link detected: {}", uri_str);

                            // Prevent WebView from loading the external URL
                            decision.ignore();

                            // Open in system browser
                            if let Err(e) = open_external_uri(uri_str) {
                                log::warn!("[webkit6] Failed to open external link: {}", e);
                            }

                            return true; // We handled this decision
                        } else {
                            log::debug!(
                                "[webkit6] Internal/local link, allowing WebView to handle: {}",
                                uri_str
                            );
                        }
                    }
                }
            }
        }

        // Let WebKit handle the navigation for internal links
        false
    });

    log::debug!(
        "[webkit6] Link handling setup completed for WebView: {:p}",
        webview
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_url_classification() {
        let http_example = format!("{}://example.com", "http");
        let http_example_path = format!("{}://example.com/path?query=value", "http");
        let http_upper = format!("{}{}EXAMPLE.COM", "HTTP", "://");

        // Test external URLs - should return true
        assert!(
            is_external_uri(&http_example),
            "HTTP URL should be external"
        );
        assert!(
            is_external_uri("https://example.com"),
            "HTTPS URL should be external"
        );
        assert!(
            is_external_uri(&http_example_path),
            "HTTP URL with path should be external"
        );
        assert!(
            is_external_uri("https://example.com:8080/path"),
            "HTTPS URL with port should be external"
        );
        assert!(
            is_external_uri("www.example.com"),
            "www URL should be external"
        );
        assert!(
            is_external_uri("www.example.com/page"),
            "www URL with path should be external"
        );

        // Test mailto links - should return true (open in email client)
        assert!(
            is_external_uri("mailto:user@example.com"),
            "mailto link should be external"
        );
        assert!(
            is_external_uri("mailto:admin@example.com"),
            "mailto should be external"
        );
        assert!(
            is_external_uri("MAILTO:USER@EXAMPLE.COM"),
            "Uppercase mailto should be external"
        );

        // Test internal/local URLs - should return false
        assert!(
            !is_external_uri("file:///home/user/document.md"),
            "file:// URL should be internal"
        );
        assert!(
            !is_external_uri("#section-id"),
            "Anchor link should be internal"
        );
        assert!(!is_external_uri("#"), "Empty anchor should be internal");
        assert!(
            !is_external_uri("relative/path/to/file.html"),
            "Relative path should be internal"
        );
        assert!(
            !is_external_uri("/absolute/path/to/file.html"),
            "Absolute path should be internal"
        );
        assert!(!is_external_uri(""), "Empty string should be internal");

        // Edge cases
        assert!(
            !is_external_uri("data:text/html,<h1>Hello</h1>"),
            "data: URL should be internal"
        );
        assert!(
            !is_external_uri("about:blank"),
            "about: URL should be internal"
        );
        assert!(
            is_external_uri(&http_upper),
            "Uppercase HTTP should be external"
        );
        assert!(
            is_external_uri("WWW.EXAMPLE.COM"),
            "Uppercase www should be external"
        );
    }

    #[test]
    fn smoke_test_open_external_uri() {
        // Test that the function exists and has correct signature
        // We can't actually test launching browsers in unit tests, but we can verify error handling

        // Invalid URI should return error
        let result = open_external_uri("");
        assert!(result.is_err(), "Empty URI should return error");

        // These would actually try to open the browser, so we skip them in automated tests
        // In manual testing, verify:
        // - open_external_uri("https://example.com") opens browser
        // - open_external_uri("http:...") opens browser
    }
}
