//! Preview Rendering Module for Marco
//!
//! This module handles rendering markdown content to HTML and displaying it in a WebView.
//! It provides functions for refreshing preview content with optimal performance using caching.
//!
//!
//! # Key Features
//!
//! - Async rendering to keep UI responsive
//! - Full HTML caching for optimal performance
//! - Theme-aware syntax highlighting
//! - Smooth content updates without page reloads
//! - Base URI support for relative file references
//!
//! # Cross-platform note
//!
//! The renderer is fully platform-agnostic — it dispatches all WebView I/O
//! through [`crate::components::viewer::backend`], whose Linux/Windows
//! implementations live in `webkit6`/`wry_platform_webview` respectively.
//!
//! Until the Windows refresh closure in `editor::ui` adopts this module
//! (planned Step 4b of the webkit6→wry parity work), several public items
//! here have no Windows caller. The module-level `dead_code` allow below
//! suppresses the resulting warnings without affecting the Linux build.
#![cfg_attr(target_os = "windows", allow(dead_code))]

use gtk4::prelude::*;
use marco_core::RenderOptions;
use marco_shared::cache::global_parser_cache;
use std::cell::RefCell;

use crate::components::viewer::backend;
use crate::components::viewer::preview_types::PageViewState;

/// Parameters for preview refresh operations
pub struct PreviewRefreshParams<'a> {
    pub webview: &'a backend::PreviewWebView,
    pub css: &'a RefCell<String>,
    pub html_options: &'a RenderOptions,
    pub buffer: &'a sourceview5::Buffer,
    pub wheel_js: &'a str,
    pub theme_mode: &'a RefCell<String>,
    pub base_uri: Option<&'a str>,
    /// When `Some`, paged.js page view is active with these settings.
    /// A full HTML reload is used; the smooth update path is bypassed.
    pub page_view: Option<std::rc::Rc<RefCell<PageViewState>>>,
}

/// Generate test HTML content when the editor is empty
fn generate_test_html(wheel_js: &str) -> String {
    let welcome_html = r#"<div id="welcome-message" style="
  text-align:center; 
  margin-top:20%; 
  opacity:0.7; 
  font-family:sans-serif;">
    <h1>Welcome to marco</h1>
  <p>Start typing or open a file to begin your writing journey ✍️</p>
</div>"#;
    let mut html_with_js = welcome_html.to_string();
    html_with_js.push_str(wheel_js);
    html_with_js
}

