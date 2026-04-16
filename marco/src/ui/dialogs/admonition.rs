//! Insert Admonition Dialog
//!
//! Provides a list-style dialog for inserting GFM/Marco admonitions.

use gtk4::{
    glib, prelude::*, Align, Box, Button, CheckButton, Entry, EntryCompletion, Label, ListStore,
    Orientation, PolicyType, ScrolledWindow, TextView, Window,
};
use sourceview5::{Buffer, View};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdmonitionType {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
    Custom,
}

#[derive(Debug, Clone)]
struct AdmonitionOptions {
    kind: AdmonitionType,
    custom_icon: Option<String>,
    custom_title: Option<String>,
    body: String,
}

fn generate_admonition_markdown(options: &AdmonitionOptions) -> String {
    let marker = match options.kind {
        AdmonitionType::Note => "[!NOTE]".to_string(),
        AdmonitionType::Tip => "[!TIP]".to_string(),
        AdmonitionType::Important => "[!IMPORTANT]".to_string(),
        AdmonitionType::Warning => "[!WARNING]".to_string(),
        AdmonitionType::Caution => "[!CAUTION]".to_string(),
        AdmonitionType::Custom => {
            let icon = options
                .custom_icon
                .clone()
                .unwrap_or_else(|| ":smile:".to_string());
            let title = options
                .custom_title
                .clone()
                .unwrap_or_else(|| "Custom Admonition".to_string());
            format!("[{} {}]", icon, title)
        }
    };

    let mut lines = vec![format!("> {}", marker)];

    if options.body.trim().is_empty() {
        lines.push("> ".to_string());
    } else {
        lines.extend(options.body.lines().map(|line| format!("> {}", line)));
    }

    lines.join("\n")
}

fn normalize_custom_icon_input(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(shortcode) = extract_embedded_shortcode(trimmed) {
        return Some(shortcode);
    }

    if let Some(shortcode) = extract_trailing_shortcode_name(trimmed) {
        return Some(shortcode);
    }

    let normalized = marco_shared::logic::text_completion::normalize_completion_query(trimmed);
    if !normalized.is_empty()
        && normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'))
    {
        return Some(format!(":{}:", normalized));
    }

    Some(trimmed.to_string())
}

fn extract_trailing_shortcode_name(raw: &str) -> Option<String> {
    let token = raw.split_whitespace().last()?;
    let normalized = marco_shared::logic::text_completion::normalize_completion_query(token);

    if normalized.is_empty()
        || !normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'))
    {
        return None;
    }

    let candidate = format!(":{}:", normalized);
    if marco_shared::logic::text_completion::emoji_shortcodes_for_completion().contains(&candidate) {
        return Some(candidate);
    }

    None
}

fn extract_embedded_shortcode(raw: &str) -> Option<String> {
    let is_valid_shortcode_char =
        |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+');

    for (start_idx, _) in raw.match_indices(':') {
        let rest = &raw[start_idx + 1..];
        for (end_rel_idx, _) in rest.match_indices(':') {
            if end_rel_idx == 0 {
                continue;
            }

            let candidate = &rest[..end_rel_idx];
            if candidate.chars().all(is_valid_shortcode_char) {
                return Some(format!(":{}:", candidate.to_ascii_lowercase()));
            }
        }
    }

    None
}

#[allow(deprecated)]
fn attach_emoji_completion(entry: &Entry) {
    let completion = EntryCompletion::new();
    completion.set_inline_completion(true);
    completion.set_inline_selection(true);
    completion.set_popup_completion(true);
    completion.set_popup_single_match(false);
    completion.set_minimum_key_length(1);

    let model = ListStore::new(&[String::static_type(), String::static_type()]);
    for item in marco_shared::logic::text_completion::emoji_completion_items() {
        let iter = model.append();
        model.set(&iter, &[(0, &item.display), (1, &item.shortcode)]);
    }

    completion.set_model(Some(&model));
    completion.set_text_column(0);

    completion.set_match_func(|completion, key, iter| {
        let Some(model) = completion.model() else {
            return false;
        };
        let candidate: String = model.get(iter, 1);
        marco_shared::logic::text_completion::emoji_shortcode_matches_query(&candidate, key)
    });

    entry.set_completion(Some(&completion));
}

