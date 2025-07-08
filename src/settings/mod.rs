pub mod core;
pub mod preferences;
pub mod ui;
pub mod dialogs;

// Re-export main types and functions
pub use core::{get_app_preferences, initialize_global_settings, initialize_settings};
pub use preferences::{
    apply_settings_to_app, save_app_state_to_settings, load_app_state_from_settings,
    save_window_geometry, restore_window_geometry, connect_settings_changes
};
pub use ui::{
    create_settings_button, add_settings_button_to_header_bar, apply_settings_css, show_notification, get_available_css_themes,
    get_available_languages, is_file_readable
};

// Re-export all modular dialog page entrypoints and helpers
pub use dialogs::common::{
    show_settings_dialog,
    create_settings_section_header,
    create_settings_row
};
pub use dialogs::editor::create_editor_settings_page;
pub use dialogs::layout::create_layout_settings_page;
pub use dialogs::appearance::create_appearance_settings_page;
pub use dialogs::language::create_language_settings_page;
pub use dialogs::advanced::create_advanced_settings_page;
