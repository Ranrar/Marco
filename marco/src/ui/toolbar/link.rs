//! Toolbar link-insert helpers for Markdown links.
//! Opens a compact popover near the editor cursor to collect URL + optional link metadata.

use gtk4::prelude::*;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

#[cfg(target_os = "linux")]
use gtk4::{FileChooserAction, FileChooserNative};

const LINK_POPOVER_WIDTH: i32 = 280;
const LINK_POPOVER_HORIZONTAL_SAFE_PADDING: i32 = 8;

pub fn connect_link_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    parent_window: &gtk4::Window,
    current_file_provider: Rc<dyn Fn() -> Option<PathBuf>>,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-link")
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let parent_window = parent_window.clone();
        let current_file_provider = current_file_provider.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            show_insert_link_popover(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                editor_view.upcast_ref::<gtk4::TextView>(),
                &parent_window,
                current_file_provider.clone(),
            );
        });
    }
}

pub fn connect_reference_link_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    parent_window: &gtk4::Window,
    current_file_provider: Rc<dyn Fn() -> Option<PathBuf>>,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) = find_button_by_css_class(
        toolbar.upcast_ref::<gtk4::Widget>(),
        "toolbar-btn-link-reference",
    ) {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let parent_window = parent_window.clone();
        let current_file_provider = current_file_provider.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            show_insert_reference_link_popover(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                editor_view.upcast_ref::<gtk4::TextView>(),
                &parent_window,
                current_file_provider.clone(),
            );
        });
    }
}

pub fn connect_image_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    parent_window: &gtk4::Window,
    current_file_provider: Rc<dyn Fn() -> Option<PathBuf>>,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-image")
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let parent_window = parent_window.clone();
        let current_file_provider = current_file_provider.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            show_insert_image_popover(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                editor_view.upcast_ref::<gtk4::TextView>(),
                &parent_window,
                current_file_provider.clone(),
            );
        });
    }
}

