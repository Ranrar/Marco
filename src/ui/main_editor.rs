use crate::components::marco_engine::parser::MarkdownSyntaxMap;
use crate::footer::{FooterLabels, FooterUpdate};
// No need to import source_remove; use SourceId::remove()
use gtk4::glib::ControlFlow;
/// Wires up debounced footer updates to buffer events
pub fn wire_footer_updates(
    buffer: &sourceview5::Buffer,
    labels: Rc<FooterLabels>,
    syntax_map: Rc<std::cell::RefCell<Option<MarkdownSyntaxMap>>>,
    insert_mode_state: Rc<RefCell<bool>>,
) {
    use std::cell::Cell;
    let debounce_ms = 300;

    // State variable for insert/overwrite mode (true = insert, false = overwrite)
    // Accept insert_mode_state as an argument instead of creating it here

    // Separate timeout IDs for each event type to avoid conflicts
    let buffer_timeout_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
    let cursor_timeout_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

    let update_footer = {
        let buffer = buffer.clone();
        let syntax_map = Rc::clone(&syntax_map);
        let insert_mode_state = Rc::clone(&insert_mode_state);
        move || {
            crate::footer_dbg!("[wire_footer_updates] update_footer closure called");
            // Gather snapshot of footer data
            let offset = buffer.cursor_position();
            let iter = buffer.iter_at_offset(offset);
            let row = (iter.line() + 1) as usize;
            let col = (iter.line_offset() + 1) as usize;
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string();
            let word_count = text.split_whitespace().filter(|w| !w.is_empty()).count();
            let char_count = text.chars().count();
            crate::footer_dbg!(
                "[wire_footer_updates] Calculated stats - Row: {}, Col: {}, Words: {}, Chars: {}",
                row,
                col,
                word_count,
                char_count
            );
            // Syntax trace for current line
            let current_line = iter.line();
            let start_iter_opt = buffer.iter_at_line(current_line);
            let end_iter_opt = buffer.iter_at_line(current_line + 1);
            let line_text = match (start_iter_opt, end_iter_opt) {
                (Some(ref start), Some(ref end)) => buffer.text(start, end, false).to_string(),
                (Some(ref start), None) => {
                    buffer.text(start, &buffer.end_iter(), false).to_string()
                }
                _ => String::new(),
            };
            let syntax_display = if let Some(ref map) = *syntax_map.borrow() {
                crate::footer::format_syntax_trace(&line_text, map)
            } else {
                let dummy_map = crate::components::marco_engine::parser::MarkdownSyntaxMap {
                    rules: std::collections::HashMap::new(),
                    display_hints: None,
                };
                crate::footer::format_syntax_trace(&line_text, &dummy_map)
            };

            let is_insert = *insert_mode_state.borrow();
            let msg = FooterUpdate::Snapshot {
                row,
                col,
                words: word_count,
                chars: char_count,
                syntax_display,
                encoding: "UTF-8".to_string(),
                is_insert,
            };
            crate::footer_dbg!(
                "[wire_footer_updates] About to call apply_footer_update with: {:?}",
                msg
            );
            // Apply directly on the main context: wire_footer_updates runs in main-loop callbacks
            crate::footer::apply_footer_update(&labels, msg);
        }
    };

    // Debounce logic for buffer changes
    let buffer_timeout_clone = Rc::clone(&buffer_timeout_id);
    let update_footer_clone = update_footer.clone();
    buffer.connect_changed(move |_| {
        crate::footer_dbg!("[wire_footer_updates] Buffer changed event triggered");
        // Cancel existing timeout if any (safe - ignore errors if already removed)
        if let Some(id) = buffer_timeout_clone.replace(None) {
            id.remove();
        }
        let buffer_timeout_clone_inner = Rc::clone(&buffer_timeout_clone);
        let update_footer_clone = update_footer_clone.clone();
        let id =
            glib::timeout_add_local(std::time::Duration::from_millis(debounce_ms), move || {
                crate::footer_dbg!(
                    "[wire_footer_updates] Debounced buffer change timeout executing"
                );
                // Clear the timeout ID since we're executing now
                buffer_timeout_clone_inner.set(None);
                update_footer_clone();
                ControlFlow::Break
            });
        buffer_timeout_clone.set(Some(id));
    });

    // Debounce logic for cursor position changes using buffer notify signal
    let cursor_timeout_clone = Rc::clone(&cursor_timeout_id);
    let update_footer_clone2 = update_footer.clone();
    buffer.connect_notify_local(Some("cursor-position"), move |_, _| {
        crate::footer_dbg!("[wire_footer_updates] Cursor position change event triggered");
        // Cancel existing timeout if any (safe - ignore errors if already removed)
        if let Some(id) = cursor_timeout_clone.replace(None) {
            id.remove();
        }
        let cursor_timeout_clone_inner = Rc::clone(&cursor_timeout_clone);
        let update_footer_clone2 = update_footer_clone2.clone();
        let id =
            glib::timeout_add_local(std::time::Duration::from_millis(debounce_ms), move || {
                crate::footer_dbg!(
                    "[wire_footer_updates] Debounced cursor position timeout executing"
                );
                // Clear the timeout ID since we're executing now
                cursor_timeout_clone_inner.set(None);
                update_footer_clone2();
                ControlFlow::Break
            });
        cursor_timeout_clone.set(Some(id));
    });

    // Initial update (send snapshot)
    crate::footer_dbg!("[wire_footer_updates] Calling initial footer update");
    update_footer();
}
use crate::components::marco_engine::render::{markdown_to_html, MarkdownOptions};
use crate::ui::html_viewer::wrap_html_document;
use gtk4::Paned;
use std::cell::RefCell;
/// Create a split editor with live HTML preview using WebKit6
use std::rc::Rc;
/// This is the markdown editor
use webkit6::prelude::*;
type EditorReturn = (
    Paned,
    webkit6::WebView,
    Rc<RefCell<String>>,
    Box<dyn Fn()>,
    Box<dyn Fn(&str)>,
    Box<dyn Fn(&str)>,
    sourceview5::Buffer,
    Rc<RefCell<bool>>,
);

