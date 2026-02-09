//! Advanced settings tab
//!
//! This tab provides access to advanced settings including:
//! - Telemetry opt-in/opt-out toggle (note: telemetry functionality is  disabled)
//! - "My Data" viewer (placeholder - shows no data)

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Switch, Window};
use log::{debug, error};
use std::rc::Rc;

// Import unified helper and i18n registry
use super::helpers::{add_setting_row_i18n, SettingsI18nRegistry};
use crate::components::language::{SettingsAdvancedTranslations, Translations};

/// Build the advanced settings tab
pub fn build_advanced_tab(
    settings_path: &str,
    translations: &SettingsAdvancedTranslations,
    i18n: &SettingsI18nRegistry,
) -> GtkBox {
    // Initialize SettingsManager for this tab
    let settings_manager_opt = match core::logic::swanson::SettingsManager::initialize(
        std::path::PathBuf::from(settings_path),
    ) {
        Ok(sm) => Some(sm),
        Err(e) => {
            log::warn!(
                "Failed to initialize SettingsManager in advanced tab: {}",
                e
            );
            None
        }
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // === Telemetry Section ===

    // Telemetry Toggle
    let telemetry_switch = Switch::new();
    telemetry_switch.add_css_class("marco-switch");

    // Load current telemetry enabled state
    let current_telemetry_enabled = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings
            .telemetry
            .as_ref()
            .and_then(|t| t.enabled)
            .unwrap_or(false)
    } else {
        false
    };
    telemetry_switch.set_active(current_telemetry_enabled);

    // Connect telemetry toggle to save settings (numbed - no actual telemetry provider interaction)
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        telemetry_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!(
                "Telemetry toggle changed to: {} (note: telemetry is disabled)",
                enabled
            );

            // Update telemetry setting using SettingsManager
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.telemetry.is_none() {
                    settings.telemetry = Some(core::logic::swanson::TelemetrySettings::default());
                }
                if let Some(ref mut telemetry) = settings.telemetry {
                    telemetry.enabled = Some(enabled);
                }
            }) {
                error!("Failed to save telemetry setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Note: telemetry provider interactions are disabled
            debug!("Telemetry setting saved (provider interaction disabled)");

            glib::Propagation::Proceed
        });
    }

    // Create telemetry toggle row
    let telemetry_row = add_setting_row_i18n(
        i18n,
        &translations.telemetry_label,
        &translations.telemetry_description,
        Rc::new(|t: &Translations| t.settings.advanced.telemetry_label.clone()),
        Rc::new(|t: &Translations| t.settings.advanced.telemetry_description.clone()),
        &telemetry_switch,
        true, // First row
    );
    container.append(&telemetry_row);

    // Note: telemetry is disabled
    let note = Label::new(Some(
        "Note: Telemetry functionality is currently disabled in Marco.",
    ));
    note.set_wrap(true);
    note.add_css_class("settings-note");
    note.set_margin_top(6);
    container.append(&note);

    // === Logging Section ===

    // Log-to-file toggle
    let log_to_file_switch = Switch::new();
    log_to_file_switch.add_css_class("marco-switch");

    let current_log_to_file_enabled = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager.get_settings().log_to_file.unwrap_or(false)
    } else {
        false
    };
    log_to_file_switch.set_active(current_log_to_file_enabled);

    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        log_to_file_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Log-to-file toggle changed to: {}", enabled);

            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                settings.log_to_file = Some(enabled);
            }) {
                error!("Failed to save log_to_file setting: {}", e);
            }

            glib::Propagation::Proceed
        });
    }

    let log_to_file_row = add_setting_row_i18n(
        i18n,
        &translations.log_to_file_label,
        &translations.log_to_file_description,
        Rc::new(|t: &Translations| t.settings.advanced.log_to_file_label.clone()),
        Rc::new(|t: &Translations| t.settings.advanced.log_to_file_description.clone()),
        &log_to_file_switch,
        false,
    );
    container.append(&log_to_file_row);

    // "My Data" button to view queued telemetry events
    let my_data_button = Button::with_label(&translations.my_data_button);
    my_data_button.add_css_class("marco-btn");
    my_data_button.add_css_class("marco-btn-blue");

    my_data_button.connect_clicked({
        let i18n = i18n.clone();
        move |button| {
            show_my_data_dialog(
                button
                    .root()
                    .and_then(|r| r.downcast::<gtk4::Window>().ok())
                    .as_ref(),
                &i18n,
            );
        }
    });

    // Create "My Data" button row
    let my_data_row = add_setting_row_i18n(
        i18n,
        &translations.my_data_label,
        &translations.my_data_description,
        Rc::new(|t: &Translations| t.settings.advanced.my_data_label.clone()),
        Rc::new(|t: &Translations| t.settings.advanced.my_data_description.clone()),
        &my_data_button,
        false, // Not first row
    );
    container.append(&my_data_row);

    container
}

