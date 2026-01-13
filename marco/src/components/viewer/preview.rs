use core::global_parser_cache;
use core::RenderOptions;
use gtk4::prelude::*;
use std::cell::RefCell;

/// Parameters for preview refresh operations
pub struct PreviewRefreshParams<'a> {
    pub webview: &'a webkit6::WebView,
    pub css: &'a RefCell<String>,
    pub html_options: &'a RenderOptions,
    pub buffer: &'a sourceview5::Buffer,
    pub wheel_js: &'a str,
    pub theme_mode: &'a RefCell<String>,
    pub base_uri: Option<&'a str>,
}

/// Simplified parameters for smooth content updates
pub struct SmoothUpdateParams<'a> {
    pub webview: &'a webkit6::WebView,
    pub html_options: &'a RenderOptions,
    pub buffer: &'a sourceview5::Buffer,
    pub wheel_js: &'a str,
}

/// Generate test HTML content when the editor is empty
fn generate_test_html(wheel_js: &str) -> String {
    let welcome_html = r#"<div id="welcome-message" style="
  text-align:center; 
  margin-top:20%; 
  opacity:0.7; 
  font-family:sans-serif;">
  <h1>Welcome to Marco</h1>
  <p>Start typing or open a file to begin your writing journey ✍️</p>
</div>"#;
    let mut html_with_js = welcome_html.to_string();
    html_with_js.push_str(wheel_js);
    html_with_js
}

/// Generate CSS for syntax highlighting based on current theme mode
fn generate_syntax_highlighting_css(theme_mode: &str) -> String {
    use crate::components::viewer::syntax_highlighter::{
        generate_css_with_global, global_syntax_highlighter,
    };

    // Initialize global highlighter if needed
    if let Err(e) = global_syntax_highlighter() {
        log::warn!(
            "[viewer] Failed to initialize syntax highlighter for CSS generation: {}",
            e
        );
        return String::new();
    }

    // Generate CSS for the current theme mode
    match generate_css_with_global(theme_mode) {
        Ok(css) => {
            log::debug!(
                "[viewer] Generated syntax highlighting CSS for theme: {}",
                theme_mode
            );
            css
        }
        Err(e) => {
            log::warn!("[viewer] Failed to generate syntax highlighting CSS: {}", e);
            String::new()
        }
    }
}

/// Parse markdown text into HTML using the Marco engine with full HTML caching
/// Uses the current theme mode from params for syntax highlighting
fn parse_markdown_to_html_with_theme(
    text: &str,
    base_html_options: &RenderOptions,
    theme_mode: &str,
) -> String {
    // Create fresh RenderOptions with the current theme mode for syntax highlighting
    let html_options = RenderOptions {
        theme: theme_mode.to_string(),
        ..base_html_options.clone()
    };

    // Use full HTML caching for optimal performance
    match global_parser_cache().render_with_cache(text, html_options) {
        Ok(html) => html,
        Err(e) => {
            log::error!("[viewer] Error rendering HTML with cache: {}", e);
            format!("Error rendering HTML: {}", e)
        }
    }
}

/// Parse markdown text into HTML using the Marco engine with full HTML caching
fn parse_markdown_to_html(text: &str, html_options: &RenderOptions) -> String {
    // Fallback for backwards compatibility - use light theme if no theme specified
    parse_markdown_to_html_with_theme(text, html_options, "light")
}

/// Small helper to wrap markdown -> html and load into webview using the new rendering system.
/// If document_path is provided, it will be used to generate a base URI for resolving relative paths.
pub fn refresh_preview_into_webview(
    webview: &webkit6::WebView,
    css: &RefCell<String>,
    html_options: &RenderOptions,
    buffer: &sourceview5::Buffer,
    wheel_js: &str,
    theme_mode: &RefCell<String>,
    document_path: Option<&std::path::Path>,
) {
    let base_uri =
        document_path.and_then(crate::components::viewer::webkit6::generate_base_uri_from_path);
    refresh_preview_into_webview_with_base_uri(
        webview,
        css,
        html_options,
        buffer,
        wheel_js,
        theme_mode,
        base_uri.as_deref(),
    );
}

/// Small helper to wrap markdown -> html and load into webview using the new rendering system.
/// If base_uri is provided, it will be used directly as the base URI for resolving relative paths.
pub fn refresh_preview_into_webview_with_base_uri(
    webview: &webkit6::WebView,
    css: &RefCell<String>,
    html_options: &RenderOptions,
    buffer: &sourceview5::Buffer,
    wheel_js: &str,
    theme_mode: &RefCell<String>,
    base_uri: Option<&str>,
) {
    let params = PreviewRefreshParams {
        webview,
        css,
        html_options,
        buffer,
        wheel_js,
        theme_mode,
        base_uri,
    };
    refresh_preview_into_webview_with_base_uri_and_doc_buffer(params);
}

