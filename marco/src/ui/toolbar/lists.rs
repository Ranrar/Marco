//! Toolbar list button connector
//!
//! Connects the list button to show the Insert List dialog.

use gtk4::prelude::*;
use sourceview5::{Buffer, View};

/// Connect the list toolbar button to show the insert list dialog
pub fn connect_list_toolbar_action(
    toolbar: &gtk4::Box,
    parent_window: &gtk4::ApplicationWindow,
    editor_buffer: &Buffer,
    editor_view: &View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-lists")
    {
        let parent_window = parent_window.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            crate::ui::dialogs::lists::show_insert_list_dialog(
                parent_window.upcast_ref::<gtk4::Window>(),
                &editor_buffer,
                &editor_view,
            );
        });
    }
}

/// Recursively search for a button with the given CSS class
fn find_button_by_css_class(root: &gtk4::Widget, css_class: &str) -> Option<gtk4::Button> {
    if let Ok(button) = root.clone().downcast::<gtk4::Button>() {
        if button.has_css_class(css_class) {
            return Some(button);
        }
    }

    let mut child = root.first_child();
    while let Some(widget) = child {
        if let Some(found) = find_button_by_css_class(&widget, css_class) {
            return Some(found);
        }
        child = widget.next_sibling();
    }

    None
}
