use gtk4::prelude::*;
use gtk4::{Box, Orientation, Button, Separator, DropDown, StringList};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{editor, localization};

/// Toolbar button references for updating active states
pub struct ToolbarButtons {
    pub headings_dropdown: DropDown,
    pub bold_button: Button,
    pub italic_button: Button,
    pub code_button: Button,
    pub strikethrough_button: Button,
    pub updating_dropdown: std::cell::RefCell<bool>, // Flag to prevent recursive updates
}

pub fn create_markdown_toolbar(editor: &editor::MarkdownEditor) -> (Box, Rc<RefCell<ToolbarButtons>>) {
    // BASIC SYNTAX ONLY - Markdown formatting toolbar
    let markdown_toolbar = Box::new(Orientation::Horizontal, 5);
    markdown_toolbar.set_margin_top(5);
    markdown_toolbar.set_margin_bottom(5);
    markdown_toolbar.set_margin_start(10);
    markdown_toolbar.set_margin_end(10);
    
    // Create headings dropdown
    let headings_list = StringList::new(&[
        "H1",
        "H2", 
        "H3",
        "H4",
        "H5",
        "H6",
    ]);
    
    let headings_dropdown = DropDown::new(Some(headings_list.upcast::<gtk4::gio::ListModel>()), None::<&gtk4::Expression>);
    headings_dropdown.set_selected(0); // Default to "H1"
    headings_dropdown.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.headings")));
    
    markdown_toolbar.append(&headings_dropdown);
    
    // Separator
    let sep1 = Separator::new(Orientation::Vertical);
    markdown_toolbar.append(&sep1);
    
    // Text formatting buttons (Basic)
    let bold_button = Button::with_label("𝐁");
    bold_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.bold")));
    bold_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_bold();
        }
    });
    markdown_toolbar.append(&bold_button);
    
    let italic_button = Button::with_label("𝐼");
    italic_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.italic")));
    italic_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_italic();
        }
    });
    markdown_toolbar.append(&italic_button);
    
    let code_button = Button::with_label("{}");
    code_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.inline_code")));
    code_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_inline_code();
        }
    });
    markdown_toolbar.append(&code_button);
    
    let strikethrough_button = Button::with_label("S̶");
    strikethrough_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.strikethrough")));
    strikethrough_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_strikethrough();
        }
    });
    markdown_toolbar.append(&strikethrough_button);
    
    // Store references to formatting buttons for state tracking
    let toolbar_buttons = Rc::new(RefCell::new(ToolbarButtons {
        headings_dropdown: headings_dropdown.clone(),
        bold_button: bold_button.clone(),
        italic_button: italic_button.clone(),
        code_button: code_button.clone(),
        strikethrough_button: strikethrough_button.clone(),
        updating_dropdown: std::cell::RefCell::new(false),
    }));
    
    // Set up dropdown callback after toolbar_buttons is created
    headings_dropdown.connect_selected_notify({
        let editor = editor.clone();
        let toolbar_buttons_clone = toolbar_buttons.clone();
        move |dropdown| {
            // Check if we're updating the dropdown programmatically to avoid infinite loops
            if let Ok(toolbar_btns) = toolbar_buttons_clone.try_borrow() {
                if *toolbar_btns.updating_dropdown.borrow() {
                    return; // Skip if we're updating programmatically
                }
            }
            
            let selected = dropdown.selected();
            match selected {
                0 => editor.insert_heading(1), // H1
                1 => editor.insert_heading(2), // H2
                2 => editor.insert_heading(3), // H3
                3 => editor.insert_heading(4), // H4
                4 => editor.insert_heading(5), // H5
                5 => editor.insert_heading(6), // H6
                _ => {},
            }
        }
    });
    
    // Connect cursor tracking for visual feedback
    setup_cursor_tracking(editor, toolbar_buttons.clone());
    
    // Separator
    let sep2 = Separator::new(Orientation::Vertical);
    markdown_toolbar.append(&sep2);
    
    // List buttons (Basic)
    let bullet_list_button = Button::with_label("•");
    bullet_list_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.unordered_list")));
    bullet_list_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_bullet_list();
        }
    });
    markdown_toolbar.append(&bullet_list_button);
    
    let numbered_list_button = Button::with_label("1.");
    numbered_list_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.ordered_list")));
    numbered_list_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_numbered_list();
        }
    });
    markdown_toolbar.append(&numbered_list_button);
    
    let quote_button = Button::with_label("❝");
    quote_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.blockquote")));
    quote_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_blockquote();
        }
    });
    markdown_toolbar.append(&quote_button);
    
    // Separator
    let sep3 = Separator::new(Orientation::Vertical);
    markdown_toolbar.append(&sep3);
    
    // Insert buttons (Basic)
    let link_button = Button::with_label("🔗");
    link_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.link")));
    link_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_link();
        }
    });
    markdown_toolbar.append(&link_button);
    
    let image_button = Button::with_label("🖼");
    image_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.image")));
    image_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_image();
        }
    });
    markdown_toolbar.append(&image_button);
    
    let hr_button = Button::with_label("—");
    hr_button.set_tooltip_text(Some(&localization::tr("toolbar.tooltip.horizontal_rule")));
    hr_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_horizontal_rule();
        }
    });
    markdown_toolbar.append(&hr_button);
    
    (markdown_toolbar, toolbar_buttons)
}

