//! Search Navigation - Match Navigation and Scrolling
//!
//! Handles navigation through search results and preview synchronization.

use super::state::*;
use gtk4::prelude::*;
use log::debug;

#[cfg(target_os = "linux")]
use webkit6::prelude::WebViewExt;

/// Find the position of the match at or immediately after the cursor
/// Returns the 0-based index of the match position, or None if no match found
pub(crate) fn find_position_from_cursor() -> Option<i32> {
    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    let cursor_iter = buffer.iter_at_offset(buffer.cursor_position());
                    let match_count = search_state.search_context.occurrences_count();

                    if match_count <= 0 {
                        return None;
                    }

                    let mut search_iter = cursor_iter;
                    let mut position = 0;

                    // Special case: if cursor is at the very end (no more text ahead)
                    // Always return the first match (wrapping behavior)
                    if cursor_iter.is_end() {
                        return Some(0);
                    }

                    // Iterate through all matches to find the one at or after the cursor
                    while let Some((match_start, match_end, has_wrapped)) =
                        search_state.search_context.forward(&search_iter)
                    {
                        let match_offset = match_start.offset();
                        let cursor_offset = cursor_iter.offset();

                        // If the match is at or after the cursor, this is our target
                        if match_offset >= cursor_offset {
                            return Some(position);
                        }

                        // Check if we wrapped - means we're back at the first match
                        if has_wrapped {
                            // Cursor is past all matches, so wrap to the first one
                            return Some(0);
                        }

                        position += 1;

                        // Move search position forward to find next match
                        search_iter = match_end;

                        // Prevent infinite loop
                        if position >= match_count {
                            break;
                        }
                    }

                    // If we didn't find a match at or after cursor, wrap to first match
                    Some(0)
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
}

/// Find the position of the match immediately before the cursor
/// Returns position + 1 (to account for the decrement in navigation logic)
pub(crate) fn find_position_before_cursor() -> Option<i32> {
    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    let cursor_iter = buffer.iter_at_offset(buffer.cursor_position());
                    let match_count = search_state.search_context.occurrences_count();

                    if match_count <= 0 {
                        return None;
                    }

                    // Start searching from the beginning
                    let mut search_iter = buffer.start_iter();
                    let mut last_valid_position: Option<i32> = None;
                    let mut position = 0;

                    // Iterate through all matches to find the last one before the cursor
                    while let Some((match_start, match_end, has_wrapped)) =
                        search_state.search_context.forward(&search_iter)
                    {
                        let match_offset = match_start.offset();
                        let cursor_offset = cursor_iter.offset();

                        // If we wrapped, we've seen all matches
                        if has_wrapped {
                            break;
                        }

                        // If this match starts before the cursor, it's a candidate
                        if match_offset < cursor_offset {
                            last_valid_position = Some(position);
                        } else {
                            // We've reached matches at or after cursor, stop
                            break;
                        }

                        position += 1;
                        search_iter = match_end;

                        // Prevent infinite loop
                        if position >= match_count {
                            break;
                        }
                    }

                    // Return position + 1 to account for the decrement in navigate_previous
                    last_valid_position
                        .map(|pos| {
                            if pos == match_count - 1 {
                                // If the last match before cursor is the final match,
                                // returning match_count will wrap to last match (match_count - 1)
                                match_count
                            } else {
                                pos + 1
                            }
                        })
                        .or({
                            // No match before cursor - wrap to last match
                            Some(match_count)
                        })
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
}

