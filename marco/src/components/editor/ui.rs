// Main editor construction with integrated preview.
//
// This module builds the complete editor interface including:
// - Split pane layout (editor + preview)
// - SourceView5 text editing
// - WebKit6 HTML preview rendering
// - Bidirectional scroll synchronization
// - Theme management and syntax highlighting
// - Document buffer integration
// - Debounced content processing (intelligence, rendering, extensions)
//
// # Architecture
//
// The editor is constructed as a GTK Paned widget containing:
// 1. **Left pane**: SourceView editor with syntax highlighting
// 2. **Right pane**: WebKit WebView for HTML preview
//
// Content changes trigger debounced processing:
// - Preview rendering (300ms debounce)
// - Intelligence syntax highlighting (150ms debounce)
// - Extension processing (500ms debounce)
//
// # Platform Support
//
// Currently Linux-only (uses WebKit6 for preview).
// Cross-platform support planned using wry/WebView2 for Windows.
//
// When adding Windows support, `create_editor_with_preview_and_buffer()`
// will need conditional compilation for WebView creation.

use crate::components::editor::display_config::extract_xml_color_value;
use crate::components::editor::sourceview::render_editor_with_view;
use crate::components::editor::utilities::AsyncExtensionManager;
use crate::components::viewer::javascript::{wheel_js, SCROLL_REPORT_JS, SCROLL_RESTORE_JS};
#[cfg(target_os = "windows")]
use crate::components::viewer::javascript::{HOVER_REPORT_JS, WIN_ZOOM_BAR_HTML};
use crate::components::viewer::preview_types::{EditorReturn, ViewMode};
use crate::footer::FooterLabels;
#[cfg(target_os = "linux")]
use crate::logic::signal_manager::safe_source_remove;
use crate::ui::splitview::setup_split_percentage_indicator_with_cascade_prevention;
#[cfg(target_os = "windows")]
use gio;
#[cfg(target_os = "windows")]
use glib;
use gtk4::prelude::*;
use gtk4::Paned;
use marco_core::RenderOptions; // New parser API
use marco_shared::cache::global_parser_cache;
use sourceview5::prelude::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

// Thread-local handle to the active TOC panel, set once during editor init.
thread_local! {
    static TOC_PANEL_HANDLE: RefCell<Option<crate::ui::toc_panel::TocPanelHandle>> =
        const { RefCell::new(None) };
}

/// Store the TOC panel handle so the footer button and the debounce pipeline
/// can reach it without threading access through the whole call stack.
pub fn set_toc_panel_handle(handle: crate::ui::toc_panel::TocPanelHandle) {
    TOC_PANEL_HANDLE.with(|cell| {
        *cell.borrow_mut() = Some(handle);
    });
}

/// Access the TOC panel handle if it has been registered.
pub fn with_toc_panel<F: FnOnce(&crate::ui::toc_panel::TocPanelHandle)>(f: F) {
    TOC_PANEL_HANDLE.with(|cell| {
        if let Some(handle) = cell.borrow().as_ref() {
            f(handle);
        }
    });
}

use crate::components::viewer::css_utils::{pretty_print_html, webkit_scrollbar_css};

// Renderer functions are Linux-only (keep use guarded where necessary)

/// Parameters for creating the editor with preview
pub struct EditorParams {
    pub preview_theme_filename: String,
    pub preview_theme_dir: String,
    pub theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    pub theme_mode: Rc<RefCell<String>>,
}

/// Return the debounce delay for preview rendering based on document size.
///
/// Larger documents need a longer quiet period so rapid typing does not
/// saturate the render thread pool.  Values are anchored to the existing
/// 400ms baseline so small files feel unchanged.
fn preview_debounce_duration(line_count: i32) -> std::time::Duration {
    std::time::Duration::from_millis(match line_count {
        0..=500 => 400, // existing baseline — no change for small files
        501..=2000 => 600,
        2001..=10000 => 900,
        _ => 1200,
    })
}

pub(crate) fn split_hover_content(raw: &str) -> (String, String) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return ("Info".to_string(), String::new());
    }

    let (title, details) = if let Some(rest) = trimmed.strip_prefix("**") {
        if let Some(end_idx) = rest.find("**") {
            let title = rest[..end_idx].trim().to_string();
            let details = rest[end_idx + 2..].trim().to_string();
            (title, details)
        } else {
            ("Info".to_string(), trimmed.to_string())
        }
    } else {
        ("Info".to_string(), trimmed.to_string())
    };

    (title, details.replace('`', "").trim().to_string())
}

pub(crate) fn diagnostic_at_offset(
    diagnostics: &[marco_core::intelligence::Diagnostic],
    byte_offset: usize,
) -> Option<marco_core::intelligence::Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| d.span.start.offset <= byte_offset && byte_offset < d.span.end.offset)
        // Narrowest span wins (most specific diagnostic)
        .min_by_key(|d| d.span.end.offset.saturating_sub(d.span.start.offset))
        .cloned()
}

