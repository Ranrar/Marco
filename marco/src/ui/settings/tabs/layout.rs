use core::logic::swanson::WindowSettings;
use gtk4::prelude::*;
use gtk4::Box;
use log::debug;

// Import unified helper
use super::helpers::add_setting_row;

// Build the layout tab. `initial_view_mode` optionally sets which entry is
// active when the tab is first shown (e.g. Some("Source Code") to select
// the code preview). The optional callback will be called when the View Mode
// dropdown changes and receives the selected value as a String.
// `settings_path` is used to load/save window settings including split ratio.
// `on_split_ratio_changed` is called when the split ratio slider changes to update the UI in real-time.
// `on_sync_scrolling_changed` is called when the sync scrolling toggle changes.
pub fn build_layout_tab(
    initial_view_mode: Option<String>,
    on_view_mode_changed: Option<std::boxed::Box<dyn Fn(String) + 'static>>,
    settings_path: Option<&str>,
    on_split_ratio_changed: Option<std::boxed::Box<dyn Fn(i32) + 'static>>,
    on_sync_scrolling_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
    on_line_numbers_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
) -> Box {
    use gtk4::{
        Adjustment, Box as GtkBox, DropDown, Expression, Orientation, PropertyExpression,
        SpinButton, StringList, StringObject, Switch,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Initialize SettingsManager once if settings_path is available
    let settings_manager_opt = if let Some(settings_path) = settings_path {
        match core::logic::swanson::SettingsManager::initialize(std::path::PathBuf::from(
            settings_path,
        )) {
            Ok(sm) => Some(sm),
            Err(e) => {
                debug!("Failed to initialize SettingsManager in layout tab: {}", e);
                None
            }
        }
    } else {
        None
    };

    // View Mode (Dropdown)
    let view_mode_options = StringList::new(&["HTML Preview", "Source Code"]);
    let view_mode_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let view_mode_combo = DropDown::new(Some(view_mode_options), Some(view_mode_expression));
    view_mode_combo.add_css_class("marco-dropdown");

    // Set active index based on saved setting if provided.
    let active_index = match initial_view_mode.as_deref() {
        Some(s)
            if s.eq_ignore_ascii_case("source code") || s.eq_ignore_ascii_case("code preview") =>
        {
            1
        }
        _ => 0,
    };
    view_mode_combo.set_selected(active_index);
    // Connect change handler to notify owner if provided. Convert selected index
    // to String when invoking the provided callback so callers receive a
    // straightforward String value.
    if let Some(cb) = on_view_mode_changed {
        view_mode_combo.connect_selected_notify(move |dropdown| {
            let selected_index = dropdown.selected() as usize;
            let mode_text = match selected_index {
                0 => "HTML Preview",
                1 => "Source Code",
                _ => "HTML Preview", // Default fallback
            };
            cb(mode_text.to_string());
        });
    }

    // Create view mode row using unified helper (first row)
    let view_mode_row = add_setting_row(
        "View Mode",
        "Choose the default mode for previewing Markdown content.",
        &view_mode_combo,
        true, // First row - no top margin
    );
    container.append(&view_mode_row);

    // Sync Scrolling (Toggle)
    let sync_scroll_switch = Switch::new();
    sync_scroll_switch.add_css_class("marco-switch");

    // Load current sync scrolling setting using existing SettingsManager
    let current_sync_scrolling = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager
            .get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.sync_scrolling)
            .unwrap_or(true)
    } else {
        true // Default to true
    };

    sync_scroll_switch.set_active(current_sync_scrolling);

    // Save sync scrolling setting when it changes
    if let Some(ref settings_manager) = settings_manager_opt {
        let settings_manager_clone = settings_manager.clone();
        sync_scroll_switch.connect_state_set(move |_switch, is_active| {
            debug!("Sync scrolling changed to: {}", is_active);

            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure layout settings exist
                if settings.layout.is_none() {
                    use core::logic::swanson::LayoutSettings;
                    settings.layout = Some(LayoutSettings::default());
                }

                // Update sync scrolling setting
                if let Some(ref mut layout) = settings.layout {
                    layout.sync_scrolling = Some(is_active);
                }
            }) {
                debug!("Failed to save sync scrolling setting: {}", e);
            } else {
                debug!("Sync scrolling saved: {}", is_active);
            }

            glib::Propagation::Proceed
        });
    }

    // Also connect runtime callback if provided
    if let Some(callback) = on_sync_scrolling_changed {
        sync_scroll_switch.connect_state_set(move |_switch, is_active| {
            debug!("Calling runtime sync scrolling update: {}", is_active);
            callback(is_active);
            glib::Propagation::Proceed
        });
    }

    // Create sync scrolling row using unified helper
    let sync_scroll_row = add_setting_row(
        "Sync Scrolling",
        "Synchronize scrolling between the editor and the preview pane.",
        &sync_scroll_switch,
        false, // Not first row
    );
    container.append(&sync_scroll_row);

    // Editor/View Split (SpinButton)
    // Load current split ratio from settings or use default (60%)
    let current_split_ratio = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager
            .get_settings()
            .window
            .as_ref()
            .and_then(|w| w.split_ratio)
            .unwrap_or(60)
    } else {
        60
    };

    let split_adj = Adjustment::new(current_split_ratio as f64, 10.0, 90.0, 1.0, 0.0, 0.0);
    let split_spin = SpinButton::new(Some(&split_adj), 1.0, 0);
    split_spin.add_css_class("marco-spinbutton");

    // Save split ratio when it changes
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        split_adj.connect_value_changed(move |adj| {
            let new_ratio = adj.value() as i32;
            debug!("Split ratio changed to: {}%", new_ratio);

            // Use SettingsManager to update split ratio setting
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure window settings exist
                if settings.window.is_none() {
                    settings.window = Some(WindowSettings::default());
                }

                // Update split ratio
                if let Some(ref mut window) = settings.window {
                    window.split_ratio = Some(new_ratio);
                }
            }) {
                debug!("Failed to save split ratio setting: {}", e);
            } else {
                debug!("Split ratio saved: {}%", new_ratio);
            }
        });
    }

    // Also connect live split ratio updates if callback provided
    if let Some(callback) = on_split_ratio_changed {
        split_adj.connect_value_changed(move |adj| {
            let new_ratio = adj.value() as i32;
            debug!("Calling live split ratio update: {}%", new_ratio);
            callback(new_ratio);
        });
    } else {
        debug!("No callback provided for split ratio changes");
    }

    // Create split ratio row using unified helper
    let split_row = add_setting_row(
        "Editor/View Split",
        "Adjust how much horizontal space the editor takes.",
        &split_spin,
        false, // Not first row
    );
    container.append(&split_row);

    // Show Line Numbers (Toggle)
    let line_numbers_switch = Switch::new();
    line_numbers_switch.add_css_class("marco-switch");

    // Load current line numbers setting from SettingsManager
    let current_line_numbers = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager
            .get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.show_line_numbers)
            .unwrap_or(true) // Default to true if not set
    } else {
        true // Default to true
    };

    line_numbers_switch.set_active(current_line_numbers);

    // Save line numbers setting when it changes
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        line_numbers_switch.connect_state_set(move |_switch, is_active| {
            debug!("Line numbers changed to: {}", is_active);

            // Use SettingsManager to update line numbers setting
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure layout settings exist
                if settings.layout.is_none() {
                    use core::logic::swanson::LayoutSettings;
                    settings.layout = Some(LayoutSettings::default());
                }

                // Update line numbers setting
                if let Some(ref mut layout) = settings.layout {
                    layout.show_line_numbers = Some(is_active);
                }
            }) {
                debug!("Failed to save line numbers setting: {}", e);
            } else {
                debug!("Line numbers saved: {}", is_active);
            }

            glib::Propagation::Proceed
        });
    }

    // Also connect runtime callback if provided
    if let Some(callback) = on_line_numbers_changed {
        line_numbers_switch.connect_state_set(move |_switch, is_active| {
            debug!("Calling runtime line numbers update: {}", is_active);
            callback(is_active);
            glib::Propagation::Proceed
        });
    }

    // Create line numbers row using unified helper
    let line_numbers_row = add_setting_row(
        "Show Line Numbers",
        "Display line numbers in the editor gutter.",
        &line_numbers_switch,
        false, // Not first row
    );
    container.append(&line_numbers_row);

    // Text Direction (Dropdown)
    let text_dir_options = StringList::new(&["Left-to-Right (LTR)", "Right-to-Left (RTL)"]);
    let text_dir_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let text_dir_combo = DropDown::new(Some(text_dir_options), Some(text_dir_expression));
    text_dir_combo.add_css_class("marco-dropdown");
    text_dir_combo.set_selected(0);

    // Create text direction row using unified helper
    let text_dir_row = add_setting_row(
        "Text Direction",
        "Switch between Left-to-Right (LTR) and Right-to-Left (RTL) layout.",
        &text_dir_combo,
        false, // Not first row
    );
    container.append(&text_dir_row);

    container
}
