use crate::components::marco_engine::grammar::Rule;
use crate::components::marco_engine::render_html::{HtmlOptions, HtmlRenderer};
use crate::components::marco_engine::{AstBuilder, MarcoParser};
use gtk4::prelude::*;
use pest::Parser;
use std::cell::RefCell;
use webkit6::prelude::*;

/// Parameters for preview refresh operations
pub struct PreviewRefreshParams<'a> {
    pub webview: &'a webkit6::WebView,
    pub css: &'a RefCell<String>,
    pub html_options: &'a HtmlOptions,
    pub buffer: &'a sourceview5::Buffer,
    pub wheel_js: &'a str,
    pub theme_mode: &'a RefCell<String>,
    pub base_uri: Option<&'a str>,
    pub document_buffer: Option<&'a std::rc::Rc<std::cell::RefCell<crate::logic::buffer::DocumentBuffer>>>,
}

/// Simplified parameters for smooth content updates
pub struct SmoothUpdateParams<'a> {
    pub webview: &'a webkit6::WebView,
    pub html_options: &'a HtmlOptions,
    pub buffer: &'a sourceview5::Buffer,
    pub wheel_js: &'a str,
    pub document_buffer: Option<&'a std::rc::Rc<std::cell::RefCell<crate::logic::buffer::DocumentBuffer>>>,
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

/// Parse markdown text into HTML using the Marco engine with caching
fn parse_markdown_to_html(text: &str, html_options: &HtmlOptions) -> String {
    use crate::components::marco_engine::global_ast_cache;
    
    match global_ast_cache().parse_cached(text) {
        Ok(ast) => {
            log::debug!("[viewer] AST parsed and cached successfully");
            let renderer = HtmlRenderer::new(html_options.clone());
            let html = renderer.render(&ast);
            log::debug!(
                "[viewer] HTML rendered: '{}'",
                html.chars().take(100).collect::<String>()
            );
            html
        }
        Err(e) => {
            log::error!("[viewer] Error parsing markdown with cache: {}", e);
            
            // Fallback to non-cached parsing in case of cache issues
            log::info!("[viewer] Falling back to non-cached parsing");
            match MarcoParser::parse(Rule::document, text) {
                Ok(pairs) => {
                    match AstBuilder::build(pairs) {
                        Ok(ast) => {
                            log::debug!("[viewer] Fallback AST built successfully");
                            let renderer = HtmlRenderer::new(html_options.clone());
                            renderer.render(&ast)
                        }
                        Err(e) => {
                            log::error!("[viewer] Error building AST in fallback: {}", e);
                            format!("Error building AST: {}", e)
                        }
                    }
                }
                Err(e) => {
                    log::error!("[viewer] Error parsing markdown in fallback: {}", e);
                    format!("Error parsing markdown: {}", e)
                }
            }
        }
    }
}

/// Small helper to wrap markdown -> html and load into webview using the new rendering system.
/// If document_path is provided, it will be used to generate a base URI for resolving relative paths.
pub fn refresh_preview_into_webview(
    webview: &webkit6::WebView,
    css: &RefCell<String>,
    html_options: &HtmlOptions,
    buffer: &sourceview5::Buffer,
    wheel_js: &str,
    theme_mode: &RefCell<String>,
    document_path: Option<&std::path::Path>,
) {
    let base_uri = document_path.and_then(|path| {
        crate::components::viewer::webkit6::generate_base_uri_from_path(path)
    });
    refresh_preview_into_webview_with_base_uri(
        webview, css, html_options, buffer, wheel_js, theme_mode, base_uri.as_deref()
    );
}

