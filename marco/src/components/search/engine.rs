//! Search Engine - Search Logic and Highlighting
//!
//! Performs searches, manages highlighting, and handles search operations.

use super::state::*;
use super::ui::OptionsWidgets;
use gtk4::prelude::*;
use gtk4::{Entry, Label};
use log::debug;
use sourceview5::prelude::*;
use sourceview5::{SearchContext, SearchSettings};

/// Apply enhanced dual-color search highlighting
///
/// Uses two distinct colors:
/// - Standard highlights for all matches (from search-match style)
/// - Enhanced highlight for the current selected match (from search-match-selected style)
///
/// # Theme Requirements
/// The theme files should define both:
/// - `search-match` style for regular matches
/// - `search-match-selected` style for the selected match
pub fn apply_enhanced_search_highlighting(
    search_context: &SearchContext,
    current_match_start: Option<&gtk4::TextIter>,
    current_match_end: Option<&gtk4::TextIter>,
) {
    CURRENT_BUFFER.with(|buffer_ref| {
        if let Some(buffer) = buffer_ref.borrow().as_ref() {
            // FIRST: Clear any existing tags to prevent accumulation
            let tag_table = buffer.tag_table();
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();

            // Remove all search-related tags that might exist
            if let Some(tag) = tag_table.lookup("search-match-selected-custom") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("search-match") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("search-occurrence") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }

            // Get the style scheme to check for available styles
            if let Some(style_scheme) = buffer.style_scheme() {
                // Check if we have the enhanced highlighting styles
                let has_selected_style = style_scheme.style("search-match-selected").is_some();

                if has_selected_style {
                    debug!("Applying enhanced search highlighting with dual colors");

                    // Apply standard highlighting to all matches
                    search_context.set_highlight(true);

                    // If we have a current match, add additional highlighting for the selected match
                    if let (Some(start), Some(end)) = (current_match_start, current_match_end) {
                        // Create a text tag for the selected match highlighting
                        let tag_table = buffer.tag_table();

                        // Check if we already have a selected match tag, or create a new one
                        let selected_tag = if let Some(existing_tag) = tag_table.lookup("search-match-selected-custom") {
                            existing_tag
                        } else {
                            let new_tag = gtk4::TextTag::new(Some("search-match-selected-custom"));

                            // Get the colors from the style scheme
                            if let Some(selected_style) = style_scheme.style("search-match-selected") {
                                // Apply the style properties from the scheme
                                if let Some(bg_color) = selected_style.background() {
                                    new_tag.set_background(Some(&bg_color));
                                }
                                if let Some(fg_color) = selected_style.foreground() {
                                    new_tag.set_foreground(Some(&fg_color));
                                }
                                if selected_style.is_bold() {
                                    new_tag.set_weight(700); // Bold weight
                                }
                            } else {
                                // Fallback colors if style is not found
                                new_tag.set_background(Some("#FF6B35")); // Orange background
                                new_tag.set_foreground(Some("#FFFFFF")); // White text
                                new_tag.set_weight(700); // Bold weight
                            }

                            tag_table.add(&new_tag);
                            new_tag
                        };

                        // Remove any existing selected match highlighting
                        let start_iter = buffer.start_iter();
                        let end_iter = buffer.end_iter();
                        buffer.remove_tag(&selected_tag, &start_iter, &end_iter);

                        // Apply the selected match highlighting to the current match
                        buffer.apply_tag(&selected_tag, start, end);

                        let line_number = start.line() + 1;
                        debug!("Applied enhanced highlighting to current match at line {}", line_number);
                    }
                } else {
                    debug!("Enhanced highlighting styles not found in theme, using standard highlighting");
                    search_context.set_highlight(true);
                }
            } else {
                debug!("No style scheme available, using default highlighting");
                search_context.set_highlight(true);
            }
        }
    });
}

/// Clear enhanced search highlighting including custom selected match tags
pub fn clear_enhanced_search_highlighting() {
    CURRENT_BUFFER.with(|buffer_ref| {
        if let Some(buffer) = buffer_ref.borrow().as_ref() {
            // Clear standard search highlighting
            CURRENT_SEARCH_STATE.with(|state_ref| {
                if let Some(search_state) = state_ref.borrow().as_ref() {
                    search_state.search_context.set_highlight(false);
                }
            });

            // Get tag table for manual tag removal
            let tag_table = buffer.tag_table();
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();

            // Remove custom selected match highlighting
            if let Some(selected_tag) = tag_table.lookup("search-match-selected-custom") {
                buffer.remove_tag(&selected_tag, &start_iter, &end_iter);
                debug!("Removed custom selected match tag");
            }

            // IMPORTANT: Remove SourceView's built-in search highlighting tags
            // These persist even after set_highlight(false)
            if let Some(search_match_tag) = tag_table.lookup("search-match") {
                buffer.remove_tag(&search_match_tag, &start_iter, &end_iter);
                debug!("Removed search-match tag");
            }

            // Also try common variants that might exist
            if let Some(search_occurrence_tag) = tag_table.lookup("search-occurrence") {
                buffer.remove_tag(&search_occurrence_tag, &start_iter, &end_iter);
                debug!("Removed search-occurrence tag");
            }

            debug!("Cleared all search highlighting tags");
        }
    });
}