/// Enhanced version that checks both GTK TextBuffer and DocumentBuffer to determine if welcome message should show
pub fn refresh_preview_into_webview_with_base_uri_and_doc_buffer(params: PreviewRefreshParams<'_>) {
    let text = params
        .buffer
        .text(
            &params.buffer.start_iter(),
            &params.buffer.end_iter(),
            false,
        )
        .to_string();

    // Keep the main thread responsive: do not render Markdown to HTML synchronously here.

    // If empty, show the welcome message immediately.
    if text.trim().is_empty() {
        let html_body_with_js = generate_test_html(params.wheel_js);

        // Generate syntax highlighting CSS and combine with theme CSS
        let theme_css = params.css.borrow().clone();
        let theme_mode = params.theme_mode.borrow().clone();
        let syntax_css = generate_syntax_highlighting_css(&theme_mode);
        let combined_css = format!(
            "{}\n\n/* Syntax Highlighting CSS */\n{}",
            theme_css, syntax_css
        );

        let html = crate::components::viewer::webkit6::wrap_html_document(
            &html_body_with_js,
            &combined_css,
            &theme_mode,
            None,
        );

        let base_uri = params.base_uri.map(|s| s.to_string());
        let webview = params.webview.clone();

        crate::components::viewer::webkit6::load_html_when_ready(&webview, html, base_uri);

        return;
    }

    // Non-empty: render HTML in the background.
    let html_options = params.html_options.clone();
    let theme_mode = params.theme_mode.borrow().clone();
    let theme_mode_for_render = theme_mode.clone();
    let wheel_js = params.wheel_js.to_string();
    let theme_css = params.css.borrow().clone();
    let syntax_css = generate_syntax_highlighting_css(&theme_mode);
    let base_uri = params.base_uri.map(|s| s.to_string());
    let webview = params.webview.clone();

    glib::spawn_future_local(async move {
        let rendered = gio::spawn_blocking(move || {
            parse_markdown_to_html_with_theme(&text, &html_options, &theme_mode_for_render)
        })
        .await;

        glib::idle_add_local_once(move || match rendered {
            Ok(html_body) => {
                let mut html_body_with_js = html_body;
                html_body_with_js.push_str(&wheel_js);

                let combined_css = format!(
                    "{}\n\n/* Syntax Highlighting CSS */\n{}",
                    theme_css, syntax_css
                );

                let html = crate::components::viewer::webkit6::wrap_html_document(
                    &html_body_with_js,
                    &combined_css,
                    &theme_mode,
                    None,
                );

                crate::components::viewer::webkit6::load_html_when_ready(&webview, html, base_uri);
            }
            Err(e) => {
                log::error!("[viewer] Background render task panicked: {:?}", e);
            }
        });
    });
}

/// Enhanced version that checks both GTK TextBuffer and DocumentBuffer to determine if welcome message should show
pub fn refresh_preview_content_smooth_with_doc_buffer(params: SmoothUpdateParams<'_>) {
    let text = params
        .buffer
        .text(
            &params.buffer.start_iter(),
            &params.buffer.end_iter(),
            false,
        )
        .to_string();

    // Keep the main thread responsive: do not render Markdown to HTML synchronously here.

    if text.trim().is_empty() {
        let html_body_with_js = generate_test_html(params.wheel_js);
        crate::components::viewer::webkit6::update_html_content_smooth(
            params.webview,
            &html_body_with_js,
        );
        return;
    }

    let html_options = params.html_options.clone();
    let wheel_js = params.wheel_js.to_string();
    let webview = params.webview.clone();

    glib::spawn_future_local(async move {
        let rendered =
            gio::spawn_blocking(move || parse_markdown_to_html(&text, &html_options)).await;

        glib::idle_add_local_once(move || match rendered {
            Ok(html_body) => {
                let mut html_body_with_js = html_body;
                html_body_with_js.push_str(&wheel_js);

                crate::components::viewer::webkit6::update_html_content_smooth(
                    &webview,
                    &html_body_with_js,
                );
            }
            Err(e) => {
                log::error!("[viewer] Background smooth render task panicked: {:?}", e);
            }
        });
    });
}