/// Generate CSS for syntax highlighting based on current theme mode
pub fn generate_syntax_highlighting_css(theme_mode: &str) -> String {
    use crate::logic::syntax_highlighter::{generate_css_with_global, global_syntax_highlighter};

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

/// Renders markdown to HTML and loads it into the webview, checking both GTK TextBuffer and DocumentBuffer to determine if welcome message should show
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

    // Capture page view state before entering async context
    let page_view_snapshot = params
        .page_view
        .as_ref()
        .map(|pv| pv.borrow().clone())
        .filter(|pv| pv.enabled);

    // If empty, show the welcome message immediately.
    if text.trim().is_empty() {
        // Generate syntax highlighting CSS and combine with theme CSS
        let theme_css = params.css.borrow().clone();
        let theme_mode = params.theme_mode.borrow().clone();
        let syntax_css = generate_syntax_highlighting_css(&theme_mode);
        let combined_css = format!(
            "{}\n\n/* Syntax Highlighting CSS */\n{}",
            theme_css, syntax_css
        );

        let base_uri = params.base_uri.map(|s| s.to_string());
        let webview = params.webview.clone();
        let wheel_js_str = params.wheel_js;

        if let Some(pv) = &page_view_snapshot {
            // Page view enabled — wrap welcome through paged.js so it renders
            // as a proper page instead of raw unstyled content.
            let welcome_body = generate_test_html("");
            let page_opts = marco_core::render::PageViewOptions {
                paged_js_source: crate::components::viewer::pagedjs::PAGED_POLYFILL_JS,
                paper: &pv.paper,
                orientation: &pv.orientation,
                margin_mm: pv.margin_mm,
                show_page_numbers: pv.show_page_numbers,
                wheel_js: wheel_js_str,
                columns_per_row: pv.columns_per_row,
                for_export: false,
                title: "",
                standalone_export: false,
            };
            let html = backend::wrap_html_document_paged(
                &welcome_body,
                &combined_css,
                &theme_mode,
                None,
                &page_opts,
            );
            backend::load_html_when_ready(&webview, html, base_uri);
        } else {
            let html_body_with_js = generate_test_html(wheel_js_str);
            let html =
                backend::wrap_html_document(&html_body_with_js, &combined_css, &theme_mode, None);
            backend::load_html_when_ready(&webview, html, base_uri);
        }

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
                let combined_css = format!(
                    "{}\n\n/* Syntax Highlighting CSS */\n{}",
                    theme_css, syntax_css
                );

                if let Some(pv) = page_view_snapshot {
                    // Page view mode: inject paged.js for true CSS Paged Media simulation.
                    // Full HTML reload required — smooth updates are incompatible with paged.js.
                    let page_opts = marco_core::render::PageViewOptions {
                        paged_js_source: crate::components::viewer::pagedjs::PAGED_POLYFILL_JS,
                        paper: &pv.paper,
                        orientation: &pv.orientation,
                        margin_mm: pv.margin_mm,
                        show_page_numbers: pv.show_page_numbers,
                        wheel_js: &wheel_js,
                        columns_per_row: pv.columns_per_row,
                        for_export: false,
                        title: "",
                        standalone_export: false,
                    };
                    let html = backend::wrap_html_document_paged(
                        &html_body,
                        &combined_css,
                        &theme_mode,
                        None,
                        &page_opts,
                    );
                    backend::load_html_when_ready(&webview, html, base_uri);
                } else {
                    // Normal mode: wrap with wheel JS for scroll sync, then load.
                    let mut html_body_with_js = html_body;
                    html_body_with_js.push_str(&wheel_js);
                    let html = backend::wrap_html_document(
                        &html_body_with_js,
                        &combined_css,
                        &theme_mode,
                        None,
                    );
                    backend::load_html_when_ready(&webview, html, base_uri);
                }
            }
            Err(e) => {
                log::error!("[viewer] Background render task panicked: {:?}", e);
            }
        });
    });
}

