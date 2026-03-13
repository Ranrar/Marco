#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

#[cfg(target_os = "linux")]
use webkit6::prelude::*;

mod components;
mod footer;
mod logic;
mod menu;
mod settings {}
mod theme;
mod toolbar;
pub mod ui;

/*
╔═══════════════════════════════════════════════════════════════════════════╗
║    CRITICAL: This file (main.rs) serves ONLY as an APPLICATION GATEWAY    ║
╚═══════════════════════════════════════════════════════════════════════════╝
*/

use crate::components::bookmarks::BookmarkManager;
use crate::components::editor::footer::{refresh_footer_snapshot, wire_footer_updates};
use crate::components::editor::ui::create_editor_with_preview_and_buffer;
use crate::components::language::{LocalizationProvider, SimpleLocalizationManager};
use crate::components::viewer::preview_types::ViewMode;
use crate::theme::ThemeManager;
use crate::ui::menu_items::files::FileDialogs;
use crate::ui::menu_items::FileOperations;
use core::logic::{DocumentBuffer, RecentFiles};
use core::paths::MarcoPaths;
#[cfg(target_os = "windows")]
use gio::prelude::*;
#[cfg(target_os = "windows")]
use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box as GtkBox, Orientation};
use log::trace;
use sourceview5::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const APP_ID: &str = "io.github.ranrar.Marco";
const BOOKMARK_MARK_CATEGORY: &str = "marco-bookmark";

fn main() -> glib::ExitCode {
    // Very early audit: record entering main (before initialization)
    log::trace!("audit: main() entry - very early");

    // Install panic hook to ensure panics are logged and logger is flushed
    crate::logic::panic_hook::install_panic_hook();

    // path detection and environment setup
    use core::paths::MarcoPaths;
    let marco_paths = match MarcoPaths::new() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error initializing Marco paths: {:?}", e);
            std::process::exit(1);
        }
    };

    // Icon font support removed - icon fonts (IcoMoon) are no longer used. Use SVGs instead.

    let settings_path = marco_paths.settings_file();
    if !settings_path.exists() {
        eprintln!("Warning: Settings file not found at {:?}", settings_path);
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    // Register OS-level signal handlers for graceful shutdown
    crate::logic::signal_handlers::setup_signal_handlers(&app);

    // Clone marco_paths for closures
    let marco_paths_for_open = std::rc::Rc::new(marco_paths);
    let marco_paths_for_activate = marco_paths_for_open.clone();

    // Handle file opening via command line or file manager
    app.connect_open(move |app, files, _hint| {
        let file_path = if !files.is_empty() {
            Some(files[0].path().unwrap().to_string_lossy().to_string())
        } else {
            None
        };
        build_ui(app, file_path, marco_paths_for_open.clone());
    });

    // Handle normal activation (no files)
    app.connect_activate(move |app| {
        build_ui(app, None, marco_paths_for_activate.clone());
    });

    trace!("audit: app starting");
    let exit_code = app.run();
    trace!("audit: app exiting with code {:?}", exit_code);

    // Clean up global resources before shutting down logger
    crate::components::editor::editor_manager::shutdown_editor_manager();
    core::shutdown_global_parser_cache();
    core::logic::cache::shutdown_global_cache();

    // Ensure file logger is flushed and closed on normal exit
    core::logic::logger::shutdown_file_logger();
    exit_code
}

