use core::logic::swanson::WindowSettings;
use gtk4::prelude::*;
use gtk4::Box;
use log::debug;
use std::rc::Rc;

// Import unified helper
use super::helpers::{add_setting_row_i18n, SettingsI18nRegistry};
use crate::components::language::SettingsLayoutTranslations;
use crate::components::language::Translations;

#[derive(Default)]
pub struct LayoutTabCallbacks {
    pub on_view_mode_changed: Option<std::boxed::Box<dyn Fn(String) + 'static>>,
    pub on_split_ratio_changed: Option<std::boxed::Box<dyn Fn(i32) + 'static>>,
    pub on_sync_scrolling_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
    pub on_line_numbers_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
}

// Build the layout tab. `initial_view_mode` optionally sets which entry is
// active when the tab is first shown (e.g. Some("Source Code") to select
// the code preview). The optional callback will be called when the View Mode
// dropdown changes and receives the selected value as a String.
// `settings_path` is used to load/save window settings including split ratio.
// `callbacks` can optionally receive events when the user changes settings.
pub fn build_layout_tab(
    initial_view_mode: Option<String>,
    callbacks: LayoutTabCallbacks,
    settings_path: Option<&str>,
    translations: &SettingsLayoutTranslations,
    i18n: &SettingsI18nRegistry,
) -> Box {
    use gtk4::{
        Adjustment, Box as GtkBox, DropDown, Expression, Orientation, PropertyExpression,
        SpinButton, StringList, StringObject, Switch,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    let LayoutTabCallbacks {
        on_view_mode_changed,
        on_split_ratio_changed,
        on_sync_scrolling_changed,
        on_line_numbers_changed,
    } = callbacks;

    // Initialize SettingsManager once if settings_path is available
    let settings_manager_opt = if let Some(settings_path) = settings_path {
        match core::logic::swanson::SettingsManager::initialize(std::path::PathBuf::from(
            settings_path,
        )) {
            Ok(sm) => Some(sm),
            Err(e) => {
                debug!("Failed to initialize SettingsManager in layout tab: {}", e);
                None
            }
        }
    } else {
        None
    };

    // View Mode (Dropdown)
    let view_mode_labels = [
        translations.view_mode_html.as_str(),
        translations.view_mode_source.as_str(),
    ];
    let view_mode_values = ["HTML Preview", "Source Code"];
    let view_mode_options = StringList::new(&view_mode_labels);
    i18n.bind_string_list_item(
        &view_mode_options,
        0,
        Rc::new(|t: &Translations| t.settings.layout.view_mode_html.clone()),
    );
    i18n.bind_string_list_item(
        &view_mode_options,
        1,
        Rc::new(|t: &Translations| t.settings.layout.view_mode_source.clone()),
    );
    let view_mode_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let view_mode_combo =
        DropDown::new(Some(view_mode_options.clone()), Some(view_mode_expression));
    view_mode_combo.add_css_class("marco-dropdown");

    // Set active index based on saved setting if provided.
    let active_index = match initial_view_mode.as_deref() {
        Some(s)
            if s.eq_ignore_ascii_case("source code") || s.eq_ignore_ascii_case("code preview") =>
        {
            1
        }
        _ => 0,
    };
    view_mode_combo.set_selected(active_index);
    // Connect change handler to notify owner if provided. Convert selected index
    // to String when invoking the provided callback so callers receive a
    // straightforward String value.
    {
        view_mode_combo.connect_selected_notify(move |dropdown| {
            let selected_index = dropdown.selected() as usize;
            let mode_value = view_mode_values
                .get(selected_index)
                .copied()
                .unwrap_or("HTML Preview");
            if let Some(ref cb) = on_view_mode_changed {
                cb(mode_value.to_string());
            }
        });
    }

    // Create view mode row using unified helper (first row)
    let view_mode_row = add_setting_row_i18n(
        i18n,
        &translations.view_mode_label,
        &translations.view_mode_description,
        Rc::new(|t: &Translations| t.settings.layout.view_mode_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.view_mode_description.clone()),
        &view_mode_combo,
        true, // First row - no top margin
    );
    container.append(&view_mode_row);

    // Sync Scrolling (Toggle)
    let sync_scroll_switch = Switch::new();
    sync_scroll_switch.add_css_class("marco-switch");

    // Load current sync scrolling setting using existing SettingsManager
    let current_sync_scrolling = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager
            .get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.sync_scrolling)
            .unwrap_or(true)
    } else {
        true // Default to true
    };

    sync_scroll_switch.set_active(current_sync_scrolling);

    // Save sync scrolling setting when it changes
    if let Some(ref settings_manager) = settings_manager_opt {
        let settings_manager_clone = settings_manager.clone();
        sync_scroll_switch.connect_state_set(move |_switch, is_active| {
            debug!("Sync scrolling changed to: {}", is_active);

            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure layout settings exist
                if settings.layout.is_none() {
                    use core::logic::swanson::LayoutSettings;
                    settings.layout = Some(LayoutSettings::default());
                }

                // Update sync scrolling setting
                if let Some(ref mut layout) = settings.layout {
                    layout.sync_scrolling = Some(is_active);
                }
            }) {
                debug!("Failed to save sync scrolling setting: {}", e);
            } else {
                debug!("Sync scrolling saved: {}", is_active);
            }

            glib::Propagation::Proceed
        });
    }

    // Also connect runtime callback if provided
    if let Some(callback) = on_sync_scrolling_changed {
        sync_scroll_switch.connect_state_set(move |_switch, is_active| {
            debug!("Calling runtime sync scrolling update: {}", is_active);
            callback(is_active);
            glib::Propagation::Proceed
        });
    }

    // Create sync scrolling row using unified helper
    let sync_scroll_row = add_setting_row_i18n(
        i18n,
        &translations.sync_scrolling_label,
        &translations.sync_scrolling_description,
        Rc::new(|t: &Translations| t.settings.layout.sync_scrolling_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.sync_scrolling_description.clone()),
        &sync_scroll_switch,
        false, // Not first row
    );
    container.append(&sync_scroll_row);

    // Editor/View Split (SpinButton)
    // Load current split ratio from settings or use default (60%)
    let current_split_ratio = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager
            .get_settings()
            .window
            .as_ref()
            .and_then(|w| w.split_ratio)
            .unwrap_or(60)
    } else {
        60
    };

    let split_adj = Adjustment::new(current_split_ratio as f64, 10.0, 90.0, 1.0, 0.0, 0.0);
    let split_spin = SpinButton::new(Some(&split_adj), 1.0, 0);
    split_spin.add_css_class("marco-spinbutton");

    // Save split ratio when it changes
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        split_adj.connect_value_changed(move |adj| {
            let new_ratio = adj.value() as i32;
            debug!("Split ratio changed to: {}%", new_ratio);

            // Use SettingsManager to update split ratio setting
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure window settings exist
                if settings.window.is_none() {
                    settings.window = Some(WindowSettings::default());
                }

                // Update split ratio
                if let Some(ref mut window) = settings.window {
                    window.split_ratio = Some(new_ratio);
                }
            }) {
                debug!("Failed to save split ratio setting: {}", e);
            } else {
                debug!("Split ratio saved: {}%", new_ratio);
            }
        });
    }

    // Also connect live split ratio updates if callback provided
    if let Some(callback) = on_split_ratio_changed {
        split_adj.connect_value_changed(move |adj| {
            let new_ratio = adj.value() as i32;
            debug!("Calling live split ratio update: {}%", new_ratio);
            callback(new_ratio);
        });
    } else {
        debug!("No callback provided for split ratio changes");
    }

    // Create split ratio row using unified helper
    let split_row = add_setting_row_i18n(
        i18n,
        &translations.split_label,
        &translations.split_description,
        Rc::new(|t: &Translations| t.settings.layout.split_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.split_description.clone()),
        &split_spin,
        false, // Not first row
    );
    container.append(&split_row);

    // Show Line Numbers (Toggle)
    let line_numbers_switch = Switch::new();
    line_numbers_switch.add_css_class("marco-switch");

    // Load current line numbers setting from SettingsManager
    let current_line_numbers = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager
            .get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.show_line_numbers)
            .unwrap_or(true) // Default to true if not set
    } else {
        true // Default to true
    };

    line_numbers_switch.set_active(current_line_numbers);

    // Save line numbers setting when it changes
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        line_numbers_switch.connect_state_set(move |_switch, is_active| {
            debug!("Line numbers changed to: {}", is_active);

            // Use SettingsManager to update line numbers setting
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure layout settings exist
                if settings.layout.is_none() {
                    use core::logic::swanson::LayoutSettings;
                    settings.layout = Some(LayoutSettings::default());
                }

                // Update line numbers setting
                if let Some(ref mut layout) = settings.layout {
                    layout.show_line_numbers = Some(is_active);
                }
            }) {
                debug!("Failed to save line numbers setting: {}", e);
            } else {
                debug!("Line numbers saved: {}", is_active);
            }

            glib::Propagation::Proceed
        });
    }

    // Also connect runtime callback if provided
    if let Some(callback) = on_line_numbers_changed {
        line_numbers_switch.connect_state_set(move |_switch, is_active| {
            debug!("Calling runtime line numbers update: {}", is_active);
            callback(is_active);
            glib::Propagation::Proceed
        });
    }

    // Create line numbers row using unified helper
    let line_numbers_row = add_setting_row_i18n(
        i18n,
        &translations.line_numbers_label,
        &translations.line_numbers_description,
        Rc::new(|t: &Translations| t.settings.layout.line_numbers_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.line_numbers_description.clone()),
        &line_numbers_switch,
        false, // Not first row
    );
    container.append(&line_numbers_row);

    // Text Direction (Dropdown)
    let text_dir_labels = [
        translations.text_direction_ltr.as_str(),
        translations.text_direction_rtl.as_str(),
    ];
    let text_dir_values = ["ltr", "rtl"];
    let text_dir_options = StringList::new(&text_dir_labels);
    i18n.bind_string_list_item(
        &text_dir_options,
        0,
        Rc::new(|t: &Translations| t.settings.layout.text_direction_ltr.clone()),
    );
    i18n.bind_string_list_item(
        &text_dir_options,
        1,
        Rc::new(|t: &Translations| t.settings.layout.text_direction_rtl.clone()),
    );
    let text_dir_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let text_dir_combo = DropDown::new(Some(text_dir_options.clone()), Some(text_dir_expression));
    text_dir_combo.add_css_class("marco-dropdown");
    let current_text_direction = if let Some(ref settings_manager) = settings_manager_opt {
        let settings_snapshot = settings_manager.get_settings();
        settings_snapshot
            .layout
            .as_ref()
            .and_then(|l| l.text_direction.clone())
            .unwrap_or_else(|| "ltr".to_string())
    } else {
        "ltr".to_string()
    };
    let text_dir_index = text_dir_values
        .iter()
        .position(|value| *value == current_text_direction.as_str())
        .unwrap_or(0);
    text_dir_combo.set_selected(text_dir_index as u32);

    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        text_dir_combo.connect_selected_notify(move |combo| {
            let selected_index = combo.selected() as usize;
            let direction = text_dir_values
                .get(selected_index)
                .copied()
                .unwrap_or("ltr");
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.layout.is_none() {
                    settings.layout = Some(core::logic::swanson::LayoutSettings::default());
                }
                if let Some(ref mut layout) = settings.layout {
                    layout.text_direction = Some(direction.to_string());
                }
            }) {
                debug!("Failed to save text direction setting: {}", e);
            }
        });
    }

    // Create text direction row using unified helper
    let text_dir_row = add_setting_row_i18n(
        i18n,
        &translations.text_direction_label,
        &translations.text_direction_description,
        Rc::new(|t: &Translations| t.settings.layout.text_direction_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.text_direction_description.clone()),
        &text_dir_combo,
        false, // Not first row
    );
    container.append(&text_dir_row);

    container
}
