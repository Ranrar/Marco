//! Insert List Dialog
//!
//! Provides a fast, keyboard-first dialog for generating Markdown list scaffolding.
//!
//! ## Features
//! - 6 list types: Bullet, Ordered, Unordered, Task (with/without dot), Definition
//! - Configurable item count (1-100, default 3)
//! - Session memory for last used list type
//! - Keyboard-optimized: Enter to insert, Esc to cancel
//! - Respects cursor position and indentation
//!
//! ## Usage
//! ```no_run
//! show_insert_list_dialog(&parent_window, &editor_buffer, &editor_view);
//! ```

use gtk4::{
    glib, prelude::*, Align, Box, Button, CheckButton, Label, Orientation, SpinButton, Window,
};
use sourceview5::{Buffer, View};
use std::cell::RefCell;

/// List type options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    Bullet,
    Ordered,
    Unordered,
    TaskWithDot,
    TaskWithoutDot,
    Definition,
}

/// List generation options
#[derive(Debug, Clone)]
pub struct ListOptions {
    pub list_type: ListType,
    pub count: u32,
}

// Session memory for last used list type (in-memory only, not persisted)
thread_local! {
    static LAST_LIST_TYPE: RefCell<ListType> = const { RefCell::new(ListType::Bullet) };
}

/// Generate Markdown list text based on options
///
/// # Examples
/// ```
/// let options = ListOptions {
///     list_type: ListType::Bullet,
///     count: 3,
/// };
/// let markdown = generate_list(&options);
/// assert_eq!(markdown, "- Item 1\n- Item 2\n- Item 3");
/// ```
pub fn generate_list(options: &ListOptions) -> String {
    let mut lines = Vec::new();

    match options.list_type {
        ListType::Bullet => {
            for i in 1..=options.count {
                lines.push(format!("- Item {}", i));
            }
        }
        ListType::Ordered => {
            for i in 1..=options.count {
                lines.push(format!("{}. Item {}", i, i));
            }
        }
        ListType::Unordered => {
            for i in 1..=options.count {
                lines.push(format!("{}) Item {}", i, i));
            }
        }
        ListType::TaskWithDot => {
            for i in 1..=options.count {
                lines.push(format!("- [ ] Task {}", i));
            }
        }
        ListType::TaskWithoutDot => {
            for i in 1..=options.count {
                lines.push(format!("[ ] Task {}", i));
            }
        }
        ListType::Definition => {
            for i in 1..=options.count {
                lines.push(format!("Term {}", i));
                lines.push(format!(": Definition {}", i));
                if i < options.count {
                    lines.push(String::new()); // Blank line between definition pairs
                }
            }
        }
    }

    lines.join("\n")
}

