use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

// Import unified helper
use super::helpers::{add_setting_row_i18n, SettingsI18nRegistry};

// Import your theme manager
use crate::components::language::SettingsAppearanceTranslations;
use crate::components::language::Translations;
use crate::logic::signal_manager::SignalManager;
use core::logic::loaders::theme_loader::list_html_view_themes;

pub struct AppearanceTabCallbacks {
    pub on_preview_theme_changed: Box<dyn Fn(String) + 'static>,
    pub refresh_preview: Rc<RefCell<Box<dyn Fn()>>>,
    pub on_editor_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
}

pub fn build_appearance_tab(
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    settings_path: PathBuf,
    asset_dir: &std::path::Path,
    callbacks: AppearanceTabCallbacks,
    translations: &SettingsAppearanceTranslations,
    i18n: &SettingsI18nRegistry,
) -> (gtk4::Box, Rc<RefCell<SignalManager>>) {
    use gtk4::{
        Adjustment, Box as GtkBox, Button, DropDown, Expression, Orientation, PropertyExpression,
        SpinButton, StringList, StringObject,
    };

    let AppearanceTabCallbacks {
        on_preview_theme_changed,
        refresh_preview,
        on_editor_theme_changed,
    } = callbacks;

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

    let preview_theme_row = add_setting_row_i18n(
        i18n,
        &translations.preview_theme_label,
        &translations.preview_theme_description,
        Rc::new(|t: &Translations| t.settings.appearance.preview_theme_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.preview_theme_description.clone()),
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

    let color_mode_options = [
        translations.color_mode_light.as_str(),
        translations.color_mode_dark.as_str(),
    ];
    let color_mode_string_list = StringList::new(&color_mode_options);
    i18n.bind_string_list_item(
        &color_mode_string_list,
        0,
        Rc::new(|t: &Translations| t.settings.appearance.color_mode_light.clone()),
    );
    i18n.bind_string_list_item(
        &color_mode_string_list,
        1,
        Rc::new(|t: &Translations| t.settings.appearance.color_mode_dark.clone()),
    );
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

    let color_mode_row = add_setting_row_i18n(
        i18n,
        &translations.color_mode_label,
        &translations.color_mode_description,
        Rc::new(|t: &Translations| t.settings.appearance.color_mode_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.color_mode_description.clone()),
        &color_mode_combo,
        false, // Not first row
    );
    container.append(&color_mode_row);

    // === ROW 3: Custom CSS for Preview ===
    let custom_css_button = Button::with_label(&translations.custom_css_button);
    custom_css_button.add_css_class("marco-btn");
    custom_css_button.add_css_class("marco-btn-blue");
    i18n.bind_button_label(
        &custom_css_button,
        Rc::new(|t: &Translations| t.settings.appearance.custom_css_button.clone()),
    );
    let custom_css_row = add_setting_row_i18n(
        i18n,
        &translations.custom_css_label,
        &translations.custom_css_description,
        Rc::new(|t: &Translations| t.settings.appearance.custom_css_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.custom_css_description.clone()),
        &custom_css_button,
        false, // Not first row
    );
    container.append(&custom_css_row);

    // === ROW 4: UI Font ===
    let ui_font_options = [
        translations.ui_font_system_default.as_str(),
        translations.ui_font_sans.as_str(),
        translations.ui_font_serif.as_str(),
        translations.ui_font_monospace.as_str(),
    ];
    let ui_font_values = ["system", "sans", "serif", "monospace"];
    let ui_font_string_list = StringList::new(&ui_font_options);
    i18n.bind_string_list_item(
        &ui_font_string_list,
        0,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_system_default.clone()),
    );
    i18n.bind_string_list_item(
        &ui_font_string_list,
        1,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_sans.clone()),
    );
    i18n.bind_string_list_item(
        &ui_font_string_list,
        2,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_serif.clone()),
    );
    i18n.bind_string_list_item(
        &ui_font_string_list,
        3,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_monospace.clone()),
    );
    let ui_font_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");
    let ui_font_combo = DropDown::new(Some(ui_font_string_list), Some(ui_font_expression));
    ui_font_combo.add_css_class("marco-dropdown");
    let settings_snapshot = theme_manager.borrow().get_settings();
    let current_ui_font = settings_snapshot
        .appearance
        .as_ref()
        .and_then(|a| a.ui_font.as_deref())
        .unwrap_or("system");
    let ui_font_index = ui_font_values
        .iter()
        .position(|value| *value == current_ui_font)
        .unwrap_or(0);
    ui_font_combo.set_selected(ui_font_index as u32);

    if let Ok(settings_manager) =
        core::logic::swanson::SettingsManager::initialize(settings_path.clone())
    {
        ui_font_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            let value = ui_font_values.get(idx).copied().unwrap_or("system");
            if let Err(e) = settings_manager.update_settings(|settings| {
                if settings.appearance.is_none() {
                    settings.appearance = Some(core::logic::swanson::AppearanceSettings::default());
                }
                if let Some(ref mut appearance) = settings.appearance {
                    appearance.ui_font = Some(value.to_string());
                }
            }) {
                log::error!("Failed to save UI font setting: {}", e);
            }
        });
    }

    let ui_font_row = add_setting_row_i18n(
        i18n,
        &translations.ui_font_label,
        &translations.ui_font_description,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_description.clone()),
        &ui_font_combo,
        false, // Not first row
    );
    container.append(&ui_font_row);

    // === ROW 5: UI Font Size ===
    let settings_snapshot = theme_manager.borrow().get_settings();
    let current_ui_font_size = settings_snapshot
        .appearance
        .as_ref()
        .and_then(|a| a.ui_font_size)
        .unwrap_or(12) as f64;
    let ui_font_size_adj = Adjustment::new(current_ui_font_size, 10.0, 24.0, 1.0, 0.0, 0.0);
    let ui_font_size_spin = SpinButton::new(Some(&ui_font_size_adj), 1.0, 0);
    ui_font_size_spin.add_css_class("marco-spinbutton");

    if let Ok(settings_manager) =
        core::logic::swanson::SettingsManager::initialize(settings_path.clone())
    {
        ui_font_size_adj.connect_value_changed(move |adj| {
            let new_size = adj.value() as u8;
            if let Err(e) = settings_manager.update_settings(|settings| {
                if settings.appearance.is_none() {
                    settings.appearance = Some(core::logic::swanson::AppearanceSettings::default());
                }
                if let Some(ref mut appearance) = settings.appearance {
                    appearance.ui_font_size = Some(new_size);
                }
            }) {
                log::error!("Failed to save UI font size setting: {}", e);
            }
        });
    }

    let ui_font_size_row = add_setting_row_i18n(
        i18n,
        &translations.ui_font_size_label,
        &translations.ui_font_size_description,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_size_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_size_description.clone()),
        &ui_font_size_spin,
        false, // Not first row
    );
    container.append(&ui_font_size_row);

    (container, signal_manager)
}