pub(crate) fn diagnostic_hover_markup(
    diagnostic: &marco_core::intelligence::Diagnostic,
) -> (String, String, (usize, usize, String)) {
    let severity = match diagnostic.severity {
        marco_core::intelligence::DiagnosticSeverity::Error => "Error",
        marco_core::intelligence::DiagnosticSeverity::Warning => "Warning",
        marco_core::intelligence::DiagnosticSeverity::Info => "Info",
        marco_core::intelligence::DiagnosticSeverity::Hint => "Hint",
    };

    let title_text = diagnostic
        .title_resolved()
        .unwrap_or(diagnostic.message.as_str());
    let title = format!("{}: {}", severity, title_text);

    let mut body_lines = vec![format!("Code: {}", diagnostic.code_id())];
    if let Some(description) = diagnostic.description_resolved() {
        if !description.trim().is_empty() {
            body_lines.push(format!("About: {}", description));
        }
    }
    body_lines.push(format!("Fix: {}", diagnostic.fix_suggestion_resolved()));

    let body = body_lines.join("\n");
    let signature = (
        diagnostic.span.start.offset,
        diagnostic.span.end.offset,
        format!("{}|{}", title, body),
    );
    (title, body, signature)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeIntelligenceSettings {
    pub(crate) markdown_intelligence_enabled: bool,
    pub(crate) diagnostics_underlines_enabled: bool,
    pub(crate) diagnostics_hover_enabled: bool,
    pub(crate) markdown_hover_enabled: bool,
    pub(crate) syntax_colors_enabled: bool,
    pub(crate) level_1_enabled: bool,
    pub(crate) level_2_enabled: bool,
    pub(crate) level_3_enabled: bool,
    pub(crate) level_4_enabled: bool,
}

impl Default for RuntimeIntelligenceSettings {
    fn default() -> Self {
        Self {
            markdown_intelligence_enabled: true,
            diagnostics_underlines_enabled: true,
            diagnostics_hover_enabled: true,
            markdown_hover_enabled: true,
            syntax_colors_enabled: true,
            level_1_enabled: true,
            level_2_enabled: true,
            // Keep runtime defaults in sync with footer diagnostics defaults:
            // errors + warnings on, infos + hints off unless explicitly enabled.
            level_3_enabled: false,
            level_4_enabled: false,
        }
    }
}

fn read_runtime_intelligence_settings(
    settings_manager: Option<&std::sync::Arc<marco_shared::logic::swanson::SettingsManager>>,
) -> RuntimeIntelligenceSettings {
    let Some(settings_manager) = settings_manager else {
        return RuntimeIntelligenceSettings::default();
    };

    // Reload from disk so changes made by the settings window (which uses
    // its own SettingsManager instance) are picked up immediately.
    if let Err(e) = settings_manager.reload_settings() {
        log::debug!("Failed to reload settings for intelligence: {}", e);
    }

    let settings = settings_manager.get_settings();
    let editor = settings.editor.unwrap_or_default();
    let filter = editor.diagnostics_filter.unwrap_or(
        marco_shared::logic::swanson::DiagnosticsFilterSettings {
            errors: Some(true),
            warnings: Some(true),
            infos: Some(false),
            hints: Some(false),
        },
    );

    let diagnostics_underlines_enabled = editor.diagnostics_underlines_enabled.unwrap_or(true);
    let diagnostics_hover_enabled = editor.diagnostics_hover_enabled.unwrap_or(true);
    let markdown_hover_enabled = editor.markdown_hover_enabled.unwrap_or(true);
    let syntax_colors_enabled = editor.syntax_colors.unwrap_or(true);

    // Master enablement is derived from visible feature toggles.
    // This avoids a hidden legacy master flag from unexpectedly disabling all
    // intelligence behavior after the UI master switch was removed.
    let markdown_intelligence_enabled = diagnostics_underlines_enabled
        || diagnostics_hover_enabled
        || markdown_hover_enabled
        || syntax_colors_enabled;

    RuntimeIntelligenceSettings {
        markdown_intelligence_enabled,
        diagnostics_underlines_enabled,
        diagnostics_hover_enabled,
        markdown_hover_enabled,
        syntax_colors_enabled,
        level_1_enabled: filter.errors.unwrap_or(true),
        level_2_enabled: filter.warnings.unwrap_or(true),
        level_3_enabled: filter.infos.unwrap_or(true),
        level_4_enabled: filter.hints.unwrap_or(true),
    }
}

fn diagnostic_severity_enabled(
    severity: &marco_core::intelligence::DiagnosticSeverity,
    settings: RuntimeIntelligenceSettings,
) -> bool {
    match severity {
        marco_core::intelligence::DiagnosticSeverity::Error => settings.level_1_enabled,
        marco_core::intelligence::DiagnosticSeverity::Warning => settings.level_2_enabled,
        marco_core::intelligence::DiagnosticSeverity::Info => settings.level_3_enabled,
        marco_core::intelligence::DiagnosticSeverity::Hint => settings.level_4_enabled,
    }
}

pub fn create_editor_with_preview_and_buffer(
    _window: &gtk4::ApplicationWindow,
    params: EditorParams,
    labels: Rc<FooterLabels>,
    settings_path: &str,
    _document_buffer: Option<Rc<RefCell<marco_shared::logic::buffer::DocumentBuffer>>>,
) -> EditorReturn {
    let preview_theme_filename = &params.preview_theme_filename;
    let preview_theme_dir = &params.preview_theme_dir;
    let theme_manager = params.theme_manager;
    let theme_mode = params.theme_mode;
    let intelligence_settings_manager =
        match marco_shared::logic::swanson::SettingsManager::initialize(std::path::PathBuf::from(
            settings_path,
        )) {
            Ok(manager) => Some(manager),
            Err(err) => {
                log::warn!(
                    "Failed to initialize settings manager for intelligence runtime settings: {}",
                    err
                );
                None
            }
        };
    let resolve_runtime_intelligence_settings: Rc<dyn Fn() -> RuntimeIntelligenceSettings> = {
        let settings_manager = intelligence_settings_manager.clone();
        Rc::new(move || read_runtime_intelligence_settings(settings_manager.as_ref()))
    };
    // Implementation largely copied from previous editor.rs but using helper modules
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(600);

    // Create split controller to manage position constraints and locking
    use crate::components::viewer::layout_controller::SplitController;
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

    // Make the editor scroller discoverable for detached preview windows.
    crate::components::editor::editor_manager::set_primary_editor_scrolled_window(
        &editor_scrolled_window,
    );
    editor_widget.set_hexpand(true);
    editor_widget.set_vexpand(true);

    // Editor widget goes directly as the start child of the outer editor/preview paned.
    // The TOC panel will be wired at the outermost level after the preview is set up.
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
        if crate::logic::tables::is_table_auto_align_enabled() {
            // Hold the guard for the entire duration of the navigation so that
            // cursor-position signals fired by our own replace_table_in_buffer
            // calls do NOT trigger the cursor-leave reformat path.
            let _nav_guard = crate::logic::tables::NavigationGuard::new();
            if crate::components::editor::table_edit::handle_table_navigation_key(
                &source_view_clone,
                keyval,
                state,
            ) {
                return Propagation::Stop;
            }
        }

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
                // Shift+Enter: Insert a visible blank line (spacer paragraph).
                //
                // CommonMark ignores consecutive blank lines between blocks, so
                // inserting "\n\n" produces no rendered output between elements.
                //
                // Inserting a non-breaking space (\u{00A0}) on its own line creates
                // a real paragraph node because trim() does NOT strip \u{00A0}.
                // The renderer emits <p>&#xa0;</p>, which has full line-height height
                // in the preview — visually pushing content down. The character is
                // invisible in the editor and doesn't interfere with the document.
                buffer.insert_at_cursor("\u{00A0}\n\n");
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

    // Auto-align table when the cursor leaves a table region.
    // We track the previous cursor line; when it was inside a table and the
    // new line is outside, we reformat the table in-place without moving the cursor.
    //
    // IMPORTANT: `replace_table_in_buffer` (delete+insert) itself moves the
    // GTK cursor, which re-fires `cursor-position`.  The `is_reformatting`
    // guard prevents infinite re-entry (and the resulting stack-overflow
    // segfault).
    {
        use gtk4::prelude::TextBufferExt as _;
        let prev_line: Rc<std::cell::Cell<i32>> = Rc::new(std::cell::Cell::new(-1));
        let is_reformatting: Rc<std::cell::Cell<bool>> = Rc::new(std::cell::Cell::new(false));
        let buffer_for_leave = source_view.buffer();
        buffer_for_leave.connect_notify_local(Some("cursor-position"), move |buf, _| {
            // Block re-entrant calls triggered by our own buffer writes.
            if is_reformatting.get() {
                return;
            }
            // Block calls triggered by Tab/Enter navigation's buffer writes.
            if crate::logic::tables::is_table_navigation_in_progress() {
                return;
            }
            if !crate::logic::tables::is_table_auto_align_enabled() {
                return;
            }
            let text_buf: gtk4::TextBuffer = buf.clone().upcast();
            let cursor = text_buf.iter_at_offset(text_buf.cursor_position());
            let new_line = cursor.line();
            let old_line = prev_line.get();
            prev_line.set(new_line);

            if old_line >= 0
                && old_line != new_line
                && crate::components::editor::table_edit::line_is_in_table(&text_buf, old_line)
                && !crate::components::editor::table_edit::line_is_in_table(&text_buf, new_line)
            {
                // Defer the buffer modification to the next idle cycle.
                //
                // When the cursor moves via a mouse click, GTK still holds an
                // internal TextIter pointing at the click position after this
                // signal fires.  Any synchronous delete+insert in this handler
                // invalidates that iter and triggers:
                //   "gtk_text_buffer_set_mark: assertion … failed"
                //
                // By scheduling the reformat as an idle callback we let GTK
                // finish all cursor-placement housekeeping first, then safely
                // rewrite the buffer in the next idle cycle.
                let text_buf_idle = text_buf.clone();
                let is_reformatting_idle = is_reformatting.clone();
                glib::idle_add_local_once(move || {
                    is_reformatting_idle.set(true);
                    crate::components::editor::table_edit::format_table_at_line_no_cursor(
                        &text_buf_idle,
                        old_line,
                    );
                    is_reformatting_idle.set(false);
                });
            }
        });
    }

    // SourceView5 native hover provider.
    // Uses the built-in GtkSourceHover infrastructure instead of a custom
    // EventControllerMotion + Popover approach.
    let current_diagnostics: Rc<RefCell<Vec<marco_core::intelligence::Diagnostic>>> =
        Rc::new(RefCell::new(Vec::new()));

    {
        let hover = source_view.hover();
        hover.set_hover_delay(350);

        let provider = crate::components::editor::hover_provider::MarcoHoverProvider::new(
            Rc::clone(&current_diagnostics),
            Rc::clone(&resolve_runtime_intelligence_settings),
        );
        hover.add_provider(&provider);
    }

    // Editor callback registration moved later so buffer handle can be captured

    // Register this editor for line numbers updates
    {
        let source_view_for_line_numbers = source_view.clone();
        if let Some(_line_numbers_id) =
            crate::components::editor::editor_manager::register_line_numbers_callback_globally(
                move |show_line_numbers: bool| {
                    log::debug!(
                        "Applying line numbers setting to SourceView: {}",
                        show_line_numbers
                    );
                    crate::logic::rtl::apply_rtl_line_numbers(
                        show_line_numbers,
                        &source_view_for_line_numbers,
                    );
                },
            )
        {
            log::debug!(
                "Registered line numbers callback with editor manager: ID {:?}",
                _line_numbers_id
            );
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
    css.push_str(&crate::components::viewer::css_utils::complete_indentation_css());

    // Add syntect CSS for code block highlighting.
    // On Linux this is also used in WebKit helpers; on Windows we include it here
    // so preview code blocks are styled consistently.
    {
        let normalized_theme = if theme_mode.borrow().contains("dark") {
            "dark"
        } else {
            "light"
        };

        match crate::logic::syntax_highlighter::generate_css_with_global(normalized_theme) {
            Ok(syntect_css) => {
                css.push('\n');
                css.push_str(&syntect_css);
            }
            Err(e) => {
                log::warn!(
                    "Failed to generate syntect CSS for theme '{}': {}",
                    normalized_theme,
                    e
                );
            }
        }
    }

    // wheel JS with scroll report for bidirectional sync
    let scroll_scale: f64 = std::env::var("MARCO_SCROLL_SCALE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);
    let wheel_js = wheel_js(scroll_scale);
    let mut wheel_with_report = wheel_js.clone();
    wheel_with_report.push_str(SCROLL_REPORT_JS);
    wheel_with_report.push_str(SCROLL_RESTORE_JS);
    // Windows-only: native wry/WebView2 lacks a hit-test signal for hovered
    // links and the GTK zoom-bar overlay is hidden behind the WebView2 child
    // window. Inject a JS bridge that posts hovered link URLs and an in-page
    // zoom toolbar via IPC instead.
    #[cfg(target_os = "windows")]
    {
        wheel_with_report.push_str(HOVER_REPORT_JS);
        wheel_with_report.push_str(WIN_ZOOM_BAR_HTML);
    }
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
        crate::components::viewer::css_utils::gtk_scrollbar_css(&initial_thumb, &initial_track);
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

                log::debug!(
                    "Paned separator CSS updated: scrollbar_visible={}",
                    scrollbar_visible
                );
            }
        }

        glib::ControlFlow::Continue
    });

    let buffer_rc: Rc<sourceview5::Buffer> = Rc::new(buffer);
    // Apply intelligence syntax tag colors for the current theme so tags exist before
    // any intelligence/UI code attempts to lookup them by name. Only apply if the
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
                move |new_settings: &crate::components::editor::display_config::EditorDisplaySettings| {
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

                    // Apply line numbers setting (direction-aware: right gutter in RTL mode)
                    crate::logic::rtl::apply_rtl_line_numbers(
                        new_settings.show_line_numbers,
                        &source_view_for_callback,
                    );

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

    // Precreated ScrolledWindow for code view - shared between platforms
    let precreated_code_sw: Rc<gtk4::ScrolledWindow> = {
        let sw = gtk4::ScrolledWindow::new();
        sw.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        sw.add_css_class("editor-scrolled");
        Rc::new(sw)
    };

    #[cfg(target_os = "windows")]
    {
        // On Windows we don't have WebKit scroll integration yet, but we can still
        // synchronize the editor with the HTML code view (TextView) scroller.
        if let Some(global_sync) =
            crate::components::editor::editor_manager::get_global_scroll_synchronizer()
        {
            if global_sync.is_enabled() {
                global_sync.connect_scrolled_windows_bidirectional(
                    &editor_scrolled_window,
                    precreated_code_sw.as_ref(),
                );
                log::debug!("Scroll synchronization initialized between editor and code view");
            } else {
                log::debug!("Scroll sync disabled; skipping editor/code sync wiring");
            }
        } else {
            log::warn!(
                "Failed to initialize scroll synchronization: global scroll synchronizer not available"
            );
        }
    }

    #[cfg(target_os = "windows")]
    let code_view_widget_for_windows: Rc<
        RefCell<Option<crate::components::viewer::preview_types::PlatformWebView>>,
    > = Rc::new(RefCell::new(None));

    // Shared stack and optional webview wrapper so both platforms can add children
    let stack = gtk4::Stack::new();
    let webview_rc_opt: Option<
        Rc<RefCell<crate::components::viewer::preview_types::PlatformWebView>>,
    >;

    // For large documents skip the synchronous initial render to keep the GTK
    // main thread responsive.  `refresh_preview_impl()` fires immediately after
    // WebView setup (see below) and uses `gio::spawn_blocking` for the actual
    // render, so the content appears without ever blocking the event loop.
    // For small documents the synchronous path is kept: it's fast (<5 ms) and
    // avoids a brief visual blank-then-content flash on startup.
    let initial_line_count = initial_text.lines().count();
    let initial_html_body = if initial_line_count < 300 {
        match global_parser_cache().render_with_cache(&initial_text, (*html_opts_rc).clone()) {
            Ok(html) => html,
            Err(e) => format!("Error rendering HTML: {}", e),
        }
    } else {
        // Large document: use an empty body now; the async refresh below fills it in.
        String::new()
    };

    #[cfg(target_os = "linux")]
    let pretty_initial = pretty_print_html(&initial_html_body);

    #[cfg(target_os = "linux")]
    {
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

        // Wire link-hover → footer: when the cursor enters a link in the preview,
        // show its URL in the footer spacer area; clear it when the cursor leaves.
        {
            let labels_for_hover = Rc::clone(&labels);
            crate::components::viewer::webkit6::setup_link_hover_status(
                &webview,
                move |url: Option<String>| {
                    crate::footer::update_hovered_link(&labels_for_hover, url.as_deref());
                },
            );
        }

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
            None,              // No base URI needed for code view
            bg_init,           // Pass editor background color
            fg_init,           // Pass editor foreground color
            Some(&thumb_init), // Pass scrollbar thumb color
            Some(&track_init), // Pass scrollbar track color
        )
        .expect("Failed to create code viewer WebView");

        // Wrap WebView into precreated ScrolledWindow for consistency
        precreated_code_sw.set_child(Some(&webview_code));

        let _precreated_code_sw_holder: Rc<RefCell<Option<Rc<gtk4::ScrolledWindow>>>> =
            Rc::new(RefCell::new(Some(precreated_code_sw.clone())));

        // Wrap the WebView in a loading overlay so we can show a centered
        // indeterminate progress bar while large files are parsed/rendered.
        // The bar uses GTK's default theme so it follows light/dark mode
        // automatically — no HTML/CSS involvement.
        let loading_overlay =
            crate::components::viewer::loading_overlay::LoadingOverlay::new(&webview);
        crate::components::viewer::loading_overlay::set_global(loading_overlay.clone());

        // Hide the overlay only when the WebView has actually finished
        // painting the new page — load_html / load_html_when_ready merely
        // *queue* the load, so hiding right after returning would dismiss
        // the bar seconds before the new content replaces the old welcome
        // HTML on screen.
        {
            use webkit6::prelude::WebViewExt;
            webview.connect_load_changed(|_wv, event| {
                if event == webkit6::LoadEvent::Finished {
                    crate::components::viewer::loading_overlay::hide();
                }
            });
        }

        stack.add_named(loading_overlay.widget(), Some("html_preview"));
        stack.add_named(precreated_code_sw.as_ref(), Some("code_preview"));
        stack.set_visible_child(loading_overlay.widget());
        paned.set_end_child(Some(&stack));

        // Expose webview wrapper for reparenting/return
        webview_rc_opt = Some(Rc::clone(&webview_rc));
    }

    #[cfg(target_os = "windows")]
    {
        // Windows (and other) fallback: create PlatformWebView (wry) where possible
        // Use test HTML for empty document to mirror Linux behaviour (welcome message)
        let initial_html_body_with_js = if initial_html_body.trim().is_empty() {
            crate::components::viewer::wry::generate_test_html(&wheel_js_rc)
        } else {
            let mut s = initial_html_body.clone();
            s.push_str(&wheel_js_rc);
            s
        };

        // Wrap into a full HTML document, and compute a base URI for relative paths.
        let combined_css = css_rc.borrow().clone();
        let theme_mode_for_wrap = theme_mode_rc.borrow().clone();
        let base_uri = _document_buffer
            .as_ref()
            .and_then(|buf| buf.borrow().get_file_path().map(|p| p.to_path_buf()))
            .and_then(crate::components::viewer::backend::generate_base_uri_from_path);
        crate::components::viewer::wry::set_latest_preview_base_uri(base_uri.clone());

        let full_html = crate::components::viewer::backend::wrap_html_document(
            &initial_html_body_with_js,
            &combined_css,
            &theme_mode_for_wrap,
            None,
        );

        if let Ok(mut guard) = crate::components::viewer::wry::LATEST_PREVIEW_HTML
            .get_or_init(|| std::sync::Mutex::new(String::new()))
            .lock()
        {
            *guard = full_html.clone();
        }

        let pretty_initial = pretty_print_html(&initial_html_body_with_js);

        // Try to create a native Windows embedded WebView (wry). PlatformWebView
        // will fallback to a placeholder container if it cannot obtain a Win32 handle.
        let platform_webview =
            crate::components::viewer::wry_platform_webview::PlatformWebView::new(_window);

        // Set initial WebView background to match the current theme mode so the
        // GTK container (visible while the HWND is offscreen during loading) shows
        // the correct colour instead of a white GTK-widget flash.
        {
            let is_dark = theme_mode_for_wrap.eq_ignore_ascii_case("dark");
            let initial_bg = if is_dark {
                gtk4::gdk::RGBA::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0, 1.0)
            } else {
                gtk4::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)
            };
            platform_webview.set_background_color_rgba(&initial_bg);
        }

        // Wire footer hovered-link updates from the preview's JS hover-report
        // bridge. webkit6 provides this natively via `connect_mouse_target_changed`
        // (see Linux branch above); on Windows we receive `marco_hover:<url>`
        // IPC messages and forward them to the same footer label.
        {
            let labels_for_hover = Rc::clone(&labels);
            platform_webview.set_hover_link_callback(move |url: Option<String>| {
                crate::footer::update_hovered_link(&labels_for_hover, url.as_deref());
            });
        }

        // Initialize scroll synchronization between editor and the embedded wry preview.
        if let Some(global_sync) =
            crate::components::editor::editor_manager::get_global_scroll_synchronizer()
        {
            if global_sync.is_enabled() {
                global_sync.connect_scrolled_window_and_platform_webview(
                    &editor_scrolled_window,
                    &platform_webview,
                );
                log::debug!("Scroll synchronization initialized between editor and wry preview");
            }
        }

        // Load initial HTML into the platform webview (safe no-op if inner not ready)
        crate::components::viewer::backend::load_html_when_ready(
            &platform_webview,
            full_html.clone(),
            base_uri.clone(),
        );

        let webview_widget: gtk4::Widget = platform_webview.widget();
        let webview_rc = Rc::new(RefCell::new(platform_webview));

        // Create a syntax-highlighted code preview backed by its own
        // `PlatformWebView` (Step 5b). Uses the shared `code_view_html`
        // builders so the rendered output matches the webkit6 / Linux branch
        // byte-for-byte.
        let code_view_pv = crate::components::viewer::wry::create_html_source_viewer_webview(
            _window,
            &pretty_initial,
            &theme_mode_for_wrap,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("Failed to create Windows code preview widget");
        let code_view_widget: gtk4::Widget = code_view_pv.widget();
        precreated_code_sw.set_child(Some(&code_view_widget));
        *code_view_widget_for_windows.borrow_mut() = Some(code_view_pv);

        let _precreated_code_sw_holder: Rc<RefCell<Option<Rc<gtk4::ScrolledWindow>>>> =
            Rc::new(RefCell::new(Some(precreated_code_sw.clone())));

        // Wrap the wry-backed webview widget in a loading overlay so the
        // centered indeterminate progress bar can appear over the preview
        // while large files are parsed/rendered.
        let loading_overlay =
            crate::components::viewer::loading_overlay::LoadingOverlay::new(&webview_widget);
        crate::components::viewer::loading_overlay::set_global(loading_overlay.clone());
        // On Windows the wry HWND paints on top of all GTK content, so the GTK
        // progress frame is never visible while the WebView is in its normal
        // position.  Wire up the offscreen hook so that show()/hide() move the
        // HWND out of the way while rendering and restore it when done.
        #[cfg(target_os = "windows")]
        {
            let webview_for_hook = webview_rc.borrow().clone();
            loading_overlay.set_offscreen_hook(move |offscreen| {
                webview_for_hook.set_offscreen_for_loading(offscreen);
            });
        }

        stack.add_named(loading_overlay.widget(), Some("html_preview"));
        stack.add_named(precreated_code_sw.as_ref(), Some("code_preview"));
        stack.set_visible_child(loading_overlay.widget());
        paned.set_end_child(Some(&stack));

        // Expose webview wrapper for reparenting/return
        webview_rc_opt = Some(Rc::clone(&webview_rc));
    }

    // refresh_preview closure
    let wheel_js_for_refresh = wheel_js_rc.clone();

    // Read page view settings from SettingsManager and create a shared state handle.
    // The handle is captured in refresh_preview_impl and updated live when the settings dialog changes.
    let page_view_rc: std::rc::Rc<
        RefCell<crate::components::viewer::preview_types::PageViewState>,
    > = {
        use crate::components::viewer::preview_types::PageViewState;
        let state = if let Some(ref sm) = intelligence_settings_manager {
            let settings = sm.get_settings();
            let layout = settings.layout.as_ref();
            PageViewState {
                enabled: layout.and_then(|l| l.page_view_enabled).unwrap_or(false),
                paper: layout
                    .and_then(|l| l.page_view_paper.clone())
                    .unwrap_or_else(|| "A4".to_string()),
                orientation: layout
                    .and_then(|l| l.page_view_orientation.clone())
                    .unwrap_or_else(|| "portrait".to_string()),
                margin_mm: layout.and_then(|l| l.page_view_margin_mm).unwrap_or(20),
                show_page_numbers: layout
                    .and_then(|l| l.page_view_show_page_numbers)
                    .unwrap_or(true),
                columns_per_row: layout
                    .and_then(|l| l.page_view_columns)
                    .unwrap_or(1)
                    .clamp(1, 4),
            }
        } else {
            PageViewState {
                enabled: false,
                paper: "A4".to_string(),
                orientation: "portrait".to_string(),
                margin_mm: 20,
                show_page_numbers: true,
                columns_per_row: 1,
            }
        };
        std::rc::Rc::new(RefCell::new(state))
    };

    #[cfg(target_os = "linux")]
    let is_initial_load = Rc::new(RefCell::new(true)); // Track if this is the first load
    #[cfg(target_os = "linux")]
    let last_css_hash = Rc::new(RefCell::new(0u64)); // Track CSS changes for theme updates
    #[cfg(target_os = "linux")]
    let last_document_path = Rc::new(RefCell::new(None::<std::path::PathBuf>)); // Track document path changes
    #[cfg(target_os = "linux")]
    let last_page_view_enabled = Rc::new(RefCell::new(false)); // Track page-view transitions (enable→disable needs full reload)
                                                               // Phase 8 Layer 1: content-hash guard — prevents re-rendering when the text
                                                               // hasn't actually changed (undo/redo, cursor moves, settings refreshes, etc.).
                                                               // Reset to 0 on new file open so the first render always fires.
    #[cfg(target_os = "linux")]
    let last_preview_hash: Rc<Cell<u64>> = Rc::new(Cell::new(0u64));
    // Phase 9 differential section DOM updates: hashes from the previous section
    // render.  Empty = force a full rebuild on the next section render.
    #[cfg(target_os = "linux")]
    let prev_section_hashes: Rc<RefCell<Vec<u64>>> = Rc::new(RefCell::new(Vec::new()));
    // Layer 2 — generation counter: every render request increments this.
    // When a render completes, it checks its captured generation against the
    // current value; if they differ, a newer request was made while it was
    // running (stale render) and the result is discarded.  On discard, the
    // content-hash guard is also reset so the next debounce fires a fresh render.
    #[cfg(target_os = "linux")]
    let preview_generation: Rc<Cell<u64>> = Rc::new(Cell::new(0u64));
    // Layer 2 — at-most-1-in-flight guard: prevents multiple renders queued
    // concurrently on the thread pool for the same document.
    #[cfg(target_os = "linux")]
    let preview_in_flight: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    // Clone document_buffer for use in refresh closure (Linux only)
    #[cfg(target_os = "linux")]
    let document_buffer_for_refresh = _document_buffer.as_ref().map(Rc::clone);

    // Windows refresh-state cells (Step 4b / §14.4). Mirror a subset of the
    // Linux state cells above. Used to decide between a full WebView reload
    // (which navigates and may flash) and an in-place
    // `update_html_content_smooth` patch (no navigation, scroll preserved).
    //
    // Full reload is required on:
    //   * the first refresh,
    //   * any CSS / theme change (the smooth path swaps body innerHTML but
    //     leaves the `<html data-theme>` root attribute untouched),
    //   * any document path change (clean slate for a new file),
    //   * any page-view (paged.js) transition (DOM is restructured).
    //
    // Content-hash dedup lets the closure skip cheap no-op refreshes
    // (cursor moves, settings refreshes, undo/redo to the same text).
    #[cfg(target_os = "windows")]
    let is_initial_load_win = Rc::new(RefCell::new(true));
    #[cfg(target_os = "windows")]
    let last_css_hash_win = Rc::new(RefCell::new(0u64));
    #[cfg(target_os = "windows")]
    let last_document_path_win = Rc::new(RefCell::new(None::<std::path::PathBuf>));
    #[cfg(target_os = "windows")]
    let last_page_view_enabled_win = Rc::new(RefCell::new(false));
    #[cfg(target_os = "windows")]
    let last_preview_hash_win: Rc<Cell<u64>> = Rc::new(Cell::new(0u64));
    // Generation counter for Windows: incremented on every render entry so a
    // render that finds the generation advanced (stale) can reset the hash guard
    // and let the next debounce fire a fresh pass.
    #[cfg(target_os = "windows")]
    let preview_generation_win: Rc<Cell<u64>> = Rc::new(Cell::new(0u64));
    // In-flight guard for Windows: set while a render is executing so a
    // reentrant trigger (impossible on the GTK main thread, but defensive)
    // is dropped rather than producing concurrent renders.
    #[cfg(target_os = "windows")]
    let preview_in_flight_win: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    #[cfg(target_os = "linux")]
    let refresh_preview_impl: std::rc::Rc<dyn Fn()> = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let webview_rc: Rc<RefCell<crate::components::viewer::preview_types::PlatformWebView>> =
            webview_rc_opt.as_ref().expect("webview_rc not set").clone();
        let webview = Rc::clone(&webview_rc);
        let theme_mode = Rc::clone(&theme_mode_rc);
        let html_opts = std::rc::Rc::clone(&html_opts_rc);
        let wheel_js_local = wheel_js_for_refresh.clone();
        let is_initial_load_clone = Rc::clone(&is_initial_load);
        let last_css_hash_clone = Rc::clone(&last_css_hash);
        let last_document_path_clone = Rc::clone(&last_document_path);
        let last_page_view_enabled_clone = Rc::clone(&last_page_view_enabled);
        let document_buffer_capture = document_buffer_for_refresh.clone();
        let page_view_capture = std::rc::Rc::clone(&page_view_rc);
        let last_preview_hash_capture = Rc::clone(&last_preview_hash);
        let prev_section_hashes_capture = Rc::clone(&prev_section_hashes);
        let preview_generation_capture = Rc::clone(&preview_generation);
        let preview_in_flight_capture = Rc::clone(&preview_in_flight);
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

            // Page view mode always requires a full reload (paged.js restructures DOM).
            // Also force a full reload on any transition so disabling clears the paged DOM.
            let page_view_active = page_view_capture.borrow().enabled;
            let page_view_changed = *last_page_view_enabled_clone.borrow() != page_view_active;
            *last_page_view_enabled_clone.borrow_mut() = page_view_active;

            if is_first_load
                || css_changed
                || doc_path_changed
                || page_view_active
                || page_view_changed
            {
                // Phase 8: reset content hash on any full reload so the next
                // smooth-path call always fires even if text hasn't changed.
                if doc_path_changed {
                    last_preview_hash_capture.set(0);
                }
                // Phase 9: clear section hashes so the next section render
                // performs a full DOM rebuild (new document / layout change).
                prev_section_hashes_capture.borrow_mut().clear();
                // Layer 2: reset in-flight and advance generation on full reload
                // so any render in progress knows it is stale.
                preview_in_flight_capture.set(false);
                preview_generation_capture.set(preview_generation_capture.get().wrapping_add(1));

                // For large documents without paged.js, always use the section
                // render even on the initial/reload path.  The full-document async
                // render and the first edit's section render are both async and
                // race to call load_html_when_ready.  Whichever lands last wins,
                // but only the section render produces the mc-s-N DOM structure
                // that subsequent incremental patches require.  If the full render
                // lands after the first section render, all patch JS calls become
                // no-ops and the preview freezes.  Using section render from the
                // start eliminates the race entirely.
                //
                // All non-paged-view documents (small or large) now go through the
                // section path.  For a small doc this produces one section and
                // behaves identically to the old smooth-update path, but with
                // incremental patching instead of a full-body swap on every edit.
                let use_section_render = !page_view_active;

                // Empty buffer (e.g. fresh app launch with an untitled
                // document): the section render path produces an empty
                // page. Route empty text through the welcome path so the
                // "Welcome to marco" placeholder is shown.
                let empty_text = buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), false)
                    .to_string();
                if use_section_render && empty_text.trim().is_empty() {
                    let base_uri = document_buffer_capture
                        .as_ref()
                        .and_then(|buf| buf.borrow().get_base_uri_for_webview());
                    let params = crate::components::viewer::renderer::PreviewRefreshParams {
                        webview: &webview.borrow(),
                        css: &css,
                        html_options: html_opts.as_ref(),
                        buffer: buffer.as_ref(),
                        wheel_js: &wheel_js_local,
                        theme_mode: &theme_mode,
                        base_uri: base_uri.as_deref(),
                        page_view: Some(std::rc::Rc::clone(&page_view_capture)),
                    };
                    crate::components::viewer::renderer::refresh_preview_into_webview_with_base_uri_and_doc_buffer(params);
                    *is_initial_load_clone.borrow_mut() = false;
                    return;
                }

                if use_section_render {
                    // Large doc, no page view: initial render via section path.
                    let text = buffer
                        .text(&buffer.start_iter(), &buffer.end_iter(), false)
                        .to_string();
                    let content_hash = marco_shared::cache::hash_content(&text);
                    last_preview_hash_capture.set(content_hash);

                    let my_gen = preview_generation_capture.get();
                    preview_in_flight_capture.set(true);

                    let hashes_rc = Rc::clone(&prev_section_hashes_capture);
                    let gen_rc = Rc::clone(&preview_generation_capture);
                    let in_flight_rc = Rc::clone(&preview_in_flight_capture);
                    let hash_reset_rc = Rc::clone(&last_preview_hash_capture);
                    let base_uri = document_buffer_capture
                        .as_ref()
                        .and_then(|buf| buf.borrow().get_base_uri_for_webview());
                    let theme_css = css.borrow().clone();
                    let theme_name = theme_mode.borrow().clone();
                    let syntax_css =
                        crate::components::viewer::renderer::generate_syntax_highlighting_css(
                            &theme_name,
                        );
                    let combined_css = format!(
                        "{}\n\n/* Syntax Highlighting CSS */\n{}",
                        theme_css, syntax_css
                    );
                    let params = crate::components::viewer::renderer::SectionRenderParams {
                        webview: webview.borrow().clone(),
                        html_options: (*html_opts).clone(),
                        wheel_js: (*wheel_js_local).clone(),
                        theme_mode: theme_name,
                        text,
                        prev_hashes: Vec::new(), // empty → full mc-s-N rebuild
                        css: combined_css,
                        base_uri,
                        cursor_line: 0, // initial load — cursor-first not needed
                    };
                    crate::components::viewer::renderer::refresh_preview_content_sections(
                        params,
                        move |new_hashes| {
                            *hashes_rc.borrow_mut() = new_hashes;
                            in_flight_rc.set(false);
                            if gen_rc.get() != my_gen {
                                hash_reset_rc.set(0);
                            }
                        },
                    );

                    // Warm the full-document AST in the background so the hover
                    // provider and footer diagnostics can reuse it without re-parsing.
                    // (The section render above already warms per-section entries; this
                    // warms the full-document hash needed by get_cached_ast.)
                    if doc_path_changed || is_first_load {
                        let text_for_ast = buffer
                            .text(&buffer.start_iter(), &buffer.end_iter(), false)
                            .to_string();
                        std::thread::spawn(move || {
                            let _ = marco_shared::cache::global_parser_cache()
                                .parse_and_cache_ast(&text_for_ast);
                            log::debug!("[editor] Full-document AST cached");
                        });
                    }
                } else {
                    // Small doc or page view: full-document render.
                    let base_uri = document_buffer_capture
                        .as_ref()
                        .and_then(|buf| buf.borrow().get_base_uri_for_webview());
                    let params = crate::components::viewer::renderer::PreviewRefreshParams {
                        webview: &webview.borrow(),
                        css: &css,
                        html_options: html_opts.as_ref(),
                        buffer: buffer.as_ref(),
                        wheel_js: &wheel_js_local,
                        theme_mode: &theme_mode,
                        base_uri: base_uri.as_deref(),
                        page_view: Some(std::rc::Rc::clone(&page_view_capture)),
                    };
                    crate::components::viewer::renderer::refresh_preview_into_webview_with_base_uri_and_doc_buffer(params);
                }

                // Mark as no longer initial load
                *is_initial_load_clone.borrow_mut() = false;
            } else {
                // Phase 8 Layer 1: content-hash guard for the smooth path.
                // text() is extracted here (once per debounce window) and hashed;
                // skip the render entirely if the content hasn't changed since the
                // last queued render (handles undo/redo, settings refreshes, etc.).
                let text = buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), false)
                    .to_string();
                let content_hash = marco_shared::cache::hash_content(&text);
                if last_preview_hash_capture.get() == content_hash {
                    return;
                }
                last_preview_hash_capture.set(content_hash);

                // Section-based incremental rendering for all document sizes.
                // Splitting and rendering happen off the main thread.  Only
                // sections whose content hash changed since the last debounce
                // are re-rendered; all others are served from the section HTML
                // cache.  Small documents produce a single section and behave
                // like the old smooth-update path but with a targeted DOM patch
                // instead of a full-body innerHTML swap.

                // Layer 2: increment generation and apply in-flight guard.
                let my_gen = preview_generation_capture.get().wrapping_add(1);
                preview_generation_capture.set(my_gen);
                if preview_in_flight_capture.get() {
                    // A render is already running.  The generation counter has
                    // been advanced so that render will detect it is stale on
                    // completion and will reset the hash guard to trigger one
                    // more render.
                    return;
                }
                preview_in_flight_capture.set(true);

                // Cursor line for cursor-section-first rendering (step 3).
                let cursor_line = {
                    let pos = buffer.cursor_position();
                    buffer.iter_at_offset(pos).line() as usize
                };

                let prev_hashes = prev_section_hashes_capture.borrow().clone();
                let hashes_rc = Rc::clone(&prev_section_hashes_capture);
                let gen_rc = Rc::clone(&preview_generation_capture);
                let in_flight_rc = Rc::clone(&preview_in_flight_capture);
                let hash_reset_rc = Rc::clone(&last_preview_hash_capture);
                let base_uri = document_buffer_capture
                    .as_ref()
                    .and_then(|buf| buf.borrow().get_base_uri_for_webview());
                let theme_css = css.borrow().clone();
                let theme_name = theme_mode.borrow().clone();
                let syntax_css =
                    crate::components::viewer::renderer::generate_syntax_highlighting_css(
                        &theme_name,
                    );
                let combined_css = format!(
                    "{}\n\n/* Syntax Highlighting CSS */\n{}",
                    theme_css, syntax_css
                );
                let params = crate::components::viewer::renderer::SectionRenderParams {
                    webview: webview.borrow().clone(),
                    html_options: (*html_opts).clone(),
                    wheel_js: (*wheel_js_local).clone(),
                    theme_mode: theme_name,
                    text,
                    prev_hashes,
                    css: combined_css,
                    base_uri,
                    cursor_line,
                };
                crate::components::viewer::renderer::refresh_preview_content_sections(
                    params,
                    move |new_hashes| {
                        *hashes_rc.borrow_mut() = new_hashes;
                        in_flight_rc.set(false);
                        // If generation advanced while we were rendering,
                        // invalidate the content-hash guard so the next
                        // debounce fires a fresh render with the latest text.
                        if gen_rc.get() != my_gen {
                            hash_reset_rc.set(0);
                        }
                    },
                );
            }
        })
    };

    #[cfg(target_os = "windows")]
    let refresh_preview_impl: std::rc::Rc<dyn Fn()> = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let code_view_widget_for_windows = Rc::clone(&code_view_widget_for_windows);
        let html_opts = std::rc::Rc::clone(&html_opts_rc);
        let wheel_js_local = wheel_js_for_refresh.clone();
        let theme_mode_for_preview = Rc::clone(&theme_mode_rc);
        let page_view_capture = std::rc::Rc::clone(&page_view_rc);
        // Capture the in-editor platform webview if present
        let webview_for_preview = webview_rc_opt.clone();
        let document_buffer_capture = _document_buffer.as_ref().map(Rc::clone);
        // Step 4b: refresh-state cells for full-reload-vs-smooth-update decision.
        let is_initial_load = Rc::clone(&is_initial_load_win);
        let last_css_hash = Rc::clone(&last_css_hash_win);
        let last_document_path = Rc::clone(&last_document_path_win);
        let last_page_view_enabled = Rc::clone(&last_page_view_enabled_win);
        let last_preview_hash = Rc::clone(&last_preview_hash_win);
        // Gap #4: generation counter + in-flight guard (mirrors Linux behaviour).
        let preview_generation = Rc::clone(&preview_generation_win);
        let preview_in_flight = Rc::clone(&preview_in_flight_win);
        std::rc::Rc::new(move || {
            // In-flight guard: the GTK main thread is single-threaded, but if a
            // stale timer or signal fires while a synchronous render is underway
            // (e.g. from an inner gtk::main_iteration call), skip this entry.
            if preview_in_flight.get() {
                log::trace!("[preview-win] skip refresh: render already in flight");
                return;
            }
            // Basic behaviour: re-render HTML into the code view (TextView) for preview
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string();

            let base_uri = document_buffer_capture
                .as_ref()
                .and_then(|buf| buf.borrow().get_file_path().map(|p| p.to_path_buf()))
                .and_then(crate::components::viewer::wry::generate_base_uri_from_path);
            crate::components::viewer::wry::set_latest_preview_base_uri(base_uri.clone());

            // ---- Step 4b: compute reload-trigger booleans ----
            // Any of these forces a full WebView navigation (load_html_with_base);
            // otherwise we use update_html_content_smooth which keeps the page
            // alive (no white flash, scroll preserved, MarcoCorePreview caches kept).
            let current_doc_path: Option<std::path::PathBuf> = document_buffer_capture
                .as_ref()
                .and_then(|buf| buf.borrow().get_file_path().map(|p| p.to_path_buf()));
            let doc_path_changed = {
                let prev = last_document_path.borrow();
                *prev != current_doc_path
            };

            let current_css_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                css.borrow().hash(&mut hasher);
                hasher.finish()
            };
            let css_changed = current_css_hash != *last_css_hash.borrow();

            let current_page_view_enabled = page_view_capture.borrow().enabled;
            let page_view_changed = current_page_view_enabled != *last_page_view_enabled.borrow();
            // Treat any page-view-active render as a full reload as well — paged.js
            // restructures the DOM in a way that the smooth innerHTML swap cannot
            // replicate (multi-column flow, page numbers, etc.).
            let page_view_active = current_page_view_enabled;

            let first_load = *is_initial_load.borrow();
            let force_full_reload = first_load
                || css_changed
                || doc_path_changed
                || page_view_changed
                || page_view_active;

            // Content-hash dedup: if no reload trigger and the buffer text hasn't
            // changed since the last render, skip the work entirely (cursor
            // moves, focus events, settings refreshes all become no-ops).
            let current_content_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                hasher.finish()
            };
            if !force_full_reload && current_content_hash == last_preview_hash.get() {
                log::trace!("[preview-win] skip refresh: content unchanged, no reload triggers");
                return;
            }

            // Update tracked state for the next call.
            *is_initial_load.borrow_mut() = false;
            *last_css_hash.borrow_mut() = current_css_hash;
            *last_document_path.borrow_mut() = current_doc_path.clone();
            *last_page_view_enabled.borrow_mut() = current_page_view_enabled;
            last_preview_hash.set(current_content_hash);

            // Increment the generation counter and arm the in-flight guard for
            // this render pass. The guard prevents the same closure from running
            // concurrently (impossible on the GTK main thread in normal use, but
            // defensive against edge cases). The generation counter lets us detect
            // if a newer request was queued while we were rendering and, if so,
            // reset the content-hash guard so the next debounce re-renders.
            let my_gen = preview_generation.get().wrapping_add(1);
            preview_generation.set(my_gen);
            preview_in_flight.set(true);

            let use_smooth_update = !force_full_reload;
            log::debug!(
                "[preview-win] refresh: first={} css_changed={} doc_changed={} page_view_changed={} page_view_active={} → {}",
                first_load,
                css_changed,
                doc_path_changed,
                page_view_changed,
                page_view_active,
                if use_smooth_update { "smooth" } else { "full-reload" }
            );

            // Fast path: empty document — no parsing needed, immediately show
            // the welcome HTML and release the in-flight guard.
            if text.trim().is_empty() {
                let html_body = crate::components::viewer::wry::generate_test_html(&wheel_js_local);
                let combined_css = css.borrow().clone();
                let theme_mode = theme_mode_for_preview.borrow().clone();
                let page_view = page_view_capture.borrow().clone();
                let full_html = if page_view.enabled {
                    let page_opts = marco_core::render::PageViewOptions {
                        paged_js_source: crate::components::viewer::pagedjs::PAGED_POLYFILL_JS,
                        paper: &page_view.paper,
                        orientation: &page_view.orientation,
                        margin_mm: page_view.margin_mm,
                        show_page_numbers: page_view.show_page_numbers,
                        wheel_js: &wheel_js_local,
                        columns_per_row: page_view.columns_per_row,
                        for_export: false,
                        title: "",
                        standalone_export: false,
                    };
                    crate::components::viewer::backend::wrap_html_document_paged(
                        &html_body,
                        &combined_css,
                        &theme_mode,
                        None,
                        &page_opts,
                    )
                } else {
                    crate::components::viewer::wry::wrap_html_document(
                        &html_body,
                        &combined_css,
                        &theme_mode,
                        None,
                    )
                };
                crate::components::viewer::wry::set_latest_live_html(&full_html);

                if let Ok(mut guard) = crate::components::viewer::wry::LATEST_PREVIEW_HTML
                    .get_or_init(|| std::sync::Mutex::new(String::new()))
                    .lock()
                {
                    *guard = full_html.clone();
                }

                // Step 4b: on the welcome/empty branch we always full-reload.
                if let Some(ref wv_rc) = webview_for_preview {
                    if let Ok(wv) = wv_rc.try_borrow() {
                        wv.load_html_with_base(&full_html, base_uri.as_deref());
                    } else {
                        log::debug!("In-editor webview borrow busy; skipping load");
                    }
                } else {
                    log::debug!(
                        "No embedded webview available; stored welcome HTML for detached preview"
                    );
                }

                let formatted = pretty_print_html(&html_body);
                if let Some(ref pv) = *code_view_widget_for_windows.borrow() {
                    let _ = crate::components::viewer::wry::update_code_view_smooth(
                        pv,
                        &formatted,
                        &theme_mode_for_preview.borrow(),
                        None,
                        None,
                        None,
                        None,
                    );
                }

                preview_in_flight.set(false);
                if preview_generation.get() != my_gen {
                    last_preview_hash.set(0);
                }
                return;
            }

            // Slow path: non-empty document — offload parsing to a thread-pool
            // worker so the GTK main thread (and its event loop) stays responsive
            // while large files are being rendered.
            //
            // `glib::spawn_future_local` schedules the async block on the GTK
            // main thread, so all Rc<…> values remain accessible.  The inner
            // `gio::spawn_blocking` dispatches only Send-safe owned data to the
            // thread pool.

            // Clone Rc handles — these stay on the main thread.
            let css_a = Rc::clone(&css);
            let theme_a = Rc::clone(&theme_mode_for_preview);
            let page_view_a = Rc::clone(&page_view_capture);
            let wheel_js_a = Rc::clone(&wheel_js_local);
            let webview_a = webview_for_preview.clone();
            let code_view_a = Rc::clone(&code_view_widget_for_windows);
            let flight_a = Rc::clone(&preview_in_flight);
            let gen_a = Rc::clone(&preview_generation);
            let hash_a = Rc::clone(&last_preview_hash);

            // Owned, Send values captured by the thread-pool closure.
            let text_bg = text.clone();
            let html_opts_bg = (*html_opts).clone();
            let base_uri_a = base_uri.clone();

            glib::spawn_future_local(async move {
                // --- thread pool: parse + render (may be slow for large files) ---
                let html_body = gio::spawn_blocking(move || {
                    match marco_shared::cache::global_parser_cache()
                        .render_with_cache(&text_bg, html_opts_bg)
                    {
                        Ok(html) => html,
                        Err(e) => format!("Error rendering HTML: {}", e),
                    }
                })
                .await
                .unwrap_or_default();

                // --- back on GTK main thread ---

                let formatted = pretty_print_html(&html_body);

                // Store a clean live HTML snapshot (no wheel JS) for export.
                {
                    let combined_css = css_a.borrow().clone();
                    let theme_mode = theme_a.borrow().clone();
                    let page_view = page_view_a.borrow().clone();
                    let live_html = if page_view.enabled {
                        let page_opts = marco_core::render::PageViewOptions {
                            paged_js_source: crate::components::viewer::pagedjs::PAGED_POLYFILL_JS,
                            paper: &page_view.paper,
                            orientation: &page_view.orientation,
                            margin_mm: page_view.margin_mm,
                            show_page_numbers: page_view.show_page_numbers,
                            wheel_js: "",
                            columns_per_row: page_view.columns_per_row,
                            for_export: false,
                            title: "",
                            standalone_export: false,
                        };
                        crate::components::viewer::backend::wrap_html_document_paged(
                            &html_body,
                            &combined_css,
                            &theme_mode,
                            None,
                            &page_opts,
                        )
                    } else {
                        crate::components::viewer::wry::wrap_html_document(
                            &html_body,
                            &combined_css,
                            &theme_mode,
                            None,
                        )
                    };
                    crate::components::viewer::wry::set_latest_live_html(&live_html);
                }

                // Build the full HTML (with wheel JS) for WebView and detached windows.
                let full_html = {
                    let mut html_with_js = html_body.clone();
                    html_with_js.push_str(&*wheel_js_a);
                    let combined_css = css_a.borrow().clone();
                    let theme_mode = theme_a.borrow().clone();
                    let page_view = page_view_a.borrow().clone();
                    if page_view.enabled {
                        let page_opts = marco_core::render::PageViewOptions {
                            paged_js_source: crate::components::viewer::pagedjs::PAGED_POLYFILL_JS,
                            paper: &page_view.paper,
                            orientation: &page_view.orientation,
                            margin_mm: page_view.margin_mm,
                            show_page_numbers: page_view.show_page_numbers,
                            wheel_js: &*wheel_js_a,
                            columns_per_row: page_view.columns_per_row,
                            for_export: false,
                            title: "",
                            standalone_export: false,
                        };
                        crate::components::viewer::backend::wrap_html_document_paged(
                            &html_body,
                            &combined_css,
                            &theme_mode,
                            None,
                            &page_opts,
                        )
                    } else {
                        crate::components::viewer::wry::wrap_html_document(
                            &html_with_js,
                            &combined_css,
                            &theme_mode,
                            None,
                        )
                    }
                };

                if let Ok(mut guard) = crate::components::viewer::wry::LATEST_PREVIEW_HTML
                    .get_or_init(|| std::sync::Mutex::new(String::new()))
                    .lock()
                {
                    *guard = full_html.clone();
                }

                // Push HTML to the embedded WebView.
                if let Some(ref wv_rc) = webview_a {
                    if let Ok(wv) = wv_rc.try_borrow() {
                        if use_smooth_update {
                            wv.update_html_content_smooth(&full_html);
                        } else {
                            wv.load_html_with_base(&full_html, base_uri_a.as_deref());
                        }
                    } else {
                        log::debug!("In-editor webview borrow busy; skipping load");
                    }
                }

                // Update the code view.
                if let Some(ref pv) = *code_view_a.borrow() {
                    let _ = crate::components::viewer::wry::update_code_view_smooth(
                        pv,
                        &formatted,
                        &theme_a.borrow(),
                        None,
                        None,
                        None,
                        None,
                    );
                }

                flight_a.set(false);
                if gen_a.get() != my_gen {
                    hash_a.set(0);
                }
            });
            // The sync closure returns immediately; the async block above finishes
            // on the GTK main thread once the thread-pool render completes.
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

        #[cfg(target_os = "windows")]
        let code_view_widget_for_windows = Rc::clone(&code_view_widget_for_windows);

        #[cfg(target_os = "linux")]
        let editor_bg_for_code = Rc::clone(&editor_bg_color);
        #[cfg(target_os = "linux")]
        let editor_fg_for_code = Rc::clone(&editor_fg_color);
        #[cfg(target_os = "linux")]
        let scrollbar_thumb_for_code = Rc::clone(&scrollbar_thumb_color);
        #[cfg(target_os = "linux")]
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
            let html_body = match global_parser_cache()
                .render_with_cache(&text, (*html_opts_for_code).clone())
            {
                Ok(html) => html,
                Err(e) => format!("<!-- Error rendering HTML: {} -->", e),
            };

            log::debug!(
                "[editor_ui] Generated HTML length: {} bytes",
                html_body.len()
            );

            // Format the HTML for better readability in code view
            let formatted_html = pretty_print_html(&html_body);

            log::debug!(
                "[editor_ui] Formatted HTML length: {} bytes",
                formatted_html.len()
            );

            // Get current theme mode
            let current_theme = theme_mode_for_code.borrow().clone();

            log::debug!("[editor_ui] Current theme: {}", current_theme);

            // Update the code view
            if let Some(sw_child) = precreated_code_sw_for_code.child() {
                log::debug!(
                    "[editor_ui] Code view has child widget: {:?}",
                    sw_child.type_()
                );

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
                    let widget_type = widget.type_();
                    log::debug!("[editor_ui] Actual widget type: {:?}", widget_type);

                    // Get current theme and check if it changed
                    let theme_changed = *last_code_view_theme.borrow() != current_theme;
                    if theme_changed {
                        log::debug!(
                            "[editor_ui] Theme changed: {} -> {}",
                            last_code_view_theme.borrow(),
                            current_theme
                        );
                        *last_code_view_theme.borrow_mut() = current_theme.clone();
                    }

                    // Update WebView with smooth transition (Linux) or TextView content on other platforms
                    #[cfg(target_os = "linux")]
                    {
                        if widget.is::<webkit6::WebView>() {
                            log::debug!(
                                "[editor_ui] Widget is WebView, updating with smooth transition"
                            );

                            if let Ok(webview) = widget.clone().downcast::<webkit6::WebView>() {
                                // Get editor colors
                                let bg_owned = editor_bg_for_code.borrow().clone();
                                let fg_owned = editor_fg_for_code.borrow().clone();
                                let bg = bg_owned.as_deref();
                                let fg = fg_owned.as_deref();

                                // Get scrollbar colors
                                let thumb = scrollbar_thumb_for_code.borrow().clone();
                                let track = scrollbar_track_for_code.borrow().clone();

                                // Use smooth update to avoid flickering
                                if let Err(e) =
                                    crate::components::viewer::webkit6::update_code_view_smooth(
                                        &webview,
                                        &formatted_html,
                                        &current_theme,
                                        bg,
                                        fg,
                                        Some(&thumb),
                                        Some(&track),
                                    )
                                {
                                    log::error!("Failed to smooth update code view: {}", e);
                                }
                            }
                        } else {
                            log::warn!(
                                "[editor_ui] Code view widget is not a WebView: {:?}",
                                widget_type
                            );
                        }
                    }

                    #[cfg(target_os = "windows")]
                    {
                        if let Some(ref pv) = *code_view_widget_for_windows.borrow() {
                            if let Err(e) = crate::components::viewer::wry::update_code_view_smooth(
                                pv,
                                &formatted_html,
                                &current_theme,
                                None,
                                None,
                                None,
                                None,
                            ) {
                                log::warn!("[editor_ui] Failed to update Windows code view: {}", e);
                            }
                        }
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

    // Create debouncers for different types of processing.
    //
    // Important: preview + intelligence highlighting can be expensive on large documents.
    // Use trailing-edge debouncing so we update only after the user pauses typing.
    let preview_debouncer = Rc::new(crate::components::editor::debounce::Debouncer::new(400));
    let extension_debouncer = Rc::new(crate::components::editor::debounce::Debouncer::new(400));
    let intelligence_debouncer = Rc::new(crate::components::editor::debounce::Debouncer::new(250));

    // Guard against re-entrant buffer "changed" notifications caused by applying
    // syntax highlight tags. Applying/removing tags can emit `changed`, which would
    // otherwise schedule more parsing/highlighting and cause flicker.
    let applying_intelligence_tags = Rc::new(Cell::new(false));

    // Track in-flight intelligence computations so we can drop stale results.
    // (A slow parse should not overwrite newer highlights.)
    let intelligence_request_id = Rc::new(Cell::new(0u64));

    // Also update preview whenever buffer content changes (e.g. when opening a file).
    let refresh_for_signal = std::rc::Rc::clone(&refresh_preview_impl);
    let update_code_for_signal = Rc::clone(&update_html_code_view_rc);
    let view_mode_for_signal = Rc::clone(&current_view_mode);
    let extension_manager_for_signal = extension_manager.clone();
    let buffer_rc_clone = Rc::clone(&buffer_rc);
    let preview_debouncer_for_signal = Rc::clone(&preview_debouncer);
    let extension_debouncer_for_signal = Rc::clone(&extension_debouncer);
    let intelligence_debouncer_for_signal = Rc::clone(&intelligence_debouncer);
    let applying_intelligence_tags_for_signal = Rc::clone(&applying_intelligence_tags);
    // Shared intelligence pipeline closure. Called from:
    // 1. Buffer changed handler (via debounce) on each text edit
    // 2. Global intelligence refresh (immediately) when settings change
    let run_intelligence: Rc<dyn Fn()> = {
        let buffer_rc = Rc::clone(&buffer_rc);
        let applying_tags = Rc::clone(&applying_intelligence_tags);
        let request_id = Rc::clone(&intelligence_request_id);
        let current_diagnostics = Rc::clone(&current_diagnostics);
        let resolve_settings = Rc::clone(&resolve_runtime_intelligence_settings);

        Rc::new(move || {
            let runtime_settings = resolve_settings();

            if !runtime_settings.markdown_intelligence_enabled {
                current_diagnostics.borrow_mut().clear();
                applying_tags.set(true);
                let buffer_for_clear = buffer_rc.clone();
                let applying_for_clear = Rc::clone(&applying_tags);
                crate::components::editor::intelligence::apply_intelligence_highlights_chunked(
                    &buffer_rc,
                    Vec::new(),
                    move || {
                        crate::components::editor::intelligence::apply_diagnostics_markers_chunked(
                            &buffer_for_clear,
                            Vec::new(),
                            move || applying_for_clear.set(false),
                        );
                    },
                );
                return;
            }

            let rid = request_id.get().wrapping_add(1);
            request_id.set(rid);

            let current_text = buffer_rc
                .text(&buffer_rc.start_iter(), &buffer_rc.end_iter(), false)
                .to_string();

            let buffer_for_apply = buffer_rc.clone();
            let request_id_for_apply = Rc::clone(&request_id);
            let applying_flag_for_apply = Rc::clone(&applying_tags);
            let current_diagnostics_for_async = Rc::clone(&current_diagnostics);
            let resolve_settings_for_async = Rc::clone(&resolve_settings);

            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(move || {
                    let src = current_text;
                    let content_hash = marco_shared::cache::hash_content(&src);
                    // Reuse cached AST when available (warmed by the render pipeline).
                    match marco_shared::cache::global_parser_cache().parse_and_cache_ast(&src) {
                        Ok(doc) => {
                            let highlights =
                                marco_core::intelligence::compute_highlights_with_source(
                                    doc.as_ref(),
                                    &src,
                                );
                            // Diagnostics are cached by content_hash — immediate return
                            // if footer or a previous intelligence run already computed them.
                            let cached_diags = marco_shared::cache::global_parser_cache()
                                .get_or_compute_diagnostics_for_doc(&doc, content_hash);
                            Ok((highlights, (*cached_diags).clone()))
                        }
                        Err(e) => Err(e.to_string()),
                    }
                })
                .await;

                let current_diagnostics_for_idle = Rc::clone(&current_diagnostics_for_async);

                glib::idle_add_local_once(move || {
                    if request_id_for_apply.get() != rid {
                        return;
                    }

                    let clear_intelligence_ui = || {
                        applying_flag_for_apply.set(true);
                        let buffer_for_clear = buffer_for_apply.clone();
                        let applying_for_clear = Rc::clone(&applying_flag_for_apply);
                        crate::components::editor::intelligence::apply_intelligence_highlights_chunked(
                            &buffer_for_apply,
                            Vec::new(),
                            move || {
                                crate::components::editor::intelligence::apply_diagnostics_markers_chunked(
                                    &buffer_for_clear,
                                    Vec::new(),
                                    move || {
                                        applying_for_clear.set(false);
                                    },
                                );
                            },
                        );
                    };

                    match result {
                        Ok(Ok((highlights, diagnostics))) => {
                            let rs = resolve_settings_for_async();

                            let filtered_diagnostics: Vec<marco_core::intelligence::Diagnostic> =
                                diagnostics
                                    .into_iter()
                                    .filter(|d| diagnostic_severity_enabled(&d.severity, rs))
                                    .collect();

                            log::debug!(
                                "Computed {} intelligence highlights and {} diagnostics",
                                highlights.len(),
                                filtered_diagnostics.len()
                            );

                            if rs.diagnostics_hover_enabled {
                                *current_diagnostics_for_idle.borrow_mut() =
                                    filtered_diagnostics.clone();
                            } else {
                                current_diagnostics_for_idle.borrow_mut().clear();
                            }

                            let highlights_to_apply = if rs.syntax_colors_enabled {
                                highlights
                            } else {
                                Vec::new()
                            };
                            let diagnostics_to_apply = if rs.diagnostics_underlines_enabled {
                                filtered_diagnostics
                            } else {
                                Vec::new()
                            };

                            applying_flag_for_apply.set(true);
                            let buffer_for_diagnostics = buffer_for_apply.clone();
                            crate::components::editor::intelligence::apply_intelligence_highlights_chunked(
                                &buffer_for_apply,
                                highlights_to_apply,
                                move || {
                                    crate::components::editor::intelligence::apply_diagnostics_markers_chunked(
                                        &buffer_for_diagnostics,
                                        diagnostics_to_apply,
                                        move || {
                                            applying_flag_for_apply.set(false);
                                        },
                                    );
                                },
                            );
                        }
                        Ok(Err(e)) => {
                            current_diagnostics_for_idle.borrow_mut().clear();
                            clear_intelligence_ui();
                            log::warn!(
                                "Failed to parse markdown for intelligence highlighting: {}",
                                e
                            );
                        }
                        Err(e) => {
                            current_diagnostics_for_idle.borrow_mut().clear();
                            clear_intelligence_ui();
                            log::error!("Intelligence highlight task panicked: {:?}", e);
                        }
                    }
                });
            });
        })
    };

    // Register the intelligence pipeline for global refresh from the settings tab.
    {
        let run_intelligence_for_refresh = Rc::clone(&run_intelligence);
        crate::components::editor::editor_manager::register_intelligence_refresh(move || {
            run_intelligence_for_refresh();
        });
    }

    // Register the page-view state handle so the settings dialog can update it live.
    {
        let page_view_for_reg = std::rc::Rc::clone(&page_view_rc);
        let refresh_for_page_view = std::rc::Rc::clone(&refresh_preview_impl);
        crate::components::editor::editor_manager::register_page_view_state(
            page_view_for_reg,
            move || {
                refresh_for_page_view();
            },
        );
    }

    buffer_rc_clone.connect_changed(move |buffer| {
        // SourceView5's native hover handles dismissal on buffer changes.

        // Ignore change events caused by applying/removing highlight tags.
        if applying_intelligence_tags_for_signal.get() {
            return;
        }

        // Use debounced HTML preview updates to batch rapid typing
        let refresh_clone = Rc::clone(&refresh_for_signal);
        let update_code_clone = Rc::clone(&update_code_for_signal);
        let view_mode_clone = Rc::clone(&view_mode_for_signal);

        // Capture buffer text for TOC rebuild.
        let buffer_for_toc = buffer.clone();

        // Phase 8 Layer 3: adaptive debounce — larger files get a longer quiet
        // period before a render fires so the thread pool is not saturated.
        // line_count() is a cheap O(1) GTK call; no text extraction here.
        let line_count = buffer.line_count();
        let debounce_delay = preview_debounce_duration(line_count);
        preview_debouncer_for_signal.debounce_trailing_with_timeout(debounce_delay, move || {
            // Update HTML preview (trailing edge only)
            refresh_clone();

            // Also update code view if we're currently in CodePreview mode
            if *view_mode_clone.borrow() == ViewMode::CodePreview {
                update_code_clone();
            }

            // Rebuild TOC sidebar if it is currently visible.
            with_toc_panel(|handle| {
                if !handle.is_visible() {
                    return;
                }
                let text = buffer_for_toc
                    .text(
                        &buffer_for_toc.start_iter(),
                        &buffer_for_toc.end_iter(),
                        false,
                    )
                    .to_string();
                let depth = handle.depth.get();
                let entries = marco_shared::cache::global_parser_cache().get_or_compute_toc(&text);
                handle.rebuild(&entries, depth);
            });
        });

        // Apply intelligence with debouncing
        let run_intelligence_for_debounce = Rc::clone(&run_intelligence);
        intelligence_debouncer_for_signal.debounce_trailing(move || {
            run_intelligence_for_debounce();
        });

        // Use new debounced extension processing with change delta detection
        if let Some(ref manager) = extension_manager_for_signal {
            let extension_manager_clone = manager.clone();

            let buffer_for_extensions = buffer.clone();

            extension_debouncer_for_signal.debounce_trailing(move || {
                let current_text_for_extensions = buffer_for_extensions
                    .text(
                        &buffer_for_extensions.start_iter(),
                        &buffer_for_extensions.end_iter(),
                        false,
                    )
                    .to_string();

                let cursor_position = {
                    let cursor_iter = buffer_for_extensions.cursor_position();
                    if cursor_iter >= 0 {
                        Some(cursor_iter as u32)
                    } else {
                        None
                    }
                };

                // Process extensions using the simplified parallel method.
                if let Ok(manager_ref) = extension_manager_clone.try_borrow() {
                    if let Err(e) = manager_ref.process_extensions_parallel(
                        current_text_for_extensions,
                        cursor_position,
                        |results| {
                            log::debug!(
                                "Extension processing completed: {} results",
                                results.len()
                            );
                        },
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

    // Preview theme updater (Linux sets a real implementation; other platforms use a no-op)
    let update_preview_theme: Box<dyn Fn(&str)>;

    // Clones for preview theme updater (Linux-only implementation overrides the default)
    #[cfg(target_os = "linux")]
    {
        let theme_manager_for_preview = Rc::clone(&theme_manager);
        let css_rc_for_preview = Rc::clone(&css_rc);
        // Capture the main refresh closure so that a theme change re-renders via
        // the correct code path (including paged.js when page view is active).
        let refresh_impl_for_theme = std::rc::Rc::clone(&refresh_preview_impl);
        let theme_mode_for_preview = Rc::clone(&theme_mode_rc);
        let editor_dir_for_preview = theme_manager.borrow().editor_theme_dir.clone();
        let editor_bg_color_for_preview = Rc::clone(&editor_bg_color);
        let editor_fg_color_for_preview = Rc::clone(&editor_fg_color);
        let scrollbar_thumb_for_preview = Rc::clone(&scrollbar_thumb_color);
        let scrollbar_track_for_preview = Rc::clone(&scrollbar_track_color);
        let editor_sw_for_preview = editor_scrolled_window.clone(); // For checking scrollbar state
        let preview_theme_timeout: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
        let preview_theme_timeout_clone = Rc::clone(&preview_theme_timeout);
        let update_html_code_view_for_preview = Rc::clone(&update_html_code_view_rc);
        update_preview_theme = Box::new(move |scheme_id: &str| {
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
                                    if let Some(v) = extract_xml_color_value(&contents, "dark-text")
                                    {
                                        *editor_fg_color_for_preview.borrow_mut() = Some(v);
                                    } else if let Some(v) =
                                        extract_xml_color_value(&contents, "light-text")
                                    {
                                        *editor_fg_color_for_preview.borrow_mut() = Some(v);
                                    }

                                    // Extract scrollbar colors for code view
                                    if let Some(v) =
                                        extract_xml_color_value(&contents, "scrollbar-thumb")
                                    {
                                        *scrollbar_thumb_for_preview.borrow_mut() = v;
                                    }
                                    if let Some(v) =
                                        extract_xml_color_value(&contents, "scrollbar-track")
                                    {
                                        *scrollbar_track_for_preview.borrow_mut() = v;
                                    }

                                    // Update webkit scrollbar CSS in the preview CSS string
                                    // This ensures the HTML preview scrollbar matches the theme
                                    let new_thumb = scrollbar_thumb_for_preview.borrow().clone();
                                    let new_track = scrollbar_track_for_preview.borrow().clone();
                                    let new_webkit_css =
                                        webkit_scrollbar_css(&new_thumb, &new_track);

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
                                    log::debug!(
                                        "Updated webkit scrollbar CSS in preview CSS string"
                                    );

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
                                            crate::components::viewer::css_utils::gtk_scrollbar_css(
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
            let update_code_clone = Rc::clone(&update_html_code_view_for_preview);
            let refresh_impl_clone = std::rc::Rc::clone(&refresh_impl_for_theme);
            let id = glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                // Use the main refresh closure.  It already handles page-view mode
                // (forces a full paged.js reload when enabled) as well as normal mode.
                // This ensures a theme switch never silently downgraded to non-paged rendering.
                refresh_impl_clone();

                // Also update code view if it exists (for theme changes)
                (update_code_clone)();

                preview_theme_timeout_clone2.set(None);
                glib::ControlFlow::Break
            });
            preview_theme_timeout_clone.set(Some(id));
        }) as Box<dyn Fn(&str)>;
    }

    #[cfg(target_os = "windows")]
    {
        let theme_manager_for_preview = Rc::clone(&theme_manager);
        let theme_mode_for_preview = Rc::clone(&theme_mode_rc);
        let refresh_for_preview = std::rc::Rc::clone(&refresh_preview_impl);
        let update_code_for_preview = Rc::clone(&update_html_code_view_rc);
        let css_rc_for_preview = Rc::clone(&css_rc);
        let scrollbar_thumb_for_preview = Rc::clone(&scrollbar_thumb_color);
        let scrollbar_track_for_preview = Rc::clone(&scrollbar_track_color);
        let editor_dir_for_preview = theme_manager.borrow().editor_theme_dir.clone();

        let webview_rc: Rc<RefCell<crate::components::viewer::preview_types::PlatformWebView>> =
            webview_rc_opt.as_ref().expect("webview_rc not set").clone();

        update_preview_theme = Box::new(move |scheme_id: &str| {
            let new_theme_mode = theme_manager_for_preview
                .borrow()
                .preview_theme_mode_from_scheme(scheme_id);

            *theme_mode_for_preview.borrow_mut() = new_theme_mode.clone();

            // Keep WebView2 background in sync to reduce white/flash artifacts.
            let is_dark = new_theme_mode.eq_ignore_ascii_case("dark");
            let rgba = if is_dark {
                gtk4::gdk::RGBA::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0, 1.0)
            } else {
                gtk4::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)
            };

            if let Ok(wv) = webview_rc.try_borrow() {
                wv.set_background_color_rgba(&rgba);
            }

            // Extract updated scrollbar colors from the new editor theme scheme
            // and update the HTML preview CSS so the WebView scrollbar changes too.
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
                                    if let Some(v) =
                                        extract_xml_color_value(&contents, "scrollbar-thumb")
                                    {
                                        *scrollbar_thumb_for_preview.borrow_mut() = v;
                                    }
                                    if let Some(v) =
                                        extract_xml_color_value(&contents, "scrollbar-track")
                                    {
                                        *scrollbar_track_for_preview.borrow_mut() = v;
                                    }

                                    // Rebuild the embedded webkit scrollbar CSS in the preview CSS
                                    // string so the next full HTML load carries the correct colors.
                                    let new_thumb = scrollbar_thumb_for_preview.borrow().clone();
                                    let new_track = scrollbar_track_for_preview.borrow().clone();
                                    let new_webkit_css =
                                        webkit_scrollbar_css(&new_thumb, &new_track);
                                    let mut updated_css = css_rc_for_preview.borrow().clone();
                                    if let Some(pos) = updated_css.rfind("::-webkit-scrollbar") {
                                        if let Some(start) = updated_css[..pos].rfind("\n/*") {
                                            updated_css.truncate(start);
                                        } else {
                                            updated_css.truncate(pos);
                                        }
                                    }
                                    updated_css.push('\n');
                                    updated_css.push_str(&new_webkit_css);
                                    *css_rc_for_preview.borrow_mut() = updated_css;

                                    // Also update the GTK scrollbar CSS so native scrollbars match.
                                    if let Some(display) = gtk4::gdk::Display::default() {
                                        let gtk_css =
                                            crate::components::viewer::css_utils::gtk_scrollbar_css(
                                                &new_thumb, &new_track,
                                            );
                                        let provider = gtk4::CssProvider::new();
                                        provider.load_from_data(&gtk_css);
                                        gtk4::style_context_add_provider_for_display(
                                            &display,
                                            &provider,
                                            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                                        );
                                        log::debug!(
                                            "[win] Updated GTK scrollbar CSS: thumb={}, track={}",
                                            new_thumb,
                                            new_track
                                        );
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Apply the new theme mode to the preview + source code view.
            (refresh_for_preview)();
            (update_code_for_preview)();
        });
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
    {
        update_preview_theme = Box::new(|_s: &str| {});
    }

    // Set up split percentage indicator with cascade prevention from split controller
    let split_indicator = setup_split_percentage_indicator_with_cascade_prevention(
        &paned,
        Some(split_controller.position_being_set()),
    );
    let overlay = split_indicator.widget().clone();

    // ── Zoom overlay bar (bottom-right of the preview split) ───────────────
    // Create the floating zoom control bar and add it to the same gtk4::Overlay
    // that wraps the paned.  The zoom-changed callback in editor_manager keeps
    // the label in sync with keyboard shortcuts as well.
    let _zoom_bar = crate::ui::zoom_overlay::create_zoom_bar(
        &overlay,
        &paned,
        intelligence_settings_manager.clone(),
    );

    // Wrap the entire editor+preview paned in the TOC sidebar paned.
    // Placing it here (outside the inner split) means the TOC panel is visible
    // in all LayoutState modes — including ViewOnly and EditorAndViewSeparate —
    // because the SplitController only manages `paned`, not `toc_paned`.
    let (toc_paned, toc_handle) = crate::ui::toc_panel::create_toc_panel(&source_view);
    toc_paned.set_end_child(Some(&overlay));
    toc_paned.set_hexpand(true);
    toc_paned.set_vexpand(true);
    set_toc_panel_handle(toc_handle);

    (
        paned, // 0: Inner editor/preview split (split-ratio + reparenting)
        webview_rc_opt.expect("webview wrapper not set"), // Return wrapped WebView for reparenting support
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
        toc_paned,        // 10: Outermost TOC container paned (TOC sidebar | split overlay)
        split_controller, // 11: Split position controller
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diag(
        severity: marco_core::intelligence::DiagnosticSeverity,
        code: marco_core::intelligence::DiagnosticCode,
        start: usize,
        end: usize,
        message: &str,
    ) -> marco_core::intelligence::Diagnostic {
        marco_core::intelligence::Diagnostic {
            code,
            span: marco_core::parser::Span {
                start: marco_core::parser::Position {
                    line: 1,
                    column: start.saturating_add(1),
                    offset: start,
                },
                end: marco_core::parser::Position {
                    line: 1,
                    column: end.saturating_add(1),
                    offset: end,
                },
            },
            severity,
            message: message.to_string(),
        }
    }

    #[test]
    fn smoke_test_diagnostic_at_offset_prefers_narrowest_span() {
        let wide = make_diag(
            marco_core::intelligence::DiagnosticSeverity::Warning,
            marco_core::intelligence::DiagnosticCode::MissingCodeBlockLanguage,
            10,
            30,
            "wide",
        );
        let narrow = make_diag(
            marco_core::intelligence::DiagnosticSeverity::Error,
            marco_core::intelligence::DiagnosticCode::EmptyImageUrl,
            12,
            14,
            "narrow",
        );

        let hit = diagnostic_at_offset(&[wide, narrow.clone()], 13)
            .expect("expected diagnostic hit at offset 13");
        assert_eq!(hit.message, "narrow");
    }

    #[test]
    fn smoke_test_diagnostic_at_offset_none_outside_span() {
        let diag = make_diag(
            marco_core::intelligence::DiagnosticSeverity::Error,
            marco_core::intelligence::DiagnosticCode::EmptyImageUrl,
            5,
            10,
            "hit-range",
        );

        assert!(diagnostic_at_offset(&[diag], 10).is_none()); // end-exclusive
    }

    #[test]
    fn smoke_test_diagnostic_hover_markup_contains_code_and_fix() {
        let diag = make_diag(
            marco_core::intelligence::DiagnosticSeverity::Error,
            marco_core::intelligence::DiagnosticCode::EmptyImageUrl,
            20,
            24,
            "Empty image URL",
        );

        let (title, body, signature) = diagnostic_hover_markup(&diag);
        assert!(title.contains("Error"));
        assert!(title.contains("Empty image URL"));
        assert!(body.contains(&format!("Code: {}", diag.code_id())));
        assert!(body.contains("About:"));
        assert!(body.contains("Fix:"));
        assert_eq!(signature.0, 20);
        assert_eq!(signature.1, 24);
    }

    #[test]
    fn smoke_test_runtime_intelligence_defaults_match_footer_filter_baseline() {
        let settings = RuntimeIntelligenceSettings::default();
        assert!(settings.level_1_enabled);
        assert!(settings.level_2_enabled);
        assert!(!settings.level_3_enabled);
        assert!(!settings.level_4_enabled);
    }
}
