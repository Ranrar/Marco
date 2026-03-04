pub mod about;
pub mod blocks;
pub mod bookmarks;
pub mod edit;
pub mod file_operations;
pub mod files;
pub mod inline;
pub mod modules;
pub mod tools;

use gtk4::{gio, prelude::*};

// Re-export commonly used types
pub use about::show_about_dialog;
pub use bookmarks::{refresh_bookmark_menu, setup_bookmark_actions};
pub use file_operations::{
    attach_change_tracker, register_file_actions_async, setup_recent_actions,
    update_recent_files_menu, FileOperations, InitialFileLoadContext, SaveChangesResult,
};
pub use files::FileDialogs;

pub fn setup_inline_blocks_modules_actions(
    app: &gtk4::Application,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    window: &gtk4::ApplicationWindow,
    settings_manager: std::sync::Arc<core::logic::swanson::SettingsManager>,
    current_file_provider: std::rc::Rc<dyn Fn() -> Option<std::path::PathBuf>>,
) {
    inline::setup_inline_actions(
        app,
        editor_buffer,
        editor_view,
        window,
        settings_manager,
        current_file_provider,
    );
    blocks::setup_block_actions(app, editor_buffer, editor_view, window);
    modules::setup_modules_actions(app, editor_buffer, editor_view, window);
}

fn add_format_action<F>(app: &gtk4::Application, name: &str, activate: F)
where
    F: Fn() + 'static,
{
    if app.lookup_action(name).is_some() {
        return;
    }

    let action = gio::SimpleAction::new(name, None);
    action.connect_activate(move |_, _| activate());
    app.add_action(&action);
}

fn refocus(buf: &sourceview5::Buffer, view: &sourceview5::View) {
    let mut iter = buf.iter_at_offset(buf.cursor_position());
    buf.place_cursor(&iter);
    view.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

fn insert_block_snippet(text_buffer: &gtk4::TextBuffer, snippet: &str) {
    let cursor = text_buffer.iter_at_offset(text_buffer.cursor_position());
    let at_line_start = cursor.starts_line();
    let prefix = if at_line_start { "" } else { "\n" };
    let text = format!("{}{}\n", prefix, snippet);

    text_buffer.begin_user_action();
    let mut insert_pos = text_buffer.iter_at_offset(text_buffer.cursor_position());
    text_buffer.insert(&mut insert_pos, &text);
    text_buffer.end_user_action();
}
