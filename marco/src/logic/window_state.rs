use marco_shared::logic::swanson::SettingsManager;
use gtk4::prelude::*;
use gtk4::ApplicationWindow;
use std::sync::Arc;

/// Apply the saved window state (size, maximized) from settings to the window.
pub fn apply_saved_window_state(
    window: &ApplicationWindow,
    settings_manager: &Arc<SettingsManager>,
) {
    let settings = settings_manager.get_settings();
    if let Some(window_settings) = settings.window {
        // Apply window size
        let (width, height) = window_settings.get_window_size();
        window.set_default_size(width as i32, height as i32);

        // Apply window position if saved
        if let Some((x, y)) = window_settings.get_window_position() {
            // Note: GTK4 doesn't support programmatic window positioning directly
            // This would need platform-specific implementation if required
            log::debug!(
                "Would restore window position to ({}, {}) if supported",
                x,
                y
            );
        }

        // Apply maximized state
        if window_settings.is_maximized() {
            window.maximize();
        }
    }
}

/// Connect window resize and maximize handlers to persist state in settings.
pub fn connect_window_state_persistence(
    window: &ApplicationWindow,
    settings_manager: &Arc<SettingsManager>,
    settings_tx: &std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>,
    refresh_bookmark_marks: &std::rc::Rc<dyn Fn()>,
) {
    // Width change
    {
        let settings_manager = settings_manager.clone();
        let settings_tx = settings_tx.clone();
        let refresh_bookmark_marks = refresh_bookmark_marks.clone();
        window.connect_default_width_notify(move |w| {
            let settings_manager = settings_manager.clone();
            let width = w.default_width();
            let height = w.default_height();
            let settings_tx = settings_tx.clone();
            let refresh_bookmark_marks = refresh_bookmark_marks.clone();

            gtk4::glib::idle_add_local_once(move || {
                refresh_bookmark_marks();
            });

            let task = Box::new(move || {
                if let Err(e) = settings_manager.update_settings(|s| {
                    let _ = s.update_window_settings(|ws| {
                        ws.width = Some(width as u32);
                        ws.height = Some(height as u32);
                    });
                }) {
                    log::error!("Failed to save window size: {}", e);
                } else {
                    log::debug!("Window size saved: {}x{}", width, height);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue window size save task: {}", e);
            }
        });
    }

    // Height change
    {
        let settings_manager = settings_manager.clone();
        let settings_tx = settings_tx.clone();
        let refresh_bookmark_marks = refresh_bookmark_marks.clone();
        window.connect_default_height_notify(move |w| {
            let settings_manager = settings_manager.clone();
            let width = w.default_width();
            let height = w.default_height();
            let settings_tx = settings_tx.clone();
            let refresh_bookmark_marks = refresh_bookmark_marks.clone();

            gtk4::glib::idle_add_local_once(move || {
                refresh_bookmark_marks();
            });

            let task = Box::new(move || {
                if let Err(e) = settings_manager.update_settings(|s| {
                    let _ = s.update_window_settings(|ws| {
                        ws.width = Some(width as u32);
                        ws.height = Some(height as u32);
                    });
                }) {
                    log::error!("Failed to save window size: {}", e);
                } else {
                    log::debug!("Window size saved: {}x{}", width, height);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue window size save task: {}", e);
            }
        });
    }

    // Maximize state
    {
        let settings_manager = settings_manager.clone();
        let settings_tx = settings_tx.clone();
        window.connect_maximized_notify(move |w| {
            let settings_manager = settings_manager.clone();
            let is_maximized = w.is_maximized();
            let settings_tx = settings_tx.clone();

            let task = Box::new(move || {
                if let Err(e) = settings_manager.update_settings(|s| {
                    let _ = s.update_window_settings(|ws| {
                        ws.maximized = Some(is_maximized);
                    });
                }) {
                    log::error!("Failed to save window maximized state: {}", e);
                } else {
                    log::debug!("Window maximized state saved: {}", is_maximized);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue window maximized save task: {}", e);
            }
        });
    }
}
