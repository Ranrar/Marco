use crate::components::editor::render::render_editor_with_view;
use crate::components::editor::theme_utils::extract_xml_color_value;
use crate::components::editor::processing_utilities::AsyncExtensionManager;
use core::RenderOptions;  // New parser API
use core::global_parser_cache;  // New cache API
use crate::components::viewer::preview::refresh_preview_into_webview;
use crate::components::viewer::viewmode::{EditorReturn, ViewMode};
use crate::components::viewer::webview_js::{wheel_js, SCROLL_REPORT_JS};
use crate::components::viewer::webview_utils::webkit_scrollbar_css;
use crate::footer::FooterLabels;
use crate::logic::signal_manager::safe_source_remove;
use gtk4::prelude::*;
use gtk4::Paned;
use sourceview5::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::ui::splitview::setup_split_percentage_indicator_with_cascade_prevention;

pub fn create_editor_with_preview_and_buffer(
    preview_theme_filename: &str,
    preview_theme_dir: &str,
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    theme_mode: Rc<RefCell<String>>,
    labels: Rc<FooterLabels>,
    _settings_path: &str,
    document_buffer: Option<Rc<RefCell<core::logic::buffer::DocumentBuffer>>>,
) -> EditorReturn {
    // Implementation largely copied from previous editor.rs but using helper modules
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(600);
    
    // Create split controller to manage position constraints and locking
    use crate::components::viewer::controller::SplitController;
    let split_controller = SplitController::new(paned.clone());

    let (style_scheme, font_family, font_size_pt, show_line_numbers) = {
        let tm = theme_manager.borrow();
        let style_scheme = tm.current_editor_scheme();
        let settings = tm.get_settings();
        let font_family = settings
            .appearance
            .as_ref()
            .and_then(|a| a.ui_font.as_deref())
            .unwrap_or("Fira Mono")
            .to_string();
        let font_size_pt = settings
            .appearance
            .as_ref()
            .and_then(|a| a.ui_font_size)
            .map(|v| v as f64)
            .unwrap_or(10.0);
        let show_line_numbers = settings
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

    // Editor callback registration moved later so buffer handle can be captured

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
    let scrollbar_thumb_color: Rc<RefCell<String>> = Rc::new(RefCell::new(initial_thumb.clone()));
    let scrollbar_track_color: Rc<RefCell<String>> = Rc::new(RefCell::new(initial_track.clone()));
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
                                initial_thumb = v.clone();
                                *scrollbar_thumb_color.borrow_mut() = v;
                            }
                            if let Some(v) = extract_xml_color_value(&contents, "scrollbar-track") {
                                initial_track = v.clone();
                                *scrollbar_track_color.borrow_mut() = v;
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

    // Style the Paned separator dynamically based on scrollbar visibility
    // When no scrollbar: 12px visible separator
    // When scrollbar visible: 1px minimal separator (scrollbar acts as divider)
    let paned_css_provider_holder: Rc<RefCell<Option<gtk4::CssProvider>>> =
        Rc::new(RefCell::new(None));
    
    // Function to generate CSS based on scrollbar visibility
    let generate_dynamic_paned_css = {
        let scrollbar_thumb = Rc::clone(&scrollbar_thumb_color);
        let scrollbar_track = Rc::clone(&scrollbar_track_color);
        
        move |scrollbar_visible: bool| -> String {
            let thumb = scrollbar_thumb.borrow().clone();
            let track = scrollbar_track.borrow().clone();
            
            if scrollbar_visible {
                // Minimal 1px separator when scrollbar is visible
                format!(
                    r#"
/* Paned separator: minimal (1px) when scrollbar is visible */
paned > separator {{
    min-width: 1px;
    min-height: 1px;
    background: transparent;
    border: none;
}}

paned > separator:active {{
    min-width: 1px;
    background: {thumb};
}}
                    "#,
                    thumb = thumb
                )
            } else {
                // 12px visible separator when no scrollbar - solid track color
                format!(
                    r#"
/* Paned separator: 12px solid track color when no scrollbar */
paned > separator {{
    min-width: 12px;
    min-height: 12px;
    background: {track};
    border: none;
}}
                    "#,
                    track = track
                )
            }
        }
    };
    
    // Apply initial CSS (assume no scrollbar initially)
    if let Some(display) = gtk4::gdk::Display::default() {
        let initial_css = generate_dynamic_paned_css(false);
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(&initial_css);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        *paned_css_provider_holder.borrow_mut() = Some(provider);
        log::debug!("Applied initial paned separator CSS (12px, no scrollbar)");
    }
    
    // Monitor scrollbar visibility and update separator CSS dynamically
    let paned_css_holder_for_monitor = Rc::clone(&paned_css_provider_holder);
    let editor_sw_for_monitor = editor_scrolled_window.clone();
    let generate_css_for_monitor = generate_dynamic_paned_css.clone();
    
    // Track last scrollbar state to avoid redundant CSS updates
    let last_scrollbar_state = Rc::new(RefCell::new(false));
    
    // Check scrollbar visibility periodically with fast polling (100ms)
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        // Get vertical adjustment to check if scrollbar is needed
        let vadj = editor_sw_for_monitor.vadjustment();
        let upper = vadj.upper();
        let page_size = vadj.page_size();
        let scrollbar_visible = upper > page_size;
        
        // Only update CSS if scrollbar visibility state changed
        let mut last_state = last_scrollbar_state.borrow_mut();
        if *last_state != scrollbar_visible {
            *last_state = scrollbar_visible;
            drop(last_state); // Release borrow before calling closure
            
            // Update CSS based on scrollbar visibility using the closure
            if let Some(display) = gtk4::gdk::Display::default() {
                let css = generate_css_for_monitor(scrollbar_visible);
                
                let provider = gtk4::CssProvider::new();
                provider.load_from_data(&css);
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
                *paned_css_holder_for_monitor.borrow_mut() = Some(provider);
                
                log::debug!("Paned separator CSS updated: scrollbar_visible={}", scrollbar_visible);
            }
        }
        
        glib::ControlFlow::Continue
    });

    let buffer_rc: Rc<sourceview5::Buffer> = Rc::new(buffer);
    // Apply LSP syntax tag colors for the current theme so tags exist before
    // any LSP or UI code attempts to lookup them by name. Only apply if the
    // user settings enable syntax colors.
    {
        let tm = theme_manager.borrow();
        let settings = tm.get_settings();
        let enable_syntax = settings
            .editor
            .as_ref()
            .and_then(|e| e.syntax_colors)
            .unwrap_or(true);
        if enable_syntax {
            crate::ui::css::syntax::apply_to_buffer(&buffer_rc, theme_mode.borrow().as_str());
        } else {
            crate::ui::css::syntax::remove_from_buffer(&buffer_rc);
        }
    }
        // Register this editor with the global editor manager to receive settings updates
        {
            let source_view_for_callback = source_view.clone();
            let buffer_for_callback = Rc::clone(&buffer_rc);
            let theme_manager_for_callback = Rc::clone(&theme_manager);
            if let Some(_editor_id) = crate::components::editor::editor_manager::register_editor_callback_globally(
                move |new_settings: &crate::components::editor::font_config::EditorDisplaySettings| {
                    log::debug!("Applying editor settings update to SourceView: {} {}px", 
                        new_settings.font_family, new_settings.font_size);

                    // Apply font and line height using CSS
                    let css = format!(
                        r#"
                        textview {{
                            font-family: "{}";
                            font-size: {}px;
                            line-height: {};
                        }}
                        textview text {{
                            font-family: "{}";
                            font-size: {}px;
                            line-height: {};
                        }}
                        "#,
                        new_settings.font_family, new_settings.font_size, new_settings.line_height,
                        new_settings.font_family, new_settings.font_size, new_settings.line_height
                    );

                    let css_provider = gtk4::CssProvider::new();
                    css_provider.load_from_data(&css);
                    source_view_for_callback.style_context().add_provider(
                        &css_provider, 
                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
                    );

                    // Apply line wrapping
                    let wrap_mode = if new_settings.line_wrapping {
                        gtk4::WrapMode::Word
                    } else {
                        gtk4::WrapMode::None
                    };
                    source_view_for_callback.set_wrap_mode(wrap_mode);

                    // Apply tabs to spaces setting
                    source_view_for_callback.set_insert_spaces_instead_of_tabs(new_settings.tabs_to_spaces);

                    // Apply line numbers setting
                    source_view_for_callback.set_show_line_numbers(new_settings.show_line_numbers);

                    // Apply show invisibles setting (whitespace visibility)
                    let space_drawer = source_view_for_callback.space_drawer();
                    if new_settings.show_invisibles {
                        space_drawer.set_types_for_locations(
                            sourceview5::SpaceLocationFlags::ALL,
                            sourceview5::SpaceTypeFlags::SPACE | sourceview5::SpaceTypeFlags::TAB | sourceview5::SpaceTypeFlags::NEWLINE,
                        );
                        space_drawer.set_enable_matrix(true);
                    } else {
                        space_drawer.set_types_for_locations(
                            sourceview5::SpaceLocationFlags::ALL,
                            sourceview5::SpaceTypeFlags::NONE,
                        );
                        space_drawer.set_enable_matrix(false);
                    }

                    // Apply or remove syntax colors depending on the new settings value
                    if new_settings.syntax_colors {
                        let scheme_id = theme_manager_for_callback.borrow().current_editor_scheme_id();
                        let theme_mode = theme_manager_for_callback.borrow().preview_theme_mode_from_scheme(&scheme_id);
                        crate::ui::css::syntax::apply_to_buffer(&buffer_for_callback, theme_mode.as_str());
                    } else {
                        crate::ui::css::syntax::remove_from_buffer(&buffer_for_callback);
                    }

                    log::debug!("Successfully applied editor settings to SourceView: {} {}px", 
                        new_settings.font_family, new_settings.font_size);
                }
            ) {
                log::debug!("Registered editor callback with editor manager: ID {:?}", _editor_id);
            } else {
                log::warn!("Failed to register editor with global editor manager - settings updates will not work");
            }
        }
    let css_rc = Rc::new(RefCell::new(css));
    let theme_mode_rc = Rc::clone(&theme_mode);

    // Create RenderOptions with the current theme mode for syntax highlighting
    let current_theme_mode = theme_mode_rc.borrow().clone();
    let html_opts = RenderOptions {
        syntax_highlighting: true,
        line_numbers: false,
        theme: current_theme_mode.clone(),
    };
    let html_opts_rc = std::rc::Rc::new(html_opts);

    // Precreate code scrolled window
    let initial_text = buffer_rc
        .text(&buffer_rc.start_iter(), &buffer_rc.end_iter(), false)
        .to_string();

    let initial_html_body = match global_parser_cache().render_with_cache(&initial_text, (*html_opts_rc).clone()) {
        Ok(html) => html,
        Err(e) => format!("Error rendering HTML: {}", e),
    };

    let pretty_initial =
        crate::components::viewer::html_format::pretty_print_html(&initial_html_body);

    // Build initial HTML for the WebView using the rendered markdown body and the
    // wheel JS so the preview shows content immediately.
    let mut initial_html_body_with_js = initial_html_body.clone();
    initial_html_body_with_js.push_str(&wheel_js_rc);
    // Use the CSS stored in css_rc (clone it) to avoid using the moved `css` value.
    let css_clone = css_rc.borrow().clone();
    
    // Get editor background color early for instant dark mode support (eliminates white flash)
    let bg_init_preview = editor_bg_color.borrow().clone();
    let bg_init_preview_ref = bg_init_preview.as_deref();
    
    // LAYERED DEFENSE - Inject inline background style in HTML
    let initial_html = crate::components::viewer::webkit6::wrap_html_document(
        &initial_html_body_with_js,
        &css_clone,
        &theme_mode.borrow(),
        bg_init_preview_ref, // Pass editor background color for inline style
    );
    
    // LAYERED DEFENSE - Set widget background + load HTML
    let webview = crate::components::viewer::webkit6::create_html_viewer_with_base(
        &initial_html,
        None,                // No base URI needed yet
        bg_init_preview_ref, // Pass editor background color for widget-level background
    );
    // Wrap WebView in Rc<RefCell<>> for shared ownership during reparenting
    let webview_rc = Rc::new(RefCell::new(webview.clone()));

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
    let thumb_init = scrollbar_thumb_color.borrow().clone();
    let track_init = scrollbar_track_color.borrow().clone();
    
    // Create WebView-based code viewer with syntax highlighting
    let current_theme_for_code = theme_mode_rc.borrow().clone();
    let webview_code = crate::components::viewer::webkit6::create_html_source_viewer_webview(
        &pretty_initial,
        &current_theme_for_code,
        None, // No base URI needed for code view
        bg_init, // Pass editor background color
        fg_init, // Pass editor foreground color
        Some(&thumb_init), // Pass scrollbar thumb color
        Some(&track_init), // Pass scrollbar track color
    ).expect("Failed to create code viewer WebView");
    
    // Wrap WebView in ScrolledWindow for consistency
    let sw = gtk4::ScrolledWindow::new();
    sw.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    sw.set_child(Some(&webview_code));
    sw.add_css_class("editor-scrolled"); // Match editor scrollbar style
    let precreated_code_sw = Rc::new(sw);
    
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
                    webview: &webview.borrow(),
                    css: &css,
                    html_options: html_opts.as_ref(),
                    buffer: buffer.as_ref(),
                    wheel_js: &wheel_js_local,
                    theme_mode: &theme_mode,
                    base_uri: base_uri.as_deref(),
                };
                crate::components::viewer::preview::refresh_preview_into_webview_with_base_uri_and_doc_buffer(params);
                
                // Mark as no longer initial load
                *is_initial_load_clone.borrow_mut() = false;
            } else {
                // Use smooth updates for subsequent content changes
                let params = crate::components::viewer::preview::SmoothUpdateParams {
                    webview: &webview.borrow(),
                    html_options: html_opts.as_ref(),
                    buffer: buffer.as_ref(),
                    wheel_js: &wheel_js_local,
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
        let theme_mode_for_code = Rc::clone(&theme_mode_rc);
        let editor_bg_for_code = Rc::clone(&editor_bg_color);
        let editor_fg_for_code = Rc::clone(&editor_fg_color);
        let scrollbar_thumb_for_code = Rc::clone(&scrollbar_thumb_color);
        let scrollbar_track_for_code = Rc::clone(&scrollbar_track_color);
        let last_code_view_theme = Rc::new(RefCell::new(String::new()));
        
        Box::new(move || {
            log::debug!("[editor_ui] update_html_code_view called");
            
            let text = buffer_for_code
                .text(
                    &buffer_for_code.start_iter(),
                    &buffer_for_code.end_iter(),
                    false,
                )
                .to_string();

            log::debug!("[editor_ui] Buffer text length: {} bytes", text.len());

            // Generate raw HTML using new parser cache with full HTML caching
            let html_body = match global_parser_cache().render_with_cache(&text, (*html_opts_for_code).clone()) {
                Ok(html) => html,
                Err(e) => format!("<!-- Error rendering HTML: {} -->", e),
            };

            log::debug!("[editor_ui] Generated HTML length: {} bytes", html_body.len());

            // Format the HTML for better readability in code view
            let formatted_html = crate::components::viewer::html_format::pretty_print_html(&html_body);
            
            log::debug!("[editor_ui] Formatted HTML length: {} bytes", formatted_html.len());
            
            // Get current theme mode
            let current_theme = theme_mode_for_code.borrow().clone();
            
            log::debug!("[editor_ui] Current theme: {}", current_theme);
            
            // Update the code view
            if let Some(sw_child) = precreated_code_sw_for_code.child() {
                log::debug!("[editor_ui] Code view has child widget: {:?}", sw_child.type_());
                
                // GTK ScrolledWindow may wrap widgets in a Viewport
                // Try to get the actual widget (WebView or TextView)
                let actual_widget = if sw_child.is::<gtk4::Viewport>() {
                    log::debug!("[editor_ui] Child is Viewport, getting its child");
                    if let Ok(viewport) = sw_child.downcast::<gtk4::Viewport>() {
                        viewport.child()
                    } else {
                        None
                    }
                } else {
                    Some(sw_child)
                };
                
                if let Some(widget) = actual_widget {
                    log::debug!("[editor_ui] Actual widget type: {:?}", widget.type_());
                    
                    // Get current theme and check if it changed
                    let theme_changed = *last_code_view_theme.borrow() != current_theme;
                    if theme_changed {
                        log::debug!("[editor_ui] Theme changed: {} -> {}", 
                            last_code_view_theme.borrow(), current_theme);
                        *last_code_view_theme.borrow_mut() = current_theme.clone();
                    }
                    
                    // Update WebView with smooth transition
                    if widget.is::<webkit6::WebView>() {
                        log::debug!("[editor_ui] Widget is WebView, updating with smooth transition");
                        
                        if let Ok(webview) = widget.downcast::<webkit6::WebView>() {
                            // Get editor colors
                            let bg_owned = editor_bg_for_code.borrow().clone();
                            let fg_owned = editor_fg_for_code.borrow().clone();
                            let bg = bg_owned.as_deref();
                            let fg = fg_owned.as_deref();
                            
                            // Get scrollbar colors
                            let thumb = scrollbar_thumb_for_code.borrow().clone();
                            let track = scrollbar_track_for_code.borrow().clone();
                            
                            // Use smooth update to avoid flickering
                            if let Err(e) = crate::components::viewer::webkit6::update_code_view_smooth(
                                &webview,
                                &formatted_html,
                                &current_theme,
                                bg,
                                fg,
                                Some(&thumb),
                                Some(&track),
                            ) {
                                log::error!("Failed to smooth update code view: {}", e);
                            }
                        }
                    } else {
                        log::warn!("[editor_ui] Code view widget is not a WebView: {:?}", widget.type_());
                    }
                } else {
                    log::warn!("[editor_ui] No actual widget found in code view");
                }
            } else {
                log::warn!("[editor_ui] Code view has no child widget");
            }
        }) as Box<dyn Fn()>
    };
    let update_html_code_view_rc = Rc::new(update_html_code_view);

    // Initialize AsyncExtensionManager for background extension processing
    let extension_manager = match AsyncExtensionManager::new() {
        Ok(manager) => Some(Rc::new(RefCell::new(manager))),
        Err(e) => {
            log::error!("Failed to initialize AsyncExtensionManager: {}", e);
            None
        }
    };

    // Create debouncers for different types of processing
    let preview_debouncer = Rc::new(crate::components::editor::debouncer::Debouncer::new(300));
    let extension_debouncer = Rc::new(crate::components::editor::debouncer::Debouncer::new(250));
    let lsp_debouncer = Rc::new(crate::components::editor::debouncer::Debouncer::new(150)); // Faster for syntax highlighting
    
    // Track last content for change delta detection
    let last_content = Rc::new(RefCell::new(String::new()));
    
    // Also update preview whenever buffer content changes (e.g. when opening a file).
    let refresh_for_signal = std::rc::Rc::clone(&refresh_preview_impl);
    let update_code_for_signal = Rc::clone(&update_html_code_view_rc);
    let view_mode_for_signal = Rc::clone(&current_view_mode);
    let extension_manager_for_signal = extension_manager.clone();
    let buffer_rc_clone = Rc::clone(&buffer_rc);
    let preview_debouncer_for_signal = Rc::clone(&preview_debouncer);
    let extension_debouncer_for_signal = Rc::clone(&extension_debouncer);
    let lsp_debouncer_for_signal = Rc::clone(&lsp_debouncer);
    let last_content_for_signal = Rc::clone(&last_content);
    
    buffer_rc_clone.connect_changed(move |buffer| {
        let current_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
        let cursor_position = {
            let cursor_iter = buffer.cursor_position();
            if cursor_iter >= 0 { Some(cursor_iter as u32) } else { None }
        };
        
        // Change delta detection
        let mut last_content_borrow = last_content_for_signal.borrow_mut();
        let content_changed = *last_content_borrow != current_text;
        let change_delta = if content_changed {
            let old_len = last_content_borrow.len();
            let new_len = current_text.len();
            let size_change = (new_len as i32 - old_len as i32).abs();
            *last_content_borrow = current_text.clone();
            Some((old_len, new_len, size_change as usize))
        } else {
            None
        };
        drop(last_content_borrow);
        
        // Use debounced HTML preview updates to batch rapid typing
        let refresh_clone = Rc::clone(&refresh_for_signal);
        let update_code_clone = Rc::clone(&update_code_for_signal);
        let view_mode_clone = Rc::clone(&view_mode_for_signal);
        
        preview_debouncer_for_signal.debounce(move || {
            log::debug!("Buffer changed - debounced preview update triggered");
            
            // Update HTML preview (leading edge + trailing edge)
            refresh_clone();
            
            // Also update code view if we're currently in CodePreview mode
            if *view_mode_clone.borrow() == ViewMode::CodePreview {
                update_code_clone();
            }
        });
        
        // Apply LSP syntax highlighting with debouncing
        let buffer_for_lsp = buffer.clone();
        let current_text_for_lsp = current_text.clone();
        lsp_debouncer_for_signal.debounce(move || {
            log::trace!("Buffer changed - applying LSP syntax highlighting");
            
            // Parse the markdown content
            match core::parser::parse(&current_text_for_lsp) {
                Ok(document) => {
                    // Compute highlights from AST
                    let highlights = core::lsp::compute_highlights(&document);
                    log::debug!("Computed {} LSP highlights", highlights.len());
                    
                    // Apply highlights to buffer
                    crate::components::editor::lsp_integration::apply_lsp_highlights(
                        &buffer_for_lsp,
                        &highlights,
                    );
                }
                Err(e) => {
                    log::warn!("Failed to parse markdown for LSP highlighting: {}", e);
                }
            }
        });
        
        // Use new debounced extension processing with change delta detection
        if let Some(ref manager) = extension_manager_for_signal {
            let extension_manager_clone = manager.clone();
            let current_text_clone = current_text.clone();
            
            extension_debouncer_for_signal.debounce(move || {
                log::debug!("Buffer changed - debounced extension processing triggered");
                
                // Apply change delta optimization
                if let Some((old_len, _new_len, size_change)) = change_delta {
                    // For small changes, we can potentially optimize
                    if size_change < 10 && old_len > 1000 {
                        log::debug!("Change delta detected: {} chars changed in {} char document - optimizing", size_change, old_len);
                    } else {
                        log::debug!("Large change or small document: processing all extensions");
                    }
                }
                
                // Process extensions using the simpler parallel method instead of old debouncing
                if let Ok(manager_ref) = extension_manager_clone.try_borrow() {
                    if let Err(e) = manager_ref.process_extensions_parallel(
                        current_text_clone.clone(),
                        cursor_position,
                        |results| {
                            log::debug!("Extension processing completed: {} results", results.len());
                            for result in results {
                                if result.success {
                                    log::debug!("Extension '{}' processed in {}ms", 
                                        result.extension_name, result.processing_time_ms);
                                }
                            }
                        }
                    ) {
                        log::error!("Failed to trigger extension processing: {}", e);
                    }
                }
            });
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
    let scrollbar_thumb_for_preview = Rc::clone(&scrollbar_thumb_color);
    let scrollbar_track_for_preview = Rc::clone(&scrollbar_track_color);
    let editor_sw_for_preview = editor_scrolled_window.clone(); // For checking scrollbar state
    let document_buffer_for_preview = document_buffer.as_ref().map(Rc::clone);
    use std::cell::Cell;
    let preview_theme_timeout: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
    let preview_theme_timeout_clone = Rc::clone(&preview_theme_timeout);
    let update_html_code_view_for_preview = Rc::clone(&update_html_code_view_rc);
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
                                
                                // Extract scrollbar colors for code view
                                if let Some(v) = extract_xml_color_value(&contents, "scrollbar-thumb") {
                                    *scrollbar_thumb_for_preview.borrow_mut() = v;
                                }
                                if let Some(v) = extract_xml_color_value(&contents, "scrollbar-track") {
                                    *scrollbar_track_for_preview.borrow_mut() = v;
                                }
                                
                                // Update webkit scrollbar CSS in the preview CSS string
                                // This ensures the HTML preview scrollbar matches the theme
                                let new_thumb = scrollbar_thumb_for_preview.borrow().clone();
                                let new_track = scrollbar_track_for_preview.borrow().clone();
                                let new_webkit_css = webkit_scrollbar_css(&new_thumb, &new_track);
                                
                                // Regenerate the CSS with new webkit scrollbar styling
                                let mut updated_css = css_rc_for_preview.borrow().clone();
                                // Remove old webkit scrollbar CSS (everything after the last occurrence of ::-webkit-scrollbar)
                                if let Some(pos) = updated_css.rfind("::-webkit-scrollbar") {
                                    // Find the start of the webkit CSS block (search backwards for newline before the comment)
                                    if let Some(start) = updated_css[..pos].rfind("\n/*") {
                                        updated_css.truncate(start);
                                    } else {
                                        updated_css.truncate(pos);
                                    }
                                }
                                // Append new webkit scrollbar CSS
                                updated_css.push('\n');
                                updated_css.push_str(&new_webkit_css);
                                *css_rc_for_preview.borrow_mut() = updated_css;
                                log::debug!("Updated webkit scrollbar CSS in preview CSS string");

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
                                    
                                    log::debug!("Updated GTK scrollbar CSS for new theme: thumb={}, track={}", thumb, track);
                                    
                                    // Force immediate paned separator CSS update for theme change
                                    // Check current scrollbar visibility state
                                    let vadj = editor_sw_for_preview.vadjustment();
                                    let upper = vadj.upper();
                                    let page_size = vadj.page_size();
                                    let scrollbar_visible = upper > page_size;
                                    
                                    let paned_css = if scrollbar_visible {
                                        // Scrollbar visible - use 1px separator
                                        format!(
                                            r#"
paned > separator {{
    min-width: 1px;
    min-height: 1px;
    background: transparent;
    border: none;
}}

paned > separator:active {{
    min-width: 1px;
    background: {};
    opacity: 0.5;
}}
                                            "#,
                                            thumb
                                        )
                                    } else {
                                        // No scrollbar - use 12px separator
                                        format!(
                                            r#"
paned > separator {{
    min-width: 12px;
    min-height: 12px;
    background: {};
    border: none;
}}
                                            "#,
                                            track
                                        )
                                    };
                                    
                                    let paned_provider = gtk4::CssProvider::new();
                                    paned_provider.load_from_data(&paned_css);
                                    gtk4::style_context_add_provider_for_display(
                                        &display,
                                        &paned_provider,
                                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                                    );
                                    log::debug!("Updated paned separator CSS for theme change: scrollbar_visible={}", scrollbar_visible);
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
            safe_source_remove(id);
        }
        let preview_theme_timeout_clone2 = Rc::clone(&preview_theme_timeout_clone);
        let webview_clone = webview_rc_for_preview.clone();
        let css_clone = Rc::clone(&css_rc_for_preview);
        let html_opts_clone = std::rc::Rc::clone(&html_opts_for_preview);
        let buffer_clone = Rc::clone(&buffer_rc_for_preview);
        let wheel_clone = wheel_js_for_preview.clone();
        let theme_mode_clone = Rc::clone(&theme_mode_for_preview);
        let document_buffer_clone = document_buffer_for_preview.as_ref().map(Rc::clone);
        let update_code_clone = Rc::clone(&update_html_code_view_for_preview);
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
                &webview_clone.borrow(),
                &css_clone,
                &html_opts_clone,
                &buffer_clone,
                &wheel_clone,
                &theme_mode_clone,
                doc_path.as_deref(),
            );
            
            // Also update code view if it exists (for theme changes)
            (update_code_clone)();
            
            preview_theme_timeout_clone2.set(None);
            glib::ControlFlow::Break
        });
        preview_theme_timeout_clone.set(Some(id));
    }) as Box<dyn Fn(&str)>;

    // Set up split percentage indicator with cascade prevention from split controller
    let split_indicator = setup_split_percentage_indicator_with_cascade_prevention(&paned, Some(split_controller.position_being_set()));
    let overlay = split_indicator.widget().clone();

    (
        paned,  // Return original paned for compatibility
        webview_rc,  // Return wrapped WebView for reparenting support
        css_rc,
        Box::new({
            let r = std::rc::Rc::clone(&refresh_preview_impl);
            move || r()
        }) as Box<dyn Fn()>,
        update_theme,
        update_preview_theme,
        buffer_rc.as_ref().clone(),
        source_view.clone(),
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
        overlay,           // 10: Overlay widget
        split_controller,  // 11: Split position controller
    )
}
