//! Insert Table Dialog
//!
//! Provides a simple table-builder dialog for toolbar -> modules -> table insertion.

use gtk4::{
    glib, prelude::*, Align, Box, Button, CheckButton, Label, Orientation, SpinButton,
    ToggleButton, Window,
};
use sourceview5::{Buffer, View};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct TableOptions {
    pub columns: u32,
    pub rows: u32,
    pub include_header_row: bool,
    pub edit_column_alignment: bool,
    pub alignments: Vec<ColumnAlignment>,
}

fn alignment_rule(alignment: ColumnAlignment) -> &'static str {
    match alignment {
        ColumnAlignment::Left => ":---",
        ColumnAlignment::Center => ":---:",
        ColumnAlignment::Right => "---:",
    }
}

fn build_row(values: &[String]) -> String {
    format!("| {} |", values.join(" | "))
}

pub fn generate_table_markdown(options: &TableOptions) -> String {
    let columns = options.columns.max(1) as usize;
    let rows = options.rows.max(1) as usize;

    let mut lines: Vec<String> = Vec::new();

    if options.include_header_row {
        let header_cells: Vec<String> =
            (1..=columns).map(|idx| format!("Column {}", idx)).collect();
        lines.push(build_row(&header_cells));

        let delimiter_cells: Vec<String> = (0..columns)
            .map(|idx| {
                if options.edit_column_alignment {
                    let alignment = options
                        .alignments
                        .get(idx)
                        .copied()
                        .unwrap_or(ColumnAlignment::Left);
                    alignment_rule(alignment).to_string()
                } else {
                    "---".to_string()
                }
            })
            .collect();
        lines.push(build_row(&delimiter_cells));

        for row in 1..=rows {
            let body_cells: Vec<String> = (1..=columns)
                .map(|col| format!("Cell {}-{}", row, col))
                .collect();
            lines.push(build_row(&body_cells));
        }
    } else {
        let delimiter_cells: Vec<String> = (0..columns)
            .map(|idx| {
                if options.edit_column_alignment {
                    let alignment = options
                        .alignments
                        .get(idx)
                        .copied()
                        .unwrap_or(ColumnAlignment::Left);
                    alignment_rule(alignment).to_string()
                } else {
                    "--------".to_string()
                }
            })
            .collect();
        lines.push(build_row(&delimiter_cells));

        for row in 1..=rows {
            let body_cells: Vec<String> = (1..=columns)
                .map(|col| format!("Data {}-{}", row, col))
                .collect();
            lines.push(build_row(&body_cells));
        }
    }

    lines.join("\n")
}

fn insert_table_at_cursor(buffer: &Buffer, view: &View, table_text: &str) {
    let insert_mark = buffer.get_insert();
    let mut cursor_iter = buffer.iter_at_mark(&insert_mark);

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

    let indented_text = if indent.is_empty() {
        table_text.to_string()
    } else {
        table_text
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

    buffer.insert(&mut cursor_iter, &indented_text);

    let insert_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&insert_mark);
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

fn create_alignment_row(
    column_index: u32,
    state: std::rc::Rc<std::cell::RefCell<Vec<ColumnAlignment>>>,
) -> Box {
    let row = Box::new(Orientation::Horizontal, 8);
    row.set_halign(Align::Fill);

    let col_label = Label::new(Some(&format!("Col {}", column_index + 1)));
    col_label.set_halign(Align::Start);
    col_label.set_xalign(0.0);
    col_label.set_width_chars(7);
    col_label.add_css_class("marco-dialog-option-title");

    let toggles_box = Box::new(Orientation::Horizontal, 0);
    toggles_box.add_css_class("marco-segmented-control");

    let left_toggle = ToggleButton::with_label("Left");
    let center_toggle = ToggleButton::with_label("Center");
    let right_toggle = ToggleButton::with_label("Right");

    left_toggle.add_css_class("marco-segmented-toggle");
    center_toggle.add_css_class("marco-segmented-toggle");
    right_toggle.add_css_class("marco-segmented-toggle");

    left_toggle.set_group(Some(&center_toggle));
    right_toggle.set_group(Some(&center_toggle));

    left_toggle.set_active(true);

    {
        let state = state.clone();
        left_toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                if let Ok(mut alignments) = state.try_borrow_mut() {
                    alignments[column_index as usize] = ColumnAlignment::Left;
                }
            }
        });
    }

    {
        let state = state.clone();
        center_toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                if let Ok(mut alignments) = state.try_borrow_mut() {
                    alignments[column_index as usize] = ColumnAlignment::Center;
                }
            }
        });
    }

    {
        let state = state.clone();
        right_toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                if let Ok(mut alignments) = state.try_borrow_mut() {
                    alignments[column_index as usize] = ColumnAlignment::Right;
                }
            }
        });
    }

    toggles_box.append(&left_toggle);
    toggles_box.append(&center_toggle);
    toggles_box.append(&right_toggle);

    row.append(&col_label);
    row.append(&toggles_box);
    row
}