fn build_ui(app: &Application, initial_file: Option<String>, marco_paths: Rc<MarcoPaths>) {
    // Import path functions and settings manager
    use core::logic::swanson::SettingsManager;
    use core::paths::PathProvider;

    // Load CSS using the new modular system
    crate::ui::css::load_css();

    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Marco")
        .default_width(1200)
        .default_height(800)
        .build();
    window.add_css_class("main-window");

    // Set window icon (GTK will look for icon named "marco" in the system icon theme)
    window.set_icon_name(Some("marco"));

    // --- Create window first, but defer titlebar creation until after editor ---
    window.add_css_class("main-window");

    // --- ThemeManager and settings.ron path ---
    let asset_root = marco_paths.asset_root().clone();
    let settings_path = marco_paths.settings_file();
    let editor_theme_dir = marco_paths.editor_themes_dir();
    let preview_theme_dir = marco_paths.shared().preview_themes_dir();

    // Initialize centralized settings manager - single source of truth for all settings
    let settings_manager = match SettingsManager::initialize(settings_path.clone()) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to initialize settings manager: {}", e);
            eprintln!("Using default settings and continuing...");
            // Create a fallback settings manager with default settings
            match SettingsManager::initialize(settings_path.clone()) {
                Ok(manager) => manager,
                Err(_) => {
                    eprintln!("Critical: Cannot initialize settings. Exiting.");
                    std::process::exit(1);
                }
            }
        }
    };

    let localization_manager =
        SimpleLocalizationManager::new().expect("Failed to initialize localization manager");
    let locale_code = settings_manager
        .get_settings()
        .language
        .and_then(|l| l.language)
        // "System Default" in settings means: detect OS language.
        .or_else(core::paths::detect_system_locale_iso639_1)
        .unwrap_or_else(|| "en".to_string());
    if let Err(e) = localization_manager.load_locale(&locale_code) {
        log::warn!(
            "Failed to load locale '{}': {}. Falling back to English.",
            locale_code,
            e
        );

        if locale_code != "en" {
            if let Err(e) = localization_manager.load_locale("en") {
                log::error!("Failed to load fallback locale 'en': {}", e);
            }
        }
    }
    let translations = localization_manager.translations();
    let translations_rc = Rc::new(RefCell::new(translations.clone()));
    let menu_translations_rc = Rc::new(RefCell::new(translations.menu.clone()));
    let dialog_translations_rc = Rc::new(RefCell::new(translations.dialog.clone()));

    // Discover available locales from `assets/language/*.toml` at startup.
    // This list is used to populate the Settings → Language dropdown.
    let available_locale_infos_rc = Rc::new(localization_manager.available_locale_infos());

    // Initialize file logger according to settings (runtime)
    crate::logic::logger_init::init_logging(&settings_manager);

    // Initialize monospace font cache for fast settings loading
    if let Err(e) = core::logic::loaders::font_loader::FontLoader::init_monospace_cache() {
        log::warn!("Failed to initialize monospace font cache: {}", e);
    }

    // Initialize the global editor manager with settings manager
    if let Err(e) =
        crate::components::editor::editor_manager::init_editor_manager(settings_manager.clone())
    {
        log::warn!("Failed to initialize editor manager: {}", e);
    }

    // Initialize theme manager with settings manager
    // Note: ui_theme_dir is deprecated and unused in ThemeManager
    let theme_manager = Rc::new(RefCell::new(ThemeManager::new(
        settings_manager.clone(),
        asset_root.clone(), // Placeholder - ui_theme_dir is unused
        preview_theme_dir.clone(),
        editor_theme_dir,
    )));
    // Pass settings struct to modules as needed

    // Add theme-specific CSS class based on current mode (for runtime GTK UI switching)
    let current_theme_mode = {
        let settings = settings_manager.get_settings();
        let editor_mode = settings
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.as_str())
            .unwrap_or("light");
        if editor_mode.contains("dark") {
            "dark"
        } else {
            "light"
        }
    };
    window.add_css_class(&format!("marco-theme-{}", current_theme_mode));
    log::debug!("Applied theme class: marco-theme-{}", current_theme_mode);

    // Create main vertical box layout
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.add_css_class("main-container");

    // Create basic UI components (structure only)
    let toolbar = toolbar::create_toolbar_structure(&translations);
    toolbar.add_css_class("toolbar");
    toolbar::set_toolbar_height(&toolbar, 0); // Minimum height, matches footer
    let toolbar_ref = Rc::new(RefCell::new(toolbar));

    // Wire toolbar gutter on/off buttons (binary toggle for line numbers)
    toolbar::wire_gutter_toggle(&toolbar_ref.borrow(), &settings_manager);

    // --- Determine correct HTML preview theme based on settings and app theme ---
    use core::logic::loaders::theme_loader::list_html_view_themes;
    let preview_theme_dir_str = preview_theme_dir.clone().to_string_lossy().to_string();
    let html_themes = list_html_view_themes(&preview_theme_dir.clone());
    let settings = theme_manager.borrow().get_settings();
    let mut preview_theme_filename = "standard.css".to_string();
    if let Some(appearance) = &settings.appearance {
        if let Some(ref preview_theme) = appearance.preview_theme {
            if html_themes.iter().any(|t| &t.filename == preview_theme) {
                preview_theme_filename = preview_theme.clone();
            }
        }
    }
    // Initialize theme_mode based on current editor scheme setting
    let initial_theme_mode = {
        let current_scheme = theme_manager.borrow().current_editor_scheme_id();
        theme_manager
            .borrow()
            .preview_theme_mode_from_scheme(&current_scheme)
    };
    let theme_mode = Rc::new(RefCell::new(initial_theme_mode));
    let (footer, footer_labels_rc) =
        footer::create_footer(&translations.footer, settings_manager.clone());

    // Create file operations handler early so we can pass DocumentBuffer to editor
    let file_operations = FileOperations::new(
        Rc::new(RefCell::new(DocumentBuffer::new_untitled())),
        Rc::new(RefCell::new(RecentFiles::new(settings_manager.clone()))),
    );
    let file_operations_rc = Rc::new(RefCell::new(file_operations));
    let bookmark_manager = Rc::new(BookmarkManager::new(settings_manager.clone()));
    let document_buffer_ref = Rc::clone(&file_operations_rc.borrow().buffer);

    // Active markdown schema support removed; footer uses AST parser directly.
    let _schema_root = asset_root.join("markdown_schema");
    let active_schema_map: Rc<RefCell<Option<()>>> = Rc::new(RefCell::new(None));

    let (
        split,
        editor_webview,
        preview_css_rc,
        refresh_preview,
        update_editor_theme,
        update_preview_theme,
        editor_buffer,
        editor_source_view,
        insert_mode_state,
        set_view_mode,
        split_overlay,
        split_controller,
    ) = create_editor_with_preview_and_buffer(
        &window,
        crate::components::editor::ui::EditorParams {
            preview_theme_filename,
            preview_theme_dir: preview_theme_dir_str,
            theme_manager: theme_manager.clone(),
            theme_mode: Rc::clone(&theme_mode),
        },
        footer_labels_rc.clone(),
        settings_path.to_str().unwrap(),
        Some(document_buffer_ref),
    );

    // Shared root popover tree state for menu + toolbar interaction.
    let root_popover_state = crate::ui::popover_state::RootPopoverState::new();
    crate::ui::popover_state::set_global_root_popover_state(root_popover_state.clone());

    crate::ui::toolbar::connect_simple_markdown_toolbar_actions(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_bold_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_italic_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_strikethrough_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_highlight_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_code_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_superscript_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_subscript_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_inline_math_toolbar_action(
        &toolbar_ref.borrow(),
        window.upcast_ref::<gtk4::Window>(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_inline_checkbox_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );

    let current_file_provider_for_links: Rc<dyn Fn() -> Option<std::path::PathBuf>> = {
        let file_operations_rc = file_operations_rc.clone();
        Rc::new(move || {
            file_operations_rc
                .borrow()
                .buffer
                .borrow()
                .get_file_path()
                .map(|path| path.to_path_buf())
        })
    };

    crate::ui::toolbar::connect_link_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        window.upcast_ref::<gtk4::Window>(),
        current_file_provider_for_links.clone(),
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_reference_link_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        window.upcast_ref::<gtk4::Window>(),
        current_file_provider_for_links.clone(),
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_image_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        window.upcast_ref::<gtk4::Window>(),
        current_file_provider_for_links.clone(),
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_inline_footnote_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_block_footnote_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_emoji_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        window.upcast_ref::<gtk4::Window>(),
        settings_manager.clone(),
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_hr_toolbar_action(
        &toolbar_ref.borrow(),
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_list_toolbar_action(
        &toolbar_ref.borrow(),
        &window,
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_table_toolbar_action(
        &toolbar_ref.borrow(),
        &window,
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_tab_block_toolbar_action(
        &toolbar_ref.borrow(),
        &window,
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_slider_deck_toolbar_action(
        &toolbar_ref.borrow(),
        &window,
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_mermaid_toolbar_action(
        &toolbar_ref.borrow(),
        &window,
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_admonition_toolbar_action(
        &toolbar_ref.borrow(),
        &window,
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );
    crate::ui::toolbar::connect_mention_toolbar_action(
        &toolbar_ref.borrow(),
        &window,
        &editor_buffer,
        &editor_source_view,
        root_popover_state.clone(),
    );

    // Configure bookmark gutter marks and context action.
    {
        let mark_attributes = sourceview5::MarkAttributes::new();
        mark_attributes.set_icon_name("bookmark-new-symbolic");
        // Place bookmark marks in the line-mark gutter with a compact renderer.
        // Lower priority keeps it visually close to line numbers.
        editor_source_view.set_mark_attributes(BOOKMARK_MARK_CATEGORY, &mark_attributes, 1);
        // Keep gutter at normal width unless the current document actually has bookmarks.
        editor_source_view.set_show_line_marks(false);
    }

    crate::components::editor::contextmenu::setup_editor_context_menu(
        app,
        &editor_source_view,
        &editor_buffer,
        bookmark_manager.clone(),
        file_operations_rc.clone(),
    );

    // Keep bookmarks in sync with live line edits in the current document.
    // We treat SourceView marks as source-of-truth because they track edits robustly.
    let user_edit_depth = Rc::new(Cell::new(0u32));
    let sync_bookmarks_from_marks: Rc<dyn Fn()> = {
        let bookmark_manager = bookmark_manager.clone();
        let file_operations_rc = file_operations_rc.clone();
        let editor_buffer = editor_buffer.clone();
        Rc::new(move || {
            let current_path = file_operations_rc
                .borrow()
                .buffer
                .borrow()
                .get_file_path()
                .map(|path| path.to_path_buf());

            let Some(path) = current_path else {
                return;
            };

            let line_count = editor_buffer.line_count().max(0) as u32;
            let mut lines = Vec::new();
            for line in 0..line_count {
                let marks =
                    editor_buffer.source_marks_at_line(line as i32, Some(BOOKMARK_MARK_CATEGORY));
                if !marks.is_empty() {
                    lines.push(line);
                }
            }

            bookmark_manager.replace_for_file(path, &lines);
        })
    };

    {
        let user_edit_depth = user_edit_depth.clone();
        editor_buffer.connect_begin_user_action(move |_| {
            let depth = user_edit_depth.get();
            user_edit_depth.set(depth.saturating_add(1));
        });
    }

    {
        let user_edit_depth = user_edit_depth.clone();
        let sync_bookmarks_from_marks = sync_bookmarks_from_marks.clone();
        editor_buffer.connect_end_user_action(move |_| {
            let depth = user_edit_depth.get();
            let new_depth = depth.saturating_sub(1);
            user_edit_depth.set(new_depth);

            if new_depth == 0 {
                sync_bookmarks_from_marks();
            }
        });
    }

    // Rebuild source marks whenever bookmarks or current document changes.
    let refresh_bookmark_marks: Rc<dyn Fn()> = {
        let bookmark_manager = bookmark_manager.clone();
        let file_operations_rc = file_operations_rc.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_source_view = editor_source_view.clone();
        Rc::new(move || {
            let start = editor_buffer.start_iter();
            let end = editor_buffer.end_iter();
            editor_buffer.remove_source_marks(&start, &end, Some(BOOKMARK_MARK_CATEGORY));

            let current_path = file_operations_rc
                .borrow()
                .buffer
                .borrow()
                .get_file_path()
                .map(|path| path.to_path_buf());

            let mut has_marks = false;

            if let Some(path) = current_path {
                for line in bookmark_manager.get_for_file(&path) {
                    if let Some(iter) = editor_buffer.iter_at_line(line as i32) {
                        editor_buffer.create_source_mark(None, BOOKMARK_MARK_CATEGORY, &iter);
                        has_marks = true;
                    }
                }
            }

            // Only reserve gutter mark column when needed.
            editor_source_view.set_show_line_marks(has_marks);
        })
    };
    refresh_bookmark_marks();

    // Ensure bookmark marks are refreshed when the editor widget is mapped.
    // This helps keep gutter rendering stable after startup/layout changes.
    {
        let refresh_bookmark_marks = refresh_bookmark_marks.clone();
        editor_source_view.connect_map(move |_| {
            refresh_bookmark_marks();
        });
    }

    // Wrap setter into Rc so it can be cloned into action callbacks
    let set_view_mode_rc: Rc<Box<dyn Fn(ViewMode)>> = Rc::new(set_view_mode);

    // Wire up live footer updates using the actual editor buffer
    // Wire footer updates directly: wire_footer_updates will run callbacks on
    // the main loop and call `apply_footer_update` directly.
    wire_footer_updates(
        &editor_buffer,
        &editor_source_view,
        footer_labels_rc.clone(),
        insert_mode_state.clone(),
        settings_manager.clone(),
    );
    split_overlay.add_css_class("split-view"); // Apply CSS to overlay

    // --- WebView Reparenting State for EditorAndViewSeparate Mode ---
    use crate::components::viewer::layout_controller::WebViewLocationTracker;

    // Platform-specific reparenting state initialization. On Linux we create the actual
    // objects; on non-Linux we pass `None` so titlebar/menu code uses safe fallbacks.
    #[cfg(target_os = "linux")]
    let (preview_window_opt, webview_location_tracker, reparent_guard) = {
        use crate::components::viewer::webkit6_detached_window::PreviewWindow;
        let webview_location_tracker = WebViewLocationTracker::new();
        let preview_window_opt: Rc<RefCell<Option<PreviewWindow>>> = Rc::new(RefCell::new(None));
        let reparent_guard = crate::components::viewer::reparenting::ReparentGuard::new();
        log::debug!("Initialized WebView reparenting state for EditorAndViewSeparate mode (Linux)");
        (
            Some(preview_window_opt),
            Some(webview_location_tracker),
            Some(reparent_guard),
        )
    };

    #[cfg(target_os = "windows")]
    let (preview_window_opt, webview_location_tracker, reparent_guard) = {
        use crate::components::viewer::wry_detached_window::PreviewWindow;
        let webview_location_tracker = WebViewLocationTracker::new();
        let preview_window_opt: Rc<RefCell<Option<PreviewWindow>>> = Rc::new(RefCell::new(None));
        let reparent_guard = None::<()>;
        log::debug!("Initialized wry-based detached preview state for Windows");
        (
            Some(preview_window_opt),
            Some(webview_location_tracker),
            reparent_guard,
        )
    };

    // --- Create custom titlebar now that we have webview and reparenting state ---
    let (titlebar_handle, title_label, menu_state) =
        menu::create_custom_titlebar(menu::TitlebarConfig {
            window: &window,
            webview_rc: Some(editor_webview.clone()),
            split: Some(split.clone()),
            preview_window_opt,
            webview_location_tracker,
            reparent_guard,
            split_controller: Some(split_controller.clone()),
            asset_root: &asset_root,
            translations: &translations,
            root_popover_state,
        });
    let menu_state = Rc::new(menu_state);
    window.set_titlebar(Some(&titlebar_handle));

    // --- Settings Thread Pool for Proper Resource Management ---
    let settings_pool = crate::logic::settings_thread::SettingsThreadPool::new();
    let settings_tx = settings_pool.tx.clone();
    let settings_pool_rc = std::rc::Rc::new(std::cell::RefCell::new(settings_pool));

    // Apply saved split ratio after paned widget is mapped and sized
    crate::logic::split_state::apply_saved_split_ratio(&split, &settings_manager);

    // Save split ratio when user finishes manually dragging the divider
    crate::logic::split_state::connect_split_ratio_save(&split, &settings_manager, &settings_tx);

    // Apply saved view mode from settings at startup (if present)
    {
        let s = settings_manager.get_settings();
        if let Some(layout) = s.layout {
            if let Some(vm) = layout.view_mode {
                match vm.as_str() {
                    "HTML Preview" => (set_view_mode_rc)(ViewMode::HtmlPreview),
                    "Source Code" | "Code Preview" => (set_view_mode_rc)(ViewMode::CodePreview),
                    _ => {}
                }
            }
        }
    }

    // Create footer update function using weak references to prevent circular retention
    let trigger_footer_update: std::rc::Rc<dyn Fn()> = {
        // Use weak references to editor components
        let buffer_weak = editor_buffer.downgrade();
        let labels_weak = Rc::downgrade(&footer_labels_rc);
        let test_counter = std::rc::Rc::new(std::cell::Cell::new(0));

        std::rc::Rc::new(move || {
            // Check if components are still valid before using
            if let (Some(_buffer), Some(labels)) = (buffer_weak.upgrade(), labels_weak.upgrade()) {
                // Manual footer trigger invoked; terminal output suppressed.

                // Increment test counter for obvious visual changes
                let count = test_counter.get() + 1;
                test_counter.set(count);

                // Update with test values to make changes obvious
                crate::footer::update_cursor_row(&labels, count + 10);
                crate::footer::update_cursor_col(&labels, count + 20);
                crate::footer::update_word_count(&labels, count * 10);
                crate::footer::update_char_count(&labels, count * 50);
                crate::footer::update_encoding(&labels, &format!("TEST-{}", count));
                crate::footer::update_insert_mode(&labels, count.is_multiple_of(2));
            } else {
                log::debug!("Footer update callback called after editor components were dropped");
            }
        })
    };

    // Add components to main layout (menu bar is now in titlebar)
    main_box.append(&*toolbar_ref.borrow());
    main_box.append(&split_overlay); // Use overlay instead of split
    main_box.append(&footer);

    // Set editor area to expand
    split_overlay.set_vexpand(true); // Use overlay instead of split

    // Ensure footer is visible and properly positioned
    footer.set_vexpand(false); // Footer should not expand vertically
    footer.set_hexpand(true); // Footer should expand horizontally
    footer.set_visible(true); // Explicitly ensure footer is visible

    // Add main box to window
    window.set_child(Some(&main_box));

    // --- Live HTML preview theme switching ---
    // Store refresh_preview closure for use on theme changes
    let refresh_preview_rc = Rc::new(RefCell::new(refresh_preview));
    // Register 'app.settings' action to show the settings dialog with the callback
    let settings_action = gtk4::gio::SimpleAction::new("settings", None);
    let update_editor_theme_rc = Rc::new(update_editor_theme);
    let update_preview_theme_rc = Rc::new(update_preview_theme);

    // Helper to persist view mode in settings.ron without blocking the UI
    // Uses the dedicated settings thread pool to avoid orphaned threads
    let save_view_mode = {
        let settings_manager = settings_manager.clone();
        let settings_tx = settings_tx.clone();
        Rc::new(move |mode: &str| {
            let settings_manager = settings_manager.clone();
            let mode_owned = mode.to_string();
            let task = Box::new(move || {
                use core::logic::swanson::LayoutSettings;
                if let Err(e) = settings_manager.update_settings(|s| {
                    if s.layout.is_none() {
                        s.layout = Some(LayoutSettings::default());
                    }
                    if let Some(ref mut l) = s.layout {
                        l.view_mode = Some(mode_owned.clone());
                    }
                }) {
                    log::error!("Failed to save view mode settings: {}", e);
                } else {
                    log::debug!("View mode saved: {}", mode_owned);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue view mode save task: {}", e);
            }
        })
    };

    // Clone asset_root for use in multiple closures
    let asset_root_for_settings = asset_root.clone();
    let window_for_language_handler = window.clone();

    // Reusable runtime locale switcher (used by both Settings → Language and the welcome screen).
    // NOTE: `None` means "System Default".
    let language_changed_handler: Rc<dyn Fn(Option<String>) + 'static> = {
        let translations_rc = translations_rc.clone();
        let menu_translations_rc = menu_translations_rc.clone();
        let dialog_translations_rc = dialog_translations_rc.clone();
        let menu_state = menu_state.clone();
        let app_for_locale = app.clone();
        let toolbar_ref = toolbar_ref.clone();
        let footer_labels_rc = footer_labels_rc.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_source_view_for_locale = editor_source_view.clone();
        let insert_mode_state = insert_mode_state.clone();
        let file_operations_rc = file_operations_rc.clone();
        let bookmark_manager = bookmark_manager.clone();
        let title_label = title_label.clone();
        let refresh_bookmark_marks_for_language = refresh_bookmark_marks.clone();
        let settings_manager_for_language = settings_manager.clone();

        Rc::new(move |selected_code: Option<String>| {
            let locale_code = selected_code
                .or_else(core::paths::detect_system_locale_iso639_1)
                .unwrap_or_else(|| "en".to_string());

            let window_for_locale = window_for_language_handler.clone();

            let translations_rc = translations_rc.clone();
            let menu_translations_rc = menu_translations_rc.clone();
            let dialog_translations_rc = dialog_translations_rc.clone();
            let menu_state = menu_state.clone();
            let app_for_locale = app_for_locale.clone();
            let toolbar_ref = toolbar_ref.clone();
            let footer_labels_rc = footer_labels_rc.clone();
            let editor_buffer = editor_buffer.clone();
            let editor_source_view_for_locale = editor_source_view_for_locale.clone();
            let insert_mode_state = insert_mode_state.clone();
            let file_operations_rc = file_operations_rc.clone();
            let bookmark_manager = bookmark_manager.clone();
            let title_label = title_label.clone();
            let refresh_bookmark_marks_for_language = refresh_bookmark_marks_for_language.clone();
            let settings_manager_for_locale = settings_manager_for_language.clone();

            glib::idle_add_local(move || {
                let localization_manager = match SimpleLocalizationManager::new() {
                    Ok(manager) => manager,
                    Err(e) => {
                        log::warn!("Failed to initialize localization manager: {}", e);
                        return glib::ControlFlow::Break;
                    }
                };

                if let Err(e) = localization_manager.load_locale(&locale_code) {
                    log::warn!(
                        "Failed to load locale '{}': {}. Falling back to English.",
                        locale_code,
                        e
                    );
                    if locale_code != "en" {
                        if let Err(e) = localization_manager.load_locale("en") {
                            log::error!("Failed to load fallback locale 'en': {}", e);
                        }
                    }
                }

                let new_translations = localization_manager.translations();
                *translations_rc.borrow_mut() = new_translations.clone();
                *menu_translations_rc.borrow_mut() = new_translations.menu.clone();
                *dialog_translations_rc.borrow_mut() = new_translations.dialog.clone();

                menu::update_menu_translations(menu_state.as_ref(), &new_translations);

                let recent_files = file_operations_rc.borrow().get_recent_files();
                crate::ui::menu_items::update_recent_files_menu(
                    &menu_state.recent_menu,
                    &recent_files,
                    &new_translations.menu,
                );

                let current_file_provider: Rc<dyn Fn() -> Option<std::path::PathBuf>> = {
                    let file_operations_rc = file_operations_rc.clone();
                    Rc::new(move || {
                        file_operations_rc
                            .borrow()
                            .buffer
                            .borrow()
                            .get_file_path()
                            .map(|path| path.to_path_buf())
                    })
                };
                let jump_to_bookmark: Rc<dyn Fn(std::path::PathBuf, u32)> = {
                    let file_operations_rc = file_operations_rc.clone();
                    let window = window_for_locale.clone();
                    let editor_buffer = editor_buffer.clone();
                    let source_view = editor_source_view_for_locale.clone();
                    let title_label = title_label.clone();
                    let dialog_translations_rc = dialog_translations_rc.clone();
                    let refresh_bookmark_marks = refresh_bookmark_marks_for_language.clone();
                    Rc::new(move |target_path: std::path::PathBuf, line: u32| {
                        let current_path = file_operations_rc
                            .borrow()
                            .buffer
                            .borrow()
                            .get_file_path()
                            .map(|path| path.to_path_buf());

                        let jump_in_current = || {
                            let _ = crate::components::editor::editor_manager::suppress_preview_to_editor_sync_globally();
                            let mut iter = editor_buffer
                                .iter_at_line(line as i32)
                                .unwrap_or_else(|| editor_buffer.end_iter());
                            editor_buffer.place_cursor(&iter);
                            source_view.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                            refresh_bookmark_marks();
                            window.present();
                            gtk4::prelude::GtkWindowExt::set_focus(
                                &window,
                                Some(source_view.upcast_ref::<gtk4::Widget>()),
                            );
                            source_view.grab_focus();

                            let editor_buffer_retry = editor_buffer.clone();
                            let source_view_retry = source_view.clone();
                            let window_retry = window.clone();
                            let refresh_bookmark_marks_retry = refresh_bookmark_marks.clone();
                            glib::timeout_add_local_once(
                                std::time::Duration::from_millis(220),
                                move || {
                                    refresh_bookmark_marks_retry();
                                    let mut iter = editor_buffer_retry
                                        .iter_at_line(line as i32)
                                        .unwrap_or_else(|| editor_buffer_retry.end_iter());
                                    editor_buffer_retry.place_cursor(&iter);
                                    source_view_retry
                                        .scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                                    window_retry.present();
                                    gtk4::prelude::GtkWindowExt::set_focus(
                                        &window_retry,
                                        Some(source_view_retry.upcast_ref::<gtk4::Widget>()),
                                    );
                                    source_view_retry.grab_focus();
                                    let _ = crate::components::editor::editor_manager::resume_preview_to_editor_sync_globally();
                                },
                            );
                        };

                        if current_path.as_ref().is_some_and(|p| p == &target_path) {
                            jump_in_current();
                            return;
                        }

                        let file_ops = file_operations_rc.clone();
                        let window = window.clone();
                        let editor_buffer = editor_buffer.clone();
                        let source_view_async = source_view.clone();
                        let title_label = title_label.clone();
                        let refresh_bookmark_marks = refresh_bookmark_marks.clone();
                        let dialog_translations = dialog_translations_rc.borrow().clone();
                        glib::MainContext::default().spawn_local(async move {
                            let _ = crate::components::editor::editor_manager::suppress_preview_to_editor_sync_globally();
                            let gtk_window: &gtk4::Window = window.upcast_ref();
                            let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                            let result = FileOperations::open_file_by_path_from_rc_async(
                                &file_ops,
                                &target_path,
                                gtk_window,
                                text_buffer,
                                &dialog_translations,
                                |w, doc_name, action| {
                                    FileDialogs::save_changes_dialog_callback(
                                        dialog_translations.clone(),
                                    )(w, doc_name, action)
                                },
                                |w, title, suggested| {
                                    FileDialogs::save_dialog_callback(dialog_translations.clone())(
                                        w, title, suggested,
                                    )
                                },
                            )
                            .await;

                            if let Err(e) = result {
                                let err_msg = e.to_string();
                                if err_msg.contains("cancelled by user") {
                                    log::debug!(
                                        "Bookmark switch cancelled by user (save/discard/stay): {}:{}",
                                        target_path.display(),
                                        line + 1
                                    );
                                } else {
                                    log::warn!(
                                        "Failed to open bookmark target {}:{} -> {}",
                                        target_path.display(),
                                        line + 1,
                                        err_msg
                                    );
                                }
                                let _ = crate::components::editor::editor_manager::resume_preview_to_editor_sync_globally();
                                return;
                            }

                            title_label.set_text(&file_ops.borrow().get_document_title());
                            refresh_bookmark_marks();

                            let window_for_focus = window.clone();
                            let refresh_bookmark_marks_idle = refresh_bookmark_marks.clone();
                            glib::idle_add_local_once(move || {
                                refresh_bookmark_marks_idle();
                                let mut iter = editor_buffer
                                    .iter_at_line(line as i32)
                                    .unwrap_or_else(|| editor_buffer.end_iter());
                                editor_buffer.place_cursor(&iter);
                                source_view_async.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                                window_for_focus.present();
                                gtk4::prelude::GtkWindowExt::set_focus(
                                    &window_for_focus,
                                    Some(source_view_async.upcast_ref::<gtk4::Widget>()),
                                );
                                source_view_async.grab_focus();

                                let window_retry = window_for_focus.clone();
                                let source_view_retry = source_view_async.clone();
                                let editor_buffer_retry = editor_buffer.clone();
                                let refresh_bookmark_marks_retry = refresh_bookmark_marks_idle.clone();
                                glib::timeout_add_local_once(
                                    std::time::Duration::from_millis(220),
                                    move || {
                                        refresh_bookmark_marks_retry();
                                        let mut iter = editor_buffer_retry
                                            .iter_at_line(line as i32)
                                            .unwrap_or_else(|| editor_buffer_retry.end_iter());
                                        editor_buffer_retry.place_cursor(&iter);
                                        source_view_retry.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                                        window_retry.present();
                                        gtk4::prelude::GtkWindowExt::set_focus(&window_retry, Some(
                                            source_view_retry.upcast_ref::<gtk4::Widget>(),
                                        ));
                                        source_view_retry.grab_focus();
                                        let _ = crate::components::editor::editor_manager::resume_preview_to_editor_sync_globally();
                                    },
                                );
                            });
                        });
                    })
                };
                crate::ui::menu_items::refresh_bookmark_menu(
                    &app_for_locale,
                    &menu_state.bookmarks_menu,
                    &bookmark_manager,
                    &current_file_provider,
                    &jump_to_bookmark,
                    &new_translations.menu,
                );

                // Update toolbar tooltips in place instead of rebuilding
                toolbar::update_toolbar_translations(&toolbar_ref.borrow(), &new_translations);

                if let Ok(mut buffer) = file_operations_rc.borrow().buffer.try_borrow_mut() {
                    if buffer.get_file_path().is_none() {
                        buffer.display_name = new_translations.messages.untitled_document.clone();
                    }
                }

                let is_insert = *insert_mode_state.borrow();
                crate::footer::update_footer_translations(
                    footer_labels_rc.as_ref(),
                    &new_translations.footer,
                    is_insert,
                );
                refresh_footer_snapshot(
                    &editor_buffer,
                    footer_labels_rc.clone(),
                    insert_mode_state.clone(),
                    settings_manager_for_locale.clone(),
                );

                if file_operations_rc
                    .borrow()
                    .buffer
                    .borrow()
                    .get_file_path()
                    .is_none()
                {
                    let title = file_operations_rc.borrow().get_document_title();
                    title_label.set_text(&title);
                }

                // Settings dialog now updates itself via polling, no need to clear cache
                glib::ControlFlow::Break
            });
        })
    };

    settings_action.connect_activate({
        // Clone directly from original sources to avoid intermediate reference chains
        let window = window.clone();
        let theme_manager = theme_manager.clone();
        let settings_path = settings_path.clone();
        let translations_rc = translations_rc.clone();
        let available_locale_infos_rc = available_locale_infos_rc.clone();
        let preview_css_rc = preview_css_rc.clone();
        let refresh_preview_rc = refresh_preview_rc.clone();
        let update_editor_theme_rc = update_editor_theme_rc.clone();
        let update_preview_theme_rc = update_preview_theme_rc.clone();
        let set_view_mode_rc = set_view_mode_rc.clone();
        let save_view_mode = save_view_mode.clone();
        let language_changed_handler = language_changed_handler.clone();
        move |_, _| {
            use crate::ui::settings::dialog::show_settings_dialog;

            // Create editor theme callback that updates both editor and preview
            let editor_callback = {
                let update_editor = update_editor_theme_rc.clone();
                let update_preview = update_preview_theme_rc.clone();
                let window_for_theme = window.clone();
                Box::new(move |scheme_id: String| {
                    update_editor(&scheme_id);
                    update_preview(&scheme_id);

                    // Toggle window CSS class for runtime GTK UI theme switching
                    // This cascades to all descendants (toolbar, footer, menu, etc.)
                    let new_mode = if scheme_id.contains("dark") { "dark" } else { "light" };
                    let old_class = if new_mode == "dark" { "marco-theme-light" } else { "marco-theme-dark" };
                    let new_class = format!("marco-theme-{}", new_mode);

                    // Update window - this automatically affects all child widgets via CSS cascade
                    window_for_theme.remove_css_class(old_class);
                    window_for_theme.add_css_class(&new_class);

                    log::debug!("Switched CSS class from {} to {} (window and all descendants)", old_class, new_class);
                }) as Box<dyn Fn(String) + 'static>
            };

            trace!("audit: opened settings dialog");
            // Build the callbacks struct for the settings dialog to keep the
            // callsite compact and satisfy the updated API.
            use crate::ui::settings::dialog::SettingsDialogCallbacks;

            let callbacks = SettingsDialogCallbacks {
                on_preview_theme_changed: Some(Box::new({
                    // Use weak references to prevent circular retention
                    let theme_manager_weak = Rc::downgrade(&theme_manager);
                    let preview_css_weak = Rc::downgrade(&preview_css_rc);
                    let refresh_preview_weak = Rc::downgrade(&refresh_preview_rc);
                    move |theme_filename: String| {
                        // Check if references are still valid before using
                        if let (Some(theme_manager), Some(preview_css_rc), Some(refresh_preview_rc)) = (
                            theme_manager_weak.upgrade(),
                            preview_css_weak.upgrade(),
                            refresh_preview_weak.upgrade(),
                        ) {
                            // On preview theme change, update CSS and call refresh
                            use std::fs;
                            let theme_manager = theme_manager.borrow();
                            let preview_theme_dir = theme_manager.preview_theme_dir.clone();
                            let css_path = preview_theme_dir.join(&theme_filename);
                            let css = fs::read_to_string(&css_path).unwrap_or_default();
                            *preview_css_rc.borrow_mut() = css;
                            (refresh_preview_rc.borrow())();
                        } else {
                            log::debug!("Preview theme callback called after main components were dropped");
                        }
                    }
                })),
                refresh_preview: Some(refresh_preview_rc.clone()),
                on_editor_theme_changed: Some(editor_callback),
                on_schema_changed: Some(Box::new({
                    // Use weak references to prevent circular retention
                    let active_schema_map_weak = Rc::downgrade(&active_schema_map);
                    let trigger_weak = Rc::downgrade(&trigger_footer_update);
                    move |_selected: Option<String>| {
                        // Check if references are still valid before using
                        if let (Some(active_schema_map), Some(trigger)) = (
                            active_schema_map_weak.upgrade(),
                            trigger_weak.upgrade(),
                        ) {
                            // Schema support removed; clear any existing schema and trigger footer update
                            *active_schema_map.borrow_mut() = None;
                            (trigger)();
                        } else {
                            log::debug!("Schema callback called after main components were dropped");
                        }
                    }
                })),
                // on_view_mode_changed: persist and forward to runtime setter
                on_view_mode_changed: Some(Box::new({
                    // Use weak reference to prevent circular retention
                    let set_view_mode_weak = Rc::downgrade(&set_view_mode_rc);
                    let save = save_view_mode.clone(); // This closure is self-contained, no circular ref risk
                    move |selected: String| {
                        // Persist the selection asynchronously (always works)
                        save(&selected);

                        // Check if view mode setter is still valid before using
                        if let Some(set_view_mode_rc) = set_view_mode_weak.upgrade() {
                            match selected.as_str() {
                                "HTML Preview" => (set_view_mode_rc)(ViewMode::HtmlPreview),
                                "Source Code" | "Code Preview" => (set_view_mode_rc)(ViewMode::CodePreview),
                                _ => {}
                            }
                        } else {
                            log::debug!("View mode callback called after main components were dropped");
                        }
                    }
                }) as Box<dyn Fn(String) + 'static>),
                // on_split_ratio_changed: update the actual paned widget position in real-time
                on_split_ratio_changed: Some(Box::new({
                    // GTK widgets have their own reference counting, but use weak ref for consistency
                    let split_paned_weak = split.downgrade();
                    move |ratio: i32| {
                        log::debug!("[SPLIT LIVE] Callback received ratio: {}%", ratio);
                        // Check if widget is still valid before using
                        if let Some(split_paned) = split_paned_weak.upgrade() {
                            // Calculate the pixel position based on the current paned width
                            let paned_width = split_paned.allocated_width();
                            let new_position = if paned_width > 0 {
                                (paned_width as f64 * ratio as f64 / 100.0) as i32
                            } else {
                                // Fallback to default width calculation
                                (1200.0 * ratio as f64 / 100.0) as i32
                            };

                            split_paned.set_position(new_position);
                            log::debug!(
                                "[SPLIT LIVE] Applied ratio: {}% -> {}px (width: {}px)",
                                ratio,
                                new_position,
                                paned_width
                            );
                        } else {
                            log::debug!("[SPLIT LIVE] Split paned widget was dropped");
                        }
                    }
                }) as Box<dyn Fn(i32) + 'static>),
                // on_sync_scrolling_changed: enable/disable scroll synchronization
                on_sync_scrolling_changed: Some(Box::new({
                    move |enabled: bool| {
                        // Use the global scroll sync API to enable/disable synchronization
                        use crate::components::editor::editor_manager::set_scroll_sync_enabled_globally;
                        let _ = set_scroll_sync_enabled_globally(enabled);
                        log::debug!("Scroll sync toggled: {}", enabled);
                    }
                }) as Box<dyn Fn(bool) + 'static>),
                // on_line_numbers_changed: enable/disable line numbers in the editor
                on_line_numbers_changed: Some(Box::new({
                    move |enabled: bool| {
                        // Use the global line numbers API to update all editors
                        use crate::components::editor::editor_manager::update_line_numbers_globally;
                        let _ = update_line_numbers_globally(enabled);
                        log::debug!("Line numbers toggled: {}", enabled);
                    }
                }) as Box<dyn Fn(bool) + 'static>),
                on_language_changed: Some(Box::new({
                    let handler = language_changed_handler.clone();
                    move |selected_code: Option<String>| {
                        (handler)(selected_code);
                    }
                }) as Box<dyn Fn(Option<String>) + 'static>),
            };

            show_settings_dialog(
                window.upcast_ref(),
                theme_manager.clone(),
                settings_path.clone(),
                &asset_root_for_settings,
                translations_rc.clone(),
                available_locale_infos_rc.clone(),
                callbacks,
            );
        }
    });
    app.add_action(&settings_action);

    // Register about action
    let about_action = gtk4::gio::SimpleAction::new("about", None);
    about_action.connect_activate({
        let window = window.clone();
        let translations_rc = translations_rc.clone();
        move |_, _| {
            let dialog_translations = translations_rc.borrow().dialog.clone();
            crate::ui::menu_items::about::show_about_dialog(&window, &dialog_translations);
        }
    });
    app.add_action(&about_action);

    // Register diagnostics reference action
    let diagnostics_reference_action = gtk4::gio::SimpleAction::new("diagnostics_reference", None);
    diagnostics_reference_action.connect_activate({
        let window = window.clone();
        move |_, _| {
            crate::ui::dialogs::diagnostics_reference::show_diagnostics_reference_dialog(
                window.upcast_ref::<gtk4::Window>(),
            );
        }
    });
    app.add_action(&diagnostics_reference_action);

    crate::ui::menu_items::edit::setup_edit_actions(app, &editor_buffer, &editor_source_view);

    let current_file_provider_for_menu: std::rc::Rc<dyn Fn() -> Option<std::path::PathBuf>> = {
        let file_operations_rc = file_operations_rc.clone();
        std::rc::Rc::new(move || {
            file_operations_rc
                .borrow()
                .buffer
                .borrow()
                .get_file_path()
                .map(|path| path.to_path_buf())
        })
    };

    crate::ui::menu_items::setup_inline_blocks_modules_actions(
        app,
        &editor_buffer,
        &editor_source_view,
        &window,
        settings_manager.clone(),
        current_file_provider_for_menu,
    );
    crate::ui::menu_items::tools::setup_tools_actions(
        app,
        &menu_state.tools_menu,
        translations_rc.clone(),
        settings_manager.clone(),
        &editor_source_view,
        set_view_mode_rc.clone(),
    );

    // Register search & replace action
    let search_action = gtk4::gio::SimpleAction::new("search", None);
    search_action.connect_activate({
        let window = window.clone();
        let buffer = Rc::new(editor_buffer.clone());
        let source_view = Rc::new(editor_source_view.clone());
        let translations_rc = translations_rc.clone();
        #[cfg(target_os = "linux")]
        let webview = editor_webview.clone(); // Already Rc<RefCell<WebView>>
        let cache = Rc::new(RefCell::new(core::logic::cache::SimpleFileCache::new()));
        move |_, _| {
            let search_translations = translations_rc.borrow().search.clone();
            #[cfg(target_os = "linux")]
            {
                use crate::ui::dialogs::search::show_search_window;
                show_search_window(
                    window.upcast_ref(),
                    cache.clone(),
                    Rc::clone(&buffer),
                    Rc::clone(&source_view),
                    webview.clone(),
                    &search_translations,
                );
            }
            #[cfg(target_os = "windows")]
            {
                use crate::ui::dialogs::search::show_search_window_no_webview;
                show_search_window_no_webview(
                    window.upcast_ref(),
                    cache.clone(),
                    Rc::clone(&buffer),
                    Rc::clone(&source_view),
                    &search_translations,
                );
            }
        }
    });
    app.add_action(&search_action);
    app.set_accels_for_action("app.search", &["<Control>f"]);

    // Populate the Recent Files submenu from FileOperations' recent list
    crate::ui::menu_items::register_file_actions_async(
        app.clone(),
        file_operations_rc.clone(),
        &window,
        &editor_buffer,
        &title_label,
        &dialog_translations_rc.borrow(),
        FileDialogs::open_dialog_callback(dialog_translations_rc.borrow().clone()),
        FileDialogs::save_changes_dialog_callback(dialog_translations_rc.borrow().clone()),
        FileDialogs::save_dialog_callback(dialog_translations_rc.borrow().clone()),
    );

    // Wire dynamic recent-file actions using the recent_menu from the UI
    crate::ui::menu_items::setup_recent_actions(
        app,
        file_operations_rc.clone(),
        &menu_state.recent_menu,
        &window,
        &editor_buffer,
        &title_label,
        menu_translations_rc.clone(),
        dialog_translations_rc.clone(),
        FileDialogs::save_changes_dialog_callback(dialog_translations_rc.borrow().clone()),
        FileDialogs::save_dialog_callback(dialog_translations_rc.borrow().clone()),
    );

    // Wire bookmarks menu actions and dynamic updates.
    {
        let current_file_provider: Rc<dyn Fn() -> Option<std::path::PathBuf>> = {
            let file_operations_rc = file_operations_rc.clone();
            Rc::new(move || {
                file_operations_rc
                    .borrow()
                    .buffer
                    .borrow()
                    .get_file_path()
                    .map(|path| path.to_path_buf())
            })
        };

        let jump_to_bookmark: Rc<dyn Fn(std::path::PathBuf, u32)> = {
            let file_operations_rc = file_operations_rc.clone();
            let window = window.clone();
            let editor_buffer = editor_buffer.clone();
            let source_view = editor_source_view.clone();
            let title_label = title_label.clone();
            let dialog_translations_rc = dialog_translations_rc.clone();
            let refresh_bookmark_marks = refresh_bookmark_marks.clone();
            Rc::new(move |target_path: std::path::PathBuf, line: u32| {
                let current_path = file_operations_rc
                    .borrow()
                    .buffer
                    .borrow()
                    .get_file_path()
                    .map(|path| path.to_path_buf());

                let jump_in_current = || {
                    let _ = crate::components::editor::editor_manager::suppress_preview_to_editor_sync_globally();
                    let mut iter = editor_buffer
                        .iter_at_line(line as i32)
                        .unwrap_or_else(|| editor_buffer.end_iter());
                    editor_buffer.place_cursor(&iter);
                    source_view.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                    refresh_bookmark_marks();
                    window.present();
                    gtk4::prelude::GtkWindowExt::set_focus(
                        &window,
                        Some(source_view.upcast_ref::<gtk4::Widget>()),
                    );
                    source_view.grab_focus();

                    let editor_buffer_retry = editor_buffer.clone();
                    let source_view_retry = source_view.clone();
                    let window_retry = window.clone();
                    let refresh_bookmark_marks_retry = refresh_bookmark_marks.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(220),
                        move || {
                            refresh_bookmark_marks_retry();
                            let mut iter = editor_buffer_retry
                                .iter_at_line(line as i32)
                                .unwrap_or_else(|| editor_buffer_retry.end_iter());
                            editor_buffer_retry.place_cursor(&iter);
                            source_view_retry.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                            window_retry.present();
                            gtk4::prelude::GtkWindowExt::set_focus(
                                &window_retry,
                                Some(source_view_retry.upcast_ref::<gtk4::Widget>()),
                            );
                            source_view_retry.grab_focus();
                            let _ = crate::components::editor::editor_manager::resume_preview_to_editor_sync_globally();
                        },
                    );
                };

                if current_path.as_ref().is_some_and(|p| p == &target_path) {
                    jump_in_current();
                    return;
                }

                let file_ops = file_operations_rc.clone();
                let window = window.clone();
                let editor_buffer = editor_buffer.clone();
                let source_view_async = source_view.clone();
                let title_label = title_label.clone();
                let refresh_bookmark_marks = refresh_bookmark_marks.clone();
                let dialog_translations = dialog_translations_rc.borrow().clone();
                glib::MainContext::default().spawn_local(async move {
                    let _ = crate::components::editor::editor_manager::suppress_preview_to_editor_sync_globally();
                    let gtk_window: &gtk4::Window = window.upcast_ref();
                    let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                    let result = FileOperations::open_file_by_path_from_rc_async(
                        &file_ops,
                        &target_path,
                        gtk_window,
                        text_buffer,
                        &dialog_translations,
                        |w, doc_name, action| {
                            FileDialogs::save_changes_dialog_callback(
                                dialog_translations.clone(),
                            )(w, doc_name, action)
                        },
                        |w, title, suggested| {
                            FileDialogs::save_dialog_callback(dialog_translations.clone())(
                                w, title, suggested,
                            )
                        },
                    )
                    .await;

                    if let Err(e) = result {
                        let err_msg = e.to_string();
                        if err_msg.contains("cancelled by user") {
                            log::debug!(
                                "Bookmark switch cancelled by user (save/discard/stay): {}:{}",
                                target_path.display(),
                                line + 1
                            );
                        } else {
                            log::warn!(
                                "Failed to open bookmark target {}:{} -> {}",
                                target_path.display(),
                                line + 1,
                                err_msg
                            );
                        }
                        let _ = crate::components::editor::editor_manager::resume_preview_to_editor_sync_globally();
                        return;
                    }

                    title_label.set_text(&file_ops.borrow().get_document_title());
                    refresh_bookmark_marks();

                    let window_for_focus = window.clone();
                    let refresh_bookmark_marks_idle = refresh_bookmark_marks.clone();
                    glib::idle_add_local_once(move || {
                        refresh_bookmark_marks_idle();
                        let mut iter = editor_buffer
                            .iter_at_line(line as i32)
                            .unwrap_or_else(|| editor_buffer.end_iter());
                        editor_buffer.place_cursor(&iter);
                        source_view_async.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                        window_for_focus.present();
                        gtk4::prelude::GtkWindowExt::set_focus(
                            &window_for_focus,
                            Some(source_view_async.upcast_ref::<gtk4::Widget>()),
                        );
                        source_view_async.grab_focus();

                        let window_retry = window_for_focus.clone();
                        let source_view_retry = source_view_async.clone();
                        let editor_buffer_retry = editor_buffer.clone();
                        let refresh_bookmark_marks_retry = refresh_bookmark_marks_idle.clone();
                        glib::timeout_add_local_once(
                            std::time::Duration::from_millis(220),
                            move || {
                                refresh_bookmark_marks_retry();
                                let mut iter = editor_buffer_retry
                                    .iter_at_line(line as i32)
                                    .unwrap_or_else(|| editor_buffer_retry.end_iter());
                                editor_buffer_retry.place_cursor(&iter);
                                source_view_retry.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
                                window_retry.present();
                                gtk4::prelude::GtkWindowExt::set_focus(
                                    &window_retry,
                                    Some(source_view_retry.upcast_ref::<gtk4::Widget>()),
                                );
                                source_view_retry.grab_focus();
                                let _ = crate::components::editor::editor_manager::resume_preview_to_editor_sync_globally();
                            },
                        );
                    });
                });
            })
        };

        crate::ui::menu_items::setup_bookmark_actions(
            app,
            &menu_state.bookmarks_menu,
            bookmark_manager.clone(),
            current_file_provider.clone(),
            jump_to_bookmark.clone(),
            menu_translations_rc.clone(),
        );

        // Refresh bookmarks menu whenever recent-file callbacks indicate file changes.
        {
            let app_owned = app.clone();
            let bookmarks_menu = menu_state.bookmarks_menu.clone();
            let bookmark_manager = bookmark_manager.clone();
            let current_file_provider = current_file_provider.clone();
            let jump_to_bookmark = jump_to_bookmark.clone();
            let menu_translations_rc = menu_translations_rc.clone();
            let refresh_bookmark_marks = refresh_bookmark_marks.clone();
            file_operations_rc
                .borrow()
                .register_recent_changed_callback(move || {
                    crate::ui::menu_items::refresh_bookmark_menu(
                        &app_owned,
                        &bookmarks_menu,
                        &bookmark_manager,
                        &current_file_provider,
                        &jump_to_bookmark,
                        &menu_translations_rc.borrow(),
                    );
                    refresh_bookmark_marks();
                });
        }

        {
            let refresh_bookmark_marks = refresh_bookmark_marks.clone();
            bookmark_manager.register_changed_callback(move || {
                refresh_bookmark_marks();
            });
        }
    }

    // Open initial file if provided via command line
    if let Some(file_path) = initial_file {
        let dialog_translations = dialog_translations_rc.borrow().clone();
        let load_context = crate::ui::menu_items::InitialFileLoadContext {
            file_path,
            window: window.clone(),
            editor_buffer: editor_buffer.clone(),
            title_label: title_label.clone(),
            dialog_translations: dialog_translations.clone(),
            show_save_changes_dialog: FileDialogs::save_changes_dialog_callback(
                dialog_translations.clone(),
            ),
            show_save_dialog: FileDialogs::save_dialog_callback(dialog_translations),
        };
        FileOperations::load_initial_file_async(file_operations_rc.clone(), load_context);
    }

    // Apply startup editor settings to ensure editor uses settings.ron values
    if let Err(e) = crate::components::editor::editor_manager::apply_startup_editor_settings() {
        log::warn!("Failed to apply startup editor settings: {}", e);
    }

    // Load and apply saved window state
    crate::logic::window_state::apply_saved_window_state(&window, &settings_manager);

    // Connect window state change handlers to persist settings
    crate::logic::window_state::connect_window_state_persistence(
        &window,
        &settings_manager,
        &settings_tx,
        &refresh_bookmark_marks,
    );

    // Connect to window destroy signal to clean up settings thread
    window.connect_destroy({
        let settings_pool_rc = settings_pool_rc.clone();
        move |_| {
            log::debug!("Window destroyed, cleaning up settings thread");
            settings_pool_rc.borrow_mut().shutdown();

            // Clean up global resources
            crate::components::editor::editor_manager::shutdown_editor_manager();
            core::shutdown_global_parser_cache();
            core::logic::cache::shutdown_global_cache();
        }
    });

    // Present the window first
    window.present();

    // Run a post-present refresh so bookmarks loaded from settings are redrawn
    // against the final startup geometry.
    {
        let refresh_bookmark_marks = refresh_bookmark_marks.clone();
        glib::idle_add_local_once(move || {
            refresh_bookmark_marks();
        });
    }

    {
        let refresh_bookmark_marks = refresh_bookmark_marks.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(120), move || {
            refresh_bookmark_marks();
        });
    }

    // Show welcome screen on first run (Week 4)
    // This is non-blocking - appears on top of the main window
    log::info!("main: Checking if welcome screen should be shown");
    if crate::ui::dialogs::welcome_screen::should_show_welcome_screen(&settings_manager) {
        log::info!("main: Showing welcome screen");
        crate::ui::dialogs::welcome_screen::show_welcome_screen(
            &settings_manager,
            Some(window.upcast_ref::<gtk4::Window>()),
            Some(Box::new({
                let handler = language_changed_handler.clone();
                move |selected_code: Option<String>| {
                    (handler)(selected_code);
                }
            })),
        );
    }
}
