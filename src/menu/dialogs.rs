use gtk4::prelude::*;
use gtk4::{Dialog, Grid, Entry, ResponseType, Box, Orientation, Label, SpinButton, Adjustment};
use crate::{editor, localization};

/// Show shortcuts dialog
pub fn show_shortcuts_dialog(parent: &gtk4::Window) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("shortcuts.title")),
        Some(parent),
        gtk4::DialogFlags::MODAL,
        &[("OK", ResponseType::Ok)],
    );
    
    dialog.set_default_size(500, 600);
    
    let content_area = dialog.content_area();
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    
    // Scroll window for the content
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_vexpand(true);
    
    let shortcuts_box = Box::new(Orientation::Vertical, 16);
    
    // Basic Formatting Section
    let basic_section = create_shortcuts_section(
        &localization::tr("shortcuts.basic_formatting"),
        &[
            (&localization::tr("shortcuts.ctrl_b"), &localization::tr("shortcuts.bold_text")),
            (&localization::tr("shortcuts.ctrl_i"), &localization::tr("shortcuts.italic_text")),
            (&localization::tr("shortcuts.ctrl_u"), &localization::tr("shortcuts.strikethrough_text")),
            (&localization::tr("shortcuts.ctrl_k"), &localization::tr("shortcuts.insert_link")),
            (&localization::tr("shortcuts.ctrl_backtick"), &localization::tr("shortcuts.inline_code")),
        ]
    );
    shortcuts_box.append(&basic_section);
    
    // Headings Section
    let headings_section = create_shortcuts_section(
        &localization::tr("shortcuts.headings"),
        &[
            (&localization::tr("shortcuts.ctrl_1"), &localization::tr("shortcuts.heading_1")),
            (&localization::tr("shortcuts.ctrl_2"), &localization::tr("shortcuts.heading_2")),
            (&localization::tr("shortcuts.ctrl_3"), &localization::tr("shortcuts.heading_3")),
            (&localization::tr("shortcuts.ctrl_4"), &localization::tr("shortcuts.heading_4")),
            (&localization::tr("shortcuts.ctrl_5"), &localization::tr("shortcuts.heading_5")),
            (&localization::tr("shortcuts.ctrl_6"), &localization::tr("shortcuts.heading_6")),
        ]
    );
    shortcuts_box.append(&headings_section);
    
    // Lists and Quotes Section
    let lists_section = create_shortcuts_section(
        &localization::tr("shortcuts.lists_and_quotes"),
        &[
            (&localization::tr("shortcuts.ctrl_shift_8"), &localization::tr("shortcuts.bullet_list")),
            (&localization::tr("shortcuts.ctrl_shift_7"), &localization::tr("shortcuts.numbered_list")),
            (&localization::tr("shortcuts.ctrl_shift_period"), &localization::tr("shortcuts.blockquote")),
        ]
    );
    shortcuts_box.append(&lists_section);
    
    scroll.set_child(Some(&shortcuts_box));
    main_box.append(&scroll);
    content_area.append(&main_box);
    
    dialog.show();
    
    dialog.connect_response(|dialog, _response| {
        dialog.close();
    });
}

/// Create a shortcuts section with a title and list of shortcuts
fn create_shortcuts_section(title: &str, shortcuts: &[(&str, &str)]) -> gtk4::Widget {
    let section_box = Box::new(Orientation::Vertical, 8);
    
    // Section title
    let title_label = Label::new(Some(title));
    title_label.set_halign(gtk4::Align::Start);
    title_label.add_css_class("heading");
    title_label.set_markup(&format!("<b>{}</b>", title));
    section_box.append(&title_label);
    
    // Shortcuts grid
    let grid = Grid::new();
    grid.set_row_spacing(6);
    grid.set_column_spacing(20);
    grid.set_margin_start(16);
    
    for (row, (shortcut, description)) in shortcuts.iter().enumerate() {
        // Shortcut key
        let shortcut_label = Label::new(Some(shortcut));
        shortcut_label.set_halign(gtk4::Align::Start);
        shortcut_label.add_css_class("caption");
        shortcut_label.set_markup(&format!("<tt><b>{}</b></tt>", shortcut));
        grid.attach(&shortcut_label, 0, row as i32, 1, 1);
        
        // Description
        let desc_label = Label::new(Some(description));
        desc_label.set_halign(gtk4::Align::Start);
        grid.attach(&desc_label, 1, row as i32, 1, 1);
    }
    
    section_box.append(&grid);
    section_box.upcast()
}

