/// Sets the height of the toolbar widget (Box or similar)
pub fn set_toolbar_height(toolbar_box: &gtk4::Box, height: i32) {
    toolbar_box.set_height_request(height);
}

/// Updates toolbar button tooltips with new translations (in-place, without rebuilding)
pub fn update_toolbar_translations(toolbar: &gtk4::Box, translations: &Translations) {
    use gtk4::prelude::*;

    // Toolbar children order: [headings_button, sep1, bold, italic, code, strikethrough, sep2, bullet, number]
    // Indices: 0=headings, 2=bold, 3=italic, 4=code, 5=strikethrough, 7=bullet, 8=number

    // Update headings button tooltip (index 0)
    if let Some(child) = toolbar.observe_children().item(0) {
        if let Ok(button) = child.downcast::<Button>() {
            button.set_tooltip_text(Some(&translations.toolbar.headings));

            // Update heading popover button tooltips (H1-H6)
            if let Some(popover) = button.first_child() {
                if let Ok(popover_widget) = popover.downcast::<gtk4::Popover>() {
                    if let Some(popover_child) = popover_widget.child() {
                        if let Ok(popover_box) = popover_child.downcast::<gtk4::Box>() {
                            let headings = ["H1", "H2", "H3", "H4", "H5", "H6"];
                            for (i, heading) in headings.iter().enumerate() {
                                if let Some(btn_widget) =
                                    popover_box.observe_children().item(i as u32)
                                {
                                    if let Ok(btn) = btn_widget.downcast::<Button>() {
                                        btn.set_tooltip_text(Some(&format!(
                                            "{} {}",
                                            translations.toolbar.insert, heading
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Update bold button tooltip (index 2)
    if let Some(child) = toolbar.observe_children().item(2) {
        if let Ok(button) = child.downcast::<Button>() {
            button.set_tooltip_text(Some(&translations.toolbar.bold));
        }
    }

    // Update italic button tooltip (index 3)
    if let Some(child) = toolbar.observe_children().item(3) {
        if let Ok(button) = child.downcast::<Button>() {
            button.set_tooltip_text(Some(&translations.toolbar.italic));
        }
    }

    // Update code button tooltip (index 4)
    if let Some(child) = toolbar.observe_children().item(4) {
        if let Ok(button) = child.downcast::<Button>() {
            button.set_tooltip_text(Some(&translations.toolbar.code));
        }
    }

    // Update strikethrough button tooltip (index 5)
    if let Some(child) = toolbar.observe_children().item(5) {
        if let Ok(button) = child.downcast::<Button>() {
            button.set_tooltip_text(Some(&translations.toolbar.strikethrough));
        }
    }

    // Update bullet list button tooltip (index 7)
    if let Some(child) = toolbar.observe_children().item(7) {
        if let Ok(button) = child.downcast::<Button>() {
            button.set_tooltip_text(Some(&translations.toolbar.bullet_list));
        }
    }

    // Update numbered list button tooltip (index 8)
    if let Some(child) = toolbar.observe_children().item(8) {
        if let Ok(button) = child.downcast::<Button>() {
            button.set_tooltip_text(Some(&translations.toolbar.numbered_list));
        }
    }
}

use gtk4::prelude::*;
use gtk4::{Box, Button, DropDown, Orientation, Separator};
use log::trace;

use crate::components::language::Translations;

/// Toolbar button references for updating active states
// Note: This struct is not currently used, but may be useful for managing toolbar state (e.g., enabling/disabling buttons, updating active states, or connecting signals) in the future.
#[allow(dead_code)]
pub struct ToolbarButtons {
    pub headings_dropdown: DropDown,
    pub bold_button: Button,
    pub italic_button: Button,
    pub code_button: Button,
    pub strikethrough_button: Button,
}

pub fn create_toolbar_structure(translations: &Translations) -> Box {
    // Create basic toolbar structure with spacing between buttons
    let toolbar = Box::new(Orientation::Horizontal, 4); // 4px spacing between children
    toolbar.add_css_class("toolbar");
    toolbar.set_margin_top(0);
    toolbar.set_margin_bottom(0);
    toolbar.set_margin_start(0);
    toolbar.set_margin_end(0);

    // Create headings button with popover
    let headings_button = Button::with_label("H1");
    headings_button.set_tooltip_text(Some(&translations.toolbar.headings));
    headings_button.add_css_class("toolbar-headings-btn");

    // Create popover for headings
    let headings_popover = gtk4::Popover::new();
    headings_popover.set_parent(&headings_button);
    let popover_box = Box::new(Orientation::Vertical, 4);
    for heading in &["H1", "H2", "H3", "H4", "H5", "H6"] {
        let btn = Button::with_label(heading);
        btn.set_tooltip_text(Some(&format!(
            "{} {}",
            translations.toolbar.insert, heading
        )));
        btn.add_css_class("toolbar-headings-popover-btn");
        popover_box.append(&btn);
    }
    headings_popover.set_child(Some(&popover_box));
    headings_popover.set_position(gtk4::PositionType::Bottom);
    let popover_ref = headings_popover.clone();
    headings_button.connect_clicked(move |_| {
        popover_ref.popup();
        trace!("audit: headings button clicked (popover opened)");
    });
    toolbar.append(&headings_button);

    // Separator
    let sep1 = Separator::new(Orientation::Vertical);
    sep1.add_css_class("toolbar-separator");
    toolbar.append(&sep1);

    // Text formatting buttons
    let bold_button = Button::with_label("𝐁");
    bold_button.set_tooltip_text(Some(&translations.toolbar.bold));
    bold_button.add_css_class("toolbar-btn-bold");
    toolbar.append(&bold_button);

    let italic_button = Button::with_label("𝐼");
    italic_button.set_tooltip_text(Some(&translations.toolbar.italic));
    italic_button.add_css_class("toolbar-btn-italic");
    toolbar.append(&italic_button);

    let code_button = Button::with_label("{ }");
    code_button.set_tooltip_text(Some(&translations.toolbar.code));
    code_button.add_css_class("toolbar-btn-code");
    toolbar.append(&code_button);

    let strikethrough_button = Button::with_label("S̶");
    strikethrough_button.set_tooltip_text(Some(&translations.toolbar.strikethrough));
    strikethrough_button.add_css_class("toolbar-btn-strikethrough");
    toolbar.append(&strikethrough_button);

    // Separator
    let sep2 = Separator::new(Orientation::Vertical);
    sep2.add_css_class("toolbar-separator");
    toolbar.append(&sep2);

    // List buttons
    let bullet_button = Button::with_label("• ");
    bullet_button.set_tooltip_text(Some(&translations.toolbar.bullet_list));
    bullet_button.add_css_class("toolbar-btn-bullet");
    toolbar.append(&bullet_button);

    let number_button = Button::with_label("1.");
    number_button.set_tooltip_text(Some(&translations.toolbar.numbered_list));
    number_button.add_css_class("toolbar-btn-number");
    toolbar.append(&number_button);

    toolbar
}