pub fn create_editor_with_preview(
    preview_theme_filename: &str,
    preview_theme_dir: &str,
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    theme_mode: Rc<RefCell<String>>,
    labels: Rc<FooterLabels>,
) -> EditorReturn {
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(600);

    // Get style scheme and font settings from ThemeManager
    let (style_scheme, font_family, font_size_pt) = {
        let tm = theme_manager.borrow();
        let style_scheme = tm.current_editor_scheme();
        let font_family = tm
            .settings
            .appearance
            .as_ref()
            .and_then(|a| a.ui_font.as_deref())
            .unwrap_or("Fira Mono")
            .to_string(); // default font name
        let font_size_pt = tm
            .settings
            .appearance
            .as_ref()
            .and_then(|a| a.ui_font_size)
            .map(|v| v as f64)
            .unwrap_or(10.0); // default font size
        (style_scheme, font_family, font_size_pt)
    };

    // Editor (left)
    // Retrieve current editor scheme id so we can choose appropriate scrollbar colors
    let scheme_id = theme_manager.borrow().current_editor_scheme_id();
    let (editor_widget, buffer, source_view, scrolled_css_provider) = render_editor_with_view(
        &scheme_id,
        style_scheme.as_ref(),
        &font_family,
        font_size_pt,
    );
    editor_widget.set_hexpand(true);
    editor_widget.set_vexpand(true);
    paned.set_start_child(Some(&editor_widget));

    // Insert/overwrite mode state
    let insert_mode_state: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));

    // Wire up key event handler for Insert key using EventControllerKey
    use gtk4::gdk::Key;
    use gtk4::glib::Propagation;
    let event_controller = gtk4::EventControllerKey::new();
    let insert_mode_state_clone = Rc::clone(&insert_mode_state);
    let labels_clone = Rc::clone(&labels);
    let source_view_clone = source_view.clone();
    event_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
        if keyval == Key::Insert {
            let mut mode = insert_mode_state_clone.borrow_mut();
            *mode = !*mode;
            // Set overwrite mode in the editor view
            source_view_clone.set_overwrite(!*mode); // overwrite=true when mode==false
            crate::footer::update_insert_mode(&labels_clone, *mode);
            return Propagation::Stop;
        }
        Propagation::Proceed
    });
    source_view.add_controller(event_controller.upcast::<gtk4::EventController>());

    // Load the current HTML preview theme CSS
    use std::fs;
    use std::path::Path;
    let css_path = Path::new(preview_theme_dir).join(preview_theme_filename);
    let mut css = fs::read_to_string(&css_path)
        .unwrap_or_else(|_| String::from("body { background: #fff; color: #222; }"));

    // Helper to generate webkit scrollbar CSS given thumb/track colors
    fn webkit_scrollbar_css(thumb: &str, track: &str) -> String {
        format!(
            r#"
        /* Match editor scrollbar styling for WebView */
        ::-webkit-scrollbar {{ width: 12px; height: 12px; background: {track}; }}
        ::-webkit-scrollbar-track {{ background: {track}; }}
    ::-webkit-scrollbar-thumb {{ background: {thumb}; border-radius: 0px; }}
    ::-webkit-scrollbar-thumb:hover {{ background: {thumb}; opacity: 0.9; }}
        "#,
            thumb = thumb,
            track = track
        )
    }

    // WebView (right)
    // Initially, try to pick scrollbar colors from the editor theme XML so
    // preview scrollbar matches editor. Fallback to light defaults.
    let mut initial_thumb = String::from("#D0D4D8");
    let mut initial_track = String::from("#F0F0F0");
    let editor_dir = theme_manager.borrow().editor_theme_dir.clone();
    if editor_dir.exists() && editor_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&editor_dir) {
            let scheme_id = theme_manager.borrow().current_editor_scheme_id();
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.eq_ignore_ascii_case("xml"))
                    .unwrap_or(false)
                {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        let id_search = format!("id=\"{}\"", scheme_id);
                        if contents.contains(&id_search) {
                            if let Some(v) = extract_xml_color_value(&contents, "scrollbar-thumb") {
                                initial_thumb = v;
                            }
                            if let Some(v) = extract_xml_color_value(&contents, "scrollbar-track") {
                                initial_track = v;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    // Prepare wheel-normalizer JS to make WebView scroll speed match GTK.
    let scroll_scale: f64 = std::env::var("MARCO_SCROLL_SCALE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);
    let wheel_js = format!(
        r#"<script>
    (function(){{
        const scale = {scale};
        function findScroll(el){{
            while(el && el !== document){{
                if (el.scrollHeight > el.clientHeight) return el;
                el = el.parentNode;
            }}
            return document.scrollingElement || document.documentElement || document.body;
        }}
        window.addEventListener('wheel', function(e){{
            if (Math.abs(e.deltaY) < 0.0001) return;
            const sc = findScroll(e.target);
            sc.scrollBy({{ top: e.deltaY * scale, left: e.deltaX * scale, behavior: 'auto' }});
            e.preventDefault();
        }}, {{ passive: false }});
    }})();
    </script>"#,
        scale = scroll_scale
    );
    let wheel_js_rc = Rc::new(wheel_js);

    // Append webkit scrollbar CSS so the WebView preview uses the same colors
    css.push_str(&webkit_scrollbar_css(&initial_thumb, &initial_track));

    // Initial HTML: include wheel JS so scroll behavior is normalized
    let initial_html = wrap_html_document(&wheel_js_rc, &css, &theme_mode.borrow());
    let webview = crate::ui::html_viewer::create_html_viewer(&initial_html);
    paned.set_end_child(Some(&webview));

    // Pointer debug printing removed to avoid terminal output in normal runs.
    // To re-enable pointer debug output, set MARCO_DEBUG_POINTERS and
    // implement a logging facility that writes to a file or logger instead
    // of directly to stderr.

    // Shared state for refresh
    let buffer_rc: Rc<sourceview5::Buffer> = Rc::new(buffer);
    let css_rc = Rc::new(RefCell::new(css));
    let webview_rc = Rc::new(webview.clone());
    let theme_mode_rc = Rc::clone(&theme_mode);
    // Clone source_view for optional debug inspection in closures
    let _source_view_dbg = source_view.clone();

    // Prepare markdown options with GFM extensions enabled
    let mut markdown_opts = MarkdownOptions::default();
    markdown_opts.extension.table = true;
    markdown_opts.extension.autolink = true;
    markdown_opts.extension.strikethrough = true;
    markdown_opts.extension.tasklist = true;
    markdown_opts.extension.footnotes = true;
    markdown_opts.extension.tagfilter = true;
    // Share options across closures
    let markdown_opts_rc = std::rc::Rc::new(markdown_opts);

    // Closure to refresh preview
    // Clone wheel_js for the refresh_preview closure so we don't move the
    // original Rc into multiple closures.
    let wheel_js_for_refresh = wheel_js_rc.clone();
    let refresh_preview = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let webview = Rc::clone(&webview_rc);
        let theme_mode = Rc::clone(&theme_mode_rc);
        let markdown_opts = std::rc::Rc::clone(&markdown_opts_rc);
        let wheel_js_local = wheel_js_for_refresh.clone();
        move || {
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string();
            let html_body = markdown_to_html(&text, &markdown_opts);
            // Append wheel JS so preview scroll behavior matches GTK
            let mut html_body_with_js = html_body.clone();
            html_body_with_js.push_str(&wheel_js_local);
            let html = wrap_html_document(&html_body_with_js, &css.borrow(), &theme_mode.borrow());
            // Defer the actual load to idle and ensure the webview is mapped first
            let html_clone = html.clone();
            let webview_idle = webview.clone();
            glib::idle_add_local(move || {
                if webview_idle.is_mapped() {
                    webview_idle.load_html(&html_clone, None);
                } else if std::env::var("MARCO_DEBUG_WEBVIEW_LOAD").is_ok() {
                    // Debug message intentionally suppressed from terminal.
                }
                glib::ControlFlow::Break
            });
        }
    };

    // Live update: on buffer change, re-render and update WebView
    let css_clone = Rc::clone(&css_rc);
    let theme_mode = Rc::clone(&theme_mode_rc);
    let webview_clone = Rc::clone(&webview_rc);
    let buffer_for_signal = Rc::clone(&buffer_rc);
    {
        // Debounced preview refresh: cancel previous timeout and schedule a short delay
        let markdown_opts = std::rc::Rc::clone(&markdown_opts_rc);
        let wheel_js_for_signal = wheel_js_rc.clone();
        let preview_timeout_id: Rc<std::cell::Cell<Option<glib::SourceId>>> =
            Rc::new(std::cell::Cell::new(None));
        let preview_timeout_id_cloned = Rc::clone(&preview_timeout_id);
        buffer_for_signal.connect_changed(move |buf| {
            // Clone buffer handle for use inside timeout closure (must be 'static)
            let buf_clone = buf.clone();
            // cancel previous
            if let Some(id) = preview_timeout_id_cloned.replace(None) {
                id.remove();
            }
            let preview_timeout_id_inner = Rc::clone(&preview_timeout_id_cloned);
            let css_clone_inner = css_clone.clone();
            let theme_mode_inner = theme_mode.clone();
            let webview_inner = webview_clone.clone();
            let markdown_opts_inner = markdown_opts.clone();
            let wheel_js_local = wheel_js_for_signal.clone();
            // schedule timeout
            let id = glib::timeout_add_local(std::time::Duration::from_millis(400), move || {
                let text = buf_clone
                    .text(&buf_clone.start_iter(), &buf_clone.end_iter(), false)
                    .to_string();
                let html_body = markdown_to_html(&text, &markdown_opts_inner);
                let mut html_body_with_js = html_body.clone();
                html_body_with_js.push_str(&wheel_js_local);
                let html = wrap_html_document(
                    &html_body_with_js,
                    &css_clone_inner.borrow(),
                    &theme_mode_inner.borrow(),
                );
                if std::env::var("MARCO_DEBUG_WEBVIEW_LOAD").is_ok() {
                    // Debug message intentionally suppressed from terminal.
                }
                let html_clone = html.clone();
                let webview_idle = webview_inner.clone();
                glib::idle_add_local(move || {
                    if webview_idle.is_mapped() {
                        webview_idle.load_html(&html_clone, None);
                    } else if std::env::var("MARCO_DEBUG_WEBVIEW_LOAD").is_ok() {
                        // Debug message intentionally suppressed from terminal.
                    }
                    glib::ControlFlow::Break
                });
                preview_timeout_id_inner.set(None);
                glib::ControlFlow::Break
            });
            preview_timeout_id_cloned.set(Some(id));
        });
    }

    // Create theme update function for editor
    let buffer_for_theme = Rc::clone(&buffer_rc);
    let theme_manager_clone = Rc::clone(&theme_manager);
    // Update editor style scheme and reload editor scrollbar CSS to match the scheme
    let scrolled_provider_clone = scrolled_css_provider.clone();
    let theme_manager_for_scheme = Rc::clone(&theme_manager_clone);
    // Ensure update_theme can be called multiple times: capture only Rc clones
    // and avoid moving non-cloneable things into the closure so it implements
    // Fn, not FnOnce.
    let scrolled_provider_for_update = scrolled_provider_clone.clone();
    let css_rc_for_update = css_rc.clone();
    let webview_rc_for_update = webview_rc.clone();
    let buffer_for_update = buffer_for_theme.clone();
    let theme_mode_for_update = theme_mode_rc.clone();
    let markdown_opts_for_update = std::rc::Rc::clone(&markdown_opts_rc);
    let wheel_js_for_update = wheel_js_rc.clone();
    let update_theme = Box::new(move |scheme_id: &str| {
        if let Some(scheme) = theme_manager_for_scheme
            .borrow()
            .get_editor_scheme(scheme_id)
        {
            buffer_for_theme.set_style_scheme(Some(&scheme));
            // Applied theme message suppressed in normal startup. Enable debugging with MARCO_DEBUG_THEME=1
            if std::env::var("MARCO_DEBUG_THEME").is_ok() {
                // Theme applied debug output suppressed from terminal.
            }
        } else {
            eprintln!("Failed to find style scheme: {}", scheme_id);
        }

        // Attempt to read scrollbar color tokens from the editor XML file.
        // Many style-scheme files may use an internal id that doesn't match the
        // filename. Search each XML in editor_theme_dir for a <style-scheme
        // id="..."> matching `scheme_id` and use that file.
        let mut thumb = String::from("#D0D4D8");
        let mut track = String::from("#F0F0F0");
        let editor_dir = theme_manager_for_scheme.borrow().editor_theme_dir.clone();
        let mut found = false;
        if editor_dir.exists() && editor_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&editor_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.eq_ignore_ascii_case("xml"))
                        .unwrap_or(false)
                    {
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            // look for the style-scheme id attribute
                            let id_search = format!("id=\"{}\"", scheme_id);
                            if contents.contains(&id_search) {
                                // extract tokens if present
                                if let Some(v) =
                                    extract_xml_color_value(&contents, "scrollbar-thumb")
                                {
                                    thumb = v;
                                }
                                if let Some(v) =
                                    extract_xml_color_value(&contents, "scrollbar-track")
                                {
                                    track = v;
                                }
                                found = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if !found {
            // Missing editor theme XML - suppressed terminal output.
        }

        // Build CSS and load into the scrolled provider
        // Make GTK scrollbar match WebKit: 12px thickness, flat thumb, no extra border
        // Apply to both vertical (thickness = width) and horizontal (thickness = height)
        let new_css = format!(
            r#"
    .editor-scrolled scrollbar {{ background-color: {track}; padding: 0px; }}
    .editor-scrolled scrollbar.vertical {{ min-width: 12px; max-width: 12px; }}
    .editor-scrolled scrollbar.horizontal {{ min-height: 12px; max-height: 12px; }}
    .editor-scrolled scrollbar slider {{ background-color: {thumb}; border-radius: 0px; margin: 0px; border: none; }}
    .editor-scrolled scrollbar.vertical slider {{ min-width: 12px; max-width: 12px; }}
    .editor-scrolled scrollbar.horizontal slider {{ min-height: 12px; max-height: 12px; }}
    .editor-scrolled scrollbar slider:hover {{ background-color: {thumb}; opacity: 0.95; }}
    "#,
            thumb = thumb,
            track = track
        );
        scrolled_provider_for_update.load_from_data(&new_css);

        // Also update the WebView preview CSS so its scrollbars match.
        // We can't directly mutate the `css` String captured by the outer scope
        // (it's moved into other closures), so instead send the new webkit CSS
        // back via the `css_rc` RefCell we created earlier. Append the
        // webkit scrollbar rules and trigger a preview refresh by updating the
        // `css_rc` content.
        let webkit_css = webkit_scrollbar_css(&thumb, &track);
        // Append webkit rules to preview CSS stored in the shared RefCell.
        if let Ok(mut base_css) = css_rc_for_update.try_borrow_mut() {
            base_css.push('\n');
            base_css.push_str(&webkit_css);
        }
        // Trigger a preview refresh to apply the new CSS immediately using
        // the shared webview and css references.
        let webview_refresh = webview_rc_for_update.clone();
        let css_for_load = css_rc_for_update.clone();
        let buffer_clone_for_load = buffer_for_update.clone();
        let theme_mode_clone_for_load = theme_mode_for_update.clone();
        let markdown_opts_clone_for_load = markdown_opts_for_update.clone();
        let wheel_js_local = wheel_js_for_update.clone();
        glib::idle_add_local(move || {
            let text = buffer_clone_for_load
                .text(
                    &buffer_clone_for_load.start_iter(),
                    &buffer_clone_for_load.end_iter(),
                    false,
                )
                .to_string();
            let mut html_body = markdown_to_html(&text, &markdown_opts_clone_for_load);
            html_body.push_str(&wheel_js_local);
            let html = wrap_html_document(
                &html_body,
                &css_for_load.borrow(),
                &theme_mode_clone_for_load.borrow(),
            );
            if webview_refresh.is_mapped() {
                webview_refresh.load_html(&html, None);
            }
            glib::ControlFlow::Break
        });
    }) as Box<dyn Fn(&str)>;

    // Populate the scrolled provider now with the current scheme id so the
    // editor reflects the active theme immediately.
    update_theme(&scheme_id);

    // Create HTML preview theme update function
    let theme_mode_for_preview = Rc::clone(&theme_mode_rc);
    let theme_manager_for_preview = Rc::clone(&theme_manager);
    let refresh_for_preview = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let webview = Rc::clone(&webview_rc);
        let theme_mode = Rc::clone(&theme_mode_rc);
        let markdown_opts = std::rc::Rc::clone(&markdown_opts_rc);
        move || {
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string();
            let html_body = markdown_to_html(&text, &markdown_opts);
            let html = wrap_html_document(&html_body, &css.borrow(), &theme_mode.borrow());
            let html_clone = html.clone();
            let webview_idle = webview.clone();
            glib::idle_add_local(move || {
                if webview_idle.is_mapped() {
                    webview_idle.load_html(&html_clone, None);
                } else if std::env::var("MARCO_DEBUG_WEBVIEW_LOAD").is_ok() {
                    // Debug message intentionally suppressed from terminal.
                }
                glib::ControlFlow::Break
            });
        }
    };
    let update_preview_theme = Box::new(move |scheme_id: &str| {
        let new_theme_mode = theme_manager_for_preview
            .borrow()
            .preview_theme_mode_from_scheme(scheme_id);
        *theme_mode_for_preview.borrow_mut() = new_theme_mode;
        // Trigger refresh to apply the new theme mode
        refresh_for_preview();
        // Preview theme mode change - terminal output suppressed.
    }) as Box<dyn Fn(&str)>;

    // Return the paned, webview, refresh closure, editor theme update, preview theme update, and buffer
    (
        paned,
        webview,
        css_rc,
        Box::new(refresh_preview) as Box<dyn Fn()>,
        update_theme,
        update_preview_theme,
        buffer_rc.as_ref().clone(),
        insert_mode_state,
    )
}
use sourceview5::prelude::*; // For set_show_line_numbers

pub fn render_editor_with_view(
    _scheme_id: &str,
    style_scheme: Option<&sourceview5::StyleScheme>,
    font_family: &str,
    font_size_pt: f64,
) -> (
    gtk4::Box,
    sourceview5::Buffer,
    sourceview5::View,
    gtk4::CssProvider,
) {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    // Create a SourceBuffer and SourceView
    let buffer = sourceview5::Buffer::new(None);
    buffer.set_text("");
    let source_view = sourceview5::View::new();
    source_view.set_buffer(Some(&buffer));
    source_view.set_monospace(true);
    source_view.set_vexpand(true);
    source_view.set_editable(true);
    source_view.set_show_line_numbers(true);
    source_view.set_highlight_current_line(false);
    source_view.set_show_line_marks(true); //TODO for bookmarks

    // Apply the style scheme if available
    if let Some(scheme) = style_scheme {
        use sourceview5::prelude::*;
        buffer.set_style_scheme(Some(scheme));
    }

    // Apply font settings via CSS (style schemes don't control font)
    use gtk4::CssProvider;
    let css = format!(
        ".sourceview {{ font-family: '{}', 'monospace'; font-size: {}pt; }}",
        font_family, font_size_pt
    );
    let font_provider = CssProvider::new();
    font_provider.connect_parsing_error(|_provider, section, error| {
        eprintln!(
            "[Theme] CSS parsing error in SourceView: {:?} at {:?}",
            error, section
        );
    });
    font_provider.load_from_data(&css);
    source_view
        .style_context()
        .add_provider(&font_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    // Make sure CSS can override background
    use sourceview5::BackgroundPatternType;
    source_view.set_background_pattern(BackgroundPatternType::None);

    // Optionally style the ScrolledWindow for visibility (no border for clarity)
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&source_view));
    scrolled.set_vexpand(true);

    // Create a CssProvider for the scrolled window so we can inject scrollbar
    // colors from the editor theme XML at runtime. Do not hardcode any colors
    // here; the provider will be populated by the theme updater.
    let scrolled_provider = gtk4::CssProvider::new();
    // Add a class to the scrolled window so our CSS only affects the editor
    scrolled.add_css_class("editor-scrolled");
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &scrolled_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Add the ScrolledWindow (with SourceView) to the top
    container.append(&scrolled);

    // Return the scrolled window's css provider so callers can reload the CSS
    (container, buffer, source_view, scrolled_provider)
}

// Crude helper to extract a color value from a minimal editor XML file. Looks
// for: <color name="<key>" value="#RRGGBB"/> and returns the hex string.
fn extract_xml_color_value(contents: &str, key: &str) -> Option<String> {
    // Simple pattern search; avoid full XML parsing for low overhead
    let needle = format!("name=\"{}\"", key);
    if let Some(pos) = contents.find(&needle) {
        // search for value=" after the key
        if let Some(val_pos) = contents[pos..].find("value=\"") {
            let start = pos + val_pos + "value=\"".len();
            if let Some(end_rel) = contents[start..].find('"') {
                let val = contents[start..start + end_rel].trim().to_string();
                return Some(val);
            }
        }
    }
    None
}
