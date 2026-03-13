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

use crate::footer::{FooterLabels, FooterUpdate};
use crate::logic::signal_manager::safe_source_remove;
use core::logic::swanson::SettingsManager;
use gtk4::glib::ControlFlow;
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
pub fn refresh_footer_snapshot(
    buffer: &sourceview5::Buffer,
    labels: Rc<FooterLabels>,
    insert_mode_state: Rc<RefCell<bool>>,
    settings_manager: Arc<SettingsManager>,
) {
    let offset = buffer.cursor_position();
    let iter = buffer.iter_at_offset(offset);
    let row = (iter.line() + 1) as usize;
    let col = (iter.line_offset() + 1) as usize;
    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    let word_count = text.split_whitespace().filter(|w| !w.is_empty()).count();
    let char_count = text.chars().count();

    // Reload from disk to pick up updates from other SettingsManager instances
    // (e.g. settings dialog tab state changes).
    if let Err(err) = settings_manager.reload_settings() {
        log::debug!(
            "Failed to reload settings for footer snapshot diagnostics: {}",
            err
        );
    }

    let settings = settings_manager.get_settings();
    let editor = settings.editor.unwrap_or_default();

    let issues_runtime_enabled = editor.diagnostics_underlines_enabled.unwrap_or(true)
        || editor.diagnostics_hover_enabled.unwrap_or(true);

    let (errors, warnings, diagnostics) = if !issues_runtime_enabled {
        (0, 0, Vec::new())
    } else {
        match core::parser::parse(&text) {
            Ok(doc) => {
                let diagnostics = core::intelligence::compute_diagnostics_with_options(
                    &doc,
                    core::intelligence::DiagnosticsOptions::all(),
                );
                let errors = diagnostics
                    .iter()
                    .filter(|d| matches!(d.severity, core::intelligence::DiagnosticSeverity::Error))
                    .count();
                let warnings = diagnostics
                    .iter()
                    .filter(|d| {
                        matches!(d.severity, core::intelligence::DiagnosticSeverity::Warning)
                    })
                    .count();
                let diagnostics = diagnostics
                    .iter()
                    .map(|d| crate::footer::FooterDiagnosticItem {
                        severity: d.severity.clone(),
                        code: d.code_id().to_string(),
                        line: d.span.start.line,
                        column: d.span.start.column,
                        message: d.message.clone(),
                        fix_suggestion: d.fix_suggestion_resolved().into_owned(),
                    })
                    .collect();
                (errors, warnings, diagnostics)
            }
            Err(err) => {
                let parse_diagnostic = core::intelligence::Diagnostic::parse_error_at(
                    core::parser::Position {
                        line: row,
                        column: col,
                        offset: offset as usize,
                    },
                    format!("Parse error: {}", err),
                );

                (
                    1,
                    0,
                    vec![crate::footer::FooterDiagnosticItem {
                        severity: parse_diagnostic.severity.clone(),
                        code: parse_diagnostic.code_id().to_string(),
                        line: parse_diagnostic.span.start.line,
                        column: parse_diagnostic.span.start.column,
                        message: parse_diagnostic.message.clone(),
                        fix_suggestion: parse_diagnostic.fix_suggestion_resolved().into_owned(),
                    }],
                )
            }
        }
    };

    let encoding = labels.encoding_label.borrow().clone();
    let is_insert = *insert_mode_state.borrow();

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
