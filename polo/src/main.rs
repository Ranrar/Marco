#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Polo - Lightweight Markdown Viewer
// Standalone viewer for Marco markdown files
//
//! # Polo - Lightweight Markdown Viewer
//!
//! Polo is a standalone markdown viewer designed as the lightweight companion to the Marco
//! markdown editor. It provides a read-only view of markdown files with full support for
//! Marco's custom markdown extensions and syntax highlighting.
//!
//! ## Key Features
//!
//! - **Pure Viewer**: No editing capabilities - focused on viewing only
//! - **Marco Integration**: Opens files in Marco editor on demand
//! - **Theme Support**: Light/dark modes with multiple CSS themes
//! - **Fast Rendering**: Uses core's cached parser for instant previews
//! - **Minimal Dependencies**: No SourceView5, just GTK4 + WebKit6
//!
//! ## Architecture
//!
//! Polo follows Marco's architectural patterns:
//! - **main.rs**: Application gateway only - no business logic
//! - **components/**: All UI components and logic organized by function
//! - **core**: Shared parsing, rendering, and settings management
//!
//! ## Settings Integration
//!
//! Polo shares common settings with Marco (themes, appearance) while maintaining
//! its own section for viewer-specific settings (window size, last opened file).
//!
//! ## Command Line Usage
//!
//! ```bash
//! polo <file.md>           # Open markdown file
//! polo --debug <file.md>   # Open with debug logging
//! polo --help              # Show help message
//! ```

mod components;

use components::css::load_css_from_path;
use components::file_tree_panel::create_file_tree_panel;
use components::menu::create_custom_titlebar;
use components::sidebar_coordinator::SidebarCoordinator;
use components::toc_panel::{create_toc_panel, TocPanelHandle};
use components::utils::{apply_gtk_theme_preference, parse_hex_to_rgba};
use components::viewer::platform_webview::PlatformWebView;
use components::viewer::{load_and_render_markdown, show_empty_state_with_theme};
use gtk4::{gio, glib, prelude::*, Application, ApplicationWindow, Label};
use marco_shared::paths::PoloPaths;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

const APP_ID: &str = "io.github.ranrar.Polo";

/// Centralized fatal error handler
///
/// This function handles unrecoverable errors during application initialization.
/// It ensures proper cleanup (logger shutdown) before terminating the application.
///
/// # Arguments
/// * `message` - User-friendly error message to display
///
/// # Panics
/// This function never returns - it always exits the process with code 1
fn fatal_error(message: &str) -> ! {
    log::error!("FATAL: {}", message);
    eprintln!("Fatal error: {}", message);
    marco_shared::logic::file_logger::shutdown();
    std::process::exit(1);
}

/// Install a glib default-log handler that suppresses the harmless
/// `GLib-GIO-WARNING ... Error releasing name ...WebProcess-...: The
/// connection is closed` message emitted by WebKitGTK during shutdown.
/// All other messages are forwarded to glib's default handler.
fn install_glib_log_filter() {
    glib::log_set_default_handler(|domain, level, message| {
        if message.contains("Error releasing name") && message.contains("WebProcess") {
            return;
        }
        glib::log_default_handler(domain, level, Some(message));
    });
}