pub fn show_insert_link_popover(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    parent_window: &gtk4::Window,
    current_file_provider: Rc<dyn Fn() -> Option<PathBuf>>,
) {
    let popover = gtk4::Popover::new();
    popover.set_has_arrow(true);
    popover.set_autohide(true);
    popover.set_position(gtk4::PositionType::Bottom);
    popover.set_can_focus(true);
    popover.add_css_class("marco-link-popover");
    popover.set_parent(editor_view);

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    root.set_margin_start(8);
    root.set_margin_end(8);
    root.set_margin_top(6);
    root.set_margin_bottom(6);
    root.set_width_request(LINK_POPOVER_WIDTH);

    let title = gtk4::Label::new(Some("Link"));
    title.set_halign(gtk4::Align::Start);
    title.add_css_class("marco-dialog-section-label");

    let url_entry = gtk4::Entry::new();
    url_entry.set_hexpand(true);
    url_entry.set_placeholder_text(Some("URL (required)"));
    url_entry.add_css_class("marco-search-entry");

    let browse_button = gtk4::Button::with_label("Browse");
    browse_button.add_css_class("marco-btn");
    browse_button.add_css_class("marco-btn-blue");

    let url_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    url_row.append(&url_entry);
    url_row.append(&browse_button);

    let label_entry = gtk4::Entry::new();
    label_entry.set_hexpand(true);
    label_entry.set_placeholder_text(Some("Label link text (optional)"));
    label_entry.add_css_class("marco-search-entry");

    let attribute_entry = gtk4::Entry::new();
    attribute_entry.set_hexpand(true);
    attribute_entry.set_placeholder_text(Some("Attribute link text (optional)"));
    attribute_entry.add_css_class("marco-search-entry");

    let actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    actions.set_halign(gtk4::Align::End);
    actions.set_margin_top(2);

    let cancel_button = gtk4::Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let ok_button = gtk4::Button::with_label("Ok");
    ok_button.add_css_class("marco-btn");
    ok_button.add_css_class("marco-btn-blue");

    actions.append(&cancel_button);
    actions.append(&ok_button);

    root.append(&title);
    root.append(&url_row);
    root.append(&label_entry);
    root.append(&attribute_entry);
    root.append(&actions);

    popover.set_child(Some(&root));

    {
        let browse_button = browse_button.clone();
        let url_entry = url_entry.clone();
        let popover = popover.clone();
        let parent_window = parent_window.clone();
        let current_file_provider = current_file_provider.clone();

        let can_browse = current_file_provider().is_some();
        browse_button.set_sensitive(can_browse);
        browse_button.set_tooltip_text(Some(if can_browse {
            "Browse local file and insert path relative to current document"
        } else {
            "Save the current document first to insert a relative local path"
        }));

        browse_button.connect_clicked(move |_| {
            let parent_window = parent_window.clone();
            let url_entry = url_entry.clone();
            let popover = popover.clone();
            let current_file_provider = current_file_provider.clone();

            popover.popdown();

            glib::MainContext::default().spawn_local(async move {
                let Some(selected_path) = pick_local_file(&parent_window).await else {
                    popover.popup();
                    url_entry.grab_focus();
                    return;
                };

                let Some(current_file_path) = current_file_provider() else {
                    popover.popup();
                    url_entry.grab_focus();
                    return;
                };

                let link_path = local_link_path_relative_to_current_file(
                    selected_path.as_path(),
                    current_file_path.as_path(),
                );
                url_entry.set_text(&link_path);
                popover.popup();
                url_entry.grab_focus();
            });
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let url_entry_for_signal = url_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        url_entry_for_signal.connect_activate(move |_| {
            submit_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    {
        let popover = popover.clone();
        let editor_view = editor_view.clone();
        cancel_button.connect_clicked(move |_| {
            popover.popdown();
            editor_view.grab_focus();
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        ok_button.connect_clicked(move |_| {
            submit_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    {
        let signal_url_entry = url_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let ok_button = ok_button.clone();
        signal_url_entry.connect_changed(move |_| {
            update_local_label_requirement_ui(&url_entry, &label_entry, &ok_button);
        });
    }

    {
        let signal_label_entry = label_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let ok_button = ok_button.clone();
        signal_label_entry.connect_changed(move |_| {
            update_local_label_requirement_ui(&url_entry, &label_entry, &ok_button);
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let label_entry_for_signal = label_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        label_entry_for_signal.connect_activate(move |_| {
            submit_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let attribute_entry_for_signal = attribute_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        attribute_entry_for_signal.connect_activate(move |_| {
            submit_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    let caret_rect = cursor_rect(text_buffer, editor_view);
    let clamped_rect = clamp_rect_to_editor(caret_rect, editor_view);
    popover.set_pointing_to(Some(&clamped_rect));

    if let Some(text_area) = visible_text_area_widget_rect(editor_view) {
        let x_offset = compute_popover_x_offset_for_text_area(
            clamped_rect.x(),
            text_area.x(),
            text_area.x() + text_area.width(),
            LINK_POPOVER_WIDTH,
            LINK_POPOVER_HORIZONTAL_SAFE_PADDING,
        );
        popover.set_offset(x_offset, 0);
    }

    update_local_label_requirement_ui(&url_entry, &label_entry, &ok_button);

    popover.popup();
    url_entry.grab_focus();
}

pub fn show_insert_reference_link_popover(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    parent_window: &gtk4::Window,
    current_file_provider: Rc<dyn Fn() -> Option<PathBuf>>,
) {
    let popover = gtk4::Popover::new();
    popover.set_has_arrow(true);
    popover.set_autohide(true);
    popover.set_position(gtk4::PositionType::Bottom);
    popover.set_can_focus(true);
    popover.add_css_class("marco-link-popover");
    popover.set_parent(editor_view);

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    root.set_margin_start(8);
    root.set_margin_end(8);
    root.set_margin_top(6);
    root.set_margin_bottom(6);
    root.set_width_request(LINK_POPOVER_WIDTH);

    let title = gtk4::Label::new(Some("Reference Link"));
    title.set_halign(gtk4::Align::Start);
    title.add_css_class("marco-dialog-section-label");

    let url_entry = gtk4::Entry::new();
    url_entry.set_hexpand(true);
    url_entry.set_placeholder_text(Some("URL (required)"));
    url_entry.add_css_class("marco-search-entry");

    let browse_button = gtk4::Button::with_label("Browse");
    browse_button.add_css_class("marco-btn");
    browse_button.add_css_class("marco-btn-blue");

    let url_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    url_row.append(&url_entry);
    url_row.append(&browse_button);

    let label_entry = gtk4::Entry::new();
    label_entry.set_hexpand(true);
    label_entry.set_placeholder_text(Some("Label link text (optional)"));
    label_entry.add_css_class("marco-search-entry");

    let title_entry = gtk4::Entry::new();
    title_entry.set_hexpand(true);
    title_entry.set_placeholder_text(Some("Link title (optional)"));
    title_entry.add_css_class("marco-search-entry");

    let actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    actions.set_halign(gtk4::Align::End);
    actions.set_margin_top(2);

    let cancel_button = gtk4::Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let ok_button = gtk4::Button::with_label("Ok");
    ok_button.add_css_class("marco-btn");
    ok_button.add_css_class("marco-btn-blue");

    actions.append(&cancel_button);
    actions.append(&ok_button);

    root.append(&title);
    root.append(&url_row);
    root.append(&label_entry);
    root.append(&title_entry);
    root.append(&actions);

    popover.set_child(Some(&root));

    {
        let browse_button = browse_button.clone();
        let url_entry = url_entry.clone();
        let popover = popover.clone();
        let parent_window = parent_window.clone();
        let current_file_provider = current_file_provider.clone();

        let can_browse = current_file_provider().is_some();
        browse_button.set_sensitive(can_browse);
        browse_button.set_tooltip_text(Some(if can_browse {
            "Browse local file and insert path relative to current document"
        } else {
            "Save the current document first to insert a relative local path"
        }));

        browse_button.connect_clicked(move |_| {
            let parent_window = parent_window.clone();
            let url_entry = url_entry.clone();
            let popover = popover.clone();
            let current_file_provider = current_file_provider.clone();

            popover.popdown();

            glib::MainContext::default().spawn_local(async move {
                let Some(selected_path) = pick_local_file(&parent_window).await else {
                    popover.popup();
                    url_entry.grab_focus();
                    return;
                };

                let Some(current_file_path) = current_file_provider() else {
                    popover.popup();
                    url_entry.grab_focus();
                    return;
                };

                let link_path = local_link_path_relative_to_current_file(
                    selected_path.as_path(),
                    current_file_path.as_path(),
                );
                url_entry.set_text(&link_path);
                popover.popup();
                url_entry.grab_focus();
            });
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let url_entry_for_signal = url_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let title_entry = title_entry.clone();

        url_entry_for_signal.connect_activate(move |_| {
            submit_reference_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &title_entry,
            );
        });
    }

    {
        let popover = popover.clone();
        let editor_view = editor_view.clone();
        cancel_button.connect_clicked(move |_| {
            popover.popdown();
            editor_view.grab_focus();
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let title_entry = title_entry.clone();

        ok_button.connect_clicked(move |_| {
            submit_reference_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &title_entry,
            );
        });
    }

    {
        let signal_url_entry = url_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let ok_button = ok_button.clone();
        signal_url_entry.connect_changed(move |_| {
            update_local_label_requirement_ui(&url_entry, &label_entry, &ok_button);
        });
    }

    {
        let signal_label_entry = label_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let ok_button = ok_button.clone();
        signal_label_entry.connect_changed(move |_| {
            update_local_label_requirement_ui(&url_entry, &label_entry, &ok_button);
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let label_entry_for_signal = label_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let title_entry = title_entry.clone();

        label_entry_for_signal.connect_activate(move |_| {
            submit_reference_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &title_entry,
            );
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let title_entry_for_signal = title_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let title_entry = title_entry.clone();

        title_entry_for_signal.connect_activate(move |_| {
            submit_reference_link_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &title_entry,
            );
        });
    }

    let caret_rect = cursor_rect(text_buffer, editor_view);
    let clamped_rect = clamp_rect_to_editor(caret_rect, editor_view);
    popover.set_pointing_to(Some(&clamped_rect));

    if let Some(text_area) = visible_text_area_widget_rect(editor_view) {
        let x_offset = compute_popover_x_offset_for_text_area(
            clamped_rect.x(),
            text_area.x(),
            text_area.x() + text_area.width(),
            LINK_POPOVER_WIDTH,
            LINK_POPOVER_HORIZONTAL_SAFE_PADDING,
        );
        popover.set_offset(x_offset, 0);
    }

    update_local_label_requirement_ui(&url_entry, &label_entry, &ok_button);

    popover.popup();
    url_entry.grab_focus();
}

pub fn show_insert_image_popover(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    parent_window: &gtk4::Window,
    current_file_provider: Rc<dyn Fn() -> Option<PathBuf>>,
) {
    let popover = gtk4::Popover::new();
    popover.set_has_arrow(true);
    popover.set_autohide(true);
    popover.set_position(gtk4::PositionType::Bottom);
    popover.set_can_focus(true);
    popover.add_css_class("marco-link-popover");
    popover.set_parent(editor_view);

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    root.set_margin_start(8);
    root.set_margin_end(8);
    root.set_margin_top(6);
    root.set_margin_bottom(6);
    root.set_width_request(LINK_POPOVER_WIDTH);

    let title = gtk4::Label::new(Some("Image"));
    title.set_halign(gtk4::Align::Start);
    title.add_css_class("marco-dialog-section-label");

    let url_entry = gtk4::Entry::new();
    url_entry.set_hexpand(true);
    url_entry.set_placeholder_text(Some("Image URL (required)"));
    url_entry.add_css_class("marco-search-entry");

    let browse_button = gtk4::Button::with_label("Browse");
    browse_button.add_css_class("marco-btn");
    browse_button.add_css_class("marco-btn-blue");

    let url_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    url_row.append(&url_entry);
    url_row.append(&browse_button);

    let label_entry = gtk4::Entry::new();
    label_entry.set_hexpand(true);
    label_entry.set_placeholder_text(Some("Alt text (optional)"));
    label_entry.add_css_class("marco-search-entry");

    let attribute_entry = gtk4::Entry::new();
    attribute_entry.set_hexpand(true);
    attribute_entry.set_placeholder_text(Some("Image title (optional)"));
    attribute_entry.add_css_class("marco-search-entry");

    let actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    actions.set_halign(gtk4::Align::End);
    actions.set_margin_top(2);

    let cancel_button = gtk4::Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let ok_button = gtk4::Button::with_label("Ok");
    ok_button.add_css_class("marco-btn");
    ok_button.add_css_class("marco-btn-blue");

    actions.append(&cancel_button);
    actions.append(&ok_button);

    root.append(&title);
    root.append(&url_row);
    root.append(&label_entry);
    root.append(&attribute_entry);
    root.append(&actions);

    popover.set_child(Some(&root));

    {
        let browse_button = browse_button.clone();
        let url_entry = url_entry.clone();
        let popover = popover.clone();
        let parent_window = parent_window.clone();
        let current_file_provider = current_file_provider.clone();

        let can_browse = current_file_provider().is_some();
        browse_button.set_sensitive(can_browse);
        browse_button.set_tooltip_text(Some(if can_browse {
            "Browse local file and insert path relative to current document"
        } else {
            "Save the current document first to insert a relative local path"
        }));

        browse_button.connect_clicked(move |_| {
            let parent_window = parent_window.clone();
            let url_entry = url_entry.clone();
            let popover = popover.clone();
            let current_file_provider = current_file_provider.clone();

            popover.popdown();

            glib::MainContext::default().spawn_local(async move {
                let Some(selected_path) = pick_local_file(&parent_window).await else {
                    popover.popup();
                    url_entry.grab_focus();
                    return;
                };

                let Some(current_file_path) = current_file_provider() else {
                    popover.popup();
                    url_entry.grab_focus();
                    return;
                };

                let image_path = local_link_path_relative_to_current_file(
                    selected_path.as_path(),
                    current_file_path.as_path(),
                );
                url_entry.set_text(&image_path);
                popover.popup();
                url_entry.grab_focus();
            });
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let url_entry_for_signal = url_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        url_entry_for_signal.connect_activate(move |_| {
            submit_image_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    {
        let popover = popover.clone();
        let editor_view = editor_view.clone();
        cancel_button.connect_clicked(move |_| {
            popover.popdown();
            editor_view.grab_focus();
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        ok_button.connect_clicked(move |_| {
            submit_image_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    {
        let signal_url_entry = url_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let ok_button = ok_button.clone();
        signal_url_entry.connect_changed(move |_| {
            update_local_label_requirement_ui_with_placeholders(
                &url_entry,
                &label_entry,
                &ok_button,
                "Alt text (required)",
                "Alt text (optional)",
            );
        });
    }

    {
        let signal_label_entry = label_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let ok_button = ok_button.clone();
        signal_label_entry.connect_changed(move |_| {
            update_local_label_requirement_ui_with_placeholders(
                &url_entry,
                &label_entry,
                &ok_button,
                "Alt text (required)",
                "Alt text (optional)",
            );
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let label_entry_for_signal = label_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        label_entry_for_signal.connect_activate(move |_| {
            submit_image_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let attribute_entry_for_signal = attribute_entry.clone();
        let url_entry = url_entry.clone();
        let label_entry = label_entry.clone();
        let attribute_entry = attribute_entry.clone();

        attribute_entry_for_signal.connect_activate(move |_| {
            submit_image_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &url_entry,
                &label_entry,
                &attribute_entry,
            );
        });
    }

    let caret_rect = cursor_rect(text_buffer, editor_view);
    let clamped_rect = clamp_rect_to_editor(caret_rect, editor_view);
    popover.set_pointing_to(Some(&clamped_rect));

    if let Some(text_area) = visible_text_area_widget_rect(editor_view) {
        let x_offset = compute_popover_x_offset_for_text_area(
            clamped_rect.x(),
            text_area.x(),
            text_area.x() + text_area.width(),
            LINK_POPOVER_WIDTH,
            LINK_POPOVER_HORIZONTAL_SAFE_PADDING,
        );
        popover.set_offset(x_offset, 0);
    }

    update_local_label_requirement_ui_with_placeholders(
        &url_entry,
        &label_entry,
        &ok_button,
        "Alt text (required)",
        "Alt text (optional)",
    );

    popover.popup();
    url_entry.grab_focus();
}

fn submit_link_from_popover_entries(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    popover: &gtk4::Popover,
    url_entry: &gtk4::Entry,
    label_entry: &gtk4::Entry,
    attribute_entry: &gtk4::Entry,
) {
    let url = url_entry.text().to_string();
    let label = label_entry.text().to_string();
    let attribute = attribute_entry.text().to_string();
    let trimmed_url = url.trim();

    if trimmed_url.is_empty() {
        url_entry.grab_focus();
        return;
    }

    if is_local_link_target(trimmed_url) && label.trim().is_empty() {
        label_entry.grab_focus();
        return;
    }

    insert_link_markdown(text_buffer, trimmed_url, &label, &attribute);

    popover.popdown();
    editor_view.grab_focus();
}

fn submit_reference_link_from_popover_entries(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    popover: &gtk4::Popover,
    url_entry: &gtk4::Entry,
    label_entry: &gtk4::Entry,
    title_entry: &gtk4::Entry,
) {
    let url = url_entry.text().to_string();
    let label = label_entry.text().to_string();
    let title = title_entry.text().to_string();
    let trimmed_url = url.trim();

    if trimmed_url.is_empty() {
        url_entry.grab_focus();
        return;
    }

    if is_local_link_target(trimmed_url) && label.trim().is_empty() {
        label_entry.grab_focus();
        return;
    }

    insert_reference_link_markdown(text_buffer, trimmed_url, &label, &title);

    popover.popdown();
    editor_view.grab_focus();
}

fn submit_image_from_popover_entries(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    popover: &gtk4::Popover,
    url_entry: &gtk4::Entry,
    label_entry: &gtk4::Entry,
    attribute_entry: &gtk4::Entry,
) {
    let url = url_entry.text().to_string();
    let label = label_entry.text().to_string();
    let attribute = attribute_entry.text().to_string();
    let trimmed_url = url.trim();

    if trimmed_url.is_empty() {
        url_entry.grab_focus();
        return;
    }

    if is_local_link_target(trimmed_url) && label.trim().is_empty() {
        label_entry.grab_focus();
        return;
    }

    insert_image_markdown(text_buffer, trimmed_url, &label, &attribute);

    popover.popdown();
    editor_view.grab_focus();
}

fn cursor_rect(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
) -> gtk4::gdk::Rectangle {
    let iter = text_buffer.iter_at_offset(text_buffer.cursor_position());
    let rect = editor_view.iter_location(&iter);
    let (widget_x, widget_y) =
        editor_view.buffer_to_window_coords(gtk4::TextWindowType::Widget, rect.x(), rect.y());

    gtk4::gdk::Rectangle::new(
        widget_x,
        widget_y,
        rect.width().max(1),
        rect.height().max(1),
    )
}

fn clamp_rect_to_editor(
    rect: gtk4::gdk::Rectangle,
    editor_view: &gtk4::TextView,
) -> gtk4::gdk::Rectangle {
    let view_w = editor_view.allocated_width().max(1);
    let view_h = editor_view.allocated_height().max(1);
    let w = rect.width().max(1);
    let h = rect.height().max(1);

    let max_x = (view_w - w).max(0);
    let max_y = (view_h - h).max(0);
    let x = rect.x().clamp(0, max_x);
    let y = rect.y().clamp(0, max_y);

    gtk4::gdk::Rectangle::new(x, y, w, h)
}

fn visible_text_area_widget_rect(editor_view: &gtk4::TextView) -> Option<gtk4::gdk::Rectangle> {
    let visible = editor_view.visible_rect();
    if visible.width() <= 0 || visible.height() <= 0 {
        return None;
    }

    let (left, top) =
        editor_view.buffer_to_window_coords(gtk4::TextWindowType::Widget, visible.x(), visible.y());
    let (right, bottom) = editor_view.buffer_to_window_coords(
        gtk4::TextWindowType::Widget,
        visible.x() + visible.width(),
        visible.y() + visible.height(),
    );

    let x = left.min(right);
    let y = top.min(bottom);
    let w = (right - left).abs().max(1);
    let h = (bottom - top).abs().max(1);

    Some(gtk4::gdk::Rectangle::new(x, y, w, h))
}

fn compute_popover_x_offset_for_text_area(
    cursor_x: i32,
    text_left: i32,
    text_right: i32,
    popover_width: i32,
    safe_padding: i32,
) -> i32 {
    let half = (popover_width / 2).max(1);
    let desired_left = cursor_x - half;
    let desired_right = cursor_x + half;

    let min_left = text_left + safe_padding;
    let max_right = text_right - safe_padding;

    if desired_left < min_left {
        min_left - desired_left
    } else if desired_right > max_right {
        max_right - desired_right
    } else {
        0
    }
}

fn insert_link_markdown(text_buffer: &gtk4::TextBuffer, url: &str, label: &str, attribute: &str) {
    let trimmed_url = url.trim();
    if trimmed_url.is_empty() {
        return;
    }

    let (mut start_iter, mut end_iter) = insertion_bounds(text_buffer);
    let start_offset = start_iter.offset();

    let prev_char = char_before_offset(text_buffer, start_offset);
    let next_char = char_after_offset(text_buffer, end_iter.offset());

    let markdown = build_link_markdown(trimmed_url, label, attribute);

    let prefix_space = if should_insert_space_before(prev_char) {
        " "
    } else {
        ""
    };
    let suffix_space = if should_insert_space_after(next_char) {
        " "
    } else {
        ""
    };

    let insertion = format!("{prefix_space}{markdown}{suffix_space}");

    text_buffer.begin_user_action();
    text_buffer.delete(&mut start_iter, &mut end_iter);
    text_buffer.insert(&mut start_iter, &insertion);
    text_buffer.end_user_action();

    // Place cursor at the end of inserted markdown (before any trailing separator space).
    let cursor_offset =
        start_offset + prefix_space.chars().count() as i32 + markdown.chars().count() as i32;
    let cursor_iter = text_buffer.iter_at_offset(cursor_offset);
    text_buffer.place_cursor(&cursor_iter);
}

fn insert_reference_link_markdown(
    text_buffer: &gtk4::TextBuffer,
    url: &str,
    label: &str,
    title: &str,
) {
    let trimmed_url = url.trim();
    if trimmed_url.is_empty() {
        return;
    }

    let reference_id = next_available_reference_id_for_document(text_buffer);
    let definition = build_reference_definition(&reference_id, trimmed_url, title);
    let citation_label = select_reference_citation_label(text_buffer, label, trimmed_url);
    let citation = format!("[{citation_label}][{reference_id}]");

    let (mut start_iter, mut end_iter) = insertion_bounds(text_buffer);
    let start_offset = start_iter.offset();

    let prev_char = char_before_offset(text_buffer, start_offset);
    let next_char = char_after_offset(text_buffer, end_iter.offset());

    let prefix_space = if should_insert_space_before(prev_char) {
        " "
    } else {
        ""
    };
    let suffix_space = if should_insert_space_after(next_char) {
        " "
    } else {
        ""
    };
    let insertion = format!("{prefix_space}{citation}{suffix_space}");

    text_buffer.begin_user_action();

    text_buffer.delete(&mut start_iter, &mut end_iter);
    text_buffer.insert(&mut start_iter, &insertion);

    let cursor_offset =
        start_offset + prefix_space.chars().count() as i32 + citation.chars().count() as i32;
    let cursor_iter = text_buffer.iter_at_offset(cursor_offset);
    text_buffer.place_cursor(&cursor_iter);

    append_reference_definition_to_document_end(text_buffer, &definition);

    text_buffer.end_user_action();
}

fn insert_image_markdown(text_buffer: &gtk4::TextBuffer, url: &str, label: &str, attribute: &str) {
    let trimmed_url = url.trim();
    if trimmed_url.is_empty() {
        return;
    }

    let (mut start_iter, mut end_iter) = insertion_bounds(text_buffer);
    let start_offset = start_iter.offset();

    let prev_char = char_before_offset(text_buffer, start_offset);
    let next_char = char_after_offset(text_buffer, end_iter.offset());

    let markdown = build_image_markdown(trimmed_url, label, attribute);

    let prefix_space = if should_insert_space_before(prev_char) {
        " "
    } else {
        ""
    };
    let suffix_space = if should_insert_space_after(next_char) {
        " "
    } else {
        ""
    };

    let insertion = format!("{prefix_space}{markdown}{suffix_space}");

    text_buffer.begin_user_action();
    text_buffer.delete(&mut start_iter, &mut end_iter);
    text_buffer.insert(&mut start_iter, &insertion);
    text_buffer.end_user_action();

    let cursor_offset =
        start_offset + prefix_space.chars().count() as i32 + markdown.chars().count() as i32;
    let cursor_iter = text_buffer.iter_at_offset(cursor_offset);
    text_buffer.place_cursor(&cursor_iter);
}

fn select_reference_citation_label(
    text_buffer: &gtk4::TextBuffer,
    label: &str,
    fallback_url: &str,
) -> String {
    let trimmed_label = label.trim();
    if !trimmed_label.is_empty() {
        return trimmed_label.to_string();
    }

    if let Some((start, end)) = text_buffer.selection_bounds() {
        if start.offset() != end.offset() {
            let selected = text_buffer.text(&start, &end, false).to_string();
            let trimmed_selected = selected.trim();
            if !trimmed_selected.is_empty() {
                return trimmed_selected.to_string();
            }
        }
    }

    fallback_url.to_string()
}

fn append_reference_definition_to_document_end(text_buffer: &gtk4::TextBuffer, definition: &str) {
    let start = text_buffer.start_iter();
    let end = text_buffer.end_iter();
    let document = text_buffer.text(&start, &end, false).to_string();

    let separator = if document.is_empty() || document.ends_with("\n\n") {
        ""
    } else if document.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };

    let insertion = format!("{separator}{definition}");
    let mut end_iter = text_buffer.end_iter();
    text_buffer.insert(&mut end_iter, &insertion);
}

fn build_reference_definition(reference_id: &str, url: &str, title: &str) -> String {
    let trimmed_title = title.trim();
    if trimmed_title.is_empty() {
        return format!("[{reference_id}]: {url}");
    }

    let escaped_title = trimmed_title.replace('"', "\\\"");
    format!("[{reference_id}]: {url} \"{escaped_title}\"")
}

fn next_available_reference_id_for_document(text_buffer: &gtk4::TextBuffer) -> String {
    let start = text_buffer.start_iter();
    let end = text_buffer.end_iter();
    let document = text_buffer.text(&start, &end, false).to_string();

    let used_ids = collect_used_reference_ids(&document);
    let mut index = 1usize;

    loop {
        let candidate = format!("ref{index}");
        if !used_ids.contains(&candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn collect_used_reference_ids(document: &str) -> HashSet<String> {
    let mut ids = HashSet::new();

    for line in document.lines() {
        if let Some(id) = parse_reference_definition_label(line) {
            ids.insert(id);
        }
    }

    for id in parse_reference_usage_labels(document) {
        ids.insert(id);
    }

    ids
}

fn parse_reference_definition_label(line: &str) -> Option<String> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let (label, consumed) = parse_bracket_label_at(trimmed, 0)?;

    let mut idx = consumed;
    while let Some(ch) = trimmed[idx..].chars().next() {
        if ch == ' ' || ch == '\t' {
            idx += ch.len_utf8();
            continue;
        }
        break;
    }

    if !trimmed[idx..].starts_with(':') {
        return None;
    }

    normalize_reference_id(&label)
}

fn parse_reference_usage_labels(document: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let mut idx = 0usize;

    while idx < document.len() {
        if !document[idx..].starts_with('[') {
            if let Some(ch) = document[idx..].chars().next() {
                idx += ch.len_utf8();
            } else {
                break;
            }
            continue;
        }

        let Some((first_label, after_first)) = parse_bracket_label_at(document, idx) else {
            idx += 1;
            continue;
        };

        if !document[after_first..].starts_with('[') {
            idx = after_first;
            continue;
        }

        let Some((second_label, after_second)) = parse_bracket_label_at(document, after_first)
        else {
            idx = after_first + 1;
            continue;
        };

        if second_label.is_empty() {
            if let Some(normalized) = normalize_reference_id(&first_label) {
                labels.push(normalized);
            }
        } else if let Some(normalized) = normalize_reference_id(&second_label) {
            labels.push(normalized);
        }

        idx = after_second;
    }

    labels
}

fn parse_bracket_label_at(text: &str, start: usize) -> Option<(String, usize)> {
    if !text.get(start..)?.starts_with('[') {
        return None;
    }

    let mut idx = start + 1;
    let mut escaped = false;
    let mut label = String::new();

    while idx < text.len() {
        let ch = text[idx..].chars().next()?;
        idx += ch.len_utf8();

        if escaped {
            label.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == ']' {
            return Some((label, idx));
        }

        label.push(ch);
    }

    None
}

fn normalize_reference_id(raw: &str) -> Option<String> {
    let normalized = raw
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

#[cfg(target_os = "linux")]
async fn pick_local_file(parent_window: &gtk4::Window) -> Option<PathBuf> {
    let dialog = FileChooserNative::new(
        Some("Select Local File"),
        Some(parent_window),
        FileChooserAction::Open,
        Some("_Open"),
        Some("_Cancel"),
    );

    let filter_all = gtk4::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    dialog.add_filter(&filter_all);

    let response = dialog.run_future().await;
    if response == gtk4::ResponseType::Accept {
        return dialog.file().and_then(|file| file.path());
    }

    None
}

#[cfg(target_os = "windows")]
async fn pick_local_file(_parent_window: &gtk4::Window) -> Option<PathBuf> {
    use rfd::AsyncFileDialog;

    AsyncFileDialog::new()
        .add_filter("All files", &["*"])
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

fn local_link_path_relative_to_current_file(
    target_path: &Path,
    current_file_path: &Path,
) -> String {
    let base_dir = current_file_path.parent().unwrap_or_else(|| Path::new("."));

    if let Some(relative) = diff_paths_portable(target_path, base_dir) {
        return ensure_explicit_relative_prefix(&path_to_markdown_link(relative.as_path()));
    }

    log::warn!(
        "[toolbar/link] Could not compute relative path from '{}' to '{}'; falling back to absolute path",
        base_dir.display(),
        target_path.display()
    );

    path_to_markdown_link(target_path)
}

fn path_to_markdown_link(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn ensure_explicit_relative_prefix(path: &str) -> String {
    if path.starts_with("./") || path.starts_with("../") {
        return path.to_string();
    }

    if path.is_empty() {
        "./".to_string()
    } else {
        format!("./{path}")
    }
}

fn diff_paths_portable(path: &Path, base: &Path) -> Option<PathBuf> {
    let path_components: Vec<Component<'_>> = path.components().collect();
    let base_components: Vec<Component<'_>> = base.components().collect();

    let mut common_len = 0usize;
    let shared_len = path_components.len().min(base_components.len());
    while common_len < shared_len
        && components_equal(path_components[common_len], base_components[common_len])
    {
        common_len += 1;
    }

    if path.is_absolute() && base.is_absolute() && common_len == 0 {
        return None;
    }

    let mut relative = PathBuf::new();

    for component in &base_components[common_len..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }

    for component in &path_components[common_len..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        Some(PathBuf::from("."))
    } else {
        Some(relative)
    }
}

#[cfg(target_os = "windows")]
fn components_equal(left: Component<'_>, right: Component<'_>) -> bool {
    left.as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
}

#[cfg(target_os = "linux")]
fn components_equal(left: Component<'_>, right: Component<'_>) -> bool {
    left == right
}

fn insertion_bounds(text_buffer: &gtk4::TextBuffer) -> (gtk4::TextIter, gtk4::TextIter) {
    if let Some((start, end)) = text_buffer.selection_bounds() {
        if start.offset() != end.offset() {
            return (start, end);
        }
    }

    let cursor = text_buffer.iter_at_offset(text_buffer.cursor_position());
    (cursor, cursor)
}

fn char_before_offset(text_buffer: &gtk4::TextBuffer, offset: i32) -> Option<char> {
    if offset <= 0 {
        return None;
    }

    let prev = text_buffer.iter_at_offset(offset - 1);
    let curr = text_buffer.iter_at_offset(offset);
    text_buffer.text(&prev, &curr, false).chars().next()
}

fn char_after_offset(text_buffer: &gtk4::TextBuffer, offset: i32) -> Option<char> {
    if offset >= text_buffer.char_count() {
        return None;
    }

    let curr = text_buffer.iter_at_offset(offset);
    let next = text_buffer.iter_at_offset(offset + 1);
    text_buffer.text(&curr, &next, false).chars().next()
}

fn should_insert_space_before(prev_char: Option<char>) -> bool {
    prev_char.is_some_and(is_wordish)
}

fn should_insert_space_after(next_char: Option<char>) -> bool {
    next_char.is_some_and(is_wordish)
}

fn is_wordish(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn build_link_markdown(url: &str, label_input: &str, attribute_input: &str) -> String {
    let label = label_input.trim();
    let attribute = attribute_input.trim();

    if label.is_empty() && attribute.is_empty() {
        return url.to_string();
    }

    let final_label = if label.is_empty() { url } else { label };
    if attribute.is_empty() {
        format!("[{final_label}]({url})")
    } else {
        let escaped_attribute = attribute.replace('"', "\\\"");
        format!("[{final_label}]({url} \"{escaped_attribute}\")")
    }
}

fn build_image_markdown(url: &str, label_input: &str, attribute_input: &str) -> String {
    let label = label_input.trim();
    let attribute = attribute_input.trim();

    if attribute.is_empty() {
        format!("![{label}]({url})")
    } else {
        let escaped_attribute = attribute.replace('"', "\\\"");
        format!("![{label}]({url} \"{escaped_attribute}\")")
    }
}

fn update_local_label_requirement_ui(
    url_entry: &gtk4::Entry,
    label_entry: &gtk4::Entry,
    ok_button: &gtk4::Button,
) {
    update_local_label_requirement_ui_with_placeholders(
        url_entry,
        label_entry,
        ok_button,
        "Label link text (required)",
        "Label link text (optional)",
    );
}

fn update_local_label_requirement_ui_with_placeholders(
    url_entry: &gtk4::Entry,
    label_entry: &gtk4::Entry,
    ok_button: &gtk4::Button,
    required_placeholder: &str,
    optional_placeholder: &str,
) {
    let is_local = is_local_link_target(url_entry.text().trim());

    if is_local {
        label_entry.set_placeholder_text(Some(required_placeholder));
        ok_button.set_sensitive(!label_entry.text().trim().is_empty());
    } else {
        label_entry.set_placeholder_text(Some(optional_placeholder));
        ok_button.set_sensitive(true);
    }
}

fn is_local_link_target(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Anchor-only links are in-document, not local file links.
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return false;
    }

    if is_windows_drive_path(trimmed) {
        return true;
    }

    // Explicit schemes are treated as non-local, except file://
    if let Some(scheme_end) = trimmed.find(':') {
        let scheme = &trimmed[..scheme_end];
        let looks_like_scheme = !scheme.is_empty()
            && scheme
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'));

        if looks_like_scheme {
            return scheme.eq_ignore_ascii_case("file");
        }
    }

    trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || trimmed.contains('/')
        || trimmed.contains('\\')
}

fn is_windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn find_button_by_css_class(root: &gtk4::Widget, css_class: &str) -> Option<gtk4::Button> {
    if let Ok(button) = root.clone().downcast::<gtk4::Button>() {
        if button.has_css_class(css_class) {
            return Some(button);
        }
    }

    let mut child = root.first_child();
    while let Some(widget) = child {
        if let Some(found) = find_button_by_css_class(&widget, css_class) {
            return Some(found);
        }
        child = widget.next_sibling();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_builds_regular_link_without_attribute() {
        let markdown = build_link_markdown("https://example.com", "Example", "");
        assert_eq!(markdown, "[Example](https://example.com)");
    }

    #[test]
    fn smoke_test_builds_regular_link_with_attribute() {
        let markdown = build_link_markdown("https://example.com", "Example", "Title");
        assert_eq!(markdown, "[Example](https://example.com \"Title\")");
    }

    #[test]
    fn smoke_test_builds_raw_url_when_label_and_attribute_empty() {
        let markdown = build_link_markdown("https://example.com", "", "");
        assert_eq!(markdown, "https://example.com");
    }

    #[test]
    fn smoke_test_uses_url_as_label_when_only_attribute_present() {
        let markdown = build_link_markdown("https://example.com", "", "My title");
        assert_eq!(
            markdown,
            "[https://example.com](https://example.com \"My title\")"
        );
    }

    #[test]
    fn smoke_test_escapes_attribute_quotes() {
        let markdown = build_link_markdown("https://example.com", "Example", "A \"quote\"");
        assert_eq!(
            markdown,
            "[Example](https://example.com \"A \\\"quote\\\"\")"
        );
    }

    #[test]
    fn smoke_test_builds_image_markdown_without_title() {
        let markdown = build_image_markdown("https://example.com/img.png", "Alt", "");
        assert_eq!(markdown, "![Alt](https://example.com/img.png)");
    }

    #[test]
    fn smoke_test_builds_image_markdown_with_title_and_escaping() {
        let markdown = build_image_markdown("https://example.com/img.png", "Alt", "A \"quote\"");
        assert_eq!(
            markdown,
            "![Alt](https://example.com/img.png \"A \\\"quote\\\"\")"
        );
    }

    #[test]
    fn smoke_test_space_before_is_added_for_adjacent_word_char() {
        assert!(should_insert_space_before(Some('a')));
        assert!(should_insert_space_before(Some('_')));
    }

    #[test]
    fn smoke_test_space_before_not_added_for_whitespace_or_none() {
        assert!(!should_insert_space_before(Some(' ')));
        assert!(!should_insert_space_before(None));
    }

    #[test]
    fn smoke_test_space_after_is_added_for_adjacent_word_char() {
        assert!(should_insert_space_after(Some('w')));
        assert!(should_insert_space_after(Some('9')));
    }

    #[test]
    fn smoke_test_space_after_not_added_for_whitespace_or_none() {
        assert!(!should_insert_space_after(Some('\n')));
        assert!(!should_insert_space_after(None));
    }

    #[test]
    fn smoke_test_offset_pushes_right_when_near_text_left() {
        let offset = compute_popover_x_offset_for_text_area(50, 40, 640, 280, 8);
        assert!(offset > 0);
    }

    #[test]
    fn smoke_test_offset_pushes_left_when_near_text_right() {
        let offset = compute_popover_x_offset_for_text_area(620, 40, 640, 280, 8);
        assert!(offset < 0);
    }

    #[test]
    fn smoke_test_offset_zero_when_cursor_has_room() {
        let offset = compute_popover_x_offset_for_text_area(340, 40, 640, 280, 8);
        assert_eq!(offset, 0);
    }

    #[test]
    fn smoke_test_build_reference_definition_without_title() {
        let definition = build_reference_definition("ref3", "https://example.com", "");
        assert_eq!(definition, "[ref3]: https://example.com");
    }

    #[test]
    fn smoke_test_build_reference_definition_with_escaped_title() {
        let definition = build_reference_definition("ref3", "https://example.com", "A \"quote\"");
        assert_eq!(
            definition,
            "[ref3]: https://example.com \"A \\\"quote\\\"\""
        );
    }

    #[test]
    fn smoke_test_collect_used_reference_ids_from_definitions_and_usages() {
        let doc = "[ref1]: https://a\n[x][ref2]\n[Ref 3][]\n";
        let ids = collect_used_reference_ids(doc);

        assert!(ids.contains("ref1"));
        assert!(ids.contains("ref2"));
        assert!(ids.contains("ref 3"));
    }

    #[test]
    fn smoke_test_next_available_reference_prefers_smallest_missing_number() {
        let doc = "[ref1]: https://a\n[ref3]: https://c\n";
        let ids = collect_used_reference_ids(doc);

        assert!(ids.contains("ref1"));
        assert!(ids.contains("ref3"));
        assert!(!ids.contains("ref2"));
    }

    #[test]
    fn smoke_test_parse_reference_definition_allows_whitespace_before_colon() {
        let parsed = parse_reference_definition_label("   [Ref 7]   : https://example.com");
        assert_eq!(parsed.as_deref(), Some("ref 7"));
    }

    #[test]
    fn smoke_test_relative_link_for_sibling_file_gets_explicit_dot_prefix() {
        let current = Path::new("/docs/current.md");
        let target = Path::new("/docs/image.png");

        let link = local_link_path_relative_to_current_file(target, current);
        assert_eq!(link, "./image.png");
    }

    #[test]
    fn smoke_test_relative_link_for_parent_directory_uses_dotdot() {
        let current = Path::new("/docs/nested/current.md");
        let target = Path::new("/docs/image.png");

        let link = local_link_path_relative_to_current_file(target, current);
        assert_eq!(link, "../image.png");
    }

    #[test]
    fn smoke_test_detects_local_link_targets() {
        assert!(is_local_link_target("./assets/image.png"));
        assert!(is_local_link_target("../docs/file.md"));
        assert!(is_local_link_target("/usr/share/doc/readme.md"));
        assert!(is_local_link_target("file:///home/user/file.md"));
        assert!(is_local_link_target("C:/Users/test/file.md"));
        assert!(is_local_link_target("folder/sub/file.md"));
    }

    #[test]
    fn smoke_test_does_not_detect_remote_or_anchor_as_local() {
        assert!(!is_local_link_target("https://example.com"));
        assert!(!is_local_link_target("mailto:test@example.com"));
        assert!(!is_local_link_target("#section-id"));
        assert!(!is_local_link_target("//cdn.example.com/lib.js"));
    }
}
