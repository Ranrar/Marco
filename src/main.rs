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


use gtk4::prelude::*;
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
        preview_theme_dir,
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
        );
    });
    app.add_action(&settings_action);

    // Create main vertical box layout
    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Create basic UI components (structure only)
    let toolbar = toolbar::create_toolbar_structure();
    let (split, webview) = create_editor_with_preview();
    let footer = footer::create_footer_structure();

    // Add components to main layout (menu bar is now in titlebar)
    main_box.append(&toolbar);
    main_box.append(&split);
    main_box.append(&footer);

    // Set editor area to expand
    split.set_vexpand(true);

    // Add main box to window
    window.set_child(Some(&main_box));

    // --- Live HTML preview theme switching ---
    let webview_rc = Rc::new(webview);
    let theme_manager_for_settings = theme_manager.clone();
    let settings_path_for_settings = settings_path.clone();
    let webview_for_settings = webview_rc.clone();
    // Register 'app.settings' action to show the settings dialog with the callback
    let settings_action = gtk4::gio::SimpleAction::new("settings", None);
    let win_clone = window.clone();
    let theme_manager_clone = theme_manager.clone();
    let settings_path_clone = settings_path.clone();
    settings_action.connect_activate(move |_, _| {
        use std::fs;
        use crate::ui::settings::settings::show_settings_dialog;
        let webview = webview_for_settings.clone();
        let preview_theme_dir = theme_manager_clone.borrow().preview_theme_dir.clone();
        show_settings_dialog(
            win_clone.upcast_ref(),
            theme_manager_clone.clone(),
            settings_path_clone.clone(),
            Some(Box::new(move |theme_filename: String| {
                let css_path = preview_theme_dir.join(&theme_filename);
                let css = fs::read_to_string(&css_path).unwrap_or_default();
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset=\"utf-8\">
    <style>{}</style>
  </head>
  <body></body>
</html>"#,
                    css
                );
                webview.as_ref().load_html(&html, None);
            })),
        );
    });
    app.add_action(&settings_action);

    // Present the window
    window.present();
}