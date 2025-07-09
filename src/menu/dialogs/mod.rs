// Dialogs module - Refactored modular dialog system
// This module provides a clean, organized interface for all dialog types

// Common utilities and components
pub mod common;

// Dialog modules organized by complexity and functionality
pub mod advanced;
pub mod basic;
pub mod lists;

// Re-export all dialog functions for backward compatibility
pub use basic::center_text::show_center_text_dialog;
pub use basic::colored_text::show_colored_text_dialog;
pub use basic::comment::show_comment_dialog;
pub use basic::shortcuts::show_shortcuts_dialog;
pub use basic::underline::show_underline_dialog;

pub use advanced::admonition::show_admonition_dialog;
pub use advanced::html_entity::show_html_entity_dialog;
pub use advanced::image_with_caption::show_image_with_caption_dialog;
pub use advanced::image_with_size::show_image_with_size_dialog;
pub use advanced::link_open_new::show_link_open_new_dialog;
pub use advanced::youtube_video::show_youtube_video_dialog;

pub use lists::definition_list::show_definition_list_custom_dialog;
pub use lists::task_list::show_task_list_custom_dialog;
