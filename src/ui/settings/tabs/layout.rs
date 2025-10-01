use crate::logic::swanson::WindowSettings;
use gtk4::prelude::*;
use gtk4::Box;
use log::debug;

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
        Adjustment, Align, Box as GtkBox, DropDown, Label, Orientation, Scale, SpinButton,
        StringList, PropertyExpression, StringObject, Expression, Switch,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-layout");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    // Initialize SettingsManager once if settings_path is available
    let settings_manager_opt = if let Some(settings_path) = settings_path {
        match crate::logic::swanson::SettingsManager::initialize(std::path::PathBuf::from(settings_path)) {
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
    let view_mode_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let view_mode_header = Label::new(Some("View Mode"));
    view_mode_header.set_markup("<b>View Mode</b>");
    view_mode_header.set_halign(Align::Start);
    view_mode_header.set_xalign(0.0);

    let view_mode_spacer = GtkBox::new(Orientation::Horizontal, 0);
    view_mode_spacer.set_hexpand(true);

    let view_mode_options = StringList::new(&["HTML Preview", "Source Code"]);
    let view_mode_expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<&Expression>,
        "string",
    );
    let view_mode_combo = DropDown::new(Some(view_mode_options), Some(view_mode_expression));
    
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
    view_mode_combo.set_halign(Align::End);

    view_mode_hbox.append(&view_mode_header);
    view_mode_hbox.append(&view_mode_spacer);
    view_mode_hbox.append(&view_mode_combo);
    view_mode_hbox.set_margin_bottom(4);
    container.append(&view_mode_hbox);

    // Description text under header
    let view_mode_description = Label::new(Some(
        "Choose the default mode for previewing Markdown content.",
    ));
    view_mode_description.set_halign(Align::Start);
    view_mode_description.set_xalign(0.0);
    view_mode_description.set_wrap(true);
    view_mode_description.add_css_class("dim-label");
    view_mode_description.set_margin_bottom(12);
    container.append(&view_mode_description);

    // Sync Scrolling (Toggle)
    let sync_scroll_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let sync_scroll_header = Label::new(Some("Sync Scrolling"));
    sync_scroll_header.set_markup("<b>Sync Scrolling</b>");
    sync_scroll_header.set_halign(Align::Start);
    sync_scroll_header.set_xalign(0.0);

    let sync_scroll_spacer = GtkBox::new(Orientation::Horizontal, 0);
    sync_scroll_spacer.set_hexpand(true);

    let sync_scroll_switch = Switch::new();

    // Load current sync scrolling setting using existing SettingsManager
    let current_sync_scrolling = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager.get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.sync_scrolling)
            .unwrap_or(true)
    } else {
        true // Default to true
    };

    sync_scroll_switch.set_active(current_sync_scrolling);
    sync_scroll_switch.set_halign(Align::End);

    // Save sync scrolling setting when it changes
    if let Some(ref settings_manager) = settings_manager_opt {
        let settings_manager_clone = settings_manager.clone();
        sync_scroll_switch.connect_state_set(move |_switch, is_active| {
            debug!("Sync scrolling changed to: {}", is_active);
            
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure layout settings exist
                if settings.layout.is_none() {
                    use crate::logic::swanson::LayoutSettings;
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

    sync_scroll_hbox.append(&sync_scroll_header);
    sync_scroll_hbox.append(&sync_scroll_spacer);
    sync_scroll_hbox.append(&sync_scroll_switch);
    sync_scroll_hbox.set_margin_top(8);
    sync_scroll_hbox.set_margin_bottom(4);
    container.append(&sync_scroll_hbox);

    // Description text under header
    let sync_scroll_description = Label::new(Some(
        "Synchronize scrolling between the editor and the preview pane.",
    ));
    sync_scroll_description.set_halign(Align::Start);
    sync_scroll_description.set_xalign(0.0);
    sync_scroll_description.set_wrap(true);
    sync_scroll_description.add_css_class("dim-label");
    sync_scroll_description.set_margin_bottom(12);
    container.append(&sync_scroll_description);

    // Editor/View Split (Slider/SpinButton)
    let split_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let split_header = Label::new(Some("Editor/View Split"));
    split_header.set_markup("<b>Editor/View Split</b>");
    split_header.set_halign(Align::Start);
    split_header.set_xalign(0.0);

    let split_spacer = GtkBox::new(Orientation::Horizontal, 0);
    split_spacer.set_hexpand(true);

    // Load current split ratio from settings or use default (60%)
    let current_split_ratio = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager.get_settings()
            .window
            .as_ref()
            .and_then(|w| w.split_ratio)
            .unwrap_or(60)
    } else {
        60
    };

    let split_adj = Adjustment::new(current_split_ratio as f64, 10.0, 90.0, 1.0, 0.0, 0.0);
    let split_spin = SpinButton::new(Some(&split_adj), 1.0, 0);
    split_spin.set_halign(Align::End);

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

    split_hbox.append(&split_header);
    split_hbox.append(&split_spacer);
    split_hbox.append(&split_spin);
    split_hbox.set_margin_top(8);
    split_hbox.set_margin_bottom(4);
    container.append(&split_hbox);

    // Description text under header
    let split_description = Label::new(Some("Adjust how much horizontal space the editor takes."));
    split_description.set_halign(Align::Start);
    split_description.set_xalign(0.0);
    split_description.set_wrap(true);
    split_description.add_css_class("dim-label");
    split_description.set_margin_bottom(12);
    container.append(&split_description);

    let split_slider = Scale::new(Orientation::Horizontal, Some(&split_adj));
    split_slider.set_draw_value(false);
    split_slider.set_hexpand(true);
    split_slider.set_round_digits(0);
    split_slider.set_width_request(300);
    // Add marks for common splits
    for mark in [10, 25, 40, 50, 60, 75, 90].iter() {
        split_slider.add_mark(
            *mark as f64,
            gtk4::PositionType::Bottom,
            Some(&format!("{}%", mark)),
        );
    }
    split_slider.set_halign(Align::Start);
    split_slider.set_margin_bottom(12);
    container.append(&split_slider);

    // Show Line Numbers (Toggle)
    let line_numbers_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let line_numbers_header = Label::new(Some("Show Line Numbers"));
    line_numbers_header.set_markup("<b>Show Line Numbers</b>");
    line_numbers_header.set_halign(Align::Start);
    line_numbers_header.set_xalign(0.0);

    let line_numbers_spacer = GtkBox::new(Orientation::Horizontal, 0);
    line_numbers_spacer.set_hexpand(true);

    let line_numbers_switch = Switch::new();

    // Load current line numbers setting from SettingsManager
    let current_line_numbers = if let Some(ref settings_manager) = settings_manager_opt {
        settings_manager.get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.show_line_numbers)
            .unwrap_or(true) // Default to true if not set
    } else {
        true // Default to true
    };

    line_numbers_switch.set_active(current_line_numbers);
    line_numbers_switch.set_halign(Align::End);

    // Save line numbers setting when it changes
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        line_numbers_switch.connect_state_set(move |_switch, is_active| {
            debug!("Line numbers changed to: {}", is_active);

            // Use SettingsManager to update line numbers setting
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure layout settings exist
                if settings.layout.is_none() {
                    use crate::logic::swanson::LayoutSettings;
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

    line_numbers_hbox.append(&line_numbers_header);
    line_numbers_hbox.append(&line_numbers_spacer);
    line_numbers_hbox.append(&line_numbers_switch);
    line_numbers_hbox.set_margin_top(8);
    line_numbers_hbox.set_margin_bottom(4);
    container.append(&line_numbers_hbox);

    // Description text under header
    let line_numbers_description = Label::new(Some("Display line numbers in the editor gutter."));
    line_numbers_description.set_halign(Align::Start);
    line_numbers_description.set_xalign(0.0);
    line_numbers_description.set_wrap(true);
    line_numbers_description.add_css_class("dim-label");
    line_numbers_description.set_margin_bottom(12);
    container.append(&line_numbers_description);

    // Text Direction (Dropdown)
    let text_dir_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let text_dir_header = Label::new(Some("Text Direction"));
    text_dir_header.set_markup("<b>Text Direction</b>");
    text_dir_header.set_halign(Align::Start);
    text_dir_header.set_xalign(0.0);

    let text_dir_spacer = GtkBox::new(Orientation::Horizontal, 0);
    text_dir_spacer.set_hexpand(true);

    let text_dir_options = StringList::new(&["Left-to-Right (LTR)", "Right-to-Left (RTL)"]);
    let text_dir_expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<&Expression>,
        "string",
    );
    let text_dir_combo = DropDown::new(Some(text_dir_options), Some(text_dir_expression));
    text_dir_combo.set_selected(0);
    text_dir_combo.set_halign(Align::End);

    text_dir_hbox.append(&text_dir_header);
    text_dir_hbox.append(&text_dir_spacer);
    text_dir_hbox.append(&text_dir_combo);
    text_dir_hbox.set_margin_top(8);
    text_dir_hbox.set_margin_bottom(4);
    container.append(&text_dir_hbox);

    // Description text under header
    let text_dir_description = Label::new(Some(
        "Switch between Left-to-Right (LTR) and Right-to-Left (RTL) layout.",
    ));
    text_dir_description.set_halign(Align::Start);
    text_dir_description.set_xalign(0.0);
    text_dir_description.set_wrap(true);
    text_dir_description.add_css_class("dim-label");
    text_dir_description.set_margin_bottom(12);
    container.append(&text_dir_description);

    container
}
