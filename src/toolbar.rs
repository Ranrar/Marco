/// Sets the height of the toolbar widget (Box or similar)
pub fn set_toolbar_height(toolbar_box: &gtk4::Box, height: i32) {
    toolbar_box.set_height_request(height);
}
use gtk4::prelude::*;
use gtk4::{Box, Button, DropDown, Orientation, Separator};

/// Toolbar button references for updating active states
// TODO: This struct is not currently used, but may be useful for managing toolbar state (e.g., enabling/disabling buttons, updating active states, or connecting signals) in the future.
pub struct ToolbarButtons {
    pub headings_dropdown: DropDown,
    pub bold_button: Button,
    pub italic_button: Button,
    pub code_button: Button,
    pub strikethrough_button: Button,
}

pub fn create_toolbar_structure() -> Box {
    // Create basic toolbar structure
    let toolbar = Box::new(Orientation::Horizontal, 5);
    toolbar.add_css_class("toolbar");
    toolbar.set_margin_top(0);
    toolbar.set_margin_bottom(0);
    toolbar.set_margin_start(0);
    toolbar.set_margin_end(0);

    // Create headings button with popover
    let headings_button = Button::with_label("H1");
    headings_button.set_size_request(16,16);
    headings_button.set_tooltip_text(Some("Headings"));
    headings_button.add_css_class("toolbar-headings-btn");

    // Create popover for headings
    let headings_popover = gtk4::Popover::new();
    headings_popover.set_parent(&headings_button);
    let popover_box = Box::new(Orientation::Vertical, 4);
    for heading in &["H1", "H2", "H3", "H4", "H5", "H6"] {
        let btn = Button::with_label(heading);
        btn.set_size_request(16, 16);
        btn.set_tooltip_text(Some(&format!("Insert {}", heading)));
        btn.add_css_class("toolbar-headings-popover-btn");
        popover_box.append(&btn);
    }
    headings_popover.set_child(Some(&popover_box));
    headings_popover.set_position(gtk4::PositionType::Bottom);
    let popover_ref = headings_popover.clone();
    headings_button.connect_clicked(move |_| {
        popover_ref.popup();
    });
    toolbar.append(&headings_button);

    // Separator
    let sep1 = Separator::new(Orientation::Vertical);
    toolbar.append(&sep1);

    // Text formatting buttons
    let bold_button = Button::with_label("𝐁");
    bold_button.set_size_request(16,16);
    bold_button.set_tooltip_text(Some("Bold"));
    bold_button.add_css_class("toolbar-btn-bold");
    toolbar.append(&bold_button);

    let italic_button = Button::with_label("𝐼");
    italic_button.set_size_request(16, 16);
    italic_button.set_tooltip_text(Some("Italic"));
    italic_button.add_css_class("toolbar-btn-italic");
    toolbar.append(&italic_button);

    let code_button = Button::with_label("< >");
    code_button.set_size_request(16, 16);
    code_button.set_tooltip_text(Some("Code"));
    code_button.add_css_class("toolbar-btn-code");
    toolbar.append(&code_button);

    let strikethrough_button = Button::with_label("S̶");
    strikethrough_button.set_size_request(16, 16);
    strikethrough_button.set_tooltip_text(Some("Strikethrough"));
    strikethrough_button.add_css_class("toolbar-btn-strikethrough");
    toolbar.append(&strikethrough_button);

    // Separator
    let sep2 = Separator::new(Orientation::Vertical);
    toolbar.append(&sep2);

    // List buttons
    let bullet_button = Button::with_label("• ");
    bullet_button.set_size_request(16, 16);
    bullet_button.set_tooltip_text(Some("Bullet List"));
    bullet_button.add_css_class("toolbar-btn-bullet");
    toolbar.append(&bullet_button);

    let number_button = Button::with_label("1.");
    number_button.set_size_request(16, 16);
    number_button.set_tooltip_text(Some("Numbered List"));
    number_button.add_css_class("toolbar-btn-number");
    toolbar.append(&number_button);

    toolbar
}

/// Loads toolbar button CSS from external file
pub fn load_toolbar_css_from_file() {
    use gtk4::{CssProvider, gdk::Display};
    use std::path::Path;
    let provider = CssProvider::new();
    let css_path = "src/assets/themes/ui_elements/toolbar.css";
    if Path::new(css_path).exists() {
        provider.load_from_path(css_path);
        gtk4::style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    } else {
        eprintln!("[toolbar] CSS file not found: {}", css_path);
    }
}