pub fn show_insert_table_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    let theme_class = if parent.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let dialog = Window::builder()
        .modal(false)
        .transient_for(parent)
        .default_width(520)
        .default_height(420)
        .resizable(true)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(theme_class);

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert Table",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert Table dialog requires a close button");

    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    let root = Box::new(Orientation::Vertical, 0);

    let vbox = Box::new(Orientation::Vertical, 8);
    vbox.add_css_class("marco-dialog-content");
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(0);

    // Columns
    let columns_row = Box::new(Orientation::Horizontal, 8);
    columns_row.set_halign(Align::Fill);
    columns_row.set_margin_start(4);

    let columns_label = Label::new(Some("Columns"));
    columns_label.set_halign(Align::Start);
    columns_label.set_xalign(0.0);
    columns_label.set_width_chars(10);
    columns_label.add_css_class("marco-dialog-option-title");

    let columns_adj = gtk4::Adjustment::new(4.0, 1.0, 12.0, 1.0, 1.0, 0.0);
    let columns_spin = SpinButton::new(Some(&columns_adj), 1.0, 0);
    columns_spin.add_css_class("marco-spinbutton");
    columns_spin.set_width_chars(5);
    columns_spin.set_numeric(true);

    columns_row.append(&columns_label);
    columns_row.append(&columns_spin);
    vbox.append(&columns_row);

    // Rows
    let rows_row = Box::new(Orientation::Horizontal, 8);
    rows_row.set_halign(Align::Fill);
    rows_row.set_margin_start(4);

    let rows_label = Label::new(Some("Rows"));
    rows_label.set_halign(Align::Start);
    rows_label.set_xalign(0.0);
    rows_label.set_width_chars(10);
    rows_label.add_css_class("marco-dialog-option-title");

    let rows_adj = gtk4::Adjustment::new(1.0, 1.0, 100.0, 1.0, 1.0, 0.0);
    let rows_spin = SpinButton::new(Some(&rows_adj), 1.0, 0);
    rows_spin.add_css_class("marco-spinbutton");
    rows_spin.set_width_chars(5);
    rows_spin.set_numeric(true);

    rows_row.append(&rows_label);
    rows_row.append(&rows_spin);
    vbox.append(&rows_row);

    let selected_label = Label::new(Some("Selected: 4 Columns × 1 Rows"));
    selected_label.set_halign(Align::Start);
    selected_label.set_xalign(0.0);
    selected_label.set_margin_start(4);
    selected_label.set_margin_top(4);
    selected_label.add_css_class("marco-dialog-option-desc");
    vbox.append(&selected_label);

    let include_header_check = CheckButton::with_label("Include header row");
    include_header_check.add_css_class("marco-checkbutton");
    include_header_check.set_active(true);
    include_header_check.set_margin_start(4);
    include_header_check.set_margin_top(4);

    let edit_alignment_check = CheckButton::with_label("Edit column alignment");
    edit_alignment_check.add_css_class("marco-checkbutton");
    edit_alignment_check.set_active(true);
    edit_alignment_check.set_margin_start(4);

    vbox.append(&include_header_check);
    vbox.append(&edit_alignment_check);

    let alignment_section = Box::new(Orientation::Vertical, 6);
    alignment_section.set_margin_start(4);
    alignment_section.set_margin_top(2);
    alignment_section.set_margin_bottom(4);

    let alignment_title = Label::new(Some("Column Alignment"));
    alignment_title.set_halign(Align::Start);
    alignment_title.set_xalign(0.0);
    alignment_title.add_css_class("marco-dialog-section-label");
    alignment_section.append(&alignment_title);

    let alignment_rows_container = Box::new(Orientation::Vertical, 6);
    alignment_rows_container.set_halign(Align::Fill);
    alignment_section.append(&alignment_rows_container);

    vbox.append(&alignment_section);

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

    let alignment_state = std::rc::Rc::new(std::cell::RefCell::new(vec![ColumnAlignment::Left; 4]));

    let rebuild_alignment_rows = {
        let alignment_rows_container = alignment_rows_container.clone();
        let alignment_state = alignment_state.clone();

        move |columns: u32| {
            let mut child = alignment_rows_container.first_child();
            while let Some(widget) = child {
                let next = widget.next_sibling();
                alignment_rows_container.remove(&widget);
                child = next;
            }

            if let Ok(mut state) = alignment_state.try_borrow_mut() {
                state.resize(columns as usize, ColumnAlignment::Left);
            }

            for column_index in 0..columns {
                let row = create_alignment_row(column_index, alignment_state.clone());
                alignment_rows_container.append(&row);
            }
        }
    };

    rebuild_alignment_rows(4);

    {
        let selected_label = selected_label.clone();
        let rows_spin = rows_spin.clone();
        columns_spin.connect_value_changed(move |spin| {
            let columns = spin.value() as u32;
            let rows = rows_spin.value() as u32;
            selected_label.set_text(&format!("Selected: {} Columns × {} Rows", columns, rows));
        });
    }

    {
        let selected_label = selected_label.clone();
        let columns_spin = columns_spin.clone();
        rows_spin.connect_value_changed(move |spin| {
            let rows = spin.value() as u32;
            let columns = columns_spin.value() as u32;
            selected_label.set_text(&format!("Selected: {} Columns × {} Rows", columns, rows));
        });
    }

    {
        let rebuild_alignment_rows = rebuild_alignment_rows.clone();
        columns_spin.connect_value_changed(move |spin| {
            rebuild_alignment_rows(spin.value() as u32);
        });
    }

    {
        let alignment_section = alignment_section.clone();
        edit_alignment_check.connect_toggled(move |btn| {
            alignment_section.set_visible(btn.is_active());
        });
    }

    alignment_section.set_visible(edit_alignment_check.is_active());

    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    {
        let columns_spin = columns_spin.clone();
        let rows_spin = rows_spin.clone();
        let include_header_check = include_header_check.clone();
        let edit_alignment_check = edit_alignment_check.clone();
        let alignment_state = alignment_state.clone();
        let dialog_weak = dialog_weak.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();

        insert_button.connect_clicked(move |_| {
            let columns = columns_spin.value() as u32;
            let rows = rows_spin.value() as u32;
            let include_header_row = include_header_check.is_active();
            let edit_column_alignment = edit_alignment_check.is_active();

            let alignments = alignment_state
                .try_borrow()
                .map(|state| state.clone())
                .unwrap_or_else(|_| vec![ColumnAlignment::Left; columns as usize]);

            let options = TableOptions {
                columns,
                rows,
                include_header_row,
                edit_column_alignment,
                alignments,
            };

            let markdown = generate_table_markdown(&options);
            insert_table_at_cursor(&editor_buffer, &editor_view, &markdown);

            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        cancel_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        close_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

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

    columns_spin.grab_focus();
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_generate_table_with_header_and_default_alignment() {
        let options = TableOptions {
            columns: 2,
            rows: 2,
            include_header_row: true,
            edit_column_alignment: false,
            alignments: vec![ColumnAlignment::Left, ColumnAlignment::Left],
        };

        let output = generate_table_markdown(&options);
        let expected = "| Column 1 | Column 2 |\n| --- | --- |\n| Cell 1-1 | Cell 1-2 |\n| Cell 2-1 | Cell 2-2 |";
        assert_eq!(output, expected);
    }

    #[test]
    fn smoke_test_generate_table_with_alignment() {
        let options = TableOptions {
            columns: 3,
            rows: 1,
            include_header_row: true,
            edit_column_alignment: true,
            alignments: vec![
                ColumnAlignment::Left,
                ColumnAlignment::Center,
                ColumnAlignment::Right,
            ],
        };

        let output = generate_table_markdown(&options);
        assert!(output.contains("| :--- | :---: | ---: |"));
    }

    #[test]
    fn smoke_test_generate_headerless_table() {
        let options = TableOptions {
            columns: 2,
            rows: 2,
            include_header_row: false,
            edit_column_alignment: false,
            alignments: vec![ColumnAlignment::Left, ColumnAlignment::Left],
        };

        let output = generate_table_markdown(&options);
        let expected = "| -------- | -------- |\n| Data 1-1 | Data 1-2 |\n| Data 2-1 | Data 2-2 |";
        assert_eq!(output, expected);
    }
}
