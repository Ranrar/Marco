// HTML entity dialog - Insert HTML entities with preview
// Dialog with list of common HTML entities and preview

use gtk4::prelude::*;
use crate::menu::dialogs::common::*;
use crate::editor;

/// Show HTML entity selection dialog
pub fn show_html_entity_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some("Insert HTML Entity"),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(500, 400);
    
    let main_box = gtk4::Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);
    
    let instructions = Label::new(Some("Select an HTML entity to insert:"));
    instructions.set_halign(gtk4::Align::Start);
    main_box.append(&instructions);
    
    // Create scrolled window for the entity list
    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.set_size_request(-1, 300);
    
    // Create list box for entities
    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    
    // Populate with common HTML entities
    let entities = editor::MarkdownEditor::get_common_html_entities();
    for (entity_name, entity_char, description) in &entities {
        let row = gtk4::ListBoxRow::new();
        
        let entity_box = gtk4::Box::new(Orientation::Horizontal, 12);
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
