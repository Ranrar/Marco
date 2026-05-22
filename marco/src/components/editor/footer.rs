//! Footer status bar updates for the editor
//!
//! This module provides debounced footer updates that display:
//! - Current cursor position (line and column)
//! - Insert/overwrite mode status
//! - Character and word count statistics
//! - Diagnostics counters (errors/warnings)
//!
//! # Debouncing Strategy
//!
//! Footer updates are debounced (300ms) to avoid excessive GTK redraws during
//! rapid text editing. This prevents UI stutter while maintaining responsive
//! feedback for cursor movement and mode changes.
//!
//! # Integration
//!
//! Wire footer updates to a SourceView buffer using `wire_footer_updates()`:
//!
//! ```ignore
//! wire_footer_updates(&buffer, labels, insert_mode_state);
//! ```

use crate::footer::{FooterDiagnosticItem, FooterLabels, FooterUpdate};
use crate::logic::signal_manager::safe_source_remove;
use gtk4::glib;
use gtk4::glib::ControlFlow;
use marco_shared::logic::swanson::SettingsManager;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;

/// Wires up debounced footer updates to buffer events
pub fn wire_footer_updates(
    buffer: &sourceview5::Buffer,
    source_view: &sourceview5::View,
    labels: Rc<FooterLabels>,
    insert_mode_state: Rc<RefCell<bool>>,
    settings_manager: Arc<SettingsManager>,
) {
    crate::footer::bind_diagnostics_navigation(&labels, buffer, source_view);

    use std::cell::Cell;
    let debounce_ms = 300;

    let buffer_timeout_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
    let cursor_timeout_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

    let update_footer = {
        let buffer = buffer.clone();
        let labels = labels.clone();
        let insert_mode_state = Rc::clone(&insert_mode_state);
        let settings_manager = settings_manager.clone();
        move || {
            crate::footer_dbg!("[wire_footer_updates] update_footer closure called");
            refresh_footer_snapshot(
                &buffer,
                labels.clone(),
                insert_mode_state.clone(),
                settings_manager.clone(),
            );
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

/// Recompute and apply footer values using the current buffer state.
///
/// Cursor position, word/char counts, and encoding are read on the calling
/// (main) thread.  The expensive parse + diagnostics computation is offloaded
/// to a thread-pool worker via `gio::spawn_blocking` so the GTK event loop
/// stays responsive while editing large documents.
pub fn refresh_footer_snapshot(
    buffer: &sourceview5::Buffer,
    labels: Rc<FooterLabels>,
    insert_mode_state: Rc<RefCell<bool>>,
    settings_manager: Arc<SettingsManager>,
) {
    // ── Step 1: gather all GTK / Rc data on the main thread ────────────────
    let offset = buffer.cursor_position();
    let iter = buffer.iter_at_offset(offset);
    let row = (iter.line() + 1) as usize;
    let col = (iter.line_offset() + 1) as usize;
    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    let word_count = text.split_whitespace().filter(|w| !w.is_empty()).count();
    let char_count = text.chars().count();
    let encoding = labels.encoding_label.borrow().clone();
    let is_insert = *insert_mode_state.borrow();

    // ── Step 2: offload parse + diagnostics to a thread-pool worker ────────
    glib::spawn_future_local(async move {
        let offset_usize = offset.max(0) as usize;
        let compute = gtk4::gio::spawn_blocking(move || {
            // All types here are Send: String, Arc<SettingsManager>, primitives.
            if let Err(err) = settings_manager.reload_settings() {
                log::debug!(
                    "[footer] Failed to reload settings for diagnostics: {}",
                    err
                );
            }

            let settings = settings_manager.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let issues_runtime_enabled = editor.diagnostics_underlines_enabled.unwrap_or(true)
                || editor.diagnostics_hover_enabled.unwrap_or(true);

            if !issues_runtime_enabled {
                return (0usize, 0usize, Vec::<FooterDiagnosticItem>::new());
            }

            // Use AST cache: if the render pipeline already parsed this text the
            // AST is reused and parse_and_cache_ast returns instantly.
            let content_hash = marco_shared::cache::hash_content(&text);
            match marco_shared::cache::global_parser_cache().parse_and_cache_ast(&text) {
                Ok(doc) => {
                    // Diagnostics are cached too — same content_hash → immediate return.
                    let cached_diags = marco_shared::cache::global_parser_cache()
                        .get_or_compute_diagnostics_for_doc(&doc, content_hash);
                    let errors = cached_diags
                        .iter()
                        .filter(|d| {
                            matches!(
                                d.severity,
                                marco_core::intelligence::DiagnosticSeverity::Error
                            )
                        })
                        .count();
                    let warnings = cached_diags
                        .iter()
                        .filter(|d| {
                            matches!(
                                d.severity,
                                marco_core::intelligence::DiagnosticSeverity::Warning
                            )
                        })
                        .count();
                    let items = cached_diags
                        .iter()
                        .map(|d| FooterDiagnosticItem {
                            severity: d.severity,
                            code: d.code_id().to_string(),
                            line: d.span.start.line,
                            column: d.span.start.column,
                            message: d.message.clone(),
                            fix_suggestion: d.fix_suggestion_resolved().into_owned(),
                        })
                        .collect();
                    (errors, warnings, items)
                }
                Err(err) => {
                    let parse_diagnostic = marco_core::intelligence::Diagnostic::parse_error_at(
                        marco_core::parser::Position {
                            line: row,
                            column: col,
                            offset: offset_usize,
                        },
                        format!("Parse error: {}", err),
                    );
                    (
                        1,
                        0,
                        vec![FooterDiagnosticItem {
                            severity: parse_diagnostic.severity,
                            code: parse_diagnostic.code_id().to_string(),
                            line: parse_diagnostic.span.start.line,
                            column: parse_diagnostic.span.start.column,
                            message: parse_diagnostic.message.clone(),
                            fix_suggestion: parse_diagnostic.fix_suggestion_resolved().into_owned(),
                        }],
                    )
                }
            }
        })
        .await;

        // ── Step 3: apply result on the main thread ─────────────────────────
        match compute {
            Ok((errors, warnings, diagnostics)) => {
                let msg = FooterUpdate::Snapshot {
                    row,
                    col,
                    errors,
                    warnings,
                    diagnostics,
                    words: word_count,
                    chars: char_count,
                    encoding,
                    is_insert,
                };
                crate::footer::apply_footer_update(&labels, msg);
            }
            Err(e) => {
                log::error!("[footer] background diagnostics task panicked: {:?}", e);
            }
        }
    });
}
