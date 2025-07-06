use gtk4::prelude::*;
use gtk4::{Application, gio};
use crate::{editor, language, settings};

pub fn add_view_menu(menu_model: &gio::Menu, editor: &editor::MarkdownEditor, theme_manager: &crate::theme::ThemeManager) {
    let view_menu = create_view_menu(editor, theme_manager);
    menu_model.append_submenu(Some(&language::tr("menu.view")), &view_menu);
}

pub fn create_view_menu(_editor: &editor::MarkdownEditor, _theme_manager: &crate::theme::ThemeManager) -> gio::Menu {
    let view_menu = gio::Menu::new();
    
    // Get current settings
    let prefs = settings::get_app_preferences();
    let current_view_mode = prefs.get_view_mode();
    
    // Add view mode submenu
    let view_mode_menu = gio::Menu::new();
    
    // Create view mode menu items with checkmarks based on settings
    let html_label = if current_view_mode == "html" {
        "HTML\t✓"
    } else {
        "HTML"
    };
    view_mode_menu.append(Some(html_label), Some("app.view_html"));
    
    let code_label = if current_view_mode == "code" {
        "HTML Code\t✓"
    } else {
        "HTML Code"
    };
    view_mode_menu.append(Some(code_label), Some("app.view_code"));
    
    view_menu.append_submenu(Some("Preview Mode"), &view_mode_menu);
    
    // Add settings separator and menu item
    view_menu.append_section(None, &{
        let settings_section = gio::Menu::new();
        settings_section.append(Some(&language::tr("menu.settings")), Some("app.settings"));
        settings_section
    });
    
    view_menu
}

pub fn create_view_actions(app: &Application, editor: &editor::MarkdownEditor, theme_manager: &crate::theme::ThemeManager) {
    // View mode actions
    let view_html_action = gio::ActionEntry::builder("view_html")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                editor.set_view_mode("html");
                let prefs = settings::get_app_preferences();
                prefs.set_view_mode("html");
                super::rebuild_menu_bar(app, &editor, &theme_mgr);
            }
        })
        .build();

    let view_code_action = gio::ActionEntry::builder("view_code")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                editor.set_view_mode("code");
                let prefs = settings::get_app_preferences();
                prefs.set_view_mode("code");
                super::rebuild_menu_bar(app, &editor, &theme_mgr);
            }
        })
        .build();
    
    // Settings action
    let settings_action = gio::ActionEntry::builder("settings")
        .activate({
            let app = app.clone();
            let editor = editor.clone();
            let theme_manager = theme_manager.clone();
            move |_app: &Application, _action, _param| {
                println!("DEBUG: Settings menu action triggered");
                if let Some(window) = app.active_window() {
                    crate::settings::dialog::show_settings_dialog(&window, &editor, &theme_manager);
                } else {
                    println!("DEBUG: No active window found for settings dialog");
                }
            }
        })
        .build();
    
    // Add only the view and settings actions to the application
    let all_actions = vec![view_html_action, view_code_action, settings_action];
    
    app.add_action_entries(all_actions);
}
