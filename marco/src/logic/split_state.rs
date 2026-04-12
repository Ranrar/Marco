use core::logic::swanson::SettingsManager;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Apply the saved split ratio from settings after the paned widget has been mapped
/// and has a valid allocated width. Retries via timeout until the width is positive
/// (up to ~1 second).
pub fn apply_saved_split_ratio(split: &gtk4::Paned, settings_manager: &Arc<SettingsManager>) {
    let settings_manager_clone = settings_manager.clone();
    let applied = Rc::new(RefCell::new(false));

    split.connect_map(move |paned| {
        let paned_clone = paned.clone();
        let settings_manager = settings_manager_clone.clone();
        let applied_clone = applied.clone();
        let attempt_counter = Rc::new(RefCell::new(0));

        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            if *applied_clone.borrow() {
                return glib::ControlFlow::Break;
            }

            let paned_width = paned_clone.allocated_width();

            if paned_width > 0 {
                *applied_clone.borrow_mut() = true;

                let settings = settings_manager.get_settings();
                let split_ratio = settings
                    .window
                    .as_ref()
                    .map(|w| w.get_split_ratio())
                    .unwrap_or(60); // default 60% on first run
                let position = (paned_width as f64 * split_ratio as f64 / 100.0) as i32;

                log::info!(
                    "[SPLIT INIT] Applying saved ratio: {}% -> {}px (width: {}px)",
                    split_ratio,
                    position,
                    paned_width
                );
                paned_clone.set_position(position);
                return glib::ControlFlow::Break;
            }

            let mut attempt = attempt_counter.borrow_mut();
            *attempt += 1;
            if *attempt >= 20 {
                log::warn!(
                    "[SPLIT INIT] Failed to get paned width after {} attempts, giving up",
                    *attempt
                );
                *applied_clone.borrow_mut() = true;
                return glib::ControlFlow::Break;
            }

            glib::ControlFlow::Continue
        });
    });
}

/// Save the split ratio when the user finishes manually dragging the divider.
/// Debounces saves by 200ms to batch rapid position changes during drag.
pub fn connect_split_ratio_save(
    split: &gtk4::Paned,
    settings_manager: &Arc<SettingsManager>,
    settings_tx: &std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>,
) {
    let settings_manager_clone = settings_manager.clone();
    let settings_tx_clone = settings_tx.clone();
    let last_position = Rc::new(RefCell::new(-1i32));
    let save_timeout: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

    split.connect_notify_local(Some("position"), move |paned, _| {
        let paned_width = paned.allocated_width();
        if paned_width <= 0 {
            return;
        }

        let position = paned.position();

        // Check if position actually changed
        if *last_position.borrow() == position {
            return;
        }
        *last_position.borrow_mut() = position;

        // Cancel any pending save
        if let Some(id) = save_timeout.borrow_mut().take() {
            id.remove();
        }

        // Schedule save after 200ms of no changes (drag completed)
        let settings_manager = settings_manager_clone.clone();
        let settings_tx = settings_tx_clone.clone();
        let save_timeout_clone = save_timeout.clone();

        let timeout_id =
            glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
                *save_timeout_clone.borrow_mut() = None;

                let ratio = ((position as f64 / paned_width as f64) * 100.0).round() as i32;
                let ratio = ratio.clamp(10, 90);

                let task = Box::new(move || {
                    if let Err(e) = settings_manager.update_settings(|s| {
                        let _ = s.update_window_settings(|ws| {
                            ws.split_ratio = Some(ratio);
                        });
                    }) {
                        log::error!("Failed to save split ratio: {}", e);
                    } else {
                        log::info!(
                            "[SPLIT SAVE] Drag complete: {}% ({}px / {}px)",
                            ratio,
                            position,
                            paned_width
                        );
                    }
                });

                if let Err(e) = settings_tx.send(task) {
                    log::error!("Failed to queue split ratio save task: {}", e);
                }
            });

        *save_timeout.borrow_mut() = Some(timeout_id);
    });
}
