use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

// Import unified helper
use super::helpers::add_setting_row;

// Import your theme manager
use crate::logic::signal_manager::SignalManager;
use core::logic::loaders::theme_loader::list_html_view_themes;

pub fn build_appearance_tab(
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    settings_path: PathBuf,
    asset_dir: &std::path::Path,
    on_preview_theme_changed: Box<dyn Fn(String) + 'static>,
    refresh_preview: Rc<RefCell<Box<dyn Fn()>>>,
    on_editor_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
) -> (gtk4::Box, Rc<RefCell<SignalManager>>) {
    use gtk4::{
        Adjustment, Box as GtkBox, Button, DropDown, Expression, Orientation, PropertyExpression,
        SpinButton, StringList, StringObject,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Create signal manager to track all signal handlers for proper cleanup
    let signal_manager = Rc::new(RefCell::new(SignalManager::new()));

    // Prepare data for HTML preview theme dropdown
    let on_preview_theme_changed = Rc::new(on_preview_theme_changed);
    let user_selected_preview_theme = Rc::new(std::cell::Cell::new(false));
    let html_theme_dir = asset_dir.join("themes/html_viever");
    let html_themes = list_html_view_themes(&html_theme_dir);

    // === ROW 1: HTML Preview Theme ===
    // Extract theme labels for the dropdown
    let theme_labels: Vec<&str> = html_themes
        .iter()
        .map(|entry| entry.label.as_str())
        .collect();
    let theme_string_list = StringList::new(&theme_labels);
    let theme_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");
    let preview_theme_combo = DropDown::new(Some(theme_string_list), Some(theme_expression));
    preview_theme_combo.add_css_class("marco-dropdown");

    // Set current theme
    let current_preview = theme_manager
        .borrow()
        .get_settings()
        .appearance
        .as_ref()
        .and_then(|a| a.preview_theme.clone());
    let current_preview_str = current_preview.as_deref().unwrap_or("standard.css");
    let preview_active_idx = html_themes
        .iter()
        .position(|t| t.filename == current_preview_str)
        .unwrap_or(0);
    preview_theme_combo.set_selected(preview_active_idx as u32);

    // Connect signal
    {
        let theme_manager_clone = Rc::clone(&theme_manager);
        let settings_path_clone = settings_path.clone();
        let html_themes_clone = html_themes.clone();
        let on_preview_theme_changed_clone = Rc::clone(&on_preview_theme_changed);
        let user_selected_preview_theme_clone = Rc::clone(&user_selected_preview_theme);
        let signal_manager_clone = signal_manager.clone();

        let handler_id = preview_theme_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            if let Some(theme_entry) = html_themes_clone.get(idx) {
                user_selected_preview_theme_clone.set(true);
                log::info!(
                    "Saving preview_theme: {} to {:?}",
                    theme_entry.filename,
                    settings_path_clone
                );
                theme_manager_clone
                    .borrow_mut()
                    .set_preview_theme(theme_entry.filename.clone(), &settings_path_clone);
                (on_preview_theme_changed_clone)(theme_entry.filename.clone());
            }
        });

        signal_manager_clone.borrow_mut().register_handler(
            "appearance_tab",
            &preview_theme_combo.clone().upcast(),
            handler_id,
        );
    }

    let preview_theme_row = add_setting_row(
        "HTML Preview Theme",
        "Select a CSS theme for rendered Markdown preview.",
        &preview_theme_combo,
        true, // FIRST row - no top margin
    );
    container.append(&preview_theme_row);

    // === ROW 2: Light/Dark Mode ===
    let app_settings = theme_manager.borrow().get_settings();
    let current_mode = app_settings
        .appearance
        .as_ref()
        .and_then(|a| a.editor_mode.clone())
        .unwrap_or("marco-light".to_string());

    let color_mode_options = ["light", "dark"];
    let color_mode_string_list = StringList::new(&color_mode_options);
    let color_mode_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");
    let color_mode_combo = DropDown::new(Some(color_mode_string_list), Some(color_mode_expression));
    color_mode_combo.add_css_class("marco-dropdown");

    let active_idx = match current_mode.as_str() {
        "marco-dark" | "dark" => 1,
        _ => 0,
    };
    color_mode_combo.set_selected(active_idx);

    // Connect signal
    {
        let theme_manager_clone = theme_manager.clone();
        let settings_path_clone = settings_path.clone();
        let refresh_preview_clone = Rc::clone(&refresh_preview);
        let on_editor_theme_changed_clone = on_editor_theme_changed.map(Rc::new);
        let signal_manager_clone = signal_manager.clone();

        let handler_id = color_mode_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected();
            let scheme_id = if idx == 1 {
                "marco-dark"
            } else {
                "marco-light"
            };
            log::info!("Switching editor scheme to: {}", scheme_id);

            {
                let mut theme_mgr = theme_manager_clone.borrow_mut();
                theme_mgr.set_editor_scheme(scheme_id, &settings_path_clone);
            }

            if let Some(ref callback) = on_editor_theme_changed_clone {
                callback(scheme_id.to_string());
            }

            // Notify editor/theme changes; preview refresh will handle visuals.
            (refresh_preview_clone.borrow())();
        });

        signal_manager_clone.borrow_mut().register_handler(
            "appearance_tab",
            &color_mode_combo.clone().upcast(),
            handler_id,
        );
    }

    let color_mode_row = add_setting_row(
        "Light/Dark Mode",
        "Choose between light or dark user interface.",
        &color_mode_combo,
        false, // Not first row
    );
    container.append(&color_mode_row);

    // === ROW 3: Custom CSS for Preview ===
    let custom_css_button = Button::with_label("Open CSS Themes Folder");
    custom_css_button.add_css_class("marco-dialog-button");
    let custom_css_row = add_setting_row(
        "Custom CSS for Preview",
        "Add your own custom CSS to override the preview style.",
        &custom_css_button,
        false, // Not first row
    );
    container.append(&custom_css_row);

    // === ROW 4: UI Font ===
    let ui_font_options = ["System Default", "Sans", "Serif", "Monospace"];
    let ui_font_string_list = StringList::new(&ui_font_options);
    let ui_font_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");
    let ui_font_combo = DropDown::new(Some(ui_font_string_list), Some(ui_font_expression));
    ui_font_combo.add_css_class("marco-dropdown");
    ui_font_combo.set_selected(0);

    let ui_font_row = add_setting_row(
        "UI Font",
        "Customize the font used in the application's user interface (menus, sidebars).",
        &ui_font_combo,
        false, // Not first row
    );
    container.append(&ui_font_row);

    // === ROW 5: UI Font Size ===
    let ui_font_size_adj = Adjustment::new(14.0, 10.0, 24.0, 1.0, 0.0, 0.0);
    let ui_font_size_spin = SpinButton::new(Some(&ui_font_size_adj), 1.0, 0);
    ui_font_size_spin.add_css_class("marco-spinbutton");

    let ui_font_size_row = add_setting_row(
        "UI Font Size",
        "Customize the font size used in the application's user interface (menus, sidebars).",
        &ui_font_size_spin,
        false, // Not first row
    );
    container.append(&ui_font_size_row);

    (container, signal_manager)
}