fn main() -> glib::ExitCode {
    // Fix: restore environment variables modified by VS Code snap before WebKit
    // spawns its helper subprocesses.  See marco/src/main.rs for full explanation.
    #[cfg(target_os = "linux")]
    {
        const SNAP_PAIRS: &[(&str, &str)] = &[
            ("XDG_DATA_DIRS", "XDG_DATA_DIRS_VSCODE_SNAP_ORIG"),
            ("XDG_CONFIG_DIRS", "XDG_CONFIG_DIRS_VSCODE_SNAP_ORIG"),
            ("GTK_EXE_PREFIX", "GTK_EXE_PREFIX_VSCODE_SNAP_ORIG"),
            ("GTK_IM_MODULE_FILE", "GTK_IM_MODULE_FILE_VSCODE_SNAP_ORIG"),
            (
                "GSETTINGS_SCHEMA_DIR",
                "GSETTINGS_SCHEMA_DIR_VSCODE_SNAP_ORIG",
            ),
            ("GIO_MODULE_DIR", "GIO_MODULE_DIR_VSCODE_SNAP_ORIG"),
        ];
        unsafe {
            for (var, orig_var) in SNAP_PAIRS {
                if let Ok(orig_val) = std::env::var(orig_var) {
                    if orig_val.is_empty() {
                        std::env::remove_var(var);
                    } else {
                        std::env::set_var(var, &orig_val);
                    }
                }
            }
        }
    }

    // Setup font directory for IcoMoon icon font (MUST be done before GTK init)
    use marco_shared::paths::PoloPaths;
    let polo_paths = match PoloPaths::new() {
        Ok(paths) => paths,
        Err(e) => {
            // Fall back to printing — logger not yet initialized.
            eprintln!("Cannot initialize Polo paths: {:?}", e);
            std::process::exit(1);
        }
    };

    // Initialize logger based on the same `log_to_file` setting Marco uses,
    // so both binaries share consistent file-logging behavior.
    //
    // Default to `Info` to avoid massive log files: marco-core's grammar
    // parsers emit Debug/Trace lines containing the full input slice, which
    // can produce hundreds of MB of log spam for large documents (e.g.
    // stresstest.md) and effectively block on log I/O. `RUST_LOG` can opt
    // into more verbose levels.
    let level = match std::env::var("RUST_LOG") {
        Ok(v) => {
            let v = v.to_ascii_lowercase();
            if v.contains("trace") {
                log::LevelFilter::Trace
            } else if v.contains("debug") {
                log::LevelFilter::Debug
            } else if v.contains("warn") {
                log::LevelFilter::Warn
            } else if v.contains("error") {
                log::LevelFilter::Error
            } else {
                log::LevelFilter::Info
            }
        }
        Err(_) => log::LevelFilter::Info,
    };
    let rust_log_set = std::env::var("RUST_LOG").is_ok();
    // Read shared `log_to_file` setting from the Polo settings file.
    let log_to_file = {
        let settings_path = polo_paths.settings_file();
        match marco_shared::logic::swanson::SettingsManager::initialize(settings_path) {
            Ok(mgr) => mgr.get_settings().log_to_file.unwrap_or(false),
            Err(_) => false,
        }
    };
    let logging_enabled = log_to_file || rust_log_set;
    if let Err(e) = marco_shared::logic::file_logger::init(logging_enabled, level) {
        eprintln!("Failed to initialize logger: {}", e);
    } else if logging_enabled {
        let resolved = marco_shared::logic::file_logger::current_log_dir();
        println!(
            "Logging enabled (level: {:?}), log files stored under: {}",
            level,
            resolved.display()
        );
    }

    // Install a glib log filter to suppress the harmless WebKit shutdown
    // warning `Error releasing name ...WebProcess-...: The connection is
    // closed`.  This message is emitted by GLib-GIO when WebKitGTK tears
    // down its sandboxed web-process D-Bus connection during exit; there
    // is no application-side fix.
    install_glib_log_filter();

    // Icon font support removed - icon fonts (IcoMoon) are no longer used; use inline SVGs instead.

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN | gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    // Wrap polo_paths in Rc for sharing across closures
    let polo_paths = std::rc::Rc::new(polo_paths);
    let polo_paths_for_cmdline = polo_paths.clone();
    let polo_paths_for_open = polo_paths.clone();

    // Handle command-line arguments
    app.connect_command_line(move |app, cmd_line| {
        let args: Vec<String> = cmd_line
            .arguments()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        // Parse arguments
        if args.len() > 1 {
            for arg in &args[1..] {
                if arg == "--help" || arg == "-h" {
                    println!("Polo - Lightweight Markdown Viewer");
                    println!("\nUsage:");
                    println!("  polo <file.md>           Open markdown file");
                    println!("  polo --debug <file.md>   Open with debug logging");
                    println!("  polo --help              Show this help message");
                    return 0.into();
                } else if arg == "--debug" {
                    // Debug flag already handled by logger init
                    continue;
                } else if arg.ends_with(".md") || arg.ends_with(".markdown") {
                    // Found markdown file
                    eprintln!("[FileOps] Opened file by path: {}", arg);
                    build_ui(app, Some(arg.clone()), polo_paths_for_cmdline.clone());
                    return 0.into();
                } else if !arg.starts_with('-') {
                    // Treat as file path
                    eprintln!("[FileOps] Opened file by path: {}", arg);
                    build_ui(app, Some(arg.clone()), polo_paths_for_cmdline.clone());
                    return 0.into();
                }
            }
        }

        // No file specified - open empty Polo
        build_ui(app, None, polo_paths_for_cmdline.clone());
        0.into()
    });

    // Handle file opening via file manager (drag & drop, right-click)
    app.connect_open(move |app, files, _hint| {
        if let Some(file) = files.first() {
            if let Some(path) = file.path() {
                eprintln!("[FileOps] Opened file by path: {}", path.display());
                build_ui(
                    app,
                    Some(path.to_string_lossy().to_string()),
                    polo_paths_for_open.clone(),
                );
            }
        }
    });

    let exit_code = app.run();

    // Cleanup
    marco_shared::logic::file_logger::shutdown();
    exit_code
}

