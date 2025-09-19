use crate::components::editor::render::render_editor_with_view;
use crate::components::editor::theme_utils::extract_xml_color_value;
use crate::components::marco_engine::grammar::Rule;
use crate::components::marco_engine::render_html::{HtmlOptions, HtmlRenderer};
use crate::components::marco_engine::{AstBuilder, MarcoParser};
use crate::components::viewer::preview::refresh_preview_into_webview;
use crate::components::viewer::viewmode::{EditorReturn, ViewMode};
use crate::components::viewer::webview_js::{wheel_js, SCROLL_REPORT_JS};
use crate::components::viewer::webview_utils::webkit_scrollbar_css;
use crate::footer::FooterLabels;
use gtk4::prelude::*;
use gtk4::Paned;
use pest::Parser;
use sourceview5::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_editor_with_preview_and_buffer(
    preview_theme_filename: &str,
    preview_theme_dir: &str,
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    theme_mode: Rc<RefCell<String>>,
    labels: Rc<FooterLabels>,
    settings_path: &str,
    document_buffer: Option<Rc<RefCell<crate::logic::buffer::DocumentBuffer>>>,
) -> EditorReturn {
    // Implementation largely copied from previous editor.rs but using helper modules
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(600);
    
    // Add constraints to limit paned position (10% to 90% of width)
    paned.connect_notify_local(Some("position"), move |paned, _| {
        let width = paned.allocated_width();
        if width > 0 {
            let position = paned.position();
            let min_position = (width as f64 * 0.10) as i32;  // 10%
            let max_position = (width as f64 * 0.90) as i32;  // 90%
            
            if position < min_position {
                paned.set_position(min_position);
            } else if position > max_position {
                paned.set_position(max_position);
            }
        }
    });

    let (style_scheme, font_family, font_size_pt, show_line_numbers) = {
        let tm = theme_manager.borrow();
        let style_scheme = tm.current_editor_scheme();
        let font_family = tm
            .settings
            .appearance
            .as_ref()
            .and_then(|a| a.ui_font.as_deref())
            .unwrap_or("Fira Mono")
            .to_string();
        let font_size_pt = tm
            .settings
            .appearance
            .as_ref()
            .and_then(|a| a.ui_font_size)
            .map(|v| v as f64)
            .unwrap_or(10.0);
        let show_line_numbers = tm
            .settings
            .layout
            .as_ref()
            .and_then(|l| l.show_line_numbers)
            .unwrap_or(true);
        (style_scheme, font_family, font_size_pt, show_line_numbers)
    };

    let scheme_id = theme_manager.borrow().current_editor_scheme_id();
    let (editor_widget, buffer, source_view, _scrolled_css_provider, editor_scrolled_window) =
        render_editor_with_view(
            &scheme_id,
            style_scheme.as_ref(),
            &font_family,
            font_size_pt,
            show_line_numbers,
        );
    editor_widget.set_hexpand(true);
    editor_widget.set_vexpand(true);
    paned.set_start_child(Some(&editor_widget));

    let insert_mode_state: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));

    // Event controller for Insert key and line break handling
    use gtk4::gdk::Key;
    use gtk4::gdk::ModifierType;
    use gtk4::glib::Propagation;
    let event_controller = gtk4::EventControllerKey::new();
    let insert_mode_state_clone = Rc::clone(&insert_mode_state);
    let labels_clone = Rc::clone(&labels);
    let source_view_clone = source_view.clone();
    event_controller.connect_key_pressed(move |_controller, keyval, _keycode, state| {
        if keyval == Key::Insert {
            let mut mode = insert_mode_state_clone.borrow_mut();
            *mode = !*mode;
            source_view_clone.set_overwrite(!*mode);
            crate::footer::update_insert_mode(&labels_clone, *mode);
            return Propagation::Stop;
        }
        
        // Handle Enter vs Shift+Enter for different line break types
        if keyval == Key::Return {
            let buffer = source_view_clone.buffer();
            if state.contains(ModifierType::SHIFT_MASK) {
                // Shift+Enter: Insert hard line break (backslash + newline)
                buffer.insert_at_cursor("\\");
                buffer.insert_at_cursor("\n");
            } else {
                // Enter: Insert soft line break (just newline)
                buffer.insert_at_cursor("\n");
            }
            return Propagation::Stop;
        }
        
        Propagation::Proceed
    });
    
    // Set event controller to capture phase to ensure it receives events before SourceView
    event_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    source_view.add_controller(event_controller.upcast::<gtk4::EventController>());

    // Register this editor with the global editor manager to receive settings updates
    {
        let source_view_for_callback = source_view.clone();
        let settings_path_owned = settings_path.to_string();
        if let Some(_editor_id) = crate::components::editor::editor_manager::register_editor_callback_globally(
            move |new_settings: &crate::components::editor::font_config::EditorDisplaySettings| {
                use crate::components::editor::font_config::EditorConfiguration;
                
                log::debug!("Applying editor settings update to SourceView: {} {}px", 
                    new_settings.font_family, new_settings.font_size);
                
                // Create an EditorConfiguration instance to apply the settings
                match EditorConfiguration::new(&settings_path_owned) {
                    Ok(editor_config) => {
                        editor_config.apply_to_sourceview(&source_view_for_callback, new_settings);
                        log::debug!("Successfully applied editor settings to SourceView");
                    },
                    Err(e) => {
                        log::error!("Failed to create EditorConfiguration: {}", e);
                    }
                }
            }
        ) {
            log::debug!("Registered editor callback with editor manager: ID {:?}", _editor_id);
        } else {
            log::warn!("Failed to register editor with global editor manager - settings updates will not work");
        }
    }

    // Register this editor for line numbers updates
    {
        let source_view_for_line_numbers = source_view.clone();
        if let Some(_line_numbers_id) = crate::components::editor::editor_manager::register_line_numbers_callback_globally(
            move |show_line_numbers: bool| {
                log::debug!("Applying line numbers setting to SourceView: {}", show_line_numbers);
                source_view_for_line_numbers.set_show_line_numbers(show_line_numbers);
            }
        ) {
            log::debug!("Registered line numbers callback with editor manager: ID {:?}", _line_numbers_id);
        } else {
            log::warn!("Failed to register line numbers callback with global editor manager");
        }
    }

    use std::fs;
    use std::path::Path;
    let css_path = Path::new(preview_theme_dir).join(preview_theme_filename);
    let mut css = fs::read_to_string(&css_path)
        .unwrap_or_else(|_| String::from("body { background: #fff; color: #222; }"));

    // Add Marco indentation CSS to the theme CSS
    css.push('\n');
    css.push_str(&crate::components::viewer::webview_utils::complete_indentation_css());

    // wheel JS with scroll report for bidirectional sync
    let scroll_scale: f64 = std::env::var("MARCO_SCROLL_SCALE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);
    let wheel_js = wheel_js(scroll_scale);
    let mut wheel_with_report = wheel_js.clone();
    wheel_with_report.push_str(SCROLL_REPORT_JS);
    let wheel_js_rc = Rc::new(wheel_with_report);

    // Extract some theme colors from editor theme XML
    let mut initial_thumb = String::from("#D0D4D8");
    let mut initial_track = String::from("#F0F0F0");
    let editor_bg_color: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let editor_fg_color: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
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
                            if editor_bg_color.borrow().is_none() {
                                if let Some(v) = extract_xml_color_value(&contents, "dark-bg") {
                                    *editor_bg_color.borrow_mut() = Some(v);
                                } else if let Some(v) =
                                    extract_xml_color_value(&contents, "light-bg")
                                {
                                    *editor_bg_color.borrow_mut() = Some(v);
                                }
                            }
                            if editor_fg_color.borrow().is_none() {
                                if let Some(v) = extract_xml_color_value(&contents, "dark-text") {
                                    *editor_fg_color.borrow_mut() = Some(v);
                                } else if let Some(v) =
                                    extract_xml_color_value(&contents, "light-text")
                                {
                                    *editor_fg_color.borrow_mut() = Some(v);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    css.push_str(&webkit_scrollbar_css(&initial_thumb, &initial_track));

    // Register a GTK CssProvider to style application scrollbars to match
    // the editor theme (thumb/track). We'll keep the provider alive by
    // storing it in a variable and re-registering updated rules when themes
    // change.
    let gtk_scroll_css =
        crate::components::viewer::webview_utils::gtk_scrollbar_css(&initial_thumb, &initial_track);
    if let Some(display) = gtk4::gdk::Display::default() {
        let gtk_scroll_provider = gtk4::CssProvider::new();
        gtk_scroll_provider.load_from_data(&gtk_scroll_css);
        gtk4::style_context_add_provider_for_display(
            &display,
            &gtk_scroll_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        // Keep provider in scope by storing in a refcell holder inside this
        // function - prevents it from being dropped immediately.
        let _provider_holder: Rc<RefCell<Option<gtk4::CssProvider>>> =
            Rc::new(RefCell::new(Some(gtk_scroll_provider)));
    }

    let buffer_rc: Rc<sourceview5::Buffer> = Rc::new(buffer);
    let css_rc = Rc::new(RefCell::new(css));
    let theme_mode_rc = Rc::clone(&theme_mode);

    let html_opts = HtmlOptions::default();
    let html_opts_rc = std::rc::Rc::new(html_opts);

    // Precreate code scrolled window
    let initial_text = buffer_rc
        .text(&buffer_rc.start_iter(), &buffer_rc.end_iter(), false)
        .to_string();

    let initial_html_body = match MarcoParser::parse(Rule::document, &initial_text) {
        Ok(pairs) => match AstBuilder::build(pairs) {
            Ok(ast) => {
                let renderer = HtmlRenderer::new(html_opts_rc.as_ref().clone());
                renderer.render(&ast)
            }
            Err(e) => format!("Error building AST: {}", e),
        },
        Err(e) => format!("Error parsing markdown: {}", e),
    };

    let pretty_initial =
        crate::components::viewer::html_format::pretty_print_html(&initial_html_body);

    // Build initial HTML for the WebView using the rendered markdown body and the
    // wheel JS so the preview shows content immediately.
    let mut initial_html_body_with_js = initial_html_body.clone();
    initial_html_body_with_js.push_str(&wheel_js_rc);
    // Use the CSS stored in css_rc (clone it) to avoid using the moved `css` value.
    let css_clone = css_rc.borrow().clone();
    let initial_html = crate::components::viewer::webkit6::wrap_html_document(
        &initial_html_body_with_js,
        &css_clone,
        &theme_mode.borrow(),
    );
    let webview = crate::components::viewer::webkit6::create_html_viewer(&initial_html);
    let webview_rc = Rc::new(webview.clone());

    // Initialize scroll synchronization between editor and preview
    if let Some(global_sync) =
        crate::components::editor::editor_manager::get_global_scroll_synchronizer()
    {
        // Setup bidirectional scroll sync between the editor ScrolledWindow and WebView
        let webview_for_sync = webview.clone();
        let editor_sw_for_sync = editor_scrolled_window.clone();

        // Setup the bidirectional connection
        global_sync.connect_scrolled_window_and_webview(&editor_sw_for_sync, &webview_for_sync);

        log::debug!("Scroll synchronization initialized between editor and WebView preview");
    } else {
        log::warn!(
            "Failed to initialize scroll synchronization: global scroll synchronizer not available"
        );
    }

    let bg_init_owned = editor_bg_color.borrow().clone();
    let fg_init_owned = editor_fg_color.borrow().clone();
    let bg_init = bg_init_owned.as_deref();
    let fg_init = fg_init_owned.as_deref();
    let precreated_code_sw = Rc::new(
        crate::components::viewer::webkit6::create_html_source_viewer(
            &pretty_initial,
            bg_init,
            fg_init,
            true,
        ),
    );
    let _precreated_code_sw_holder: Rc<RefCell<Option<Rc<gtk4::ScrolledWindow>>>> =
        Rc::new(RefCell::new(Some(precreated_code_sw.clone())));

    let stack = gtk4::Stack::new();
    stack.add_named(&webview, Some("html_preview"));
    stack.add_named(precreated_code_sw.as_ref(), Some("code_preview"));
    stack.set_visible_child(&webview);
    paned.set_end_child(Some(&stack));

    // refresh_preview closure
    let wheel_js_for_refresh = wheel_js_rc.clone();
    let is_initial_load = Rc::new(RefCell::new(true)); // Track if this is the first load
    let last_css_hash = Rc::new(RefCell::new(0u64)); // Track CSS changes for theme updates
    let last_document_path = Rc::new(RefCell::new(None::<std::path::PathBuf>)); // Track document path changes
    // Clone document_buffer for use in refresh closure
    let document_buffer_for_refresh = document_buffer.as_ref().map(Rc::clone);
    let refresh_preview_impl: std::rc::Rc<dyn Fn()> = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let webview = Rc::clone(&webview_rc);
        let theme_mode = Rc::clone(&theme_mode_rc);
        let html_opts = std::rc::Rc::clone(&html_opts_rc);
        let wheel_js_local = wheel_js_for_refresh.clone();
        let is_initial_load_clone = Rc::clone(&is_initial_load);
        let last_css_hash_clone = Rc::clone(&last_css_hash);
        let last_document_path_clone = Rc::clone(&last_document_path);
        let document_buffer_capture = document_buffer_for_refresh.clone();
        std::rc::Rc::new(move || {
            let is_first_load = *is_initial_load_clone.borrow();
            
            // Check if the document path has changed (indicating a new file was loaded)
            let current_doc_path = document_buffer_capture
                .as_ref()
                .and_then(|buf| buf.borrow().get_file_path().map(|p| p.to_path_buf()));
            
            let doc_path_changed = {
                let last_path = last_document_path_clone.borrow();
                match (&*last_path, &current_doc_path) {
                    (None, None) => false,
                    (Some(_), None) => true,
                    (None, Some(_)) => true,
                    (Some(last), Some(current)) => last != current,
                }
            };
            
            // Update the last document path
            *last_document_path_clone.borrow_mut() = current_doc_path.clone();
            
            // Check if CSS has changed (theme update)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            css.borrow().hash(&mut hasher);
            theme_mode.borrow().hash(&mut hasher);
            let current_css_hash = hasher.finish();
            let css_changed = *last_css_hash_clone.borrow() != current_css_hash;
            *last_css_hash_clone.borrow_mut() = current_css_hash;
            
            if is_first_load || css_changed || doc_path_changed {
                // Use traditional load_html for initial load, when CSS/theme changes, or when document changes
                // Generate base URI directly from DocumentBuffer for WebKit6
                let base_uri = document_buffer_capture
                    .as_ref()
                    .and_then(|buf| buf.borrow().get_base_uri_for_webview());
                
                let params = crate::components::viewer::preview::PreviewRefreshParams {
                    webview: webview.as_ref(),
                    css: &css,
                    html_options: html_opts.as_ref(),
                    buffer: buffer.as_ref(),
                    wheel_js: &wheel_js_local,
                    theme_mode: &theme_mode,
                    base_uri: base_uri.as_deref(),
                    document_buffer: document_buffer_capture.as_ref(),
                };
                crate::components::viewer::preview::refresh_preview_into_webview_with_base_uri_and_doc_buffer(params);
                
                // Mark as no longer initial load
                *is_initial_load_clone.borrow_mut() = false;
            } else {
                // Use smooth updates for subsequent content changes
                let params = crate::components::viewer::preview::SmoothUpdateParams {
                    webview: webview.as_ref(),
                    html_options: html_opts.as_ref(),
                    buffer: buffer.as_ref(),
                    wheel_js: &wheel_js_local,
                    document_buffer: document_buffer_capture.as_ref(),
                };
                crate::components::viewer::preview::refresh_preview_content_smooth_with_doc_buffer(params);
            }
        })
    };

    // Trigger an initial preview refresh so the WebView shows content immediately.
    log::debug!("[preview] triggering initial refresh");
    refresh_preview_impl();

    // Track current view mode for real-time updates
    let current_view_mode: Rc<RefCell<ViewMode>> = Rc::new(RefCell::new(ViewMode::HtmlPreview));

    // Function to update HTML code view with raw HTML
    let update_html_code_view = {
        let buffer_for_code = Rc::clone(&buffer_rc);
        let precreated_code_sw_for_code = precreated_code_sw.clone();
        let html_opts_for_code = Rc::clone(&html_opts_rc);
        
        Box::new(move || {
            let text = buffer_for_code
                .text(
                    &buffer_for_code.start_iter(),
                    &buffer_for_code.end_iter(),
                    false,
                )
                .to_string();

            // Generate raw HTML using Marco engine (same as nested code blocks)
            let html_body = match MarcoParser::parse(Rule::document, &text) {
                Ok(pairs) => match AstBuilder::build(pairs) {
                    Ok(ast) => {
                        let renderer = HtmlRenderer::new(html_opts_for_code.as_ref().clone());
                        renderer.render(&ast)
                    }
                    Err(e) => format!("<!-- Error building AST: {} -->", e),
                },
                Err(e) => format!("<!-- Error parsing markdown: {} -->", e),
            };

            // Format the HTML for better readability in code view
            let formatted_html = crate::components::viewer::html_format::pretty_print_html(&html_body);
            
            // Show formatted HTML for easy reading and debugging
            if let Some(sw_child) = precreated_code_sw_for_code.child() {
                if let Ok(tv) = sw_child.downcast::<gtk4::TextView>() {
                    let new_buf = gtk4::TextBuffer::new(None::<&gtk4::TextTagTable>);
                    new_buf.set_text(&formatted_html);  // Pretty-formatted HTML for readability
                    tv.set_buffer(Some(&new_buf));
                }
            }
        }) as Box<dyn Fn()>
    };
    let update_html_code_view_rc = Rc::new(update_html_code_view);

    // Also update preview whenever buffer content changes (e.g. when opening a file).
    let refresh_for_signal = std::rc::Rc::clone(&refresh_preview_impl);
    let update_code_for_signal = Rc::clone(&update_html_code_view_rc);
    let view_mode_for_signal = Rc::clone(&current_view_mode);
    buffer_rc.connect_changed(move |_| {
        refresh_for_signal();  // Always update HTML preview
        
        // Also update code view if we're currently in CodePreview mode
        if *view_mode_for_signal.borrow() == ViewMode::CodePreview {
            update_code_for_signal();
        }
    });

    // theme update function
    // Prepare clones for closures so we don't move the originals
    let theme_manager_for_update = Rc::clone(&theme_manager);
    let buffer_rc_for_update = Rc::clone(&buffer_rc);
    let update_theme = Box::new(move |scheme_id: &str| {
        // actual update logic remains in editor.rs original; placeholder here
        if let Some(scheme) = theme_manager_for_update
            .borrow()
            .get_editor_scheme(scheme_id)
        {
            buffer_rc_for_update.set_style_scheme(Some(&scheme));
        }
    }) as Box<dyn Fn(&str)>;

    // Clones for preview theme updater
    let theme_manager_for_preview = Rc::clone(&theme_manager);
    let css_rc_for_preview = Rc::clone(&css_rc);
    let html_opts_for_preview = std::rc::Rc::clone(&html_opts_rc);
    let buffer_rc_for_preview = Rc::clone(&buffer_rc);
    let webview_rc_for_preview = Rc::clone(&webview_rc);
    let wheel_js_for_preview = wheel_js_for_refresh.clone();
    let theme_mode_for_preview = Rc::clone(&theme_mode_rc);
    let editor_dir_for_preview = theme_manager.borrow().editor_theme_dir.clone();
    let editor_bg_color_for_preview = Rc::clone(&editor_bg_color);
    let editor_fg_color_for_preview = Rc::clone(&editor_fg_color);
    let document_buffer_for_preview = document_buffer.as_ref().map(Rc::clone);
    use std::cell::Cell;
    let preview_theme_timeout: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
    let preview_theme_timeout_clone = Rc::clone(&preview_theme_timeout);
    let update_preview_theme = Box::new(move |scheme_id: &str| {
        // Re-extract editor bg/fg colors from the selected editor style scheme
        // so the Source Code viewer can match the editor theme.
        if editor_dir_for_preview.exists() && editor_dir_for_preview.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&editor_dir_for_preview) {
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
                                // Try to extract preferred bg/fg tokens
                                if let Some(v) = extract_xml_color_value(&contents, "dark-bg") {
                                    *editor_bg_color_for_preview.borrow_mut() = Some(v);
                                } else if let Some(v) =
                                    extract_xml_color_value(&contents, "light-bg")
                                {
                                    *editor_bg_color_for_preview.borrow_mut() = Some(v);
                                }
                                if let Some(v) = extract_xml_color_value(&contents, "dark-text") {
                                    *editor_fg_color_for_preview.borrow_mut() = Some(v);
                                } else if let Some(v) =
                                    extract_xml_color_value(&contents, "light-text")
                                {
                                    *editor_fg_color_for_preview.borrow_mut() = Some(v);
                                }

                                // Register a small CSS provider to update the source preview
                                if let Some(display) = gtk4::gdk::Display::default() {
                                    let mut css_rules = String::new();
                                    let bg_val = editor_bg_color_for_preview.borrow().clone();
                                    let fg_val = editor_fg_color_for_preview.borrow().clone();
                                    let bg = bg_val.as_deref().unwrap_or("transparent");
                                    let fg = fg_val.as_deref().unwrap_or("#000000");
                                    css_rules.push_str(&format!(
                                        ".source-preview .monospace {{ background-color: {}; color: {}; }}",
                                        bg, fg
                                    ));
                                    let provider = gtk4::CssProvider::new();
                                    provider.load_from_data(&css_rules);
                                    gtk4::style_context_add_provider_for_display(
                                        &display,
                                        &provider,
                                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                                    );
                                }
                                // Also update GTK scrollbar CSS provider so scrollbars
                                // match the newly selected editor scheme at runtime.
                                if let Some(display) = gtk4::gdk::Display::default() {
                                    let mut thumb = String::from("#D0D4D8");
                                    let mut track = String::from("#F0F0F0");
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
                                    let gtk_css =
                                        crate::components::viewer::webview_utils::gtk_scrollbar_css(
                                            &thumb, &track,
                                        );
                                    let provider = gtk4::CssProvider::new();
                                    provider.load_from_data(&gtk_css);
                                    gtk4::style_context_add_provider_for_display(
                                        &display,
                                        &provider,
                                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                                    );
                                    // Also update the HTML preview CSS so the WebView's
                                    // scrollbars (::-webkit-scrollbar) match the editor
                                    // theme at runtime.
                                    let webkit_css = crate::components::viewer::webview_utils::webkit_scrollbar_css(&thumb, &track);
                                    css_rc_for_preview.borrow_mut().push_str(&webkit_css);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        let new_theme_mode = theme_manager_for_preview
            .borrow()
            .preview_theme_mode_from_scheme(scheme_id);
        *theme_mode_for_preview.borrow_mut() = new_theme_mode;

        // debounce reloads to avoid rapid successive full-document reloads which cause blinking
        if let Some(id) = preview_theme_timeout_clone.replace(None) {
            id.remove();
        }
        let preview_theme_timeout_clone2 = Rc::clone(&preview_theme_timeout_clone);
        let webview_clone = webview_rc_for_preview.clone();
        let css_clone = Rc::clone(&css_rc_for_preview);
        let html_opts_clone = std::rc::Rc::clone(&html_opts_for_preview);
        let buffer_clone = Rc::clone(&buffer_rc_for_preview);
        let wheel_clone = wheel_js_for_preview.clone();
        let theme_mode_clone = Rc::clone(&theme_mode_for_preview);
        let document_buffer_clone = document_buffer_for_preview.as_ref().map(Rc::clone);
        let id = glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            // Extract document path from DocumentBuffer for base URI generation
            let doc_path = document_buffer_clone
                .as_ref()
                .and_then(|buf| buf.borrow().get_file_path().map(|p| p.to_path_buf()));
            
            // Debug: log the document path being passed
            if let Some(ref path) = doc_path {
                eprintln!("[DEBUG] Theme refresh: Passing document path to preview: {}", path.display());
            } else {
                eprintln!("[DEBUG] Theme refresh: No document path available (untitled document)");
            }
            
            refresh_preview_into_webview(
                webview_clone.as_ref(),
                &css_clone,
                &html_opts_clone,
                &buffer_clone,
                &wheel_clone,
                &theme_mode_clone,
                doc_path.as_deref(),
            );
            preview_theme_timeout_clone2.set(None);
            glib::ControlFlow::Break
        });
        preview_theme_timeout_clone.set(Some(id));
    }) as Box<dyn Fn(&str)>;

    (
        paned,
        webview,
        css_rc,
        Box::new({
            let r = std::rc::Rc::clone(&refresh_preview_impl);
            move || r()
        }) as Box<dyn Fn()>,
        update_theme,
        update_preview_theme,
        buffer_rc.as_ref().clone(),
        insert_mode_state,
        {
            // Provide a real runtime view-mode setter that switches the Stack
            // visible child and keeps the code-preview TextView in sync with
            // the latest rendered HTML.
            let stack_for_mode = stack.clone();
            let refresh_for_mode = std::rc::Rc::clone(&refresh_preview_impl);
            let update_code_for_mode = Rc::clone(&update_html_code_view_rc);
            let current_view_mode_for_mode = Rc::clone(&current_view_mode);
            
            Box::new(move |mode: ViewMode| {
                // Update the tracked view mode
                *current_view_mode_for_mode.borrow_mut() = mode;
                
                match mode {
                    ViewMode::HtmlPreview => {
                        // Ensure preview is up-to-date, then show HTML preview.
                        (refresh_for_mode)();
                        stack_for_mode.set_visible_child_name("html_preview");
                    }
                    ViewMode::CodePreview => {
                        // Update HTML code view with current raw HTML, then show it
                        (update_code_for_mode)();
                        stack_for_mode.set_visible_child_name("code_preview");
                    }
                }
            }) as Box<dyn Fn(ViewMode)>
        },
    )
}
