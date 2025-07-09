pub mod core;
pub mod dialogs;
pub mod preferences;
pub mod ui;

// Re-export main types and functions
pub use core::{get_app_preferences, initialize_global_settings, initialize_settings};
pub use preferences::{
    apply_settings_to_app, connect_settings_changes, load_app_state_from_settings,
    restore_window_geometry, save_app_state_to_settings, save_window_geometry,
};
pub use ui::{
    add_settings_button_to_header_bar, apply_settings_css, create_settings_button,
    get_available_css_themes, get_available_languages, is_file_readable, show_notification,
};

// Re-export all modular dialog page entrypoints and helpers
pub use dialogs::advanced::create_advanced_settings_page;
pub use dialogs::appearance::create_appearance_settings_page;
pub use dialogs::common::{
    create_settings_row, create_settings_section_header, show_settings_dialog,
};
pub use dialogs::editor::create_editor_settings_page;
pub use dialogs::language::create_language_settings_page;
pub use dialogs::layout::create_layout_settings_page;
