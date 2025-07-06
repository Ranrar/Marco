pub mod core;
pub mod dialog;
pub mod preferences;
pub mod ui;

// Re-export main types and functions
pub use core::{get_app_preferences, initialize_global_settings, initialize_settings};
pub use dialog::show_settings_dialog;
pub use preferences::{
    apply_settings_to_app, save_app_state_to_settings, load_app_state_from_settings,
    save_window_geometry, restore_window_geometry, connect_settings_changes
};
pub use ui::{
    create_settings_button, add_settings_button_to_header_bar, create_settings_section_header,
    create_settings_row, apply_settings_css, show_notification, get_available_css_themes,
    get_available_languages, is_file_readable
};