/// Update position display immediately and schedule debounced navigation
/// This provides instant feedback while preventing excessive scrolling
pub fn immediate_position_update_with_debounced_navigation(direction: i32, delay_ms: u32) {
    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            let match_count = search_state.search_context.occurrences_count();
            if match_count <= 0 {
                return;
            }

            CURRENT_MATCH_POSITION.with(|pos_ref| {
                let current_pos = *pos_ref.borrow();

                let new_pos = if let Some(pos) = current_pos {
                    // Increment or decrement based on direction
                    let next_pos = pos + direction;

                    // Handle wrapping
                    if next_pos < 0 {
                        match_count - 1 // Wrap to last match
                    } else if next_pos >= match_count {
                        0 // Wrap to first match
                    } else {
                        next_pos
                    }
                } else {
                    // No current position - determine from cursor
                    if direction > 0 {
                        find_position_from_cursor().unwrap_or(0)
                    } else {
                        find_position_before_cursor()
                            .map(|p| {
                                let wrapped = p - 1;
                                if wrapped < 0 {
                                    match_count - 1
                                } else {
                                    wrapped
                                }
                            })
                            .unwrap_or(match_count - 1)
                    }
                };

                // Update the stored position immediately
                *pos_ref.borrow_mut() = Some(new_pos);

                // Update the display label immediately for instant feedback
                CURRENT_MATCH_LABEL.with(|label_ref| {
                    if let Some(label) = label_ref.borrow().as_ref() {
                        CURRENT_BUFFER.with(|buffer_ref| {
                            if let Some(buffer) = buffer_ref.borrow().as_ref() {
                                // Find the match at the new position to get line number
                                let mut search_iter = buffer.start_iter();
                                let mut current_index = 0;

                                while let Some((match_start, _match_end, _)) =
                                    search_state.search_context.forward(&search_iter)
                                {
                                    if current_index == new_pos {
                                        let line_number = match_start.line() + 1;
                                        let display_text = format!(
                                            "{} of {} matches (line {})",
                                            new_pos + 1,
                                            match_count,
                                            line_number
                                        );
                                        label.set_text(&display_text);
                                        break;
                                    }
                                    current_index += 1;
                                    search_iter = match_start;
                                    search_iter.forward_char();
                                }
                            }
                        });
                    }
                });

                debug!(
                    "Position counter updated immediately: {} (direction: {})",
                    new_pos + 1,
                    direction
                );
            });

            // Cancel any existing navigation timer
            NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| {
                if let Some(source_id) = timer_ref.borrow_mut().take() {
                    source_id.remove();
                    debug!("Cancelled previous navigation timer");
                }

                // Schedule a new debounced navigation
                let new_timer = glib::timeout_add_local(
                    std::time::Duration::from_millis(delay_ms as u64),
                    move || {
                        navigate_to_current_position();
                        NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| {
                            *timer_ref.borrow_mut() = None;
                        });
                        glib::ControlFlow::Break
                    },
                );

                *timer_ref.borrow_mut() = Some(new_timer);
                debug!("Scheduled debounced navigation with {}ms delay", delay_ms);
            });
        }
    });
}

/// Navigate to the match at the current stored position
pub fn navigate_to_current_position() {
    // Set navigation in progress flag to prevent interference
    set_navigation_in_progress(true);

    CURRENT_MATCH_POSITION.with(|pos_ref| {
        if let Some(target_position) = *pos_ref.borrow() {
            CURRENT_SEARCH_STATE.with(|state_ref| {
                if let Some(search_state) = state_ref.borrow().as_ref() {
                    CURRENT_BUFFER.with(|buffer_ref| {
                        if let Some(buffer) = buffer_ref.borrow().as_ref() {
                            // Find the match at the target position
                            let mut search_iter = buffer.start_iter();
                            let mut current_index = 0;

                            while let Some((match_start, match_end, _)) =
                                search_state.search_context.forward(&search_iter)
                            {
                                if current_index == target_position {
                                    // Found our target match
                                    buffer.select_range(&match_start, &match_end);

                                    // Clear old enhanced highlighting first, then apply new
                                    super::engine::clear_enhanced_search_highlighting();
                                    super::engine::apply_enhanced_search_highlighting(
                                        &search_state.search_context,
                                        Some(&match_start),
                                        Some(&match_end),
                                    );

                                    // Scroll to show the match
                                    scroll_to_match(&match_start);

                                    let line_number = match_start.line() + 1;
                                    let match_count =
                                        search_state.search_context.occurrences_count();

                                    // Update match label with current position
                                    CURRENT_MATCH_LABEL.with(|label_ref| {
                                        if let Some(label) = label_ref.borrow().as_ref() {
                                            let display_text = format!(
                                                "{} of {} matches (line {})",
                                                target_position + 1,
                                                match_count,
                                                line_number
                                            );
                                            label.set_text(&display_text);
                                        }
                                    });

                                    debug!(
                                        "Navigated to match {} at line {}",
                                        target_position + 1,
                                        line_number
                                    );

                                    // Clear navigation in progress flag after navigation completes
                                    set_navigation_in_progress(false);
                                    return;
                                }

                                current_index += 1;
                                search_iter = match_start;
                                search_iter.forward_char();
                            }

                            // If we get here, target position was not found
                            debug!("Warning: Target position {} not found", target_position);
                            set_navigation_in_progress(false);
                        } else {
                            debug!("No buffer available for navigation");
                            set_navigation_in_progress(false);
                        }
                    });
                } else {
                    debug!("No active search state for navigation");
                    set_navigation_in_progress(false);
                }
            });
        } else {
            debug!("No current match position stored");
            set_navigation_in_progress(false);
        }
    });
}