/// Parameters for section-based incremental preview updates.
///
/// Used when a large document is open and only the section under the
/// cursor has changed.  Splitting and rendering both happen off the
/// main thread inside `gio::spawn_blocking`.
///
/// **Not** used when paged.js page-view is active (paged.js is
/// incompatible with partial DOM patches).
pub struct SectionRenderParams {
    pub webview: backend::PreviewWebView,
    pub html_options: RenderOptions,
    pub wheel_js: String,
    pub theme_mode: String,
    /// Full document text — sections are split inside spawn_blocking.
    pub text: String,
    /// Content hashes from the previous section render.
    /// Empty → first render for this document (full DOM rebuild required).
    pub prev_hashes: Vec<u64>,
    /// Combined CSS (theme + syntax highlighting) for wrapping the full rebuild page.
    pub css: String,
    /// Base URI for WebKit resource loading (e.g. images relative to the document).
    pub base_uri: Option<String>,
    /// 0-based line number of the editor cursor.  The section containing this
    /// line is applied first (in the same idle callback) so the user sees their
    /// own edits reflected instantly; all other changed sections follow in the
    /// next idle frame.
    pub cursor_line: usize,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Escape a string for embedding as a JS single-quoted string literal.
fn escape_for_js_string(s: &str) -> String {
    // Worst case: every character needs escaping (2× expansion).
    let mut out = String::with_capacity(s.len() * 2);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

/// Outcome produced off-thread and applied on the main thread.
enum SectionPayload {
    /// Full DOM rebuild: used only for the very first render (no DOM exists yet).
    /// Passed to `load_html_when_ready`.
    Full(String),
    /// Targeted innerHTML patches for sections whose content changed.
    /// The cursor section is applied in the current idle frame; the rest
    /// follow in the next frame (`rest_js` is `None` when only one section
    /// changed or when cursor is the only change).
    PrioritizedPatch {
        cursor_js: String,
        rest_js: Option<String>,
    },
    /// DOM morph: handles section count changes (heading added / removed)
    /// without a full page reload.  The JS removes deleted divs, renames
    /// shifted divs, and inserts new divs — only new/changed sections carry
    /// HTML payload; unchanged sections are reused from the live DOM.
    Morph(String),
    /// All sections unchanged — nothing to do.
    NoChange,
}

/// Compute what DOM update is needed for `text`, given `prev_hashes` from
/// the last render.  Runs entirely off the main thread.
fn compute_section_payload(
    text: &str,
    wheel_js: &str,
    prev_hashes: &[u64],
    opts: &RenderOptions,
    cursor_line: usize,
) -> (Vec<u64>, SectionPayload) {
    use marco_shared::cache::section_for_line;

    let sections = marco_shared::cache::split_into_sections(text);
    let new_hashes: Vec<u64> = sections.iter().map(|s| s.content_hash).collect();

    let html_arcs = match global_parser_cache().render_sections_with_cache(&sections, opts) {
        Ok(arcs) => arcs,
        Err(e) => {
            log::warn!("[viewer] section render error: {e}");
            return (new_hashes, SectionPayload::NoChange);
        }
    };

    // Full rebuild on first render (no DOM to patch yet).
    if prev_hashes.is_empty() {
        let cap = html_arcs.iter().map(|a| a.len()).sum::<usize>()
            + sections.len() * 22
            + wheel_js.len()
            + 64;
        let mut body = String::with_capacity(cap);
        for (i, arc) in html_arcs.iter().enumerate() {
            body.push_str("<div id=\"mc-s-");
            body.push_str(&i.to_string());
            body.push_str("\">");
            body.push_str(arc.as_str());
            body.push_str("</div>\n");
        }
        body.push_str(wheel_js);
        return (new_hashes, SectionPayload::Full(body));
    }

    // Section count changed: morph the existing DOM without a page reload.
    // Diff strategy: find the longest common prefix and suffix of old vs new
    // section hashes.  Sections in the common prefix/suffix are simply
    // renamed (their id may shift due to insertions/deletions); only the
    // "middle" sections need HTML payload embedded in the JS.
    if prev_hashes.len() != sections.len() {
        let n_old = prev_hashes.len();
        let n_new = sections.len();

        // Common prefix length.
        let prefix_len = prev_hashes
            .iter()
            .zip(new_hashes.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Common suffix length (capped to avoid overlap with prefix).
        let max_suffix = std::cmp::min(n_old - prefix_len, n_new - prefix_len);
        let suffix_len = prev_hashes[prefix_len..]
            .iter()
            .rev()
            .zip(new_hashes[prefix_len..].iter().rev())
            .take(max_suffix)
            .take_while(|(a, b)| a == b)
            .count();

        // Index of the first suffix section in the OLD array.
        let old_suffix_start = n_old - suffix_len;
        // Index of the first suffix section in the NEW array.
        let new_suffix_start = n_new - suffix_len;
        // How much the suffix section indices must shift.
        let index_delta: i64 = n_new as i64 - n_old as i64;

        let mut js = String::with_capacity(512 + n_new.abs_diff(n_old) * 512);
        js.push_str("(function(){");

        // Save a reference to the first suffix div BEFORE renaming so we
        // have a stable anchor for insertions.
        if suffix_len > 0 {
            js.push_str(&format!(
                "var _ref=document.getElementById('mc-s-{}');",
                old_suffix_start
            ));
        } else {
            // No suffix: insert at end of body.
            js.push_str("var _ref=null;");
        }
        js.push_str("var _par=_ref?_ref.parentNode:document.body;");

        // Remove old "middle" sections (those with no counterpart in new).
        for old_idx in prefix_len..old_suffix_start {
            js.push_str(&format!(
                "var _d{i}=document.getElementById('mc-s-{i}');if(_d{i})_d{i}.parentNode.removeChild(_d{i});",
                i = old_idx
            ));
        }

        // Rename suffix sections in REVERSE order to avoid transient ID
        // collisions (renaming mc-s-3 → mc-s-4 before mc-s-4 → mc-s-5).
        if index_delta != 0 {
            for old_idx in (old_suffix_start..n_old).rev() {
                let new_idx = (old_idx as i64 + index_delta) as usize;
                js.push_str(&format!(
                    "var _r{i}=document.getElementById('mc-s-{i}');if(_r{i})_r{i}.id='mc-s-{ni}';",
                    i = old_idx,
                    ni = new_idx
                ));
            }
        }

        // Insert new "middle" sections before _ref (which now has the new id
        // of the first suffix section, or is null if no suffix).
        for (new_idx, html_arc) in html_arcs[prefix_len..new_suffix_start].iter().enumerate() {
            let new_idx = prefix_len + new_idx;
            let html = escape_for_js_string(html_arc.as_str());
            js.push_str("var _n=document.createElement('div');");
            js.push_str(&format!("_n.id='mc-s-{}';", new_idx));
            js.push_str(&format!("_n.innerHTML='{}';", html));
            js.push_str("_par.insertBefore(_n,_ref);");
            js.push_str("if(typeof window.renderMathInElement==='function'){try{window.renderMathInElement(_n);}catch(ex){}}");
            js.push_str("if(typeof window.mermaid!=='undefined'){try{window.mermaid.init(undefined,_n.querySelectorAll('.mermaid'));}catch(ex){}}");
        }

        js.push_str("})();");
        return (new_hashes, SectionPayload::Morph(js));
    }

    // Find which sections actually changed.
    let changed_indices: Vec<usize> = sections
        .iter()
        .enumerate()
        .filter(|(i, s)| prev_hashes.get(*i).copied() != Some(s.content_hash))
        .map(|(i, _)| i)
        .collect();

    if changed_indices.is_empty() {
        return (new_hashes, SectionPayload::NoChange);
    }

    // Build a JS snippet for the given subset of section indices.
    let make_patch_js = |indices: &[usize]| -> String {
        let mut js = String::with_capacity(indices.len() * 256 + 128);
        js.push_str("(function(){var us=[");
        for (pos, &i) in indices.iter().enumerate() {
            if pos > 0 {
                js.push(',');
            }
            js.push_str("[\"mc-s-");
            js.push_str(&i.to_string());
            js.push_str("\",\'");
            js.push_str(&escape_for_js_string(html_arcs[i].as_str()));
            js.push_str("\']");
        }
        js.push_str("];for(var i=0;i<us.length;i++){");
        js.push_str("var e=document.getElementById(us[i][0]);");
        js.push_str("if(e){e.innerHTML=us[i][1];");
        js.push_str("if(typeof window.renderMathInElement==='function'){try{window.renderMathInElement(e);}catch(ex){}}");
        js.push_str("if(typeof window.mermaid!=='undefined'){try{window.mermaid.init(undefined,e.querySelectorAll('.mermaid'));}catch(ex){}}");
        js.push_str("}};})();");
        js
    };

    // Cursor-section-first: if the cursor's section changed, apply it in the
    // current idle frame; schedule all other changed sections for the next
    // idle frame so the user sees their own edit reflected instantly.
    let cursor_section = section_for_line(&sections, cursor_line);
    let cursor_changed = changed_indices.contains(&cursor_section);

    if cursor_changed && changed_indices.len() > 1 {
        let cursor_js = make_patch_js(&[cursor_section]);
        let rest: Vec<usize> = changed_indices
            .iter()
            .filter(|&&i| i != cursor_section)
            .copied()
            .collect();
        let rest_js = make_patch_js(&rest);
        (
            new_hashes,
            SectionPayload::PrioritizedPatch {
                cursor_js,
                rest_js: Some(rest_js),
            },
        )
    } else {
        // Single change or cursor section not changed: one-shot patch.
        let js = make_patch_js(&changed_indices);
        (
            new_hashes,
            SectionPayload::PrioritizedPatch {
                cursor_js: js,
                rest_js: None,
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Section-based incremental preview update.
///
/// On the **first render** (or when the section structure changes), wraps all
/// sections in `<div id="mc-s-N">` elements and pushes the full HTML via
/// `update_html_content_smooth`.
///
/// On **subsequent renders**, only the sections whose content hash changed are
/// re-rendered and the DOM is patched with a targeted JavaScript snippet via
/// `evaluate_javascript`.  For a 19K-line document this reduces the typical
/// JS payload from ~5 MB to ~1 KB.
///
/// `on_complete` is called on the GLib main thread with the new section
/// hashes.  Store them in `prev_hashes` for the next call.
pub fn refresh_preview_content_sections(
    params: SectionRenderParams,
    on_complete: impl FnOnce(Vec<u64>) + 'static,
) {
    let SectionRenderParams {
        webview,
        html_options,
        wheel_js,
        theme_mode,
        text,
        prev_hashes,
        css,
        base_uri,
        cursor_line,
    } = params;

    if text.trim().is_empty() {
        let html_body_with_js = generate_test_html(&wheel_js);
        backend::update_html_content_smooth(&webview, &html_body_with_js);
        on_complete(Vec::new());
        return;
    }

    let opts = RenderOptions {
        theme: theme_mode.clone(),
        ..html_options
    };

    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(move || {
            compute_section_payload(&text, &wheel_js, &prev_hashes, &opts, cursor_line)
        })
        .await;

        glib::idle_add_local_once(move || match result {
            Ok((new_hashes, payload)) => {
                match payload {
                    SectionPayload::Full(body) => {
                        // Full rebuild: use load_html (efficient WebKit page load) instead
                        // of evaluate_javascript.  update_html_content_smooth embeds the
                        // escaped content 3× in a format! string — for a 5 MB body that
                        // produces a 15-30 MB JS string on the main thread and blocks GTK.
                        log::debug!(
                            "[viewer] Section full rebuild ({} sections)",
                            new_hashes.len()
                        );
                        // Save the current scroll position before the page reloads so
                        // the SCROLL_RESTORE_JS injected in the new page can restore it.
                        backend::evaluate_javascript(
                            &webview,
                            "try{sessionStorage.setItem('marco-scroll',String(Math.round(window.scrollY)));}catch(e){}",
                        );
                        let full_html = backend::wrap_html_document(&body, &css, &theme_mode, None);
                        backend::load_html_when_ready(&webview, full_html, base_uri);
                        on_complete(new_hashes);
                    }
                    SectionPayload::PrioritizedPatch { cursor_js, rest_js } => {
                        // Apply the cursor section immediately so the user sees
                        // their own edit reflected in the same frame.
                        backend::evaluate_javascript(&webview, &cursor_js);
                        if let Some(rest) = rest_js {
                            // Defer on_complete until rest_js fires so in_flight
                            // stays true through both frames.  Without this, the
                            // guard is released between the two frames and a new
                            // render's cursor_js can be overwritten by this
                            // render's rest_js when it fires one frame later.
                            let webview_clone = webview.clone();
                            glib::idle_add_local_once(move || {
                                backend::evaluate_javascript(&webview_clone, &rest);
                                on_complete(new_hashes);
                            });
                        } else {
                            on_complete(new_hashes);
                        }
                    }
                    SectionPayload::Morph(js) => {
                        // Section count changed but we morph the DOM in place
                        // instead of reloading the page — scroll position is
                        // preserved and there is no white-flash.
                        log::debug!("[viewer] Section morph → {} sections", new_hashes.len());
                        backend::evaluate_javascript(&webview, &js);
                        on_complete(new_hashes);
                    }
                    SectionPayload::NoChange => {
                        on_complete(new_hashes);
                    }
                }
            }
            Err(e) => {
                log::error!("[viewer] Section render task panicked: {:?}", e);
                // Always call on_complete so the in-flight guard is reset and
                // prev_section_hashes is cleared, forcing a full rebuild on the
                // next render instead of leaving the preview permanently frozen.
                on_complete(Vec::new());
            }
        });
    });
}
