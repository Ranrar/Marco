// Admonition dialog - Insert special callout boxes with different types
// Complex dialog with standard/custom modes, emoji picker, and color selection

use crate::menu::dialogs::common::*;
use crate::{editor, language};
use gtk4::{gdk, Button, ColorButton, ComboBoxText, EmojiChooser, ToggleButton};

/// Show dialog to insert an admonition with Standard/Customize modes
pub fn show_admonition_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("advanced.admonition")),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Insert", ResponseType::Accept),
        ],
    );

    dialog.set_default_size(500, 400);

    let main_box = gtk4::Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Mode toggle buttons at the top
    let mode_box = gtk4::Box::new(Orientation::Horizontal, 0);

    let standard_toggle = ToggleButton::with_label("Standard");
    let customize_toggle = ToggleButton::with_label("Customize");

    // Group the toggle buttons (mutually exclusive)
    customize_toggle.set_group(Some(&standard_toggle));
    standard_toggle.set_active(true);

    mode_box.append(&standard_toggle);
    mode_box.append(&customize_toggle);
    main_box.append(&mode_box);

    // Standard mode widgets
    let standard_box = gtk4::Box::new(Orientation::Vertical, 12);

    // Standard type selection (only 5 types)
    let std_type_label = Label::new(Some("Type:"));
    std_type_label.set_halign(gtk4::Align::Start);
    standard_box.append(&std_type_label);

    let std_type_combo = ComboBoxText::new();
    std_type_combo.append(Some("note"), "📝 Note");
    std_type_combo.append(Some("tip"), "💡 Tip");
    std_type_combo.append(Some("important"), "❗ Important");
    std_type_combo.append(Some("warning"), "⚠️ Warning");
    std_type_combo.append(Some("caution"), "🚨 Caution");
    std_type_combo.set_active_id(Some("note"));
    standard_box.append(&std_type_combo);

    // Customize mode widgets
    let customize_box = gtk4::Box::new(Orientation::Vertical, 12);

    // Custom title input
    let custom_title_label = Label::new(Some("Title (Type):"));
    custom_title_label.set_halign(gtk4::Align::Start);
    customize_box.append(&custom_title_label);

    let custom_title_entry = Entry::new();
    custom_title_entry.set_placeholder_text(Some("Enter custom title"));
    custom_title_entry.set_text("CUSTOM");
    customize_box.append(&custom_title_entry);

    // Emoji picker button
    let emoji_label = Label::new(Some("Emoji:"));
    emoji_label.set_halign(gtk4::Align::Start);
    customize_box.append(&emoji_label);

    let emoji_button = Button::with_label("📝 Select Emoji");
    let selected_emoji = std::rc::Rc::new(std::cell::RefCell::new("📝".to_string()));

    {
        let selected_emoji = selected_emoji.clone();
        let emoji_button_for_closure = emoji_button.clone();
        emoji_button.connect_clicked(move |btn| {
            // Use the existing emoji picker
            let emoji_chooser = EmojiChooser::new();
            emoji_chooser.set_parent(btn);

            let selected_emoji = selected_emoji.clone();
            let emoji_button = emoji_button_for_closure.clone();
            emoji_chooser.connect_emoji_picked(move |chooser, emoji| {
                *selected_emoji.borrow_mut() = emoji.to_string();
                emoji_button.set_label(&format!("{} Select Emoji", emoji));
                chooser.unparent();
            });

            emoji_chooser.popup();
        });
    }

    customize_box.append(&emoji_button);

    // Color picker
    let color_label = Label::new(Some("Border Color:"));
    color_label.set_halign(gtk4::Align::Start);
    customize_box.append(&color_label);

    let color_button = ColorButton::new();
    // Set default color to blue
    let blue_rgba = gdk::RGBA::builder()
        .red(0.357) // #5bc0de
        .green(0.753)
        .blue(0.871)
        .alpha(1.0)
        .build();
    color_button.set_rgba(&blue_rgba);
    customize_box.append(&color_button);

    // Content input (shared between modes)
    let content_label = Label::new(Some("Content:"));
    content_label.set_halign(gtk4::Align::Start);

    let content_view = TextView::new();
    content_view.set_size_request(450, 100);
    let content_buffer = content_view.buffer();
    content_buffer.set_text("Add your content here...");

    let content_scroll = ScrolledWindow::new();
    content_scroll.set_child(Some(&content_view));
    content_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

    // Preview section
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);

    let preview_text = TextView::new();
    preview_text.set_editable(false);
    preview_text.set_cursor_visible(false);
    preview_text.set_size_request(450, 120);
    preview_text.set_wrap_mode(gtk4::WrapMode::Word);

    let preview_scroll = ScrolledWindow::new();
    preview_scroll.set_child(Some(&preview_text));
    preview_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

    // Add all widgets to main box
    main_box.append(&standard_box);
    main_box.append(&customize_box);
    main_box.append(&content_label);
    main_box.append(&content_scroll);
    main_box.append(&preview_label);
    main_box.append(&preview_scroll);

    dialog.content_area().append(&main_box);

    // Initially hide customize mode
    customize_box.set_visible(false);

    // Toggle mode visibility
    {
        let standard_box = standard_box.clone();
        let customize_box = customize_box.clone();
        standard_toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                standard_box.set_visible(true);
                customize_box.set_visible(false);
            }
        });
    }

    {
        let standard_box = standard_box.clone();
        let customize_box = customize_box.clone();
        customize_toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                standard_box.set_visible(false);
                customize_box.set_visible(true);
            }
        });
    }

    // Preview update function
    let update_preview = {
        let standard_toggle = standard_toggle.clone();
        let std_type_combo = std_type_combo.clone();
        let custom_title_entry = custom_title_entry.clone();
        let selected_emoji = selected_emoji.clone();
        let color_button = color_button.clone();
        let content_buffer = content_buffer.clone();
        let preview_buffer = preview_text.buffer();

        move || {
            let content = content_buffer.text(
                &content_buffer.start_iter(),
                &content_buffer.end_iter(),
                false,
            );

            let preview = if standard_toggle.is_active() {
                // Standard mode - show markdown without color indication
                let selected_id = std_type_combo.active_id().unwrap_or_default();
                let (emoji, title) = match selected_id.as_str() {
                    "note" => ("📝", "NOTE"),
                    "tip" => ("💡", "TIP"),
                    "important" => ("❗", "IMPORTANT"),
                    "warning" => ("⚠️", "WARNING"),
                    "caution" => ("🚨", "CAUTION"),
                    _ => ("📝", "NOTE"),
                };

                // Format content with proper line breaks
                let formatted_content = if content.trim().is_empty() {
                    "Add your content here...".to_string()
                } else {
                    content.trim().to_string()
                };

                format!(
                    "┌─ {} {} ─────────────────────────\n│ {}\n└─────────────────────────────────────",
                    emoji, title, formatted_content.replace('\n', "\n│ ")
                )
            } else {
                // Customize mode - show custom preview
                let title = custom_title_entry.text();
                let emoji = selected_emoji.borrow().clone();
                let rgba = color_button.rgba();
                let hex_color = format!(
                    "#{:02x}{:02x}{:02x}",
                    (rgba.red() * 255.0) as u8,
                    (rgba.green() * 255.0) as u8,
                    (rgba.blue() * 255.0) as u8
                );

                // Format content with proper line breaks
                let formatted_content = if content.trim().is_empty() {
                    "Add your content here...".to_string()
                } else {
                    content.trim().to_string()
                };

                format!(
                    "┌─ {} {} ({} border) ─────────────\n│ {}\n└─────────────────────────────────────",
                    emoji, title, hex_color, formatted_content.replace('\n', "\n│ ")
                )
            };

            preview_buffer.set_text(&preview);
        }
    };

    // Connect all update signals
    let update_preview_clone = update_preview.clone();
    std_type_combo.connect_changed(move |_| update_preview_clone());

    let update_preview_clone = update_preview.clone();
    custom_title_entry.connect_changed(move |_| update_preview_clone());

    let update_preview_clone = update_preview.clone();
    color_button.connect_color_set(move |_| update_preview_clone());

    let update_preview_clone = update_preview.clone();
    content_buffer.connect_changed(move |_| update_preview_clone());

    let update_preview_clone = update_preview.clone();
    standard_toggle.connect_toggled(move |_| update_preview_clone());

    // Initial preview update
    update_preview();

    content_view.grab_focus();

    // Handle dialog response
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let content = content_buffer.text(&content_buffer.start_iter(), &content_buffer.end_iter(), false);
            if !content.trim().is_empty() {
                if standard_toggle.is_active() {
                    // Standard mode
                    let selected_id = std_type_combo.active_id().unwrap_or_default();
                    let (emoji, title) = match selected_id.as_str() {
                        "note" => ("📝", "NOTE"),
                        "tip" => ("💡", "TIP"),
                        "important" => ("❗", "IMPORTANT"),
                        "warning" => ("⚠️", "WARNING"),
                        "caution" => ("🚨", "CAUTION"),
                        _ => ("📝", "NOTE"),
                    };
                    editor_clone.insert_admonition(emoji, title, &content);
                } else {
                    // Customize mode
                    let title = custom_title_entry.text();
                    let emoji = selected_emoji.borrow().clone();
                    let rgba = color_button.rgba();

                    // Create custom HTML with inline style for the border color
                    let hex_color = format!("#{:02x}{:02x}{:02x}",
                        (rgba.red() * 255.0) as u8,
                        (rgba.green() * 255.0) as u8,
                        (rgba.blue() * 255.0) as u8);

                    // Insert as custom HTML div with inline styling
                    let custom_admonition = format!(
                        "<div class=\"admonition\" style=\"border-left-color: {}\">\n{} <strong>{}:</strong>\n<br>\n{}\n</div>\n",
                        hex_color, emoji, title, content
                    );
                    editor_clone.insert_text_at_cursor(&custom_admonition);
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}