/// Perform search operation
pub fn perform_search(search_entry: &Entry, match_count_label: &Label, options: &OptionsWidgets) {
    let query = search_entry.text().to_string();
    if query.is_empty() {
        // Clear any existing search highlighting when query is empty
        clear_enhanced_search_highlighting();
        clear_search_highlighting();
        match_count_label.set_text("0 matches");
        return;
    }

    debug!("Performing search for: '{}'", query);

    // Clear any previous search highlighting before starting new search
    clear_enhanced_search_highlighting();
    clear_search_highlighting();

    // Get the current buffer from thread-local storage
    CURRENT_BUFFER.with(|buffer_ref| {
        if let Some(buffer) = buffer_ref.borrow().as_ref() {
            // Create search settings
            let search_settings = SearchSettings::new();
            search_settings.set_search_text(Some(&query));
            search_settings.set_case_sensitive(options.match_case_cb.is_active());
            search_settings.set_wrap_around(true);
            search_settings.set_at_word_boundaries(options.match_whole_word_cb.is_active());
            search_settings.set_regex_enabled(options.use_regex_cb.is_active());

            // Create search context
            let search_context = SearchContext::new(&**buffer, Some(&search_settings));

            // Apply enhanced highlighting initially (without a specific selected match)
            apply_enhanced_search_highlighting(&search_context, None, None);

            // Configure search highlighting with proper style scheme integration
            if let Some(style_scheme) = buffer.style_scheme() {
                // Check if the style scheme has enhanced highlighting styles
                if let Some(_search_match_style) = style_scheme.style("search-match") {
                    debug!("Using enhanced search highlighting with scheme '{}'", style_scheme.name());
                    if style_scheme.style("search-match-selected").is_some() {
                        debug!("Enhanced selected match highlighting available");
                    }
                } else {
                    // Log that we're using default highlighting
                    debug!("Style scheme '{}' does not define 'search-match' style, using SearchContext default highlighting", style_scheme.name());
                }
            } else {
                debug!("No style scheme set, using default highlighting");
            }

            // Store the search state for navigation functions
            CURRENT_SEARCH_STATE.with(|state_ref| {
                *state_ref.borrow_mut() = Some(SearchState {
                    search_context: search_context.clone(),
                });
            });

            // Reset match position tracking for new search
            CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = None);

            // Set up count monitoring with enhanced position tracking
            let label_clone = match_count_label.clone();
            let search_context_clone = search_context.clone();
            search_context.connect_occurrences_count_notify(move |ctx| {
                let count = ctx.occurrences_count();
                let text = if count == -1 {
                    "Searching...".to_string()
                } else if count == 0 {
                    "No matches".to_string()
                } else if count == 1 {
                    "1 match".to_string()
                } else {
                    format!("{} matches", count)
                };
                label_clone.set_text(&text);
                debug!("Match count updated: {}", count);

                // If scanning is complete and we have a current selection, update position display
                if count > 0 {
                    CURRENT_BUFFER.with(|buffer_ref| {
                        if let Some(buffer) = buffer_ref.borrow().as_ref() {
                            if buffer.has_selection() {
                                let (start_iter, end_iter) = buffer.selection_bounds().unwrap();
                                // Check if the current selection is a valid search match
                                let position = search_context_clone.occurrence_position(&start_iter, &end_iter);
                                if position > 0 {
                                    let line_number = start_iter.line() + 1;
                                    let updated_text = format!("{} of {} matches (line {})", position, count, line_number);
                                    label_clone.set_text(&updated_text);
                                    debug!("Updated position after scan completion: {}", updated_text);
                                }
                            }
                        }
                    });
                }
            });

            // Initial count display
            let match_count = search_context.occurrences_count();
            let match_text = if match_count == -1 {
                "Searching...".to_string()
            } else if match_count == 0 {
                "No matches".to_string()
            } else if match_count == 1 {
                "1 match".to_string()
            } else {
                format!("{} matches", match_count)
            };
            match_count_label.set_text(&match_text);

            debug!("Search initiated: initial count {} for '{}'", match_count, query);

            // Don't automatically navigate to first match during search setup
            // Let the user explicitly choose when to navigate with Enter key or buttons
            debug!("Search context created for '{}' with highlighting enabled", query);
        } else {
            debug!("No buffer available for search");
            match_count_label.set_text("No buffer");
        }
    });
}

/// Perform search operation asynchronously with debounce timer
pub fn perform_search_async(
    search_entry: &Entry,
    match_count_label: &Label,
    options: &OptionsWidgets,
    delay_ms: u32,
) {
    // Cancel any existing async search timer
    ASYNC_MANAGER.with(|manager_ref| {
        use crate::logic::signal_manager::safe_source_remove;
        if let Some(manager) = manager_ref.borrow_mut().as_mut() {
            if let Some(timer_id) = manager.current_timer_id.take() {
                safe_source_remove(timer_id);
            }
        }
    });

    // Clone the widgets for the async operation
    let search_entry_clone = search_entry.clone();
    let match_count_label_clone = match_count_label.clone();
    let options_clone = options.clone();

    // Schedule the async search
    let timer = glib::timeout_add_local(
        std::time::Duration::from_millis(delay_ms as u64),
        move || {
            perform_search(
                &search_entry_clone,
                &match_count_label_clone,
                &options_clone,
            );
            ASYNC_MANAGER.with(|manager_ref| {
                if let Some(manager) = manager_ref.borrow_mut().as_mut() {
                    manager.current_timer_id = None;
                }
            });
            glib::ControlFlow::Break
        },
    );

    // Store the timer or create a new manager
    ASYNC_MANAGER.with(|manager_ref| {
        let mut manager_borrow = manager_ref.borrow_mut();
        if let Some(manager) = manager_borrow.as_mut() {
            manager.current_timer_id = Some(timer);
        } else {
            *manager_borrow = Some(AsyncSearchManager {
                current_timer_id: Some(timer),
            });
        }
    });
}

/// Simple wrapper for debounced search
pub fn debounced_search(search_entry: &Entry, match_count_label: &Label, options: &OptionsWidgets) {
    perform_search_async(search_entry, match_count_label, options, 300);
}
