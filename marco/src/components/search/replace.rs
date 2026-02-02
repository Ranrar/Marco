//! Replace Operations
//!
//! Handles replace next and replace all functionality.

use gtk4::prelude::*;
use gtk4::Entry;
use log::debug;
use super::state::*;

/// Scroll the editor to show the match at the given position
fn scroll_to_match(match_iter: &gtk4::TextIter) {
    CURRENT_SOURCE_VIEW.with(|view_ref| {
        if let Some(source_view) = view_ref.borrow().as_ref() {
            // Check if the source view has proper allocation before scrolling
            let allocation = source_view.allocation();
            if allocation.width() <= 0 || allocation.height() <= 0 {
                debug!("Skipping scroll operation - SourceView has no allocation");
                return;
            }

            // Create a mutable copy of the iterator for scroll_to_iter
            let mut iter_copy = *match_iter;

            // Scroll to the match position with some margin
            source_view.scroll_to_iter(&mut iter_copy, 0.1, true, 0.0, 0.3);

            debug!("Scrolled editor to show match at line {}", match_iter.line() + 1);
        } else {
            debug!("No source view available for scrolling");
        }
    });
}

/// Replace the next match in the buffer
pub fn replace_next_match(search_entry: &Entry, replace_entry: &Entry) {
    let query = search_entry.text().to_string();
    let replacement = replace_entry.text().to_string();

    if query.is_empty() {
        debug!("Replace next: query is empty");
        return;
    }

    debug!("Replacing next match: '{}' -> '{}'", query, replacement);

    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    buffer.begin_user_action();

                    // Get current cursor position
                    let cursor_iter = buffer.iter_at_offset(buffer.cursor_position());

                    // If there's a selection, start search from the beginning of selection
                    // Otherwise start from cursor position
                    let search_start = if buffer.has_selection() {
                        let (start_iter, _) = buffer.selection_bounds().unwrap();
                        start_iter
                    } else {
                        cursor_iter
                    };

                    // Find the next match from the search start position
                    if let Some((match_start, match_end, _has_wrapped)) =
                        search_state.search_context.forward(&search_start)
                    {
                        // Create marks to preserve positions across buffer modifications
                        let start_mark = buffer.create_mark(None, &match_start, false);
                        let end_mark = buffer.create_mark(None, &match_end, true);

                        // Use SearchContext's replace method - this respects all search settings
                        let mut start_iter = match_start;
                        let mut end_iter = match_end;
                        match search_state.search_context.replace(
                            &mut start_iter,
                            &mut end_iter,
                            &replacement,
                        ) {
                            Ok(()) => {
                                debug!(
                                    "Successfully replaced match: '{}' -> '{}'",
                                    query, replacement
                                );

                                // Get the position after replacement using the mark
                                let replacement_end_iter = buffer.iter_at_mark(&start_mark);
                                let mut search_from_iter = replacement_end_iter;

                                // Move the search position forward by the replacement length
                                search_from_iter.forward_chars(replacement.len() as i32);
                                buffer.place_cursor(&search_from_iter);

                                // Find and select the next match for easy continuation
                                if let Some((next_start, next_end, _)) =
                                    search_state.search_context.forward(&search_from_iter)
                                {
                                    buffer.select_range(&next_start, &next_end);

                                    // Scroll to show the next match
                                    scroll_to_match(&next_start);

                                    // Position display is automatically updated by cursor-based navigation
                                } else {
                                    debug!("No more matches found after replacement");
                                }

                                // Clean up marks
                                buffer.delete_mark(&start_mark);
                                buffer.delete_mark(&end_mark);
                            }
                            Err(e) => {
                                debug!("Replace operation failed: {}", e);

                                // Clean up marks even on error
                                buffer.delete_mark(&start_mark);
                                buffer.delete_mark(&end_mark);
                            }
                        }
                    } else {
                        debug!("No matches found to replace");
                    }

                    buffer.end_user_action();
                } else {
                    debug!("No buffer available for replace operation");
                }
            });
        } else {
            debug!("No active search state - please perform a search first");
        }
    });
}

/// Replace all matches in the buffer
pub fn replace_all_matches(search_entry: &Entry, replace_entry: &Entry) {
    let query = search_entry.text().to_string();
    let replacement = replace_entry.text().to_string();

    if query.is_empty() {
        debug!("Replace all: query is empty");
        return;
    }

    debug!("Replacing all matches: '{}' -> '{}'", query, replacement);

    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    buffer.begin_user_action();

                    // Use SearchContext's replace_all method
                    match search_state.search_context.replace_all(&replacement) {
                        Ok(()) => {
                            debug!(
                                "Replace all completed successfully: '{}' -> '{}'",
                                query, replacement
                            );

                            // Update match count display after replacement
                            CURRENT_MATCH_LABEL.with(|label_ref| {
                                if let Some(label) = label_ref.borrow().as_ref() {
                                    // After replace all, there should be no matches left for the old query
                                    label.set_text("No matches");
                                }
                            });

                            // Clear current selection since all matches were replaced
                            if buffer.has_selection() {
                                let cursor_mark = buffer.get_insert();
                                let cursor_iter = buffer.iter_at_mark(&cursor_mark);
                                buffer.place_cursor(&cursor_iter);
                            }
                        }
                        Err(e) => {
                            debug!("Replace all failed: {}", e);

                            // Update match count display to show error
                            CURRENT_MATCH_LABEL.with(|label_ref| {
                                if let Some(label) = label_ref.borrow().as_ref() {
                                    label.set_text("Replace failed");
                                }
                            });
                        }
                    }

                    buffer.end_user_action();
                } else {
                    debug!("No buffer available for replace all operation");
                }
            });
        } else {
            debug!("No active search state - please perform a search first");
        }
    });
}
