use gtk4::prelude::*;
use std::cell::RefCell;
use webkit6::prelude::*;

/// Small helper to wrap markdown -> html and load into webview.
pub fn refresh_preview_into_webview(
    webview: &webkit6::WebView,
    css: &RefCell<String>,
    markdown_opts: &crate::components::marco_engine::render::MarkdownOptions,
    buffer: &sourceview5::Buffer,
    wheel_js: &str,
    theme_mode: &RefCell<String>,
) {
    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    let html_body = crate::components::marco_engine::render::markdown_to_html(&text, markdown_opts);
    let mut html_body_with_js = html_body.clone();
    html_body_with_js.push_str(wheel_js);
    let html = crate::components::viewer::webkit6::wrap_html_document(
        &html_body_with_js,
        &css.borrow(),
        &theme_mode.borrow(),
    );
    let html_clone = html.clone();
    let webview_idle = webview.clone();
    glib::idle_add_local(move || {
        // Debug: log load via logging framework rather than printing to terminal
        log::debug!("[viewer] loading html length={} ", html_clone.len());
        webview_idle.load_html(&html_clone, None);
        glib::ControlFlow::Break
    });
}
