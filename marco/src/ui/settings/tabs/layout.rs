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
    pub on_split_ratio_changed: Option<std::boxed::Box<dyn Fn(i32) + 'static>>,
    pub on_sync_scrolling_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
    pub on_toc_depth_changed: Option<std::boxed::Box<dyn Fn(u8) + 'static>>,
    /// Called with `is_rtl = true` when the user selects RTL, `false` for LTR.
    pub on_text_direction_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
}

// `settings_path` is used to load/save window settings including split ratio.
// `callbacks` can optionally receive events when the user changes settings.
pub fn build_layout_tab(
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
        on_split_ratio_changed,
        on_sync_scrolling_changed,
        on_toc_depth_changed,
        on_text_direction_changed,
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
        true, // First row - no top margin
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

    // TOC Heading Depth (SpinButton, 1-6, default 3)
    let current_toc_depth = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager
            .get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.toc_depth)
            .unwrap_or(3)
    } else {
        3
    };

    let toc_depth_adj = Adjustment::new(current_toc_depth as f64, 1.0, 6.0, 1.0, 0.0, 0.0);
    let toc_depth_spin = SpinButton::new(Some(&toc_depth_adj), 1.0, 0);
    toc_depth_spin.add_css_class("marco-spinbutton");

    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        toc_depth_adj.connect_value_changed(move |adj| {
            let new_depth = adj.value() as u8;
            debug!("TOC depth changed to: {}", new_depth);
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.layout.is_none() {
                    settings.layout = Some(core::logic::swanson::LayoutSettings::default());
                }
                if let Some(ref mut layout) = settings.layout {
                    layout.toc_depth = Some(new_depth);
                }
            }) {
                debug!("Failed to save toc_depth setting: {}", e);
            } else {
                debug!("toc_depth saved: {}", new_depth);
            }
        });
    }

    if let Some(callback) = on_toc_depth_changed {
        toc_depth_adj.connect_value_changed(move |adj| {
            let new_depth = adj.value() as u8;
            callback(new_depth);
        });
    }

    let toc_depth_row = add_setting_row_i18n(
        i18n,
        &translations.toc_depth_label,
        &translations.toc_depth_description,
        Rc::new(|t: &Translations| t.settings.layout.toc_depth_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.toc_depth_description.clone()),
        &toc_depth_spin,
        false,
    );
    container.append(&toc_depth_row);

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

    // Runtime callback: apply direction change immediately without restart
    if let Some(callback) = on_text_direction_changed {
        text_dir_combo.connect_selected_notify(move |combo| {
            let is_rtl = combo.selected() == 1;
            callback(is_rtl);
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
