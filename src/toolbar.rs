use gtk4::prelude::*;
use gtk4::{Box, Button, DropDown, Orientation, Separator, StringList};

/// Toolbar button references for updating active states
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
    toolbar.set_margin_top(5);
    toolbar.set_margin_bottom(5);
    toolbar.set_margin_start(10);
    toolbar.set_margin_end(10);

    // Create headings dropdown
    let headings_list = StringList::new(&["H1", "H2", "H3", "H4", "H5", "H6"]);
    let headings_dropdown = DropDown::new(
        Some(headings_list.upcast::<gtk4::gio::ListModel>()),
        None::<&gtk4::Expression>,
    );
    headings_dropdown.set_selected(0);
    headings_dropdown.set_tooltip_text(Some("Headings"));
    toolbar.append(&headings_dropdown);

    // Separator
    let sep1 = Separator::new(Orientation::Vertical);
    toolbar.append(&sep1);

    // Text formatting buttons
    let bold_button = Button::with_label("𝐁");
    bold_button.set_size_request(32, 32);
    bold_button.set_tooltip_text(Some("Bold"));
    toolbar.append(&bold_button);

    let italic_button = Button::with_label("𝐼");
    italic_button.set_size_request(32, 32);
    italic_button.set_tooltip_text(Some("Italic"));
    toolbar.append(&italic_button);

    let code_button = Button::with_label("< >");
    code_button.set_size_request(32, 32);
    code_button.set_tooltip_text(Some("Code"));
    toolbar.append(&code_button);

    let strikethrough_button = Button::with_label("S̶");
    strikethrough_button.set_size_request(32, 32);
    strikethrough_button.set_tooltip_text(Some("Strikethrough"));
    toolbar.append(&strikethrough_button);

    // Separator
    let sep2 = Separator::new(Orientation::Vertical);
    toolbar.append(&sep2);

    // List buttons
    let bullet_button = Button::with_label("• ");
    bullet_button.set_size_request(32, 32);
    bullet_button.set_tooltip_text(Some("Bullet List"));
    toolbar.append(&bullet_button);

    let number_button = Button::with_label("1.");
    number_button.set_size_request(32, 32);
    number_button.set_tooltip_text(Some("Numbered List"));
    toolbar.append(&number_button);

    toolbar
}