/// Show a dialog displaying telemetry information (numbed - shows placeholder)
fn show_my_data_dialog(parent: Option<&gtk4::Window>, _i18n: &SettingsI18nRegistry) {
    // Determine theme class from parent
    let theme_class = parent
        .and_then(|p| p.dynamic_cast_ref::<gtk4::Widget>())
        .map(|w| {
            if w.has_css_class("marco-theme-dark") {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            }
        })
        .unwrap_or("marco-theme-light");

    // Use the shared Window-based dialog style (matches About/Save/Welcome)
    let window = Window::builder()
        .title("My Data - Telemetry")
        .modal(true)
        .default_width(500)
        .default_height(300)
        .resizable(true)
        .build();

    if let Some(parent_window) = parent {
        window.set_transient_for(Some(parent_window));
    }

    // Apply dialog + theme CSS
    window.add_css_class("marco-dialog");
    window.add_css_class(theme_class);

    // Shared custom titlebar
    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &window,
        "My Data",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );
    let close_btn_titlebar = titlebar_controls
        .close_button
        .as_ref()
        .expect("My Data dialog requires a close button");
    {
        let window_for_close = window.clone();
        close_btn_titlebar.connect_clicked(move |_| {
            window_for_close.close();
        });
    }
    window.set_titlebar(Some(&titlebar_controls.headerbar));

    // Content container
    let content = GtkBox::new(Orientation::Vertical, 12);
    content.add_css_class("marco-dialog-content");
    content.set_halign(Align::Fill);
    content.set_valign(Align::Fill);

    // Info label
    let info_text = "Telemetry functionality is currently disabled in Marco.\n\n\
        The 'My Data' viewer would normally show telemetry events tracked locally,\n\
        but telemetry tracking is not active.";
    let info_label = Label::builder()
        .label(info_text)
        .wrap(true)
        .xalign(0.0)
        .build();
    info_label.add_css_class("marco-dialog-message");
    content.append(&info_label);

    // Bottom Close button
    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.add_css_class("marco-dialog-button-box");
    button_box.set_halign(Align::End);

    let btn_close = Button::with_label("Close");
    btn_close.add_css_class("marco-btn");
    btn_close.add_css_class("marco-btn-blue");
    button_box.append(&btn_close);
    content.append(&button_box);

    window.set_child(Some(&content));

    {
        let window_for_close = window.clone();
        btn_close.connect_clicked(move |_| {
            window_for_close.close();
        });
    }

    window.present();
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_test_advanced_tab_creation() {
        // Compile-time smoke test: ensure build_advanced_tab has expected signature
        let _fn_ptr = super::build_advanced_tab
            as fn(
                &str,
                &super::SettingsAdvancedTranslations,
                &super::SettingsI18nRegistry,
            ) -> gtk4::Box;
        let _ = _fn_ptr;
    }
}