/// Small helper to wrap markdown -> html and load into webview using the new rendering system.
/// If base_uri is provided, it will be used directly as the base URI for resolving relative paths.
pub fn refresh_preview_into_webview_with_base_uri(
    webview: &webkit6::WebView,
    css: &RefCell<String>,
    html_options: &HtmlOptions,
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
        document_buffer: None,
    };
    refresh_preview_into_webview_with_base_uri_and_doc_buffer(params);
}

    /// Enhanced version that checks both GTK TextBuffer and DocumentBuffer to determine if welcome message should show
    pub fn refresh_preview_into_webview_with_base_uri_and_doc_buffer(
        params: PreviewRefreshParams<'_>,
    ) {
    let text = params.buffer
        .text(&params.buffer.start_iter(), &params.buffer.end_iter(), false)
        .to_string();

    // Debug: log the input text (first 100 chars only to avoid massive logs)
    log::debug!("[viewer] Processing text ({} chars): '{}'", 
        text.len(), 
        text.chars().take(100).collect::<String>()
    );

    // Determine what content to show based on GTK TextBuffer and DocumentBuffer state
    let html_body_with_js = if text.trim().is_empty() {
        match params.document_buffer {
            Some(doc_buf) => {
                let doc_buf_borrowed = doc_buf.borrow();
                if doc_buf_borrowed.get_file_path().is_none() {
                    // Untitled document with empty GTK buffer -> show welcome message
                    log::debug!("[viewer] Empty GTK buffer, untitled document -> showing welcome message");
                    generate_test_html(params.wheel_js)
                } else {
                    // File document with empty GTK buffer -> try to read from DocumentBuffer
                    log::debug!("[viewer] Empty GTK buffer, but file loaded -> trying to read from DocumentBuffer");
                    match doc_buf_borrowed.read_content() {
                        Ok(file_content) if !file_content.trim().is_empty() => {
                            log::debug!("[viewer] Successfully read content from DocumentBuffer: {} chars", file_content.len());
                            let html_body = parse_markdown_to_html(&file_content, params.html_options);
                            let mut html_with_js = html_body;
                            html_with_js.push_str(params.wheel_js);
                            html_with_js
                        }
                        Ok(_) => {
                            // File exists but is empty
                            log::debug!("[viewer] File exists but is empty -> parsing empty content");
                            let html_body = parse_markdown_to_html("", params.html_options);
                            let mut html_with_js = html_body;
                            html_with_js.push_str(params.wheel_js);
                            html_with_js
                        }
                        Err(e) => {
                            log::error!("[viewer] Failed to read from DocumentBuffer: {}", e);
                            // Fallback to parsing empty text
                            let html_body = parse_markdown_to_html("", params.html_options);
                            let mut html_with_js = html_body;
                            html_with_js.push_str(params.wheel_js);
                            html_with_js
                        }
                    }
                }
            }
            None => {
                // No DocumentBuffer and empty GTK buffer -> show welcome message
                log::debug!("[viewer] No DocumentBuffer, empty GTK buffer -> showing welcome message");
                generate_test_html(params.wheel_js)
            }
        }
    } else {
        // GTK TextBuffer has content -> use it directly
        log::debug!("[viewer] GTK buffer has content -> using GTK buffer content");
        let html_body = parse_markdown_to_html(&text, params.html_options);
        let mut html_with_js = html_body;
        html_with_js.push_str(params.wheel_js);
        html_with_js
    };

    let html = crate::components::viewer::webkit6::wrap_html_document(
        &html_body_with_js,
        &params.css.borrow(),
        &params.theme_mode.borrow(),
    );
    let html_clone = html.clone();
    // Use the provided base URI directly (already converted to string)
    let base_uri_clone = params.base_uri.map(|s| s.to_string());
    let webview_idle = params.webview.clone();
    glib::idle_add_local(move || {
        log::debug!("[viewer] loading html length={} ", html_clone.len());
        if let Some(ref base_uri) = base_uri_clone {
            log::debug!("[viewer] using base URI: {}", base_uri);
            webview_idle.load_html(&html_clone, Some(base_uri));
        } else {
            webview_idle.load_html(&html_clone, None);
        }
        glib::ControlFlow::Break
    });
}

/// Enhanced version that checks both GTK TextBuffer and DocumentBuffer to determine if welcome message should show
pub fn refresh_preview_content_smooth_with_doc_buffer(
    params: SmoothUpdateParams<'_>,
) {
    let text = params.buffer
        .text(&params.buffer.start_iter(), &params.buffer.end_iter(), false)
        .to_string();

    // Debug: log the input text (first 100 chars only to avoid massive logs)
    log::debug!("[viewer] Processing text for smooth update ({} chars): '{}'", 
        text.len(), 
        text.chars().take(100).collect::<String>()
    );

    // Determine what content to show based on GTK TextBuffer and DocumentBuffer state
    let html_body_with_js = if text.trim().is_empty() {
        match params.document_buffer {
            Some(doc_buf) => {
                let doc_buf_borrowed = doc_buf.borrow();
                if doc_buf_borrowed.get_file_path().is_none() {
                    // Untitled document with empty GTK buffer -> show welcome message
                    log::debug!("[viewer] Smooth update: Empty GTK buffer, untitled document -> showing welcome message");
                    generate_test_html(params.wheel_js)
                } else {
                    // File document with empty GTK buffer -> try to read from DocumentBuffer
                    log::debug!("[viewer] Smooth update: Empty GTK buffer, but file loaded -> trying to read from DocumentBuffer");
                    match doc_buf_borrowed.read_content() {
                        Ok(file_content) if !file_content.trim().is_empty() => {
                            log::debug!("[viewer] Smooth update: Successfully read content from DocumentBuffer: {} chars", file_content.len());
                            let html_body = parse_markdown_to_html(&file_content, params.html_options);
                            let mut html_with_js = html_body;
                            html_with_js.push_str(params.wheel_js);
                            html_with_js
                        }
                        Ok(_) => {
                            // File exists but is empty
                            log::debug!("[viewer] Smooth update: File exists but is empty -> parsing empty content");
                            let html_body = parse_markdown_to_html("", params.html_options);
                            let mut html_with_js = html_body;
                            html_with_js.push_str(params.wheel_js);
                            html_with_js
                        }
                        Err(e) => {
                            log::error!("[viewer] Smooth update: Failed to read from DocumentBuffer: {}", e);
                            // Fallback to parsing empty text
                            let html_body = parse_markdown_to_html("", params.html_options);
                            let mut html_with_js = html_body;
                            html_with_js.push_str(params.wheel_js);
                            html_with_js
                        }
                    }
                }
            }
            None => {
                // No DocumentBuffer and empty GTK buffer -> show welcome message
                log::debug!("[viewer] Smooth update: No DocumentBuffer, empty GTK buffer -> showing welcome message");
                generate_test_html(params.wheel_js)
            }
        }
    } else {
        // GTK TextBuffer has content -> use it directly
        log::debug!("[viewer] Smooth update: GTK buffer has content -> using GTK buffer content");
        let html_body = parse_markdown_to_html(&text, params.html_options);
        let mut html_with_js = html_body;
        html_with_js.push_str(params.wheel_js);
        html_with_js
    };

    // Use smooth update - just update the content, not the entire page
    crate::components::viewer::webkit6::update_html_content_smooth(params.webview, &html_body_with_js);
}