/// Set up cursor tracking to update toolbar button states based on formatting at cursor
fn setup_cursor_tracking(editor: &editor::MarkdownEditor, toolbar_buttons: Rc<RefCell<ToolbarButtons>>) {
    let buffer = editor.get_source_buffer().clone();
    let editor_clone = editor.clone();
    
    buffer.connect_mark_set(move |buffer, _iter, mark| {
        // Only react to cursor movement (insert mark)
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        if mark == &gtk_buffer.get_insert() {
            update_toolbar_states(&editor_clone, &toolbar_buttons);
        }
    });
}

/// Update toolbar button states based on formatting at cursor position
fn update_toolbar_states(editor: &editor::MarkdownEditor, toolbar_buttons: &Rc<RefCell<ToolbarButtons>>) {
    let buttons = toolbar_buttons.borrow();
    
    // Set flag to indicate we're updating programmatically
    *buttons.updating_dropdown.borrow_mut() = true;
    
    // Check for heading level at cursor and update dropdown
    if let Some(level) = editor.get_heading_level_at_cursor() {
        // Set dropdown to the appropriate heading level (1-6 maps to indices 0-5)
        if level >= 1 && level <= 6 {
            buttons.headings_dropdown.set_selected((level - 1) as u32);
        }
    } else {
        // No heading, set dropdown to first item (H1, index 0)
        buttons.headings_dropdown.set_selected(0);
    }
    
    // Clear the flag
    *buttons.updating_dropdown.borrow_mut() = false;
    
    // Check each formatting type and update button appearance
    if editor.is_cursor_in_format("**", "**") || editor.is_cursor_in_format("__", "__") {
        buttons.bold_button.add_css_class("active-format");
    } else {
        buttons.bold_button.remove_css_class("active-format");
    }
    
    if editor.is_cursor_in_format("*", "*") || editor.is_cursor_in_format("_", "_") {
        buttons.italic_button.add_css_class("active-format");
    } else {
        buttons.italic_button.remove_css_class("active-format");
    }
    
    if editor.is_cursor_in_format("`", "`") {
        buttons.code_button.add_css_class("active-format");
    } else {
        buttons.code_button.remove_css_class("active-format");
    }
    
    if editor.is_cursor_in_format("~~", "~~") {
        buttons.strikethrough_button.add_css_class("active-format");
    } else {
        buttons.strikethrough_button.remove_css_class("active-format");
    }
}
