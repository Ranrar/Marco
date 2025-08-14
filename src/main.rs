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
use crate::ui::main_editor::create_editor_with_preview;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::theme::ThemeManager;

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
    // Load flat button and window control CSS
    use gtk4::{CssProvider, gdk::Display};
    let provider = CssProvider::new();
    gtk4::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
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
    let (footer, footer_labels) = footer::create_footer();

    let (split, _webview, preview_css_rc, refresh_preview, update_editor_theme, update_preview_theme) = create_editor_with_preview(
        preview_theme_filename.as_str(),
        preview_theme_dir_str.as_str(),
        theme_manager.clone(),
        Rc::clone(&theme_mode)
    );

    // Wire up live footer updates using buffer and view from editor
    use crate::ui::main_editor::{wire_footer_updates, render_editor_with_view};
    // Recreate editor to get buffer and view
    let (editor_widget, buffer, source_view) = render_editor_with_view(
        theme_manager.borrow().current_editor_scheme().as_ref(),
        &theme_manager.borrow().settings.appearance.as_ref().and_then(|a| a.ui_font.as_deref()).unwrap_or("Fira Mono"),
        theme_manager.borrow().settings.appearance.as_ref().and_then(|a| a.ui_font_size).map(|v| v as f64).unwrap_or(14.0)
    );
    wire_footer_updates(&buffer, &source_view, std::rc::Rc::new(footer_labels.clone()));
    split.add_css_class("split-view");

    // Add components to main layout (menu bar is now in titlebar)
    main_box.append(&toolbar);
    main_box.append(&split);
    main_box.append(&footer);

    // Set editor area to expand
    split.set_vexpand(true);
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
                );
            }
        });
    app.add_action(&settings_action);

    // Present the window
    window.present();
}