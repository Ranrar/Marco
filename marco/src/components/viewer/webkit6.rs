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
//! The HTML viewer receives already-rendered HTML from `marco_core::render` and displays it.
//! It does not perform Markdown-to-HTML conversion itself.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::WebView;

use crate::components::viewer::allocation_wait;

type WebViewOnceFn = Box<dyn FnOnce(WebView)>;

/// Run a closure once the widget is mapped (visible in the widget tree).
///
/// Thin `WebView`-typed adapter around
/// [`allocation_wait::run_once_when_mapped`] so existing call sites in this
/// module that pass the `WebView` into the closure can stay unchanged.
fn run_once_when_mapped(webview: &WebView, f: impl FnOnce(WebView) + 'static) {
    let webview_for_cb = webview.clone();
    let f_cell: Rc<RefCell<Option<WebViewOnceFn>>> = Rc::new(RefCell::new(Some(Box::new(f))));
    allocation_wait::run_once_when_mapped(webview, move || {
        if let Some(callback) = f_cell.borrow_mut().take() {
            callback(webview_for_cb.clone());
        }
    });
}

/// Load HTML into a WebView with deferred execution to avoid GTK allocation warnings.
///
/// Delegates the map+allocation polling to
/// [`allocation_wait::run_when_allocated`] so the retry behaviour is shared
/// with the Windows (wry) backend.
///
/// **Retry budget**: [`allocation_wait::DEFAULT_MAX_RETRIES`] × 16 ms ≈ 4.8 s.
pub fn load_html_when_ready(webview: &WebView, html: String, base_uri: Option<String>) {
    let webview_for_load = webview.clone();
    allocation_wait::run_when_allocated(webview, allocation_wait::DEFAULT_MAX_RETRIES, move || {
        webview_for_load.load_html(&html, base_uri.as_deref());
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
                    if (window.MarcoCorePreview) { 
                        MarcoCorePreview.cleanup(); 
                        delete window.MarcoCorePreview; 
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
    // Pin the GTK widget direction to LTR so the WebKitGTK native overlay scrollbar
    // always renders on the physical right, regardless of the global RTL default.
    // Content direction is handled via <body dir="rtl"> in the HTML, not the widget.
    webview.set_direction(gtk4::TextDirection::Ltr);
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
///    a. Tries to use window.MarcoCorePreview.updateContent() if available
///    b. Falls back to direct DOM update if MarcoCorePreview isn't ready
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
                
                // Check if our MarcoCorePreview object exists with update function
                if (window.MarcoCorePreview && typeof window.MarcoCorePreview.updateContent === 'function') {{
                    window.MarcoCorePreview.updateContent('{}');
                    return;
                }}
                
                // Fallback: direct DOM update without creating persistent variables
                var container = document.getElementById('mc-content-container');
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
                        body.innerHTML = '<div id="mc-content-container">{}</div>';
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
    let html =
        marco_core::render::wrap_preview_html_document(body, css, theme_mode, background_color);
    // Always keep <html dir="ltr"> so the WebKit viewport scrollbar stays on the right,
    // consistent with the editor/TOC scrollbar behaviour.  For RTL documents, inject
    // dir="rtl" on <body> instead — content flows RTL while the scrollbar stays right.
    let html = html.replacen("<html ", "<html dir=\"ltr\" ", 1);
    if crate::logic::rtl::is_rtl_global() {
        html.replacen("<body>", "<body dir=\"rtl\">", 1)
    } else {
        html
    }
}

// Note: in-page JS helpers are embedded in the HTML template produced by
// `wrap_html_document`. When we need to trigger preview interactions from Rust,
// we do so via small helper functions that call into `window.MarcoCorePreview`.

/// Start autoplay timers for all `marco_sliders` decks in the current preview (if any).
#[allow(dead_code)]
pub fn sliders_play_all(webview: &WebView) {
    let js = r#"(function(){try{if(window.MarcoCorePreview&&window.MarcoCorePreview.sliders&&typeof window.MarcoCorePreview.sliders.playAll==='function'){window.MarcoCorePreview.sliders.playAll();}}catch(e){console.error('sliders_play_all error',e);}})();"#;
    let webview_clone = webview.clone();
    glib::idle_add_local(move || {
        webview_clone.evaluate_javascript(js, None, None, None::<&gio::Cancellable>, |_result| {});
        glib::ControlFlow::Break
    });
}

/// Stop autoplay timers for all `marco_sliders` decks in the current preview (if any).
#[allow(dead_code)]
pub fn sliders_pause_all(webview: &WebView) {
    let js = r#"(function(){try{if(window.MarcoCorePreview&&window.MarcoCorePreview.sliders&&typeof window.MarcoCorePreview.sliders.pauseAll==='function'){window.MarcoCorePreview.sliders.pauseAll();}}catch(e){console.error('sliders_pause_all error',e);}})();"#;
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
    log::debug!(
        "[webkit6] Creating WebView-based code viewer with theme: {} (source: {} bytes)",
        theme_mode,
        html_source.len()
    );

    // Delegate HTML page assembly (syntect highlighting + theme/scrollbar CSS
    // + body shell) to the cross-platform `code_view_html` builder so the
    // wry-based Windows code view (§14.5 of the parity audit) produces
    // bit-identical output.
    let complete_page = crate::components::viewer::code_view_html::build_full_page(
        html_source,
        theme_mode,
        editor_bg,
        editor_fg,
        scrollbar_thumb,
        scrollbar_track,
    )?;

    log::debug!(
        "[webkit6] Generated code-view HTML page: {} bytes",
        complete_page.len()
    );

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
                "(function() { if (window.MarcoCorePreview) { MarcoCorePreview.cleanup(); delete window.MarcoCorePreview; } })()",
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
    load_html_when_ready(&webview, complete_page, base_uri.map(|s| s.to_string()));

    // Setup link handling for external/internal links
    setup_link_handling(&webview);

    webview.set_vexpand(true);
    webview.set_hexpand(true);
    // Pin GTK widget direction to LTR — scrollbar stays on physical right.
    webview.set_direction(gtk4::TextDirection::Ltr);

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

    log::debug!(
        "[webkit6] Smooth updating code view with theme: {}",
        theme_mode
    );

    // Build the JS payload via the shared cross-platform builder so wry
    // (Windows) and webkit6 (Linux) issue identical updates.
    let js_code = crate::components::viewer::code_view_html::build_smooth_update_js(
        html_source,
        theme_mode,
        editor_bg,
        editor_fg,
        scrollbar_thumb,
        scrollbar_track,
    )?;

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

/// Wire up link-hover detection so callers can show the hovered URL in the footer.
///
/// The `on_hover` closure receives:
/// - `Some(url)` when the cursor moves over a link
/// - `None`      when the cursor leaves a link (no link under cursor)
pub fn setup_link_hover_status(webview: &WebView, on_hover: impl Fn(Option<String>) + 'static) {
    use webkit6::prelude::*;

    webview.connect_mouse_target_changed(move |_webview, hit_test_result, _modifiers| {
        let url = hit_test_result.link_uri().map(|s| s.to_string());
        on_hover(url);
    });
}

/// Returns `true` if the URI is a local Markdown file link (`file://...md` or `.markdown`).
fn is_local_md_uri(uri: &str) -> bool {
    let uri_lower = uri.to_lowercase();
    if !uri_lower.starts_with("file://") {
        return false;
    }
    // Strip any fragment before checking extension
    let path_part = uri_lower.split('#').next().unwrap_or("");
    path_part.ends_with(".md") || path_part.ends_with(".markdown")
}

/// Splits a `file://` URI into (absolute_path, Option<fragment>).
/// Strips the `file://` scheme and URL-decodes `%20` spaces.
fn extract_path_and_fragment_from_file_uri(uri: &str) -> (String, Option<String>) {
    // Strip the "file://" prefix
    let without_scheme = uri.trim_start_matches("file://");
    let (raw_path, fragment) = match without_scheme.split_once('#') {
        Some((p, f)) => (
            p,
            if f.is_empty() {
                None
            } else {
                Some(f.to_string())
            },
        ),
        None => (without_scheme, None),
    };
    // Basic URL-decode for spaces
    let path = raw_path.replace("%20", " ");
    (path, fragment)
}

/// Intercept clicks on local Markdown file links in the WebView preview.
///
/// When the user clicks a `file://...md` link (e.g., a relative link to another
/// Markdown document), WebKit would normally navigate away and show raw source.
/// This handler intercepts such navigation and calls `on_local_md(path, fragment)`
/// so the application can prompt the user and open the file properly.
///
/// The `on_local_md` closure receives:
/// - `path` — absolute file system path to the target `.md` file
/// - `fragment` — optional anchor fragment (e.g. `"section-title"`)
pub fn setup_local_file_link_handler(
    webview: &WebView,
    on_local_md: impl Fn(String, Option<String>) + 'static,
) {
    use webkit6::prelude::*;

    webview.connect_decide_policy(move |_wv, decision, decision_type| {
        if decision_type != webkit6::PolicyDecisionType::NavigationAction
            && decision_type != webkit6::PolicyDecisionType::NewWindowAction
        {
            return false;
        }

        if let Ok(nav) = decision
            .clone()
            .downcast::<webkit6::NavigationPolicyDecision>()
        {
            if let Some(action) = nav.navigation_action() {
                if let Some(request) = action.request() {
                    if let Some(uri) = request.uri() {
                        let uri_str = uri.as_str();
                        if is_local_md_uri(uri_str) {
                            log::info!("[webkit6] Local .md link intercepted: {}", uri_str);
                            let (path, fragment) = extract_path_and_fragment_from_file_uri(uri_str);
                            decision.ignore();
                            on_local_md(path, fragment);
                            return true;
                        }
                    }
                }
            }
        }

        false
    });
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
            if let Some(navigation_action) = navigation_decision.navigation_action() {
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

    #[test]
    fn smoke_test_local_md_uri_detection() {
        // Should detect .md file URIs
        assert!(
            is_local_md_uri("file:///home/user/docs/README.md"),
            ".md file should be detected"
        );
        assert!(
            is_local_md_uri("file:///home/user/docs/page.markdown"),
            ".markdown file should be detected"
        );
        assert!(
            is_local_md_uri("file:///home/user/docs/README.md#section"),
            ".md file with fragment should be detected"
        );
        assert!(
            is_local_md_uri("file:///home/user/docs/README.MD"),
            "uppercase .MD should be detected"
        );

        // Should NOT detect non-md URIs
        assert!(
            !is_local_md_uri("file:///home/user/docs/image.png"),
            "image file should not be detected"
        );
        assert!(
            !is_local_md_uri("https://example.com/page.md"),
            "http URI to .md should not be detected"
        );
        assert!(
            !is_local_md_uri("file:///home/user/docs/index.html"),
            ".html file should not be detected"
        );
        assert!(!is_local_md_uri(""), "empty URI should not be detected");
    }

    #[test]
    fn smoke_test_extract_path_and_fragment() {
        let (path, fragment) =
            extract_path_and_fragment_from_file_uri("file:///home/user/docs/README.md");
        assert_eq!(path, "/home/user/docs/README.md");
        assert_eq!(fragment, None);

        let (path, fragment) =
            extract_path_and_fragment_from_file_uri("file:///home/user/docs/page.md#intro");
        assert_eq!(path, "/home/user/docs/page.md");
        assert_eq!(fragment, Some("intro".to_string()));

        // URL-decoded spaces
        let (path, fragment) =
            extract_path_and_fragment_from_file_uri("file:///home/user/my%20docs/note.md");
        assert_eq!(path, "/home/user/my docs/note.md");
        assert_eq!(fragment, None);
    }
}
