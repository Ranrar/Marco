use gtk4::glib;
use gtk4::{Button, Dialog, Notebook, Orientation, Window};

use crate::settings::core::{OriginalSettings, SettingsChangeTracker};
use crate::settings::dialogs::{
    advanced::create_advanced_settings_page, appearance::create_appearance_settings_page,
    editor::create_editor_settings_page, language::create_language_settings_page,
    layout::create_layout_settings_page,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Create and show the modular settings dialog
pub fn show_settings_dialog(
    parent: &Window,
    editor: &crate::editor::MarkdownEditor,
    theme_manager: &crate::theme::ThemeManager,
) {
    let dialog = Dialog::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .resizable(true)
        .default_width(650)
        .default_height(550)
        .build();

    let original_settings = OriginalSettings::load_current();
    let change_tracker = Rc::new(RefCell::new(SettingsChangeTracker::load_current()));
    let settings_saved = Rc::new(RefCell::new(false));
    // (Removed is_closing flag, not needed)

    let content_area = dialog.content_area();
    content_area.set_spacing(0);
    let notebook = Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_vexpand(true);
    notebook.set_hexpand(true);

    let button_box = gtk4::Box::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(12);
    button_box.set_margin_bottom(12);
    button_box.set_margin_start(12);
    button_box.set_margin_end(12);

    let reset_button = Button::with_label("Reset to Defaults");
    let cancel_button = Button::with_label("Cancel");
    cancel_button.add_css_class("destructive-action");
    let save_button = Button::with_label("Save");
    save_button.add_css_class("suggested-action");
    save_button.set_sensitive(false);

    // Add modular pages
    create_editor_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    create_layout_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    create_appearance_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    create_language_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    create_advanced_settings_page(&notebook, &change_tracker, &save_button, &original_settings);

    content_area.append(&notebook);
    button_box.append(&reset_button);
    button_box.append(&cancel_button);
    button_box.append(&save_button);
    content_area.append(&button_box);

    // --- Button signal handlers ---
    // Save button: apply changes and close dialog
    // --- Button signal handlers ---
    // Save button: apply changes and close dialog
    {
        let dialog = dialog.clone();
        let change_tracker = change_tracker.clone();
        let settings_saved = settings_saved.clone();
        let editor = editor.clone();
        let theme_manager = theme_manager.clone();
        save_button.connect_clicked(move |_| {
            change_tracker
                .borrow()
                .apply_changes(&editor, &theme_manager);
            *change_tracker.borrow_mut() = SettingsChangeTracker::load_current();
            *settings_saved.borrow_mut() = true;
            dialog.close();
        });
    }

    // Cancel button: check for unsaved changes, show dialog if needed
    {
        let dialog = dialog.clone();
        let change_tracker = change_tracker.clone();
        let original_settings = original_settings.clone();
        let settings_saved = settings_saved.clone();
        cancel_button.connect_clicked(move |_| {
            if change_tracker.borrow().has_changes(&original_settings) {
                // Show unsaved changes dialog
                let confirm_dialog = Dialog::builder()
                    .title("Unsaved Changes")
                    .transient_for(&dialog)
                    .modal(true)
                    .build();
                confirm_dialog.add_button("Don't Save", gtk4::ResponseType::Accept);
                confirm_dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
                let content = confirm_dialog.content_area();
                let message =
                    gtk4::Label::new(Some("You have unsaved changes. What would you like to do?"));
                message.set_wrap(true);
                message.set_margin_top(16);
                message.set_margin_bottom(16);
                message.set_margin_start(16);
                message.set_margin_end(16);
                content.append(&message);
                let dialog2 = dialog.clone();
                let settings_saved2 = settings_saved.clone();
                confirm_dialog.connect_response(move |d, resp| {
                    if resp == gtk4::ResponseType::Accept {
                        *settings_saved2.borrow_mut() = true;
                        d.close();
                        dialog2.close();
                    } else {
                        d.close();
                    }
                });
                confirm_dialog.show();
            } else {
                dialog.close();
            }
        });
    }

    // Reset button: show confirmation, reset settings if confirmed
    {
        let dialog = dialog.clone();
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let settings_saved_for_reset = settings_saved.clone();
        reset_button.connect_clicked(move |_| {
            let change_tracker = change_tracker.clone();
            let save_button = save_button.clone();
            let settings_saved = settings_saved_for_reset.clone();
            let dialog = dialog.clone();
            let confirm_dialog = Dialog::builder()
                .title("Reset to Defaults?")
                .transient_for(&dialog)
                .modal(true)
                .build();
            confirm_dialog.add_button("Reset", gtk4::ResponseType::Accept);
            confirm_dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
            let content = confirm_dialog.content_area();
            let message = gtk4::Label::new(Some("Are you sure you want to reset all settings to their default values? This action cannot be undone."));
            message.set_wrap(true);
            message.set_margin_top(12);
            message.set_margin_bottom(12);
            message.set_margin_start(12);
            message.set_margin_end(12);
            content.append(&message);
            confirm_dialog.connect_response(move |d, resp| {
                if resp == gtk4::ResponseType::Accept {
                    let prefs = crate::settings::core::get_app_preferences();
                    prefs.reset_to_defaults();
                    *change_tracker.borrow_mut() = SettingsChangeTracker::load_current();
                    save_button.set_sensitive(false);
                    *settings_saved.borrow_mut() = true;
                    dialog.close();
                }
                d.close();
            });
            confirm_dialog.show();
        });
    }

    // Handle close request (window X button)
    {
        let dialog = dialog.clone();
        let change_tracker = change_tracker.clone();
        let original_settings = original_settings.clone();
        let settings_saved = settings_saved.clone();
        dialog.clone().connect_close_request(move |_| {
            // If settings were saved, allow close
            if *settings_saved.borrow() {
                return glib::Propagation::Proceed;
            }
            let dialog = dialog.clone();
            let change_tracker = change_tracker.clone();
            let original_settings = original_settings.clone();
            let settings_saved = settings_saved.clone();
            if change_tracker.borrow().has_changes(&original_settings) {
                let confirm_dialog = Dialog::builder()
                    .title("Unsaved Changes")
                    .transient_for(&dialog)
                    .modal(true)
                    .build();
                confirm_dialog.add_button("Don't Save", gtk4::ResponseType::Accept);
                confirm_dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
                let content = confirm_dialog.content_area();
                let message =
                    gtk4::Label::new(Some("You have unsaved changes. What would you like to do?"));
                message.set_wrap(true);
                message.set_margin_top(16);
                message.set_margin_bottom(16);
                message.set_margin_start(16);
                message.set_margin_end(16);
                content.append(&message);
                let dialog2 = dialog.clone();
                let settings_saved2 = settings_saved.clone();
                confirm_dialog.connect_response(move |d, resp| {
                    if resp == gtk4::ResponseType::Accept {
                        *settings_saved2.borrow_mut() = true;
                        d.close();
                        dialog2.close();
                    } else {
                        d.close();
                    }
                });
                confirm_dialog.show();
                // Returning Propagation::Stop prevents the dialog from closing immediately
                return glib::Propagation::Stop;
            }
            // Returning Propagation::Proceed allows the dialog to close
            glib::Propagation::Proceed
        });
    }

    crate::settings::ui::apply_settings_css();
    dialog.add_css_class("settings-dialog");
    dialog.show();
}

use gtk4::prelude::*;

