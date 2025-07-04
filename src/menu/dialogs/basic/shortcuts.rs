// Shortcuts dialog - Shows keyboard shortcuts reference
// Simple informational dialog with scrollable content

use gtk4::prelude::*;
use crate::menu::dialogs::common::*;
use crate::language;

/// Show shortcuts dialog
pub fn show_shortcuts_dialog(parent: &gtk4::Window) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("shortcuts.title")),
        Some(parent),
        gtk4::DialogFlags::MODAL,
        &[("OK", ResponseType::Ok)],
    );
    
    dialog.set_default_size(500, 600);
    
    let content_area = dialog.content_area();
    let main_box = create_content_box(Orientation::Vertical, 12);
    
    // Scroll window for the content
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_vexpand(true);
    
    let shortcuts_box = gtk4::Box::new(Orientation::Vertical, 16);
    
    // Basic Formatting Section
    let basic_section = create_shortcuts_section(
        &language::tr("shortcuts.basic_formatting"),
        &[
            (&language::tr("shortcuts.ctrl_b"), &language::tr("shortcuts.bold_text")),
            (&language::tr("shortcuts.ctrl_i"), &language::tr("shortcuts.italic_text")),
            (&language::tr("shortcuts.ctrl_u"), &language::tr("shortcuts.strikethrough_text")),
            (&language::tr("shortcuts.ctrl_k"), &language::tr("shortcuts.insert_link")),
            (&language::tr("shortcuts.ctrl_backtick"), &language::tr("shortcuts.inline_code")),
        ]
    );
    shortcuts_box.append(&basic_section);
    
    // Headings Section
    let headings_section = create_shortcuts_section(
        &language::tr("shortcuts.headings"),
        &[
            (&language::tr("shortcuts.ctrl_1"), &language::tr("shortcuts.heading_1")),
            (&language::tr("shortcuts.ctrl_2"), &language::tr("shortcuts.heading_2")),
            (&language::tr("shortcuts.ctrl_3"), &language::tr("shortcuts.heading_3")),
            (&language::tr("shortcuts.ctrl_4"), &language::tr("shortcuts.heading_4")),
            (&language::tr("shortcuts.ctrl_5"), &language::tr("shortcuts.heading_5")),
            (&language::tr("shortcuts.ctrl_6"), &language::tr("shortcuts.heading_6")),
        ]
    );
    shortcuts_box.append(&headings_section);
    
    // Lists and Quotes Section
    let lists_section = create_shortcuts_section(
        &language::tr("shortcuts.lists_and_quotes"),
        &[
            (&language::tr("shortcuts.ctrl_shift_8"), &language::tr("shortcuts.bullet_list")),
            (&language::tr("shortcuts.ctrl_shift_7"), &language::tr("shortcuts.numbered_list")),
            (&language::tr("shortcuts.ctrl_shift_period"), &language::tr("shortcuts.blockquote")),
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
    let section_box = gtk4::Box::new(Orientation::Vertical, 8);
    
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
