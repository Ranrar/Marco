use webkit6::prelude::*;
mod logic;
// Stripped-down UI structure modules

mod footer;
mod menu;
mod settings {
    pub use crate::ui::settings::*;
}
mod theme;
mod toolbar;
pub mod ui;


use gtk4::{glib, Application, ApplicationWindow, Box as GtkBox, Orientation};
use crate::ui::main_editor::create_editor_with_preview;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::theme::ThemeManager;

const APP_ID: &str = "com.example.Marco";

fn main() -> glib::ExitCode {
    // Initialize GTK4 application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| build_ui(app));

    let no_args: [&str; 0] = [];
    app.run_with_args(&no_args)
}

fn build_ui(app: &Application) {
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

    let theme_manager = Rc::new(RefCell::new(ThemeManager::new(
        &settings_path,
        ui_theme_dir,
        preview_theme_dir.clone(),
    )));

    // Register 'app.settings' action to show the settings dialog
    let settings_action = gtk4::gio::SimpleAction::new("settings", None);
    let win_clone = window.clone();
    let theme_manager_clone = theme_manager.clone();
    let settings_path_clone = settings_path.clone();
    settings_action.connect_activate(move |_, _| {
        settings::show_settings_dialog(
            win_clone.upcast_ref(),
            theme_manager_clone.clone(),
            settings_path_clone.clone(),
            None,
            None,
            None,
        );
    });
    app.add_action(&settings_action);

    // Create main vertical box layout
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.add_css_class("main-container");

    // Create basic UI components (structure only)
    let toolbar = toolbar::create_toolbar_structure();
    toolbar.add_css_class("toolbar");
    // --- Determine correct HTML preview theme based on settings and app theme ---
    use crate::logic::theme_list::{list_html_view_themes, find_theme_by_label};
    let preview_theme_dir_str = preview_theme_dir.clone().to_string_lossy().to_string();
    let html_themes = list_html_view_themes(&preview_theme_dir.clone());
    let settings = &theme_manager.borrow().settings;
    let mut preview_theme_filename = "standard.css".to_string();
    if let Some(appearance) = &settings.appearance {
        // Try to use the preview_theme from settings, else match app_theme's is_dark
        if let Some(ref preview_theme) = appearance.preview_theme {
            // Try to find by filename
            if html_themes.iter().any(|t| &t.filename == preview_theme) {
                preview_theme_filename = preview_theme.clone();
            } else {
                // Try to match app_theme's is_dark
                if let Some(ref app_theme) = appearance.app_theme {
                    if let Some(app_theme_entry) = find_theme_by_label(&html_themes, app_theme) {
                        let is_dark = app_theme_entry.is_dark;
                        if let Some(matching) = html_themes.iter().find(|t| t.is_dark == is_dark) {
                            preview_theme_filename = matching.filename.clone();
                        }
                    }
                }
            }
        }
    }
    // Initialize theme_mode before creating the editor/preview
    let theme_mode = Rc::new(RefCell::new(String::from("theme-light")));
    if let Some(appearance) = &settings.appearance {
        if let Some(ref app_theme) = appearance.app_theme {
            if app_theme.ends_with("-dark.css") {
                theme_mode.replace("theme-dark".to_string());
            }
        }
    }
    let (split, _webview, preview_css_rc, refresh_preview) = create_editor_with_preview(
        preview_theme_filename.as_str(),
        preview_theme_dir_str.as_str(),
        Rc::clone(&theme_mode)
    );
    split.add_css_class("split-view");
    let footer = footer::create_footer_structure();

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
    let refresh_preview_for_settings = refresh_preview_rc.clone();
    let theme_mode_for_settings = theme_mode.clone();
    let refresh_preview_for_settings2 = refresh_preview_rc.clone();
    settings_action.connect_activate(move |_, _| {
        use crate::ui::settings::settings::show_settings_dialog;
        let refresh_preview_for_settings = refresh_preview_for_settings.clone();
        let preview_css_for_settings = preview_css_for_settings.clone();
        let theme_mode_for_settings = theme_mode_for_settings.clone();
        let refresh_preview_for_settings2 = refresh_preview_for_settings2.clone();
        let theme_manager_clone = theme_manager_clone.clone();
        show_settings_dialog(
            win_clone.upcast_ref(),
            theme_manager_clone.clone(),
            settings_path_clone.clone(),
            Some(Box::new(move |theme_filename: String| {
                // On preview theme change, update CSS and call refresh
                use std::fs;
                let theme_manager = theme_manager_clone.borrow();
                let preview_theme_dir = theme_manager.preview_theme_dir.clone();
                let css_path = preview_theme_dir.join(&theme_filename);
                let css = fs::read_to_string(&css_path).unwrap_or_default();
                *preview_css_for_settings.borrow_mut() = css;
                (refresh_preview_for_settings.borrow())();
            })),
            Some(theme_mode_for_settings),
            Some(refresh_preview_for_settings2),
        );
    });
    app.add_action(&settings_action);

    // Present the window
    window.present();
}