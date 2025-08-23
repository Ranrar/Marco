//! Markdown Schema settings tab
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ComboBoxText, Align};
use std::path::Path;
use crate::logic::loaders::schema_loader::list_available_schemas;
use std::rc::Rc;


/// Builds the Markdown Schema tab UI
pub fn build_schema_tab(schema_root: &str, active_schema: Option<String>, schema_disabled: bool, on_schema_changed: Rc<dyn Fn(Option<String>)>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-schema");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    // Header (bold)
    let header = Label::new(Some("Markdown Schema"));
    header.set_halign(Align::Start);
    header.set_xalign(0.0);
    header.set_markup("<b>Markdown Schema</b>");

    // Description (subtext)
    let desc = Label::new(Some("Select the Markdown schema to use for parsing and rendering. Only one schema can be active at a time. Choose ‘Disable schema selection’ to use the default behavior."));
    desc.set_halign(Align::Start);
    desc.set_xalign(0.0);
    desc.set_wrap(true);

    // Dropdown (right-aligned)
    let schemas = list_available_schemas(Path::new(schema_root));
    let combo = ComboBoxText::new();
    combo.append_text("Disable schema selection");
    for schema in &schemas {
        combo.append_text(&schema.name);
    }
    // Optionally print the first schema's paths for debugging; suppressed in normal startup.
    // To enable, temporarily uncomment the block below.
    // if let Some(first) = schemas.get(0) {
    //     eprintln!(
    //         "[schema] folder: {}\nast: {}\nsyntax: {}",
    //         first.path.display(),
    //         first.ast_path.display(),
    //         first.syntax_path.display()
    //     );
    // }
    let active_idx = if schema_disabled {
        0
    } else {
        schemas.iter().position(|s| active_schema.as_deref() == Some(&s.name)).map(|i| i + 1).unwrap_or(0)
    };
    combo.set_active(Some(active_idx as u32));
    let schemas = schemas.clone();
    combo.set_halign(Align::End);

    // Row: header/subtext left, dropdown right
    let row = GtkBox::new(Orientation::Horizontal, 0);
    let vbox = GtkBox::new(Orientation::Vertical, 2);
    vbox.append(&header);
    vbox.append(&desc);
    row.append(&vbox);
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    row.append(&spacer);
    row.append(&combo);
    // no inline info box; debug output goes to terminal instead
    row.set_hexpand(true);
    row.set_margin_bottom(24);

    // Signal
    combo.connect_changed(move |c| {
        let idx = c.active().unwrap_or(0) as usize;
        if idx == 0 {
            // selection disabled; suppressed debug output
            on_schema_changed(None);
        } else {
            let selected = schemas[idx - 1].name.clone();
            // Selection occurred; detailed debug output is suppressed in normal startup.
            // To debug, uncomment the eprintln! block below.
            // eprintln!(
            //     "[schema] selected: {}\nfolder: {}\nast: {}\nsyntax: {}",
            //     selected,
            //     schemas[idx - 1].path.display(),
            //     schemas[idx - 1].ast_path.display(),
            //     schemas[idx - 1].syntax_path.display()
            // );
            on_schema_changed(Some(selected));
        }
    });

    container.append(&row);
    container
}
