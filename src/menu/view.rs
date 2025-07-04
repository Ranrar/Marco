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
    let current_settings = settings::get_current_settings();
    
    // Add view mode submenu
    let view_mode_menu = gio::Menu::new();
    
    // Create view mode menu items with checkmarks based on settings
    let html_label = if current_settings.view_mode == "html" {
        "HTML\t✓"
    } else {
        "HTML"
    };
    view_mode_menu.append(Some(html_label), Some("app.view_html"));
    
    let code_label = if current_settings.view_mode == "code" {
        "HTML Code\t✓"
    } else {
        "HTML Code"
    };
    view_mode_menu.append(Some(code_label), Some("app.view_code"));
    
    view_menu.append_submenu(Some("Preview Mode"), &view_mode_menu);
    
    // Add CSS theme submenu for preview styling (dynamic from css/ directory)
    let css_theme_menu = gio::Menu::new();
    
    // Get available CSS themes dynamically
    let available_themes = editor::MarkdownEditor::get_available_css_themes();
    for (theme_name, display_name, sanitized_name) in available_themes {
        let _icon = match theme_name.as_str() {
            "standard" => "✓ ",
            "github" => "🖤 ",
            "minimal" => "📄 ",
            "academic" => "🎓 ",
            _ => "🎨 ",
        };
        
        // Create menu item with checkmark based on settings
        let display_label = if current_settings.css_theme == theme_name {
            format!("{}\t✓", display_name)
        } else {
            display_name.clone()
        };
        css_theme_menu.append(Some(&display_label), Some(&format!("app.css_theme_{}", sanitized_name)));
    }
    
    view_menu.append_submenu(Some("CSS Style"), &css_theme_menu);
    
    // Add theme submenu (for UI theme)
    let theme_menu = gio::Menu::new();
    
    // Create theme menu items with checkmarks based on settings
    let system_label = if current_settings.ui_theme == "system" {
        "System\t✓"
    } else {
        "System"
    };
    theme_menu.append(Some(system_label), Some("app.theme_system"));
    
    let light_label = if current_settings.ui_theme == "light" {
        "Light\t✓"
    } else {
        "Light"
    };
    theme_menu.append(Some(light_label), Some("app.theme_light"));
    
    let dark_label = if current_settings.ui_theme == "dark" {
        "Dark\t✓"
    } else {
        "Dark"
    };
    theme_menu.append(Some(dark_label), Some("app.theme_dark"));
    
    view_menu.append_submenu(Some("Theme"), &theme_menu);
    
    let language_menu = gio::Menu::new();
    
    for (code, name) in language::get_available_locales() {
        let lang_label = if current_settings.language == code {
            format!("{}\t✓", name)
        } else {
            name.to_string()
        };
        language_menu.append(Some(&lang_label), Some(&format!("app.set_language_{}", code)));
    }
    view_menu.append_submenu(Some(&language::tr("menu.language")), &language_menu);
    
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
                settings::update_view_mode("html");
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
                settings::update_view_mode("code");
                super::rebuild_menu_bar(app, &editor, &theme_mgr);
            }
        })
        .build();
    
    // Theme switching actions
    let theme_system_action = gio::ActionEntry::builder("theme_system")
        .activate({
            let theme_mgr = theme_manager.clone();
            let editor_clone = editor.clone();
            move |app: &Application, _action, _param| {
                theme_mgr.set_theme(crate::theme::Theme::System);
                editor_clone.refresh_html_view();
                settings::update_ui_theme("system");
                super::rebuild_menu_bar(app, &editor_clone, &theme_mgr);
                println!("Switched to system theme");
            }
        })
        .build();
        
    let theme_light_action = gio::ActionEntry::builder("theme_light")
        .activate({
            let theme_mgr = theme_manager.clone();
            let editor_clone = editor.clone();
            move |app: &Application, _action, _param| {
                theme_mgr.set_theme(crate::theme::Theme::Light);
                editor_clone.refresh_html_view();
                settings::update_ui_theme("light");
                super::rebuild_menu_bar(app, &editor_clone, &theme_mgr);
                println!("Switched to light theme");
            }
        })
        .build();
        
    let theme_dark_action = gio::ActionEntry::builder("theme_dark")
        .activate({
            let theme_mgr = theme_manager.clone();
            let editor_clone = editor.clone();
            move |app: &Application, _action, _param| {
                theme_mgr.set_theme(crate::theme::Theme::Dark);
                editor_clone.refresh_html_view();
                settings::update_ui_theme("dark");
                super::rebuild_menu_bar(app, &editor_clone, &theme_mgr);
                println!("Switched to dark theme");
            }
        })
        .build();

    // CSS Theme switching actions for preview (dynamic from css/ directory)
    let available_themes = editor::MarkdownEditor::get_available_css_themes();
    let mut css_theme_actions = Vec::new();
    
    for (theme_name, _display_name, sanitized_name) in available_themes {
        let action_name = format!("css_theme_{}", sanitized_name);
        let action = gio::ActionEntry::builder(&action_name)
            .activate({
                let editor = editor.clone();
                let theme_mgr = theme_manager.clone();
                let theme = theme_name.clone();
                move |app: &Application, _action, _param| {
                    editor.set_css_theme(&theme);
                    settings::update_css_theme(&theme);
                    super::rebuild_menu_bar(app, &editor, &theme_mgr);
                    println!("✓ Set CSS theme to {}", theme);
                }
            })
            .build();
        css_theme_actions.push(action);
    }

    // Language switching actions
    let available_locales = language::get_available_locales();
    let mut language_actions = Vec::new();
    
    for (code, _name) in available_locales {
        let action_name = format!("set_language_{}", code);
        let action = gio::ActionEntry::builder(&action_name)
            .activate({
                let editor = editor.clone();
                let theme_mgr = theme_manager.clone();
                let lang_code = code;
                move |app: &Application, _action, _param| {
                    language::set_locale(&lang_code);
                    settings::update_language(&lang_code);
                    super::rebuild_menu_bar(app, &editor, &theme_mgr);
                    println!("Language changed to {}", lang_code);
                }
            })
            .build();
        language_actions.push(action);
    }

    // Add all view actions to the application
    let mut all_actions = vec![view_html_action, view_code_action, theme_system_action, theme_light_action, theme_dark_action];
    all_actions.extend(css_theme_actions);
    all_actions.extend(language_actions);
    
    app.add_action_entries(all_actions);
}
