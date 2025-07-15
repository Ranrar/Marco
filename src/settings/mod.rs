pub mod core;
pub mod dialogs;
pub mod preferences;
pub mod ui;

// Re-export main types and functions
pub use core::{get_app_preferences,
};

// Re-export all modular dialog page entrypoints and helpers
pub use dialogs::common::{
    show_settings_dialog,
};