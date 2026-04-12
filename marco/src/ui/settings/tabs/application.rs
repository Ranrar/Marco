use core::logic::swanson::WindowSettings;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use super::helpers::{add_setting_row_i18n, SettingsI18nRegistry};
use crate::components::language::SettingsAppearanceTranslations;
use crate::components::language::SettingsLayoutTranslations;
use crate::components::language::Translations;
use crate::logic::signal_manager::SignalManager;
use core::logic::loaders::theme_loader::list_html_view_themes;

pub struct ApplicationTabCallbacks {
    // Appearance callbacks
    pub on_preview_theme_changed: Box<dyn Fn(String) + 'static>,
    pub refresh_preview: Rc<RefCell<Box<dyn Fn()>>>,
    pub on_editor_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
    // Layout callbacks
    pub on_split_ratio_changed: Option<std::boxed::Box<dyn Fn(i32) + 'static>>,
    pub on_sync_scrolling_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
    pub on_toc_depth_changed: Option<std::boxed::Box<dyn Fn(u8) + 'static>>,
    pub on_text_direction_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
}

pub fn build_application_tab(
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    settings_path: PathBuf,
    asset_dir: &std::path::Path,
    callbacks: ApplicationTabCallbacks,
    appearance_translations: &SettingsAppearanceTranslations,
    layout_translations: &SettingsLayoutTranslations,
    i18n: &SettingsI18nRegistry,
) -> (gtk4::Box, Rc<RefCell<SignalManager>>) {
    use gtk4::{
        Adjustment, Box as GtkBox, DropDown, Expression, Orientation, PropertyExpression,
        SpinButton, StringList, StringObject, Switch,
    };
    use log::debug;

    let ApplicationTabCallbacks {
        on_preview_theme_changed,
        refresh_preview,
        on_editor_theme_changed,
        on_split_ratio_changed,
        on_sync_scrolling_changed,
        on_toc_depth_changed,
        on_text_direction_changed,
    } = callbacks;

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    let signal_manager = Rc::new(RefCell::new(SignalManager::new()));

    // ── Appearance section ─────────────────────────────────────────────────

    let on_preview_theme_changed = Rc::new(on_preview_theme_changed);
    let user_selected_preview_theme = Rc::new(std::cell::Cell::new(false));
    let html_theme_dir = asset_dir.join("themes/html_viever");
    let html_themes = list_html_view_themes(&html_theme_dir);

    // Preview Theme (DropDown)
    let theme_labels: Vec<&str> = html_themes.iter().map(|e| e.label.as_str()).collect();
    let theme_string_list = StringList::new(&theme_labels);
    let theme_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");
    let preview_theme_combo = DropDown::new(Some(theme_string_list), Some(theme_expression));
    preview_theme_combo.add_css_class("marco-dropdown");

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

    {
        let theme_manager_clone = Rc::clone(&theme_manager);
        let settings_path_clone = settings_path.clone();
        let html_themes_clone = html_themes.clone();
        let on_preview_theme_changed_clone = Rc::clone(&on_preview_theme_changed);
        let user_selected_clone = Rc::clone(&user_selected_preview_theme);
        let sm_clone = signal_manager.clone();

        let handler_id = preview_theme_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            if let Some(entry) = html_themes_clone.get(idx) {
                user_selected_clone.set(true);
                theme_manager_clone
                    .borrow_mut()
                    .set_preview_theme(entry.filename.clone(), &settings_path_clone);
                (on_preview_theme_changed_clone)(entry.filename.clone());
            }
        });

        sm_clone.borrow_mut().register_handler(
            "application_tab",
            &preview_theme_combo.clone().upcast(),
            handler_id,
        );
    }

    let preview_theme_row = add_setting_row_i18n(
        i18n,
        &appearance_translations.preview_theme_label,
        &appearance_translations.preview_theme_description,
        Rc::new(|t: &Translations| t.settings.appearance.preview_theme_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.preview_theme_description.clone()),
        &preview_theme_combo,
        true, // first row
    );
    container.append(&preview_theme_row);

    // Color Mode (Light / Dark)
    let app_settings = theme_manager.borrow().get_settings();
    let current_mode = app_settings
        .appearance
        .as_ref()
        .and_then(|a| a.editor_mode.clone())
        .unwrap_or_else(|| "marco-light".to_string());

    let color_mode_options = [
        appearance_translations.color_mode_light.as_str(),
        appearance_translations.color_mode_dark.as_str(),
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
    let color_mode_combo =
        DropDown::new(Some(color_mode_string_list), Some(color_mode_expression));
    color_mode_combo.add_css_class("marco-dropdown");
    color_mode_combo.set_selected(match current_mode.as_str() {
        "marco-dark" | "dark" => 1,
        _ => 0,
    });

    {
        let theme_manager_clone = theme_manager.clone();
        let settings_path_clone = settings_path.clone();
        let refresh_preview_clone = Rc::clone(&refresh_preview);
        let on_editor_theme_changed_clone = on_editor_theme_changed.map(Rc::new);
        let sm_clone = signal_manager.clone();

        let handler_id = color_mode_combo.connect_selected_notify(move |combo| {
            let scheme_id = if combo.selected() == 1 {
                "marco-dark"
            } else {
                "marco-light"
            };
            {
                let mut mgr = theme_manager_clone.borrow_mut();
                mgr.set_editor_scheme(scheme_id, &settings_path_clone);
            }
            if let Some(ref cb) = on_editor_theme_changed_clone {
                cb(scheme_id.to_string());
            }
            (refresh_preview_clone.borrow())();
        });

        sm_clone.borrow_mut().register_handler(
            "application_tab",
            &color_mode_combo.clone().upcast(),
            handler_id,
        );
    }

    let color_mode_row = add_setting_row_i18n(
        i18n,
        &appearance_translations.color_mode_label,
        &appearance_translations.color_mode_description,
        Rc::new(|t: &Translations| t.settings.appearance.color_mode_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.color_mode_description.clone()),
        &color_mode_combo,
        false,
    );
    container.append(&color_mode_row);

    // UI Font (unavailable placeholder)
    let ui_font_options = [
        appearance_translations.ui_font_system_default.as_str(),
        appearance_translations.ui_font_sans.as_str(),
        appearance_translations.ui_font_serif.as_str(),
        appearance_translations.ui_font_monospace.as_str(),
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
        .position(|v| *v == current_ui_font)
        .unwrap_or(0);
    ui_font_combo.set_selected(ui_font_index as u32);
    ui_font_combo.set_sensitive(false);
    ui_font_combo.add_css_class("marco-control-unavailable");
    ui_font_combo.set_tooltip_text(Some("Not available yet"));

    if let Ok(sm) = core::logic::swanson::SettingsManager::initialize(settings_path.clone()) {
        ui_font_combo.connect_selected_notify(move |combo| {
            let value = ui_font_values
                .get(combo.selected() as usize)
                .copied()
                .unwrap_or("system");
            let _ = sm.update_settings(|s| {
                if s.appearance.is_none() {
                    s.appearance = Some(core::logic::swanson::AppearanceSettings::default());
                }
                if let Some(ref mut a) = s.appearance {
                    a.ui_font = Some(value.to_string());
                }
            });
        });
    }

    let ui_font_row = add_setting_row_i18n(
        i18n,
        &appearance_translations.ui_font_label,
        &appearance_translations.ui_font_description,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_description.clone()),
        &ui_font_combo,
        false,
    );
    ui_font_row.add_css_class("marco-settings-row-unavailable");
    ui_font_row.set_tooltip_text(Some("Not available yet"));
    container.append(&ui_font_row);

    // UI Font Size (unavailable placeholder)
    let current_ui_font_size = theme_manager
        .borrow()
        .get_settings()
        .appearance
        .as_ref()
        .and_then(|a| a.ui_font_size)
        .unwrap_or(12) as f64;
    let ui_font_size_adj = Adjustment::new(current_ui_font_size, 10.0, 24.0, 1.0, 0.0, 0.0);
    let ui_font_size_spin = SpinButton::new(Some(&ui_font_size_adj), 1.0, 0);
    ui_font_size_spin.add_css_class("marco-spinbutton");
    ui_font_size_spin.set_sensitive(false);
    ui_font_size_spin.add_css_class("marco-control-unavailable");
    ui_font_size_spin.set_tooltip_text(Some("Not available yet"));

    if let Ok(sm) = core::logic::swanson::SettingsManager::initialize(settings_path.clone()) {
        ui_font_size_adj.connect_value_changed(move |adj| {
            let new_size = adj.value() as u8;
            let _ = sm.update_settings(|s| {
                if s.appearance.is_none() {
                    s.appearance = Some(core::logic::swanson::AppearanceSettings::default());
                }
                if let Some(ref mut a) = s.appearance {
                    a.ui_font_size = Some(new_size);
                }
            });
        });
    }

    let ui_font_size_row = add_setting_row_i18n(
        i18n,
        &appearance_translations.ui_font_size_label,
        &appearance_translations.ui_font_size_description,
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_size_label.clone()),
        Rc::new(|t: &Translations| t.settings.appearance.ui_font_size_description.clone()),
        &ui_font_size_spin,
        false,
    );
    ui_font_size_row.add_css_class("marco-settings-row-unavailable");
    ui_font_size_row.set_tooltip_text(Some("Not available yet"));
    container.append(&ui_font_size_row);

    // ── Layout section ─────────────────────────────────────────────────────

    let settings_manager_opt = {
        match core::logic::swanson::SettingsManager::initialize(settings_path.clone()) {
            Ok(sm) => Some(sm),
            Err(e) => {
                debug!("Failed to initialize SettingsManager in application tab: {}", e);
                None
            }
        }
    };

    // Sync Scrolling (Toggle)
    let sync_scroll_switch = Switch::new();
    sync_scroll_switch.add_css_class("marco-switch");
    let current_sync_scrolling = settings_manager_opt
        .as_ref()
        .map(|sm| {
            sm.get_settings()
                .layout
                .as_ref()
                .and_then(|l| l.sync_scrolling)
                .unwrap_or(true)
        })
        .unwrap_or(true);
    sync_scroll_switch.set_active(current_sync_scrolling);

    if let Some(ref sm) = settings_manager_opt {
        let sm_c = sm.clone();
        sync_scroll_switch.connect_state_set(move |_sw, active| {
            debug!("Sync scrolling changed to: {}", active);
            if let Err(e) = sm_c.update_settings(|s| {
                if s.layout.is_none() {
                    s.layout = Some(core::logic::swanson::LayoutSettings::default());
                }
                if let Some(ref mut l) = s.layout {
                    l.sync_scrolling = Some(active);
                }
            }) {
                debug!("Failed to save sync scrolling: {}", e);
            }
            glib::Propagation::Proceed
        });
    }
    if let Some(cb) = on_sync_scrolling_changed {
        sync_scroll_switch.connect_state_set(move |_sw, active| {
            cb(active);
            glib::Propagation::Proceed
        });
    }

    let sync_scroll_row = add_setting_row_i18n(
        i18n,
        &layout_translations.sync_scrolling_label,
        &layout_translations.sync_scrolling_description,
        Rc::new(|t: &Translations| t.settings.layout.sync_scrolling_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.sync_scrolling_description.clone()),
        &sync_scroll_switch,
        false,
    );
    container.append(&sync_scroll_row);

    // Editor/View Split (SpinButton)
    let current_split_ratio = settings_manager_opt
        .as_ref()
        .map(|sm| {
            sm.get_settings()
                .window
                .as_ref()
                .and_then(|w| w.split_ratio)
                .unwrap_or(60)
        })
        .unwrap_or(60);
    let split_adj = Adjustment::new(current_split_ratio as f64, 10.0, 90.0, 1.0, 0.0, 0.0);
    let split_spin = SpinButton::new(Some(&split_adj), 1.0, 0);
    split_spin.add_css_class("marco-spinbutton");

    if let Some(ref sm) = settings_manager_opt {
        let sm_c = sm.clone();
        split_adj.connect_value_changed(move |adj| {
            let ratio = adj.value() as i32;
            if let Err(e) = sm_c.update_settings(|s| {
                if s.window.is_none() {
                    s.window = Some(WindowSettings::default());
                }
                if let Some(ref mut w) = s.window {
                    w.split_ratio = Some(ratio);
                }
            }) {
                debug!("Failed to save split ratio: {}", e);
            }
        });
    }
    if let Some(cb) = on_split_ratio_changed {
        split_adj.connect_value_changed(move |adj| {
            cb(adj.value() as i32);
        });
    }

    let split_row = add_setting_row_i18n(
        i18n,
        &layout_translations.split_label,
        &layout_translations.split_description,
        Rc::new(|t: &Translations| t.settings.layout.split_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.split_description.clone()),
        &split_spin,
        false,
    );
    container.append(&split_row);

    // TOC Heading Depth (SpinButton)
    let current_toc_depth = settings_manager_opt
        .as_ref()
        .map(|sm| {
            sm.get_settings()
                .layout
                .as_ref()
                .and_then(|l| l.toc_depth)
                .unwrap_or(3)
        })
        .unwrap_or(3);
    let toc_depth_adj = Adjustment::new(current_toc_depth as f64, 1.0, 6.0, 1.0, 0.0, 0.0);
    let toc_depth_spin = SpinButton::new(Some(&toc_depth_adj), 1.0, 0);
    toc_depth_spin.add_css_class("marco-spinbutton");

    if let Some(ref sm) = settings_manager_opt {
        let sm_c = sm.clone();
        toc_depth_adj.connect_value_changed(move |adj| {
            let depth = adj.value() as u8;
            if let Err(e) = sm_c.update_settings(|s| {
                if s.layout.is_none() {
                    s.layout = Some(core::logic::swanson::LayoutSettings::default());
                }
                if let Some(ref mut l) = s.layout {
                    l.toc_depth = Some(depth);
                }
            }) {
                debug!("Failed to save toc_depth: {}", e);
            }
        });
    }
    if let Some(cb) = on_toc_depth_changed {
        toc_depth_adj.connect_value_changed(move |adj| {
            cb(adj.value() as u8);
        });
    }

    let toc_depth_row = add_setting_row_i18n(
        i18n,
        &layout_translations.toc_depth_label,
        &layout_translations.toc_depth_description,
        Rc::new(|t: &Translations| t.settings.layout.toc_depth_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.toc_depth_description.clone()),
        &toc_depth_spin,
        false,
    );
    container.append(&toc_depth_row);

    // Text Direction (DropDown)
    let text_dir_labels = [
        layout_translations.text_direction_ltr.as_str(),
        layout_translations.text_direction_rtl.as_str(),
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
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");
    let text_dir_combo = DropDown::new(Some(text_dir_options), Some(text_dir_expression));
    text_dir_combo.add_css_class("marco-dropdown");

    let current_text_dir = settings_manager_opt
        .as_ref()
        .map(|sm| {
            sm.get_settings()
                .layout
                .as_ref()
                .and_then(|l| l.text_direction.clone())
                .unwrap_or_else(|| "ltr".to_string())
        })
        .unwrap_or_else(|| "ltr".to_string());
    let text_dir_index = text_dir_values
        .iter()
        .position(|v| *v == current_text_dir.as_str())
        .unwrap_or(0);
    text_dir_combo.set_selected(text_dir_index as u32);

    if let Some(ref sm) = settings_manager_opt {
        let sm_c = sm.clone();
        text_dir_combo.connect_selected_notify(move |combo| {
            let dir = text_dir_values
                .get(combo.selected() as usize)
                .copied()
                .unwrap_or("ltr");
            if let Err(e) = sm_c.update_settings(|s| {
                if s.layout.is_none() {
                    s.layout = Some(core::logic::swanson::LayoutSettings::default());
                }
                if let Some(ref mut l) = s.layout {
                    l.text_direction = Some(dir.to_string());
                }
            }) {
                debug!("Failed to save text direction: {}", e);
            }
        });
    }
    if let Some(cb) = on_text_direction_changed {
        text_dir_combo.connect_selected_notify(move |combo| {
            cb(combo.selected() == 1);
        });
    }

    let text_dir_row = add_setting_row_i18n(
        i18n,
        &layout_translations.text_direction_label,
        &layout_translations.text_direction_description,
        Rc::new(|t: &Translations| t.settings.layout.text_direction_label.clone()),
        Rc::new(|t: &Translations| t.settings.layout.text_direction_description.clone()),
        &text_dir_combo,
        false,
    );
    container.append(&text_dir_row);

    (container, signal_manager)
}