fn build_ui(app: &Application, file_path: Option<String>, polo_paths: std::rc::Rc<PoloPaths>) {
    use marco_shared::paths::PathProvider;

    // Initialize settings manager early
    let settings_path = polo_paths.settings_file();

    let settings_manager =
        match marco_shared::logic::swanson::SettingsManager::initialize(settings_path.clone()) {
            Ok(manager) => {
                log::debug!("Settings loaded successfully");
                manager
            }
            Err(e) => {
                log::warn!("Failed to load settings, using defaults: {}", e);
                // Create default settings and continue
                match marco_shared::logic::swanson::SettingsManager::initialize(settings_path) {
                    Ok(manager) => manager,
                    Err(e) => {
                        fatal_error(&format!("Cannot initialize settings: {}", e));
                    }
                }
            }
        };

    // Load settings
    let settings = settings_manager.get_settings();

    // Get saved theme from COMMON appearance settings (shared with Marco)
    let saved_theme = settings
        .appearance
        .as_ref()
        .and_then(|a| a.preview_theme.clone())
        .unwrap_or_else(|| "marco.css".to_string());

    log::debug!("Using theme from settings: {}", saved_theme);

    // Get saved window size from POLO-specific settings
    let (window_width, window_height) = if let Some(polo) = &settings.polo {
        if let Some(polo_window) = &polo.window {
            polo_window.get_window_size()
        } else {
            (1000, 800) // Default for Polo
        }
    } else {
        (1000, 800) // Default for Polo
    };

    log::debug!("Using window size: {}x{}", window_width, window_height);

    // Load CSS styling
    let asset_root = polo_paths.asset_root();
    load_css_from_path(asset_root);

    // Apply GTK dark mode preference based on settings
    apply_gtk_theme_preference(&settings_manager);

    // Get filename for titlebar
    let filename = file_path.as_ref().and_then(|p| {
        PathBuf::from(p)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
    });

    // Create shared reference to current file path (for theme switching and file opening)
    // Uses RwLock for interior mutability in GTK callbacks. Since GTK runs in a single-threaded
    // event loop, lock poisoning is extremely unlikely. All lock accesses gracefully handle
    // poisoning by using if-let-Ok patterns, treating it as a safe no-op rather than panicking.
    let current_file_path: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(file_path.clone()));

    // Set window title based on whether a file is opened
    let window_title = match filename.as_ref() {
        Some(name) => format!("Polo - {}", name),
        None => "Polo".to_string(),
    };

    // Create and show window
    let window = ApplicationWindow::builder()
        .application(app)
        .title(window_title)
        .default_width(window_width as i32)
        .default_height(window_height as i32)
        .build();
    window.add_css_class("polo-window");

    // Add theme-specific CSS class based on current mode
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

    // Set window icon (GTK will look for icon named "polo" in the system icon theme)
    window.set_icon_name(Some("polo"));

    // Create platform WebView for markdown preview
    let webview = PlatformWebView::new(&window)
        .map_err(|e| {
            log::error!("Failed to create WebView: {}", e);
            e
        })
        .unwrap_or_else(|e| fatal_error(&format!("Cannot create WebView: {}", e)));

    // Set background color to prevent flash during loading: match the HTML theme
    // background so no dark/light mismatch is visible if the HWND appears before
    // the page finishes painting.
    let bg_hex = if current_theme_mode == "dark" {
        "#1e1e1e"
    } else {
        "#ffffff"
    };
    if let Some(rgba) = parse_hex_to_rgba(bg_hex) {
        webview.set_background_color_rgba(&rgba);
    }

    // Link-hover tooltip (shows the full address of the currently hovered
    // preview link) is now entirely in-page HTML/CSS/JS — see
    // `viewer::link_hover` and `viewer::javascript::HOVER_REPORT_JS`. No
    // Rust-side wiring needed; a native `gtk4::Popover` used to be driven
    // from here, but its own show/hide/reposition calls crashed the app on
    // Windows at hover frequency (see `viewer::link_hover`'s module doc).

    // Wire link policy: external links open in browser, local .md links prompt to reload.
    // Use a shared slot so the callback can update the TOC once it is created below.
    let toc_for_links: Rc<RefCell<Option<TocPanelHandle>>> = Rc::new(RefCell::new(None));
    // Late-bound slot for the custom titlebar's title Label (the only visible
    // title text — Polo uses a client-side-decorated titlebar, so the native
    // `window.set_title()` alone is never shown on screen). The titlebar is
    // built after this closure and after the toolbar (see below), so both
    // reach the real Label through this slot once `create_custom_titlebar`
    // fills it in, instead of updating a throwaway one. Mirrors `toc_for_links`.
    let title_label_for_header: Rc<RefCell<Option<Label>>> = Rc::new(RefCell::new(None));
    {
        let webview_for_links = webview.clone();
        let window_for_links = window.clone();
        let theme_for_links = saved_theme.clone();
        let settings_for_links = settings_manager.clone();
        let asset_root_for_links = polo_paths.asset_root().to_path_buf();
        let current_file_path_for_links = current_file_path.clone();
        let toc_for_links = toc_for_links.clone();
        let title_label_for_links = title_label_for_header.clone();

        webview.setup_link_policy(move |path, _fragment| {
            let filename = std::path::Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.clone());

            let webview = webview_for_links.clone();
            let window = window_for_links.clone();
            let theme = theme_for_links.clone();
            let settings = settings_for_links.clone();
            let asset_root = asset_root_for_links.clone();
            let current_file_path = current_file_path_for_links.clone();
            let path_for_open = path.clone();
            let fname_for_open = filename.clone();
            let toc_for_open = toc_for_links.clone();
            let title_label_for_open = title_label_for_links.clone();

            components::dialog::show_open_local_file_dialog(
                &window_for_links,
                &filename,
                move || {
                    if let Ok(mut guard) = current_file_path.write() {
                        *guard = Some(path_for_open.clone());
                    }
                    let title_text = format!("Polo - {}", fname_for_open);
                    window.set_title(Some(&title_text));
                    if let Some(label) = title_label_for_open.borrow().as_ref() {
                        label.set_text(&title_text);
                    }
                    load_and_render_markdown(
                        &webview,
                        &path_for_open,
                        &theme,
                        &settings,
                        &asset_root,
                    );
                    // Update the TOC for the newly-loaded file.
                    if let Ok(text) = marco_shared::cache::cached::read_to_string(
                        std::path::Path::new(&path_for_open),
                    ) {
                        if let Some(h) = toc_for_open.borrow().as_ref() {
                            h.update_from_text_async(text);
                        }
                    }
                },
            );
        });
    }

    // Shared between the TOC and file-tree panels: once the user manually
    // resizes either one's divider, both remember and reuse that exact
    // width instead of falling back to their own default/auto-size, for as
    // long as the app stays open (not persisted across restarts). See
    // `sidebar_coordinator::SharedPanelWidth`.
    let sidebar_manual_width: components::sidebar_coordinator::SharedPanelWidth =
        Rc::new(std::cell::Cell::new(None));

    // Create TOC panel (wraps webview in a Paned)
    let (toc_paned, toc_handle) = create_toc_panel(&webview, sidebar_manual_width.clone());
    // Fill the shared slot so the link-policy callback can update the TOC.
    *toc_for_links.borrow_mut() = Some(toc_handle.clone());

    // Create the file-tree sidebar panel (wraps toc_paned in an outer
    // Paned, so it sits to the left of TOC/webview — same nesting the
    // original feature plan called for).
    //
    // Both platforms start in the panel's default Home root (a single
    // expandable node for the user's home/profile directory — labeled
    // "Home" on Linux, the username on Windows), with the panel's own
    // Home/System switcher covering full-system browsing (`/` on Linux,
    // enumerated drives on Windows) — see dir.md §3.
    let (files_paned, file_tree_handle) =
        create_file_tree_panel(None, sidebar_manual_width.clone());
    files_paned.set_end_child(Some(&toc_paned));

    // TOC and the file tree are mutually exclusive — both the toolbar
    // buttons and the View menu items go through this coordinator instead
    // of toggling either panel directly.
    let sidebar = SidebarCoordinator::new(toc_handle.clone(), file_tree_handle.clone());
    // Wrap the WebView in a loading-overlay so we can show an indeterminate
    // progress bar (centered, GTK-themed) while files are being parsed and
    // rendered.  The overlay itself becomes the Paned's end child; the
    // WebView sits inside it as the main child.
    let loading_overlay =
        components::viewer::loading_overlay::LoadingOverlay::new(&webview.widget());
    components::viewer::loading_overlay::set_global(loading_overlay.clone());
    // On Windows the wry HWND is a native child window and paints on top of all
    // GTK content, so the GTK progress frame is never visible while the WebView
    // is in its normal position.  Wire up the offscreen hook so that show()/hide()
    // move the HWND out of the way while rendering and restore it when done.
    #[cfg(target_os = "windows")]
    {
        let webview_for_hook = webview.clone();
        loading_overlay.set_offscreen_hook(move |offscreen| {
            webview_for_hook.set_offscreen_for_loading(offscreen);
        });
    }
    toc_paned.set_end_child(Some(loading_overlay.widget()));

    // Hide the overlay only once the WebView has *actually finished* painting
    // the new page — not when we merely queued it for load.  Without this the
    // bar disappears seconds before the new HTML replaces the old welcome
    // content on screen.
    webview.connect_load_finished(|| {
        components::viewer::loading_overlay::hide();
    });

    // Load and render the markdown file
    let file_path_for_render = file_path.clone();
    let asset_root_for_render = polo_paths.asset_root();
    if let Some(ref path) = file_path_for_render {
        load_and_render_markdown(
            &webview,
            path,
            &saved_theme,
            &settings_manager,
            asset_root_for_render,
        );
        // Populate TOC off the main thread so large files don't stall the event loop.
        if let Ok(text) = marco_shared::cache::cached::read_to_string(std::path::Path::new(path)) {
            toc_handle.update_from_text_async(text);
        }
        file_tree_handle.set_open_path(Some(std::path::PathBuf::from(path)));
    } else {
        // Show empty state with theme awareness
        show_empty_state_with_theme(&webview, &settings_manager);
    }

    // ── Toolbar ───────────────────────────────────────────────────────────
    // Build the icon toolbar FIRST so we can pass open_editor_btn to the titlebar.
    let asset_root = polo_paths.asset_root();

    let toolbar_state = components::toolbar::create_polo_toolbar(
        &window,
        webview.clone(),
        settings_manager.clone(),
        current_file_path.clone(),
        asset_root,
        sidebar.clone(),
    );
    // Retain a clone before `create_custom_titlebar` below moves the
    // original — needed again for the drag-and-drop and file-tree wiring
    // further down.
    let open_editor_btn_for_drop = toolbar_state.open_editor_btn.clone();

    // ── Titlebar (text menu bar) ──────────────────────────────────────────
    let (titlebar_handle, title_label) = create_custom_titlebar(
        &window,
        filename.as_deref().unwrap_or("Untitled"),
        &saved_theme,
        settings_manager.clone(),
        webview.clone(),
        current_file_path.clone(),
        asset_root,
        Some(toc_handle.clone()),
        Some(file_tree_handle.clone()),
        sidebar.clone(),
        toolbar_state.open_editor_btn,
        toolbar_state.open_find_bar,
    );
    window.set_titlebar(Some(&titlebar_handle));
    // Fill the late-bound slot now that the real title Label exists, so the
    // toolbar's Open button and the link-click-open callback (wired above,
    // before the titlebar existed) both start updating the visible header.
    *title_label_for_header.borrow_mut() = Some(title_label.clone());

    // ── Drag and drop: open a markdown file dropped onto the preview ──────
    // Wired directly on the webview via wry's native `DragDropEvent` support
    // (`PlatformWebView::set_file_drop_handler`/`set_drag_hover_handler`),
    // not a GTK-level `DropTarget` on the window — the webview is a native
    // widget on Linux and a child HWND on Windows, either of which could
    // otherwise swallow window-level drop events before a `DropTarget` ever
    // saw them. wry's own handler works identically on both backends.
    {
        let webview_for_hover = webview.clone();
        webview.set_drag_hover_handler(move |active| {
            // Both the empty state and a rendered document carry the same
            // shared drop overlay (see `viewer::drop_overlay`), so this one
            // call drives the drag-and-drop visual regardless of which is
            // currently showing.
            webview_for_hover.set_dragging_state(active);
        });
    }
    {
        let window_for_drop = window.clone();
        let webview_for_drop = webview.clone();
        let settings_manager_for_drop = settings_manager.clone();
        let current_file_path_for_drop = current_file_path.clone();
        let title_label_for_drop = title_label.clone();
        let asset_root_for_drop = polo_paths.asset_root().to_path_buf();
        let toc_handle_for_drop = toc_handle.clone();
        let open_editor_btn_for_drop = open_editor_btn_for_drop.clone();
        let file_tree_handle_for_drop = file_tree_handle.clone();

        webview.set_file_drop_handler(move |paths| {
            // Accept the first markdown file among the dropped paths (a user
            // might drop a folder + a file together, or several files at
            // once) — everything else in the drop is silently ignored, same
            // as `connect_open`'s existing `files.first()` handling for
            // file-manager "Open With" drops.
            let Some(path) = paths.into_iter().find(|p| {
                p.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                    e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown")
                })
            }) else {
                log::info!("[polo] Drag-and-drop: no markdown file among dropped paths; ignoring");
                return;
            };

            let toc_for_drop = toc_handle_for_drop.clone();
            let on_file_opened = move |path_str: &str| {
                if let Ok(text) =
                    marco_shared::cache::cached::read_to_string(std::path::Path::new(path_str))
                {
                    toc_for_drop.update_from_text_async(text);
                }
            };

            components::dialog::open_file_and_update_state(
                components::dialog::OpenFileContext {
                    window: &window_for_drop,
                    webview: &webview_for_drop,
                    settings_manager: &settings_manager_for_drop,
                    current_file_path: &current_file_path_for_drop,
                    open_editor_btn: &open_editor_btn_for_drop,
                    title_label: &title_label_for_drop,
                    asset_root: &asset_root_for_drop,
                    file_tree_handle: Some(&file_tree_handle_for_drop),
                },
                path,
                Some(&on_file_opened),
            );
        });
    }

    // ── Empty-state "Open File" button: the primary way to open a file from
    // the welcome screen (the toolbar's own "Open file" button was removed —
    // see `toolbar.rs`). The button lives inside the webview's rendered HTML
    // (`viewer::empty_state`), so it can't call into Rust directly; it posts
    // a `polo_open_file:` IPC message instead, which triggers the exact same
    // file-picker dialog every other "Open" entry point uses.
    {
        let window_for_empty_open = window.clone();
        let webview_for_empty_open = webview.clone();
        let settings_manager_for_empty_open = settings_manager.clone();
        let current_file_path_for_empty_open = current_file_path.clone();
        let title_label_for_empty_open = title_label.clone();
        let asset_root_for_empty_open = polo_paths.asset_root().to_path_buf();
        let toc_handle_for_empty_open = toc_handle.clone();
        let open_editor_btn_for_empty_open = open_editor_btn_for_drop.clone();
        let file_tree_handle_for_empty_open = file_tree_handle.clone();

        webview.set_open_file_click_handler(move || {
            let toc_for_open = toc_handle_for_empty_open.clone();
            let on_file_opened: Rc<dyn Fn(&str)> = Rc::new(move |path_str: &str| {
                if let Ok(text) =
                    marco_shared::cache::cached::read_to_string(std::path::Path::new(path_str))
                {
                    toc_for_open.update_from_text_async(text);
                }
            });

            components::dialog::show_open_file_dialog(
                &window_for_empty_open,
                webview_for_empty_open.clone(),
                settings_manager_for_empty_open.clone(),
                current_file_path_for_empty_open.clone(),
                &open_editor_btn_for_empty_open,
                &title_label_for_empty_open,
                &asset_root_for_empty_open,
                Some(on_file_opened),
                Some(file_tree_handle_for_empty_open.clone()),
            );
        });
    }

    // ── File-tree panel: open a clicked file through the shared pipeline ──
    // Fills the late-bound callback slot (see `FileTreePanelHandle`'s doc
    // comment) now that `open_editor_btn`/`title_label` exist. The panel
    // itself, and the sidebar coordinator built on it, already exist and
    // are wired into the toolbar/titlebar above — only the callback needed
    // to wait this long.
    {
        let window_for_tree = window.clone();
        let webview_for_tree = webview.clone();
        let settings_manager_for_tree = settings_manager.clone();
        let current_file_path_for_tree = current_file_path.clone();
        let title_label_for_tree = title_label.clone();
        let asset_root_for_tree = polo_paths.asset_root().to_path_buf();
        let toc_handle_for_tree = toc_handle.clone();
        let open_editor_btn_for_tree = open_editor_btn_for_drop.clone();
        let file_tree_handle_for_tree = file_tree_handle.clone();

        file_tree_handle.set_open_file_callback(Rc::new(move |path: &std::path::Path| {
            let toc_for_open = toc_handle_for_tree.clone();
            let on_file_opened = move |path_str: &str| {
                if let Ok(text) =
                    marco_shared::cache::cached::read_to_string(std::path::Path::new(path_str))
                {
                    toc_for_open.update_from_text_async(text);
                }
            };

            components::dialog::open_file_and_update_state(
                components::dialog::OpenFileContext {
                    window: &window_for_tree,
                    webview: &webview_for_tree,
                    settings_manager: &settings_manager_for_tree,
                    current_file_path: &current_file_path_for_tree,
                    open_editor_btn: &open_editor_btn_for_tree,
                    title_label: &title_label_for_tree,
                    asset_root: &asset_root_for_tree,
                    file_tree_handle: Some(&file_tree_handle_for_tree),
                },
                path.to_path_buf(),
                Some(&on_file_opened),
            );
        }));
    }

    // ── Main content layout ───────────────────────────────────────────────
    // Vertical box: toolbar (top) + paned content (fill)
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    main_box.append(&toolbar_state.toolbar);
    main_box.append(&files_paned);

    window.set_child(Some(&main_box));

    // Save window size changes to Polo-specific settings
    let settings_manager_width = settings_manager.clone();
    window.connect_default_width_notify(move |w| {
        let width = w.default_width() as u32;
        let height = w.default_height() as u32;

        let _ = settings_manager_width.update_settings(|s| {
            // Ensure polo section exists
            if s.polo.is_none() {
                s.polo = Some(marco_shared::logic::swanson::PoloSettings::default());
            }
            // Ensure polo.window exists
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(marco_shared::logic::swanson::PoloWindowSettings::default());
                }
                if let Some(ref mut win) = polo.window {
                    win.width = Some(width);
                    win.height = Some(height);
                }
            }
        });
        log::debug!("Saved Polo window width: {}", width);
    });

    let settings_manager_height = settings_manager.clone();
    window.connect_default_height_notify(move |w| {
        let width = w.default_width() as u32;
        let height = w.default_height() as u32;

        let _ = settings_manager_height.update_settings(|s| {
            if s.polo.is_none() {
                s.polo = Some(marco_shared::logic::swanson::PoloSettings::default());
            }
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(marco_shared::logic::swanson::PoloWindowSettings::default());
                }
                if let Some(ref mut win) = polo.window {
                    win.width = Some(width);
                    win.height = Some(height);
                }
            }
        });
        log::debug!("Saved Polo window height: {}", height);
    });

    // Save maximized state
    let settings_manager_max = settings_manager.clone();
    window.connect_maximized_notify(move |w| {
        let is_maximized = w.is_maximized();

        let _ = settings_manager_max.update_settings(|s| {
            if s.polo.is_none() {
                s.polo = Some(marco_shared::logic::swanson::PoloSettings::default());
            }
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(marco_shared::logic::swanson::PoloWindowSettings::default());
                }
                if let Some(ref mut win) = polo.window {
                    win.maximized = Some(is_maximized);
                }
            }
        });
        log::debug!("Saved Polo maximized state: {}", is_maximized);
    });

    // Apply saved maximized state
    if let Some(polo) = &settings.polo {
        if let Some(polo_window) = &polo.window {
            if polo_window.is_maximized() {
                window.maximize();
            }
        }
    }

    // Present window
    window.present();
}
