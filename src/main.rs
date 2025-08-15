use webkit6::prelude::*;
mod logic;
// Stripped-down UI structure modules

mod footer;
mod menu;
mod settings {
    // pub use crate::ui::settings::*; // unused import removed
}
mod theme;
mod toolbar;
pub mod ui;


use gtk4::{glib, Application, ApplicationWindow, Box as GtkBox, Orientation};
use sourceview5::Buffer;
use crate::ui::main_editor::{create_editor_with_preview, wire_footer_updates};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::theme::ThemeManager;
use crate::logic::parser::MarkdownSyntaxMap;

const APP_ID: &str = "com.example.Marco";

fn main() -> glib::ExitCode {
    // Try to use Vulkan renderer first
    std::env::set_var("GSK_RENDERER", "vulkan");
    eprintln!("[DEBUG] Trying GSK_RENDERER=vulkan");

    // Initialize GTK4 application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| build_ui(app));

    let no_args: [&str; 0] = [];
    let exit_code = app.run_with_args(&no_args);

    // If Vulkan failed, fallback to OpenGL and restart
    if exit_code != 0.into() {
        eprintln!("[DEBUG] Vulkan renderer failed, retrying with GSK_RENDERER=gl");
        std::env::set_var("GSK_RENDERER", "gl");
        let app = Application::builder().application_id(APP_ID).build();
        app.connect_activate(|app| build_ui(app));
        let gl_exit_code = app.run_with_args(&no_args);
    eprintln!("[DEBUG] OpenGL renderer exit code: {:?}", gl_exit_code);
    gl_exit_code.into()
    } else {
        eprintln!("[DEBUG] Vulkan renderer succeeded");
        exit_code
    }
}