pub fn show_insert_admonition_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    let theme_class = if parent.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let dialog = Window::builder()
        .modal(false)
        .transient_for(parent)
        .default_width(520)
        .default_height(390)
        .resizable(true)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(theme_class);

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert Admonition",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert Admonition dialog requires a close button");

    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    let root = Box::new(Orientation::Vertical, 0);

    let vbox = Box::new(Orientation::Vertical, 6);
    vbox.add_css_class("marco-dialog-content");
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(0);

    let type_label = Label::new(Some("Admonition"));
    type_label.set_halign(Align::Start);
    type_label.add_css_class("marco-dialog-section-label");
    type_label.add_css_class("marco-dialog-section-label-strong");
    vbox.append(&type_label);

    let radio_box = Box::new(Orientation::Vertical, 3);
    radio_box.set_margin_start(4);
    radio_box.set_margin_bottom(4);

    let radio_note = CheckButton::new();
    radio_note.add_css_class("marco-radio");
    radio_note.set_active(true);

    let radio_tip = CheckButton::new();
    radio_tip.add_css_class("marco-radio");
    radio_tip.set_group(Some(&radio_note));

    let radio_important = CheckButton::new();
    radio_important.add_css_class("marco-radio");
    radio_important.set_group(Some(&radio_note));

    let radio_warning = CheckButton::new();
    radio_warning.add_css_class("marco-radio");
    radio_warning.set_group(Some(&radio_note));

    let radio_caution = CheckButton::new();
    radio_caution.add_css_class("marco-radio");
    radio_caution.set_group(Some(&radio_note));

    let radio_custom = CheckButton::new();
    radio_custom.add_css_class("marco-radio");
    radio_custom.set_group(Some(&radio_note));

    let make_option_row = |button: &CheckButton, title: &str, desc: &str| {
        button.set_valign(Align::Start);
        button.set_margin_top(2);

        let row = Box::new(Orientation::Horizontal, 8);
        row.set_halign(Align::Start);

        let click = gtk4::GestureClick::new();
        let button_clone = button.clone();
        click.connect_released(move |_, _n, _x, _y| {
            button_clone.set_active(true);
        });
        row.add_controller(click);

        let text_box = Box::new(Orientation::Vertical, 0);
        text_box.set_halign(Align::Start);

        let title_label = Label::new(Some(title));
        title_label.set_halign(Align::Start);
        title_label.set_xalign(0.0);
        title_label.add_css_class("marco-dialog-option-title");

        let desc_label = Label::new(Some(desc));
        desc_label.set_halign(Align::Start);
        desc_label.set_xalign(0.0);
        desc_label.add_css_class("marco-dialog-option-desc");

        text_box.append(&title_label);
        text_box.append(&desc_label);

        row.append(button);
        row.append(&text_box);
        row
    };

    radio_box.append(&make_option_row(
        &radio_note,
        "Note",
        "General information or context.",
    ));
    radio_box.append(&make_option_row(
        &radio_tip,
        "Tip",
        "Helpful advice or best practice.",
    ));
    radio_box.append(&make_option_row(
        &radio_important,
        "Important",
        "Key details users should not miss.",
    ));
    radio_box.append(&make_option_row(
        &radio_warning,
        "Warning",
        "Potential risks or dangerous outcomes.",
    ));
    radio_box.append(&make_option_row(
        &radio_caution,
        "Caution",
        "Actions that require extra care.",
    ));
    radio_box.append(&make_option_row(
        &radio_custom,
        "Custom Admonition",
        "Define your own icon and title.",
    ));

    vbox.append(&radio_box);

    let custom_row = Box::new(Orientation::Horizontal, 6);
    custom_row.set_margin_start(4);
    custom_row.set_margin_bottom(6);

    let emoji_entry = Entry::new();
    emoji_entry.set_hexpand(true);
    emoji_entry.set_placeholder_text(Some("Search emoji (e.g. smile)"));
    emoji_entry.add_css_class("marco-textfield-entry");
    emoji_entry.add_css_class("marco-admonition-custom-field");
    emoji_entry.add_css_class(theme_class);
    attach_emoji_completion(&emoji_entry);

    let title_entry = Entry::new();
    title_entry.set_hexpand(true);
    title_entry.set_placeholder_text(Some("Title"));
    title_entry.add_css_class("marco-textfield-entry");
    title_entry.add_css_class("marco-admonition-custom-field");
    title_entry.add_css_class(theme_class);

    custom_row.append(&emoji_entry);
    custom_row.append(&title_entry);
    custom_row.set_visible(true);
    custom_row.set_sensitive(false);
    vbox.append(&custom_row);

    let text_label = Label::new(Some("Text"));
    text_label.set_halign(Align::Start);
    text_label.add_css_class("marco-dialog-section-label");
    vbox.append(&text_label);

    let body_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(108)
        .hexpand(true)
        .vexpand(true)
        .build();
    body_scroll.add_css_class("marco-textfield-scroll");

    let body_text = TextView::new();
    body_text.set_wrap_mode(gtk4::WrapMode::WordChar);
    body_text.set_hexpand(true);
    body_text.set_vexpand(true);
    body_text.add_css_class("marco-textfield-view");
    body_scroll.set_child(Some(&body_text));

    vbox.append(&body_scroll);

    let cancel_button = Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let insert_button = Button::with_label("Insert");
    insert_button.add_css_class("marco-btn");
    insert_button.add_css_class("suggested-action");

    let button_box = Box::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);
    button_box.append(&cancel_button);
    button_box.append(&insert_button);

    let bottom_frame = gtk4::Frame::new(None);
    bottom_frame.add_css_class("marco-dialog-bottom-frame");
    bottom_frame.set_height_request(48);
    bottom_frame.set_vexpand(false);
    bottom_frame.set_margin_top(2);

    let bottom_inner = Box::new(Orientation::Horizontal, 0);
    bottom_inner.set_margin_start(8);
    bottom_inner.set_margin_end(8);
    bottom_inner.set_margin_top(4);
    bottom_inner.set_margin_bottom(4);
    bottom_inner.set_halign(Align::Fill);
    bottom_inner.set_valign(Align::Center);

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bottom_inner.append(&spacer);
    bottom_inner.append(&button_box);
    bottom_frame.set_child(Some(&bottom_inner));

    root.append(&vbox);
    root.append(&bottom_frame);
    dialog.set_child(Some(&root));

    let update_insert_sensitivity = {
        let radio_custom = radio_custom.clone();
        let emoji_entry = emoji_entry.clone();
        let title_entry = title_entry.clone();
        let insert_button = insert_button.clone();

        move || {
            let enabled = if radio_custom.is_active() {
                !emoji_entry.text().trim().is_empty() && !title_entry.text().trim().is_empty()
            } else {
                true
            };
            insert_button.set_sensitive(enabled);
        }
    };

    {
        let custom_row = custom_row.clone();
        let update_insert_sensitivity = update_insert_sensitivity.clone();
        radio_custom.connect_toggled(move |btn| {
            custom_row.set_sensitive(btn.is_active());
            update_insert_sensitivity();
        });
    }

    {
        let update_insert_sensitivity = update_insert_sensitivity.clone();
        emoji_entry.connect_changed(move |_| {
            update_insert_sensitivity();
        });
    }

    {
        let update_insert_sensitivity = update_insert_sensitivity.clone();
        title_entry.connect_changed(move |_| {
            update_insert_sensitivity();
        });
    }

    update_insert_sensitivity();

    let get_selected_type = move || {
        if radio_note.is_active() {
            AdmonitionType::Note
        } else if radio_tip.is_active() {
            AdmonitionType::Tip
        } else if radio_important.is_active() {
            AdmonitionType::Important
        } else if radio_warning.is_active() {
            AdmonitionType::Warning
        } else if radio_caution.is_active() {
            AdmonitionType::Caution
        } else {
            AdmonitionType::Custom
        }
    };

    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    {
        let dialog_weak = dialog_weak.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let emoji_entry = emoji_entry.clone();
        let title_entry = title_entry.clone();
        let body_text = body_text.clone();
        let get_selected_type = get_selected_type.clone();

        insert_button.connect_clicked(move |_| {
            let selected = get_selected_type();
            let body_buffer = body_text.buffer();
            let start = body_buffer.start_iter();
            let end = body_buffer.end_iter();
            let body = body_buffer.text(&start, &end, false).to_string();

            let custom_icon = if selected == AdmonitionType::Custom {
                normalize_custom_icon_input(&emoji_entry.text())
            } else {
                None
            };

            let custom_title = if selected == AdmonitionType::Custom {
                let title = title_entry.text().trim().to_string();
                if title.is_empty() {
                    None
                } else {
                    Some(title)
                }
            } else {
                None
            };

            let options = AdmonitionOptions {
                kind: selected,
                custom_icon,
                custom_title,
                body,
            };

            let markdown = generate_admonition_markdown(&options);
            insert_admonition_at_cursor(&editor_buffer, &editor_view, &markdown);

            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        cancel_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        close_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let key_controller = gtk4::EventControllerKey::new();
        let dialog_weak = dialog_weak.clone();
        key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
            if key == gtk4::gdk::Key::Escape {
                if let Some(dialog) = dialog_weak.upgrade() {
                    dialog.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        dialog.add_controller(key_controller);
    }

    body_text.grab_focus();
    dialog.present();
}

fn insert_admonition_at_cursor(buffer: &Buffer, view: &View, admonition_text: &str) {
    let insert_mark = buffer.get_insert();
    let mut cursor_iter = buffer.iter_at_mark(&insert_mark);

    let line_start = {
        let mut iter = cursor_iter;
        iter.set_line_offset(0);
        iter
    };

    let line_end = {
        let mut iter = cursor_iter;
        if !iter.ends_line() {
            iter.forward_to_line_end();
        }
        iter
    };

    let current_line = buffer.text(&line_start, &line_end, false);
    let indent = current_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    let indented_text = if indent.is_empty() {
        admonition_text.to_string()
    } else {
        admonition_text
            .lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    buffer.insert(&mut cursor_iter, &indented_text);

    let insert_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&insert_mark);
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_generate_standard_admonition() {
        let options = AdmonitionOptions {
            kind: AdmonitionType::Note,
            custom_icon: None,
            custom_title: None,
            body: "Hello\nWorld".to_string(),
        };

        let markdown = generate_admonition_markdown(&options);
        assert_eq!(markdown, "> [!NOTE]\n> Hello\n> World");
    }

    #[test]
    fn smoke_test_generate_custom_admonition() {
        let options = AdmonitionOptions {
            kind: AdmonitionType::Custom,
            custom_icon: Some(":smile:".to_string()),
            custom_title: Some("Happy".to_string()),
            body: "Body".to_string(),
        };

        let markdown = generate_admonition_markdown(&options);
        assert_eq!(markdown, "> [:smile: Happy]\n> Body");
    }

    #[test]
    fn smoke_test_normalize_custom_icon_input() {
        assert_eq!(
            normalize_custom_icon_input("smile"),
            Some(":smile:".to_string())
        );
        assert_eq!(
            normalize_custom_icon_input(":smile:"),
            Some(":smile:".to_string())
        );
        assert_eq!(
            normalize_custom_icon_input("😄 :smile:"),
            Some(":smile:".to_string())
        );
        assert_eq!(
            normalize_custom_icon_input("😄 smile"),
            Some(":smile:".to_string())
        );
        assert_eq!(normalize_custom_icon_input("😀"), Some("😀".to_string()));
    }
}