/// Show dialog to create a custom task list with specified number of items
pub fn show_task_list_custom_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("insert.task_list_custom")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(350, 200);
    
    // Create the grid layout
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Number of items label and spin button
    let items_label = Label::new(Some("Number of tasks:"));
    items_label.set_halign(gtk4::Align::Start);
    
    // Create spin button with range 1-50, default 3
    let adjustment = Adjustment::new(3.0, 1.0, 50.0, 1.0, 5.0, 0.0);
    let items_spin = SpinButton::new(Some(&adjustment), 1.0, 0);
    items_spin.set_hexpand(true);
    
    // Preview label
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    
    // Preview text with scrolled window
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_cursor_visible(false);
    
    let preview_scroll = gtk4::ScrolledWindow::new();
    preview_scroll.set_child(Some(&preview_text));
    preview_scroll.set_size_request(300, 100);
    preview_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Update preview when spin button value changes
    let preview_buffer = preview_text.buffer();
    let update_preview = {
        let items_spin = items_spin.clone();
        let preview_buffer = preview_buffer.clone();
        move || {
            let count = items_spin.value() as usize;
            let mut preview = String::new();
            for i in 0..count.min(10) { // Show max 10 in preview
                preview.push_str(&format!("[ ] Task {}\n", i + 1));
            }
            if count > 10 {
                preview.push_str(&format!("... and {} more tasks", count - 10));
            }
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview update
    update_preview();
    
    // Connect value changed signal
    items_spin.connect_value_changed({
        let update_preview = update_preview.clone();
        move |_| {
            update_preview();
        }
    });
    
    // Add to grid
    grid.attach(&items_label, 0, 0, 1, 1);
    grid.attach(&items_spin, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_scroll, 0, 2, 2, 1);
    
    // Add grid to dialog
    dialog.content_area().append(&grid);
    
    // Focus on spin button
    items_spin.grab_focus();
    
    // Connect response
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let count = items_spin.value() as usize;
            if count > 0 {
                editor_clone.insert_custom_task_list(count);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to create a custom definition list with specified number of items
pub fn show_definition_list_custom_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("insert.definition_list_custom")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(400, 250);
    
    // Create the grid layout
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Number of items label and spin button
    let items_label = Label::new(Some("Number of definition pairs:"));
    items_label.set_halign(gtk4::Align::Start);
    
    // Create spin button with range 1-20, default 2
    let adjustment = Adjustment::new(2.0, 1.0, 20.0, 1.0, 5.0, 0.0);
    let items_spin = SpinButton::new(Some(&adjustment), 1.0, 0);
    items_spin.set_hexpand(true);
    
    // Preview label
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    
    // Preview text with scrolled window
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_cursor_visible(false);
    
    let preview_scroll = gtk4::ScrolledWindow::new();
    preview_scroll.set_child(Some(&preview_text));
    preview_scroll.set_size_request(350, 120);
    preview_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Update preview when spin button value changes
    let preview_buffer = preview_text.buffer();
    let update_preview = {
        let items_spin = items_spin.clone();
        let preview_buffer = preview_buffer.clone();
        move || {
            let count = items_spin.value() as usize;
            let mut preview = String::new();
            for i in 0..count.min(8) { // Show max 8 in preview
                if i > 0 {
                    preview.push('\n');
                }
                preview.push_str(&format!("Term {}\n: Definition of term {}.\n", i + 1, i + 1));
            }
            if count > 8 {
                preview.push_str(&format!("\n... and {} more definition pairs", count - 8));
            }
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview update
    update_preview();
    
    // Connect value changed signal
    items_spin.connect_value_changed({
        let update_preview = update_preview.clone();
        move |_| {
            update_preview();
        }
    });
    
    // Add to grid
    grid.attach(&items_label, 0, 0, 1, 1);
    grid.attach(&items_spin, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_scroll, 0, 2, 2, 1);
    
    // Add grid to dialog
    dialog.content_area().append(&grid);
    
    // Focus on spin button
    items_spin.grab_focus();
    
    // Connect response
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let count = items_spin.value() as usize;
            if count > 0 {
                editor_clone.insert_custom_definition_list(count);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert colored text
pub fn show_colored_text_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.colored_text")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(400, 200);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Text input
    let text_label = Label::new(Some("Text to color:"));
    text_label.set_halign(gtk4::Align::Start);
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Enter text here"));
    text_entry.set_hexpand(true);
    
    // Pre-fill with selected text if available
    if let Some(selected_text) = editor.get_selected_text() {
        text_entry.set_text(&selected_text);
    }
    
    // Color input
    let color_label = Label::new(Some("Color (hex or name):"));
    color_label.set_halign(gtk4::Align::Start);
    let color_entry = Entry::new();
    color_entry.set_placeholder_text(Some("#ff0000 or red"));
    color_entry.set_text("#ff0000");
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(350, 60);
    
    // Update preview function
    let update_preview = {
        let text_entry = text_entry.clone();
        let color_entry = color_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let color = color_entry.text();
            let preview = format!("<span style=\"color: {}\">{}</span>", color, 
                                if text.is_empty() { "Sample text" } else { &text });
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview
    update_preview();
    
    // Connect change signals
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    color_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 1, 1);
    grid.attach(&text_entry, 1, 0, 1, 1);
    grid.attach(&color_label, 0, 1, 1, 1);
    grid.attach(&color_entry, 1, 1, 1, 1);
    grid.attach(&preview_label, 0, 2, 2, 1);
    grid.attach(&preview_text, 0, 3, 2, 1);
    
    dialog.content_area().append(&grid);
    text_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_entry.text();
            let color = color_entry.text();
            if !text.is_empty() && !color.is_empty() {
                editor_clone.insert_colored_text(&text, &color);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert underlined text
pub fn show_underline_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.underline")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(350, 150);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    let text_label = Label::new(Some("Text to underline:"));
    text_label.set_halign(gtk4::Align::Start);
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Enter text here"));
    text_entry.set_hexpand(true);
    
    // Pre-fill with selected text if available
    if let Some(selected_text) = editor.get_selected_text() {
        text_entry.set_text(&selected_text);
    }
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(300, 50);
    
    let update_preview = {
        let text_entry = text_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let preview = format!("<u>{}</u>", if text.is_empty() { "Sample text" } else { &text });
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 1, 1);
    grid.attach(&text_entry, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_text, 0, 2, 2, 1);
    
    dialog.content_area().append(&grid);
    text_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_entry.text();
            if !text.is_empty() {
                editor_clone.insert_underline(&text);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to center text
pub fn show_center_text_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.center_text")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(350, 150);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    let text_label = Label::new(Some("Text to center:"));
    text_label.set_halign(gtk4::Align::Start);
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Enter text here"));
    text_entry.set_hexpand(true);
    
    // Pre-fill with selected text if available
    if let Some(selected_text) = editor.get_selected_text() {
        text_entry.set_text(&selected_text);
    }
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(300, 50);
    
    let update_preview = {
        let text_entry = text_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let preview = format!("<div align=\"center\">{}</div>", 
                                if text.is_empty() { "Sample text" } else { &text });
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 1, 1);
    grid.attach(&text_entry, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_text, 0, 2, 2, 1);
    
    dialog.content_area().append(&grid);
    text_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_entry.text();
            if !text.is_empty() {
                editor_clone.insert_center_text(&text);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert a comment
pub fn show_comment_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.comment")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(400, 200);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    let text_label = Label::new(Some("Comment text:"));
    text_label.set_halign(gtk4::Align::Start);
    
    // Use TextView for multi-line comment
    let text_view = gtk4::TextView::new();
    text_view.set_size_request(350, 80);
    let text_buffer = text_view.buffer();
    text_buffer.set_text("Your comment here...");
    
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_child(Some(&text_view));
    scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(350, 50);
    
    let update_preview = {
        let text_buffer = text_buffer.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false);
            let preview = format!("<!-- {} -->", text);
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    text_buffer.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 2, 1);
    grid.attach(&scroll, 0, 1, 2, 1);
    grid.attach(&preview_label, 0, 2, 2, 1);
    grid.attach(&preview_text, 0, 3, 2, 1);
    
    dialog.content_area().append(&grid);
    text_view.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false);
            if !text.trim().is_empty() {
                editor_clone.insert_comment(&text);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert an admonition (GitHub-style callout)
pub fn show_admonition_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.admonition")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(450, 300);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Type selection
    let type_label = Label::new(Some("Admonition type:"));
    type_label.set_halign(gtk4::Align::Start);
    
    let type_combo = gtk4::ComboBoxText::new();
    
    // Populate combo box with common admonitions from the helper function
    let common_admonitions = editor::MarkdownEditor::get_common_admonitions();
    for (adm_type, emoji, description) in &common_admonitions {
        let display_text = format!("{} {} ({})", emoji, adm_type.to_uppercase(), description);
        type_combo.append(Some(adm_type), &display_text);
    }
    
    // Add GitHub-style admonitions as well
    type_combo.append(Some("note"), "📝 NOTE (Note)");
    type_combo.append(Some("tip"), "💡 TIP (Tip)"); 
    type_combo.append(Some("important"), "❗ IMPORTANT (Important)");
    type_combo.append(Some("warning"), "⚠️ WARNING (Warning)");
    type_combo.append(Some("caution"), "🚨 CAUTION (Caution)");
    
    type_combo.set_active_id(Some("note"));
    
    // Content input
    let content_label = Label::new(Some("Content:"));
    content_label.set_halign(gtk4::Align::Start);
    
    let content_view = gtk4::TextView::new();
    content_view.set_size_request(400, 100);
    let content_buffer = content_view.buffer();
    content_buffer.set_text("Add your content here...");
    
    let content_scroll = gtk4::ScrolledWindow::new();
    content_scroll.set_child(Some(&content_view));
    content_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(400, 80);
    
    let preview_scroll2 = gtk4::ScrolledWindow::new();
    preview_scroll2.set_child(Some(&preview_text));
    preview_scroll2.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    let update_preview = {
        let type_combo = type_combo.clone();
        let content_buffer = content_buffer.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let selected_id = type_combo.active_id().unwrap_or_default();
            let content = content_buffer.text(&content_buffer.start_iter(), &content_buffer.end_iter(), false);
            
            // Find the corresponding emoji for the selected admonition type
            let common_admonitions = editor::MarkdownEditor::get_common_admonitions();
            let emoji = common_admonitions
                .iter()
                .find(|(adm_type, _, _)| *adm_type == selected_id.as_str())
                .map(|(_, emoji, _)| *emoji)
                .unwrap_or("📝"); // Default to note emoji
            
            let preview = format!("> :{}: **{}:** {}", emoji, selected_id.to_uppercase(), 
                                content.lines().collect::<Vec<_>>().join("\n> "));
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    type_combo.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    content_buffer.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&type_label, 0, 0, 1, 1);
    grid.attach(&type_combo, 1, 0, 1, 1);
    grid.attach(&content_label, 0, 1, 2, 1);
    grid.attach(&content_scroll, 0, 2, 2, 1);
    grid.attach(&preview_label, 0, 3, 2, 1);
    grid.attach(&preview_scroll2, 0, 4, 2, 1);
    
    dialog.content_area().append(&grid);
    content_view.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let selected_id = type_combo.active_id().unwrap_or_default();
            let content = content_buffer.text(&content_buffer.start_iter(), &content_buffer.end_iter(), false);
            if !content.trim().is_empty() {
                // Find the corresponding emoji for the selected admonition type
                let common_admonitions = editor::MarkdownEditor::get_common_admonitions();
                let emoji = common_admonitions
                    .iter()
                    .find(|(adm_type, _, _)| *adm_type == selected_id.as_str())
                    .map(|(_, emoji, _)| *emoji)
                    .unwrap_or("📝"); // Default to note emoji
                
                editor_clone.insert_admonition(emoji, &selected_id.to_uppercase(), &content);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert image with size
pub fn show_image_with_size_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.image_with_size")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(450, 250);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // URL input
    let url_label = Label::new(Some("Image URL:"));
    url_label.set_halign(gtk4::Align::Start);
    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("https://example.com/image.jpg"));
    url_entry.set_hexpand(true);
    
    // Alt text input
    let alt_label = Label::new(Some("Alt text:"));
    alt_label.set_halign(gtk4::Align::Start);
    let alt_entry = Entry::new();
    alt_entry.set_placeholder_text(Some("Description of the image"));
    
    // Width input
    let width_label = Label::new(Some("Width (px):"));
    width_label.set_halign(gtk4::Align::Start);
    let width_adjustment = Adjustment::new(300.0, 1.0, 2000.0, 1.0, 10.0, 0.0);
    let width_spin = SpinButton::new(Some(&width_adjustment), 1.0, 0);
    
    // Height input
    let height_label = Label::new(Some("Height (px):"));
    height_label.set_halign(gtk4::Align::Start);
    let height_adjustment = Adjustment::new(200.0, 1.0, 2000.0, 1.0, 10.0, 0.0);
    let height_spin = SpinButton::new(Some(&height_adjustment), 1.0, 0);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(400, 60);
    
    let update_preview = {
        let url_entry = url_entry.clone();
        let alt_entry = alt_entry.clone();
        let width_spin = width_spin.clone();
        let height_spin = height_spin.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let width = width_spin.value() as i32;
            let height = height_spin.value() as i32;
            let preview = format!(
                "<img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\">",
                if url.is_empty() { "image-url" } else { &url },
                if alt.is_empty() { "alt-text" } else { &alt },
                width, height
            );
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview and connect signals
    update_preview();
    url_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    alt_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    width_spin.connect_value_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    height_spin.connect_value_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    
    grid.attach(&url_label, 0, 0, 1, 1);
    grid.attach(&url_entry, 1, 0, 1, 1);
    grid.attach(&alt_label, 0, 1, 1, 1);
    grid.attach(&alt_entry, 1, 1, 1, 1);
    grid.attach(&width_label, 0, 2, 1, 1);
    grid.attach(&width_spin, 1, 2, 1, 1);
    grid.attach(&height_label, 0, 3, 1, 1);
    grid.attach(&height_spin, 1, 3, 1, 1);
    grid.attach(&preview_label, 0, 4, 2, 1);
    grid.attach(&preview_text, 0, 5, 2, 1);
    
    dialog.content_area().append(&grid);
    url_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let width = width_spin.value() as i32;
            let height = height_spin.value() as i32;
            if !url.is_empty() {
                editor_clone.insert_image_with_size(&url, &alt, Some(&width.to_string()), Some(&height.to_string()));
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert YouTube video
pub fn show_youtube_video_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.youtube_video")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(450, 200);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Video ID input
    let id_label = Label::new(Some("YouTube Video ID:"));
    id_label.set_halign(gtk4::Align::Start);
    let id_entry = Entry::new();
    id_entry.set_placeholder_text(Some("dQw4w9WgXcQ (from URL)"));
    id_entry.set_hexpand(true);
    
    // Title input
    let title_label = Label::new(Some("Video title (optional):"));
    title_label.set_halign(gtk4::Align::Start);
    let title_entry = Entry::new();
    title_entry.set_placeholder_text(Some("Video title"));
    
    // Help text
    let help_label = Label::new(Some("Extract the video ID from the YouTube URL (e.g., from\nhttps://www.youtube.com/watch?v=dQw4w9WgXcQ extract 'dQw4w9WgXcQ')"));
    help_label.set_halign(gtk4::Align::Start);
    help_label.add_css_class("caption");
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(400, 60);
    
    let update_preview = {
        let id_entry = id_entry.clone();
        let title_entry = title_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let id = id_entry.text();
            let title = title_entry.text();
            let preview = if !id.is_empty() {
                let display_title = if title.is_empty() { "YouTube Video" } else { &title };
                format!("[![{}](https://img.youtube.com/vi/{}/0.jpg)](https://www.youtube.com/watch?v={})", 
                       display_title, id, id)
            } else {
                "[![Video Title](https://img.youtube.com/vi/VIDEO_ID/0.jpg)](https://www.youtube.com/watch?v=VIDEO_ID)".to_string()
            };
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    id_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    title_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    
    grid.attach(&id_label, 0, 0, 1, 1);
    grid.attach(&id_entry, 1, 0, 1, 1);
    grid.attach(&title_label, 0, 1, 1, 1);
    grid.attach(&title_entry, 1, 1, 1, 1);
    grid.attach(&help_label, 0, 2, 2, 1);
    grid.attach(&preview_label, 0, 3, 2, 1);
    grid.attach(&preview_text, 0, 4, 2, 1);
    
    dialog.content_area().append(&grid);
    id_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let id = id_entry.text();
            let title = title_entry.text();
            if !id.is_empty() {
                let display_title = if title.is_empty() { "YouTube Video" } else { &title };
                editor_clone.insert_youtube_video(&id, display_title);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show HTML entity selection dialog
pub fn show_html_entity_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some("Insert HTML Entity"),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(500, 400);
    
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);
    
    let instructions = Label::new(Some("Select an HTML entity to insert:"));
    instructions.set_halign(gtk4::Align::Start);
    main_box.append(&instructions);
    
    // Create scrolled window for the entity list
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.set_size_request(-1, 300);
    
    // Create list box for entities
    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    
    // Populate with common HTML entities
    let entities = editor::MarkdownEditor::get_common_html_entities();
    for (entity_name, entity_char, description) in &entities {
        let row = gtk4::ListBoxRow::new();
        
        let entity_box = Box::new(Orientation::Horizontal, 12);
        entity_box.set_margin_top(8);
        entity_box.set_margin_bottom(8);
        entity_box.set_margin_start(12);
        entity_box.set_margin_end(12);
        
        // Entity character display
        let char_label = Label::new(Some(entity_char));
        char_label.set_size_request(30, -1);
        char_label.set_halign(gtk4::Align::Center);
        
        // Entity code
        let code_label = Label::new(Some(&format!("&{};", entity_name)));
        code_label.add_css_class("monospace");
        code_label.set_size_request(100, -1);
        code_label.set_halign(gtk4::Align::Start);
        
        // Description
        let desc_label = Label::new(Some(description));
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.set_hexpand(true);
        
        entity_box.append(&char_label);
        entity_box.append(&code_label);
        entity_box.append(&desc_label);
        
        row.set_child(Some(&entity_box));
        
        list_box.append(&row);
    }
    
    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);
    
    // Preview section
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    main_box.append(&preview_label);
    
    let preview_entry = Entry::new();
    preview_entry.set_editable(false);
    preview_entry.set_placeholder_text(Some("Selected entity will appear here"));
    main_box.append(&preview_entry);
    
    dialog.content_area().append(&main_box);
    
    // Handle selection changes
    let entities_for_preview = entities.clone();
    list_box.connect_row_selected({
        let preview_entry = preview_entry.clone();
        move |_, row| {
            if let Some(row) = row {
                let index = row.index() as usize;
                if index < entities_for_preview.len() {
                    let (entity_name, _, _) = &entities_for_preview[index];
                    preview_entry.set_text(&format!("&{};", entity_name));
                }
            }
        }
    });
    
    // Set default selection
    if let Some(first_row) = list_box.row_at_index(0) {
        list_box.select_row(Some(&first_row));
    }
    
    let entities_for_response = entities.clone();
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(selected_row) = list_box.selected_row() {
                let index = selected_row.index() as usize;
                if index < entities_for_response.len() {
                    let (entity_name, _, _) = &entities_for_response[index];
                    editor_clone.insert_html_entity(entity_name);
                }
            }
        }
        dialog.close();
    });
    
    dialog.present();
}