/// Check if a widget has valid allocation for rendering operations
fn has_valid_allocation(widget: &impl IsA<gtk4::Widget>) -> bool {
    let allocation = widget.allocation();
    allocation.width() > 0 && allocation.height() > 0
}

/// Scroll the editor to show the match at the given position
fn scroll_to_match(match_iter: &gtk4::TextIter) {
    CURRENT_SOURCE_VIEW.with(|view_ref| {
        if let Some(source_view) = view_ref.borrow().as_ref() {
            // Check if the source view has proper allocation before scrolling
            if !has_valid_allocation(source_view.as_ref()) {
                debug!("Skipping scroll operation - SourceView has no allocation");
                return;
            }

            // Create a mutable copy of the iterator for scroll_to_iter
            let mut iter_copy = *match_iter;

            // Scroll to the match position with some margin
            // Parameters: iter, within_margin, use_align, xalign, yalign
            // within_margin: 0.1 = 10% margin from edges before scrolling
            // use_align: true = use the alignment values
            // xalign: 0.0 = align to left edge
            // yalign: 0.3 = position match at 30% from top (comfortable reading position)
            source_view.scroll_to_iter(&mut iter_copy, 0.1, true, 0.0, 0.3);

            debug!(
                "Scrolled editor to show match at line {}",
                match_iter.line() + 1
            );

            // Also sync the HTML preview if scroll sync is enabled
            sync_html_preview_scroll(match_iter);
        } else {
            debug!("No source view available for scrolling");
        }
    });
}

/// Sync HTML preview scroll to match the given position (if scroll sync is enabled)
#[cfg(target_os = "linux")]
fn sync_html_preview_scroll(match_iter: &gtk4::TextIter) {
    // Check if scroll sync is enabled globally
    use crate::components::editor::editor_manager::get_global_scroll_synchronizer;
    if let Some(sync) = get_global_scroll_synchronizer() {
        // Only sync if scroll synchronization is actually enabled
        if !sync.is_enabled() {
            debug!("Scroll sync is disabled, skipping preview scroll sync");
            return;
        }
        // Access the WebView to perform sync
        CURRENT_WEBVIEW.with(|webview_ref| {
            if let Some(webview) = webview_ref.borrow().as_ref() {
                // Calculate the scroll percentage based on the match position
                CURRENT_BUFFER.with(|buffer_ref| {
                    if let Some(buffer) = buffer_ref.borrow().as_ref() {
                        let total_lines = buffer.line_count();
                        let match_line = match_iter.line();

                        // Calculate approximate scroll percentage
                        // Position the match at about 30% from the top (same as editor scroll)
                        let scroll_percentage = if total_lines > 1 {
                            (match_line as f64 / (total_lines - 1) as f64).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };

                        // Use JavaScript to scroll the WebView to the corresponding position
                        let js_code = format!(
                            r#"
                            (function() {{
                                if (window.__scroll_sync_guard) return;
                                window.__scroll_sync_guard = true;

                                const maxScroll = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
                                const targetScroll = {} * maxScroll;

                                // Adjust to position the target at 30% from top (like editor)
                                const viewportHeight = window.innerHeight;
                                const adjustedScroll = Math.max(0, targetScroll - viewportHeight * 0.3);

                                window.scrollTo({{
                                    top: adjustedScroll,
                                    behavior: 'smooth'
                                }});

                                setTimeout(() => {{
                                    window.__scroll_sync_guard = false;
                                }}, 150);
                            }})();
                            "#,
                            scroll_percentage
                        );

                        // webkit6 0.5.0 uses async futures - extract inner WebView
                        let webview_inner = webview.borrow().clone();
                        let js = js_code.clone();
                        glib::spawn_future_local(async move {
                            let _ = webview_inner.evaluate_javascript_future(&js, None, None).await;
                        });

                        debug!(
                            "Synced HTML preview scroll to line {} ({:.1}%)",
                            match_line + 1,
                            scroll_percentage * 100.0
                        );
                    }
                });
            } else {
                debug!("No WebView available for preview scroll sync");
            }
        });
    }
}

/// Fallback for Windows - no preview sync available
#[cfg(target_os = "windows")]
fn sync_html_preview_scroll(_match_iter: &gtk4::TextIter) {
    // No-op on Windows since we don't have WebView integration
    debug!("Preview scroll sync not available on Windows");
}
