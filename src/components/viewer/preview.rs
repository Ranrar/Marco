use crate::components::marco_engine::grammar::Rule;
use crate::components::marco_engine::render_html::{HtmlOptions, HtmlRenderer};
use crate::components::marco_engine::{AstBuilder, MarcoParser};
use gtk4::prelude::*;
use pest::Parser;
use std::cell::RefCell;
use webkit6::prelude::*;

/// Small helper to wrap markdown -> html and load into webview using the new rendering system.
pub fn refresh_preview_into_webview(
    webview: &webkit6::WebView,
    css: &RefCell<String>,
    html_options: &HtmlOptions,
    buffer: &sourceview5::Buffer,
    wheel_js: &str,
    theme_mode: &RefCell<String>,
) {
    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();

    // Debug: log the input text
    log::debug!("[viewer] Processing text: '{}'", text);

    // Simple test first - if it's empty or just whitespace, return early with a test message
    if text.trim().is_empty() {
        log::debug!("[viewer] Empty text, using test HTML");
        let test_html = "<h1>Test Header</h1><p>This is a test to ensure HTML rendering works.</p>";
        let mut html_with_js = test_html.to_string();
        html_with_js.push_str(wheel_js);
        let html = crate::components::viewer::webkit6::wrap_html_document(
            &html_with_js,
            &css.borrow(),
            &theme_mode.borrow(),
        );
        let html_clone = html.clone();
        let webview_idle = webview.clone();
        glib::idle_add_local(move || {
            log::debug!("[viewer] loading test html length={} ", html_clone.len());
            webview_idle.load_html(&html_clone, None);
            glib::ControlFlow::Break
        });
        return;
    }

    // Parse markdown using new marco engine
    let html_body = match MarcoParser::parse(Rule::document, &text) {
        Ok(pairs) => {
            // Debug: log what was parsed
            log::debug!(
                "[viewer] Parsed {} pairs from text: '{}'",
                pairs.len(),
                text.chars().take(50).collect::<String>()
            );

            // Debug: log each parsed pair
            let pairs_vec: Vec<_> = pairs.collect();
            for (i, pair) in pairs_vec.iter().enumerate() {
                log::debug!(
                    "[viewer] Pair {}: Rule={:?}, Text='{}'",
                    i,
                    pair.as_rule(),
                    pair.as_str()
                );
            }

            // Convert back to pairs for AST building
            let pairs_for_ast = MarcoParser::parse(Rule::document, &text).unwrap();

            match AstBuilder::build(pairs_for_ast) {
                Ok(ast) => {
                    log::debug!("[viewer] AST built successfully");
                    let renderer = HtmlRenderer::new(html_options.clone());
                    let html = renderer.render(&ast);
                    log::debug!(
                        "[viewer] HTML rendered: '{}'",
                        html.chars().take(100).collect::<String>()
                    );
                    html
                }
                Err(e) => {
                    log::error!("[viewer] Error building AST: {}", e);
                    format!("Error building AST: {}", e)
                }
            }
        }
        Err(e) => {
            log::error!("[viewer] Error parsing markdown: {}", e);
            format!("Error parsing markdown: {}", e)
        }
    };

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