/// Show the Insert List dialog
///
/// # Arguments
/// * `parent` - Parent window for modal behavior
/// * `editor_buffer` - Text buffer to insert list into
/// * `editor_view` - Editor view for scrolling and focus
pub fn show_insert_list_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    // Get current theme from parent
    let theme_class = if parent.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    // Create dialog (non-modal so the editor remains usable)
    let dialog = Window::builder()
        .modal(false)
        .transient_for(parent)
        .default_width(380)
        .default_height(300)
        .resizable(false)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(theme_class);

    // Custom titlebar
    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert List",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert List dialog requires a close button");

    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    // Root container: content + bottom action frame (like Settings)
    let root = Box::new(Orientation::Vertical, 0);

    // Main content container
    let vbox = Box::new(Orientation::Vertical, 8);
    vbox.add_css_class("marco-dialog-content");
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(0);

    // ========================================================================
    // Section 1: List Type (Radio Group)
    // ========================================================================

    let type_label = Label::new(Some("Type"));
    type_label.set_halign(Align::Start);
    type_label.add_css_class("marco-dialog-section-label");
    type_label.add_css_class("marco-dialog-section-label-strong");
    vbox.append(&type_label);

    let radio_box = Box::new(Orientation::Vertical, 4);
    radio_box.set_margin_start(4);
    radio_box.set_margin_bottom(6);

    // Indicator-only radio buttons + separate text (title + description).
    // This ensures the description is perfectly aligned under its title.
    let radio_bullet = CheckButton::new();
    radio_bullet.add_css_class("marco-radio");

    let radio_ordered = CheckButton::new();
    radio_ordered.add_css_class("marco-radio");
    radio_ordered.set_group(Some(&radio_bullet));

    let radio_unordered = CheckButton::new();
    radio_unordered.add_css_class("marco-radio");
    radio_unordered.set_group(Some(&radio_bullet));

    let radio_task_dot = CheckButton::new();
    radio_task_dot.add_css_class("marco-radio");
    radio_task_dot.set_group(Some(&radio_bullet));

    let radio_task_no_dot = CheckButton::new();
    radio_task_no_dot.add_css_class("marco-radio");
    radio_task_no_dot.set_group(Some(&radio_bullet));

    let radio_definition = CheckButton::new();
    radio_definition.add_css_class("marco-radio");
    radio_definition.set_group(Some(&radio_bullet));

    let make_option_row = |button: &CheckButton, title: &str, desc: &str| {
        button.set_valign(Align::Start);
        button.set_margin_top(2);

        let row = Box::new(Orientation::Horizontal, 8);
        row.set_halign(Align::Start);

        // Clicking anywhere on the row selects the option.
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

    // Set default based on last used type
    let last_type = LAST_LIST_TYPE.with(|t| *t.borrow());
    match last_type {
        ListType::Bullet => radio_bullet.set_active(true),
        ListType::Ordered => radio_ordered.set_active(true),
        ListType::Unordered => radio_unordered.set_active(true),
        ListType::TaskWithDot => radio_task_dot.set_active(true),
        ListType::TaskWithoutDot => radio_task_no_dot.set_active(true),
        ListType::Definition => radio_definition.set_active(true),
    }

    radio_box.append(&make_option_row(
        &radio_bullet,
        "Bullet",
        "Creates a standard list using \"-\" markers.",
    ));
    radio_box.append(&make_option_row(
        &radio_ordered,
        "Ordered",
        "Creates a numbered list like \"1. Item\".",
    ));
    radio_box.append(&make_option_row(
        &radio_unordered,
        "Unordered",
        "Creates an outline-style list like \"1) Item\".",
    ));
    radio_box.append(&make_option_row(
        &radio_task_dot,
        "Task",
        "Creates a task list like \"- [ ] Task\".",
    ));
    radio_box.append(&make_option_row(
        &radio_task_no_dot,
        "Task",
        "Creates a task list without a leading bullet: \"[ ] Task\".",
    ));
    radio_box.append(&make_option_row(
        &radio_definition,
        "Definition List",
        "Creates term/definition pairs using \"Term\" + \": Definition\".",
    ));

    vbox.append(&radio_box);

    // ========================================================================
    // Section 2: Item Count (SpinButton)
    // ========================================================================

    let count_label = Label::new(Some("Items"));
    count_label.set_halign(Align::Start);
    count_label.add_css_class("marco-dialog-section-label");
    vbox.append(&count_label);

    let count_box = Box::new(Orientation::Horizontal, 8);
    count_box.set_margin_start(4);
    count_box.set_margin_bottom(8);

    let spin_button = SpinButton::with_range(1.0, 100.0, 1.0);
    spin_button.add_css_class("marco-spinbutton");
    spin_button.set_value(3.0);
    spin_button.set_width_chars(5);
    spin_button.set_numeric(true);

    count_box.append(&spin_button);
    vbox.append(&count_box);

    // ========================================================================
    // Bottom Button Row (Settings-style bottom frame)
    // ========================================================================

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

    // ========================================================================
    // Event Handlers
    // ========================================================================

    // Helper to get selected list type
    let get_selected_type = move || {
        if radio_bullet.is_active() {
            ListType::Bullet
        } else if radio_ordered.is_active() {
            ListType::Ordered
        } else if radio_unordered.is_active() {
            ListType::Unordered
        } else if radio_task_dot.is_active() {
            ListType::TaskWithDot
        } else if radio_task_no_dot.is_active() {
            ListType::TaskWithoutDot
        } else {
            ListType::Definition
        }
    };

    // Clone references for closures
    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    // Insert button action
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let spin_button = spin_button.clone();
        let dialog_weak = dialog_weak.clone();
        let get_type = get_selected_type.clone();

        insert_button.connect_clicked(move |_| {
            let list_type = get_type();
            let count = spin_button.value() as u32;

            let options = ListOptions { list_type, count };
            let list_text = generate_list(&options);

            // Save last used type
            LAST_LIST_TYPE.with(|t| *t.borrow_mut() = list_type);

            // Insert at cursor position
            insert_list_at_cursor(&editor_buffer, &editor_view, &list_text);

            // Close dialog
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    // Cancel button
    {
        let dialog_weak = dialog_weak.clone();
        cancel_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    // Titlebar close button
    {
        let dialog_weak = dialog_weak.clone();
        close_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    // ESC key handler
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

    // Focus the SpinButton by default
    spin_button.grab_focus();

    // Show dialog
    dialog.present();
}

/// Insert list text at cursor position with proper indentation
fn insert_list_at_cursor(buffer: &Buffer, view: &View, list_text: &str) {
    // Get cursor position
    let insert_mark = buffer.get_insert();
    let mut cursor_iter = buffer.iter_at_mark(&insert_mark);

    // Get current line indentation
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

    // Apply indentation to each line
    let indented_text = if indent.is_empty() {
        list_text.to_string()
    } else {
        list_text
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

    // Insert text at cursor
    buffer.insert(&mut cursor_iter, &indented_text);

    // Move cursor to end of inserted text
    let insert_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&insert_mark);

    // Scroll to show the inserted text
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);

    // Return focus to editor
    view.grab_focus();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bullet_list() {
        let options = ListOptions {
            list_type: ListType::Bullet,
            count: 3,
        };
        let result = generate_list(&options);
        assert_eq!(result, "- Item 1\n- Item 2\n- Item 3");
    }

    #[test]
    fn test_generate_ordered_list() {
        let options = ListOptions {
            list_type: ListType::Ordered,
            count: 2,
        };
        let result = generate_list(&options);
        assert_eq!(result, "1. Item 1\n2. Item 2");
    }

    #[test]
    fn test_generate_unordered_list() {
        let options = ListOptions {
            list_type: ListType::Unordered,
            count: 2,
        };
        let result = generate_list(&options);
        assert_eq!(result, "1) Item 1\n2) Item 2");
    }

    #[test]
    fn test_generate_task_with_dot() {
        let options = ListOptions {
            list_type: ListType::TaskWithDot,
            count: 2,
        };
        let result = generate_list(&options);
        assert_eq!(result, "- [ ] Task 1\n- [ ] Task 2");
    }

    #[test]
    fn test_generate_task_without_dot() {
        let options = ListOptions {
            list_type: ListType::TaskWithoutDot,
            count: 2,
        };
        let result = generate_list(&options);
        assert_eq!(result, "[ ] Task 1\n[ ] Task 2");
    }

    #[test]
    fn test_generate_definition_list() {
        let options = ListOptions {
            list_type: ListType::Definition,
            count: 2,
        };
        let result = generate_list(&options);
        assert_eq!(result, "Term 1\n: Definition 1\n\nTerm 2\n: Definition 2");
    }

    #[test]
    fn test_single_item_list() {
        let options = ListOptions {
            list_type: ListType::Bullet,
            count: 1,
        };
        let result = generate_list(&options);
        assert_eq!(result, "- Item 1");
    }

    #[test]
    fn test_max_count_list() {
        let options = ListOptions {
            list_type: ListType::Bullet,
            count: 100,
        };
        let result = generate_list(&options);
        assert_eq!(result.lines().count(), 100);
        assert!(result.starts_with("- Item 1"));
        assert!(result.ends_with("- Item 100"));
    }
}
