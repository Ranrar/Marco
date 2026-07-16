//! Search Window Creation and Management
//!
//! Handles window creation, dialog management, and window behavior.

use gtk4::prelude::*;
use gtk4::{Label, Window};

use super::state::{AsyncSearchManager, ASYNC_MANAGER};

/// Setup all window behavior and signal connections
pub fn setup_window_behavior(
    _window: &Window,
    search_entry: &gtk4::Entry,
    replace_entry: &gtk4::Entry,
    match_count_label: &Label,
    options_widgets: &(gtk4::Box, super::ui::OptionsWidgets),
    button_widgets: &(gtk4::Box, super::ui::ButtonWidgets),
) {
    use super::engine::{debounced_search, perform_search};
    use super::navigation::immediate_position_update_with_debounced_navigation;
    use super::replace::{replace_all_matches, replace_next_match};
    use super::state::*;
    use log::debug;

    // Search entry live updates (when text is typed in the entry)
    let match_count_clone = match_count_label.clone();
    let options_clone = super::ui::OptionsWidgets {
        match_case_cb: options_widgets.1.match_case_cb.clone(),
        match_whole_word_cb: options_widgets.1.match_whole_word_cb.clone(),
        match_markdown_cb: options_widgets.1.match_markdown_cb.clone(),
        use_regex_cb: options_widgets.1.use_regex_cb.clone(),
    };
    let search_entry_clone = search_entry.clone();

    // Connect to the entry for live updates
    let options_clone_for_changed = options_clone.clone();
    search_entry.connect_changed(move |_entry| {
        use super::engine::clear_enhanced_search_highlighting;

        // Clear old highlights immediately when text changes
        clear_enhanced_search_highlighting();

        let query = search_entry_clone.text().to_string();
        if !query.is_empty() {
            debounced_search(
                &search_entry_clone,
                &match_count_clone,
                &options_clone_for_changed,
            );
        } else {
            // Clear both state and visual highlighting
            clear_search_highlighting();
            match_count_clone.set_text("");
        }
    });

    // Connect Enter key to perform search and navigate
    let search_entry_clone_enter = search_entry.clone();
    let match_count_clone_enter = match_count_label.clone();
    let options_clone_enter = options_clone.clone();
    search_entry.connect_activate(move |_entry| {
        let query = search_entry_clone_enter.text().to_string();
        if !query.is_empty() {
            let needs_search = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());
            if needs_search {
                perform_search(
                    &search_entry_clone_enter,
                    &match_count_clone_enter,
                    &options_clone_enter,
                );
            }

            let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
            if needs_position_reset {
                let position = super::navigation::find_position_from_cursor().unwrap_or(0);
                CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = Some(position));
            }

            immediate_position_update_with_debounced_navigation(1, 100);
        }
    });

    // Previous button
    let search_entry_clone_prev = search_entry.clone();
    let match_count_clone_prev = match_count_label.clone();
    let options_clone_prev = options_clone.clone();
    button_widgets.1.prev_button.connect_clicked(move |_| {
        let needs_search = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());
        if needs_search {
            let query = search_entry_clone_prev.text().to_string();
            if !query.is_empty() {
                perform_search(
                    &search_entry_clone_prev,
                    &match_count_clone_prev,
                    &options_clone_prev,
                );
            }
        }

        let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
        if needs_position_reset {
            let position = super::navigation::find_position_before_cursor().unwrap_or(2);
            CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = Some(position));
        }

        immediate_position_update_with_debounced_navigation(-1, 200);
    });

    // Next button
    let search_entry_clone_next = search_entry.clone();
    let match_count_clone_next = match_count_label.clone();
    let options_clone_next = options_clone.clone();
    button_widgets.1.next_button.connect_clicked(move |_| {
        let needs_search = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());
        if needs_search {
            let query = search_entry_clone_next.text().to_string();
            if !query.is_empty() {
                perform_search(
                    &search_entry_clone_next,
                    &match_count_clone_next,
                    &options_clone_next,
                );
            }
        }

        let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
        if needs_position_reset {
            let position = super::navigation::find_position_from_cursor().unwrap_or(0);
            CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = Some(position));
        }

        immediate_position_update_with_debounced_navigation(1, 200);
    });

    // Replace button
    let search_entry_clone_replace = search_entry.clone();
    let replace_entry_clone_replace = replace_entry.clone();
    button_widgets.1.replace_button.connect_clicked(move |_| {
        replace_next_match(&search_entry_clone_replace, &replace_entry_clone_replace);
    });

    // Replace All button
    let search_entry_clone_replace_all = search_entry.clone();
    let replace_entry_clone_replace_all = replace_entry.clone();
    button_widgets
        .1
        .replace_all_button
        .connect_clicked(move |_| {
            replace_all_matches(
                &search_entry_clone_replace_all,
                &replace_entry_clone_replace_all,
            );
        });

    // Connect option checkboxes to re-run search when changed
    let search_entry_option = search_entry.clone();
    let match_count_option = match_count_label.clone();
    let options_for_options = options_clone.clone();
    for checkbox in [
        &options_widgets.1.match_case_cb,
        &options_widgets.1.match_whole_word_cb,
        &options_widgets.1.match_markdown_cb,
        &options_widgets.1.use_regex_cb,
    ]
    .iter()
    {
        let search_entry_clone = search_entry_option.clone();
        let match_count_clone = match_count_option.clone();
        let options_clone = options_for_options.clone();
        checkbox.connect_toggled(move |_cb| {
            let query = search_entry_clone.text().to_string();
            if !query.is_empty() {
                perform_search(&search_entry_clone, &match_count_clone, &options_clone);
            }
        });
    }

    // Enable/disable replace buttons based on replace text
    let replace_button_clone = button_widgets.1.replace_button.clone();
    let replace_all_button_clone = button_widgets.1.replace_all_button.clone();
    replace_entry.connect_changed(move |entry| {
        let has_text = !entry.text().is_empty();
        replace_button_clone.set_sensitive(has_text);
        replace_all_button_clone.set_sensitive(has_text);
    });

    debug!("Window behavior setup completed");
}

/// Initialize async manager
pub fn initialize_async_manager() {
    ASYNC_MANAGER.with(|manager_ref| {
        if manager_ref.borrow().is_none() {
            *manager_ref.borrow_mut() = Some(AsyncSearchManager::new());
        }
    });
}