fn build_ui(app: &Application) {
    // --- Load settings from settings.ron ---
    use crate::logic::swanson::Settings;
    let config_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let settings_path = config_dir.join("src/assets/settings.ron");
    let settings = Settings::load_from_file(settings_path.to_str().unwrap())
        .unwrap_or_default();
    // Load toolbar CSS from external file
    toolbar::load_toolbar_css_from_file();
    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Marco")
        .default_width(1200)
        .default_height(800)
        .build();
    window.add_css_class("main-window");

    // --- Custom VS Code–like draggable titlebar from menu.rs ---
    let titlebar = menu::create_custom_titlebar(&window);
    window.set_titlebar(Some(&titlebar));

    // --- ThemeManager and settings.ron path ---
    let config_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let settings_path = config_dir.join("src/assets/settings.ron");
    let dev_ui_theme_dir = config_dir.join("src/assets/themes/gtk4");
    let prod_ui_theme_dir = config_dir.join("themes/ui");
    let ui_theme_dir = if dev_ui_theme_dir.exists() {
        dev_ui_theme_dir
    } else {
        prod_ui_theme_dir
    };

    // Use src/assets/themes/html_viever for preview themes in dev, /themes/ in prod
    let dev_preview_theme_dir = config_dir.join("src/assets/themes/html_viever");
    let prod_preview_theme_dir = config_dir.join("themes");
    let preview_theme_dir = if dev_preview_theme_dir.exists() {
        dev_preview_theme_dir
    } else {
        prod_preview_theme_dir
    };

    // Use src/assets/themes/editor for editor style schemes in dev, /themes/editor in prod
    let dev_editor_theme_dir = config_dir.join("src/assets/themes/editor");
    let prod_editor_theme_dir = config_dir.join("themes/editor");
    let editor_theme_dir = if dev_editor_theme_dir.exists() {
        dev_editor_theme_dir
    } else {
        prod_editor_theme_dir
    };

    let theme_manager = Rc::new(RefCell::new(ThemeManager::new(
        &settings_path,
        ui_theme_dir,
        preview_theme_dir.clone(),
        editor_theme_dir,
    )));
    // Pass settings struct to modules as needed

    // Create main vertical box layout
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.add_css_class("main-container");

    // Create basic UI components (structure only)
    let toolbar = toolbar::create_toolbar_structure();
    toolbar.add_css_class("toolbar");
    toolbar::set_toolbar_height(&toolbar, 0); // Minimum height, matches footer
    // --- Determine correct HTML preview theme based on settings and app theme ---
    use crate::logic::theme_loader::list_html_view_themes;
    let preview_theme_dir_str = preview_theme_dir.clone().to_string_lossy().to_string();
    let html_themes = list_html_view_themes(&preview_theme_dir.clone());
    let settings = &theme_manager.borrow().settings;
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
        theme_manager.borrow().preview_theme_mode_from_scheme(&current_scheme)
    };
    let theme_mode = Rc::new(RefCell::new(initial_theme_mode));
    let (footer, footer_labels_rc) = footer::create_footer();

    // Load active markdown schema from settings (if available)
    let schema_root = config_dir.join("src/assets/markdown_schema");
    let active_schema_map: Rc<RefCell<Option<MarkdownSyntaxMap>>> = Rc::new(RefCell::new(None));
    if let Ok(Some(map)) = MarkdownSyntaxMap::load_active_schema(settings_path.to_str().unwrap(), schema_root.to_str().unwrap()) {
        *active_schema_map.borrow_mut() = Some(map);
    }

    let (split, _webview, preview_css_rc, refresh_preview, update_editor_theme, update_preview_theme, editor_buffer, insert_mode_state) = create_editor_with_preview(
        preview_theme_filename.as_str(),
        preview_theme_dir_str.as_str(),
        theme_manager.clone(),
        Rc::clone(&theme_mode),
        footer_labels_rc.clone()
    );

    // Wire up live footer updates using the actual editor buffer
    // Wire footer updates directly: wire_footer_updates will run callbacks on
    // the main loop and call `apply_footer_update` directly.
    wire_footer_updates(&editor_buffer, footer_labels_rc.clone(), active_schema_map.clone(), insert_mode_state.clone());
    split.add_css_class("split-view");

    // Closure to trigger an immediate footer syntax update using the active schema map
    let trigger_footer_update: std::rc::Rc<dyn Fn()> = std::rc::Rc::new({
        let buffer = editor_buffer.clone();
        let labels = footer_labels_rc.clone();
        let active_schema_map = active_schema_map.clone();
        let test_counter = std::rc::Rc::new(std::cell::Cell::new(0));
        move || {
            eprintln!("[main] Manual footer trigger called!");
            
            // Increment test counter for obvious visual changes
            let count = test_counter.get() + 1;
            test_counter.set(count);
            
            // Update with test values to make changes obvious
            crate::footer::update_cursor_row(&labels, count + 10);
            crate::footer::update_cursor_col(&labels, count + 20);
            crate::footer::update_word_count(&labels, count * 10);
            crate::footer::update_char_count(&labels, count * 50);
            crate::footer::update_encoding(&labels, &format!("TEST-{}", count));
            crate::footer::update_insert_mode(&labels, count % 2 == 0);
            
            // Also do the original syntax trace logic
            let offset = buffer.cursor_position();
            let iter = buffer.iter_at_offset(offset);
            let current_line = iter.line();
            let start_iter_opt = buffer.iter_at_line(current_line);
            let end_iter_opt = buffer.iter_at_line(current_line + 1);
            let line_text = match (start_iter_opt, end_iter_opt) {
                (Some(ref start), Some(ref end)) => buffer.text(start, end, false).to_string(),
                (Some(ref start), None) => buffer.text(start, &buffer.end_iter(), false).to_string(),
                _ => String::new(),
            };
            if let Some(ref map) = *active_schema_map.borrow() {
                crate::footer::update_syntax_trace(&labels, &line_text, map);
            } else {
                let dummy_map = crate::logic::parser::MarkdownSyntaxMap { rules: std::collections::HashMap::new() };
                crate::footer::update_syntax_trace(&labels, &line_text, &dummy_map);
            }
        }
    });

    // test footer update button removed

    // Add components to main layout (menu bar is now in titlebar)
    main_box.append(&toolbar);
    main_box.append(&split);
    main_box.append(&footer);

    // Set editor area to expand
    split.set_vexpand(true);
    
    // Ensure footer is visible and properly positioned
    footer.set_vexpand(false); // Footer should not expand vertically
    footer.set_hexpand(true);  // Footer should expand horizontally
    footer.set_visible(true);  // Explicitly ensure footer is visible
    
    // Debug output to confirm footer creation
    eprintln!("[main] Footer created and added to layout");
    eprintln!("[main] Footer visible: {}", footer.is_visible());
    eprintln!("[main] Footer height request: {}", footer.height_request());
    
    // Optionally, assign classes to editor/preview if accessible here

    // Add main box to window
    window.set_child(Some(&main_box));

    // --- Live HTML preview theme switching ---
    // Store refresh_preview closure for use on theme changes
    let refresh_preview_rc = Rc::new(RefCell::new(refresh_preview));
    let preview_css_for_settings = preview_css_rc.clone();
    // Register 'app.settings' action to show the settings dialog with the callback
    let settings_action = gtk4::gio::SimpleAction::new("settings", None);
    let win_clone = window.clone();
    let theme_manager_clone = theme_manager.clone();
    let settings_path_clone = settings_path.clone();
    let refresh_preview_for_settings2 = refresh_preview_rc.clone();
    let update_editor_theme_clone = Rc::new(update_editor_theme);
    let update_preview_theme_clone = Rc::new(update_preview_theme);
        settings_action.connect_activate({
            let win_clone = win_clone.clone();
            let theme_manager_clone = theme_manager_clone.clone();
            let settings_path_clone = settings_path_clone.clone();
            let preview_css_for_settings = preview_css_for_settings.clone();
            let refresh_preview_for_settings2 = refresh_preview_for_settings2.clone();
            let update_editor_theme_clone = update_editor_theme_clone.clone();
            let update_preview_theme_clone = update_preview_theme_clone.clone();
            move |_, _| {
                use crate::ui::settings::settings::show_settings_dialog;
                
                // Create editor theme callback that updates both editor and preview
                let editor_callback = {
                    let update_editor = update_editor_theme_clone.clone();
                    let update_preview = update_preview_theme_clone.clone();
                    Box::new(move |scheme_id: String| {
                        update_editor(&scheme_id);
                        update_preview(&scheme_id);
                    }) as Box<dyn Fn(String) + 'static>
                };
                
                show_settings_dialog(
                    win_clone.upcast_ref(),
                    theme_manager_clone.clone(),
                    settings_path_clone.clone(),
                    Some(Box::new({
                        let theme_manager_clone = theme_manager_clone.clone();
                        let preview_css_for_settings = preview_css_for_settings.clone();
                        let refresh_preview_for_settings2 = refresh_preview_for_settings2.clone();
                        move |theme_filename: String| {
                            // On preview theme change, update CSS and call refresh
                            use std::fs;
                            let theme_manager = theme_manager_clone.borrow();
                            let preview_theme_dir = theme_manager.preview_theme_dir.clone();
                            let css_path = preview_theme_dir.join(&theme_filename);
                            let css = fs::read_to_string(&css_path).unwrap_or_default();
                            *preview_css_for_settings.borrow_mut() = css;
                            (refresh_preview_for_settings2.borrow())();
                        }
                    })),
                    Some(refresh_preview_for_settings2.clone()),
                    Some(editor_callback),
                    Some(Box::new({
                        let active_schema_map = active_schema_map.clone();
                        let config_dir = config_dir.clone();
                        let settings_path_clone = settings_path_clone.clone();
                        let trigger = trigger_footer_update.clone();
                        move |_selected: Option<String>| {
                            // Reload parser and update shared map
                            let schema_root = config_dir.join("src/assets/markdown_schema");
                            if let Ok(Some(map)) = crate::logic::parser::MarkdownSyntaxMap::load_active_schema(
                                settings_path_clone.to_str().unwrap(),
                                schema_root.to_str().unwrap(),
                            ) {
                                *active_schema_map.borrow_mut() = Some(map);
                            } else {
                                *active_schema_map.borrow_mut() = None;
                            }
                            // Trigger immediate footer update
                            (trigger)();
                        }
                    }) as Box<dyn Fn(Option<String>) + 'static>),
                );
            }
        });
    app.add_action(&settings_action);

    // Present the window
    window.present();
}