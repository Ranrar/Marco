use crate::footer::{FooterLabels, FooterUpdate};
use crate::logic::signal_manager::safe_source_remove;
use gtk4::glib::ControlFlow;
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

/// Wires up debounced footer updates to buffer events
pub fn wire_footer_updates(
    buffer: &sourceview5::Buffer,
    labels: Rc<FooterLabels>,
    insert_mode_state: Rc<RefCell<bool>>,
) {
    use std::cell::Cell;
    let debounce_ms = 300;

    let buffer_timeout_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
    let cursor_timeout_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

    let update_footer = {
        let buffer = buffer.clone();
        let insert_mode_state = Rc::clone(&insert_mode_state);
        move || {
            crate::footer_dbg!("[wire_footer_updates] update_footer closure called");
            let offset = buffer.cursor_position();
            let iter = buffer.iter_at_offset(offset);
            let row = (iter.line() + 1) as usize;
            let col = (iter.line_offset() + 1) as usize;
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string();
            let word_count = text.split_whitespace().filter(|w| !w.is_empty()).count();
            let char_count = text.chars().count();

            let is_insert = *insert_mode_state.borrow();
            let msg = FooterUpdate::Snapshot {
                row,
                col,
                words: word_count,
                chars: char_count,
                encoding: "UTF-8".to_string(),
                is_insert,
            };
            crate::footer::apply_footer_update(&labels, msg);
        }
    };

    // Debounce logic for buffer changes
    let buffer_timeout_clone = Rc::clone(&buffer_timeout_id);
    let update_footer_clone = update_footer.clone();
    buffer.connect_changed(move |_| {
        if let Some(id) = buffer_timeout_clone.replace(None) {
            safe_source_remove(id);
        }
        let buffer_timeout_clone_inner = Rc::clone(&buffer_timeout_clone);
        let update_footer_clone = update_footer_clone.clone();
        let id =
            glib::timeout_add_local(std::time::Duration::from_millis(debounce_ms), move || {
                buffer_timeout_clone_inner.set(None);
                update_footer_clone();
                ControlFlow::Break
            });
        buffer_timeout_clone.set(Some(id));
    });

    // Debounce logic for cursor position changes
    let cursor_timeout_clone = Rc::clone(&cursor_timeout_id);
    let update_footer_clone2 = update_footer.clone();
    buffer.connect_notify_local(Some("cursor-position"), move |_, _| {
        if let Some(id) = cursor_timeout_clone.replace(None) {
            safe_source_remove(id);
        }
        let cursor_timeout_clone_inner = Rc::clone(&cursor_timeout_clone);
        let update_footer_clone2 = update_footer_clone2.clone();
        let id =
            glib::timeout_add_local(std::time::Duration::from_millis(debounce_ms), move || {
                cursor_timeout_clone_inner.set(None);
                update_footer_clone2();
                ControlFlow::Break
            });
        cursor_timeout_clone.set(Some(id));
    });

    // Initial update
    update_footer();
}
