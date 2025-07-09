use crate::editor::MarkdownEditor;
use gtk4::prelude::*;
use gtk4::EmojiChooser;

/// Show the native GTK4 emoji chooser
pub fn show_emoji_picker_dialog(editor: &MarkdownEditor) {
    let emoji_chooser = EmojiChooser::new();

    // Set the source view as the parent for proper positioning
    let source_view = editor.source_view();
    emoji_chooser.set_parent(source_view);

    // Connect the emoji selection signal
    let editor_clone = editor.clone();
    emoji_chooser.connect_emoji_picked(move |chooser, emoji| {
        // Insert the emoji at the cursor position
        editor_clone.insert_text_at_cursor(emoji);

        // Close the emoji chooser after selection
        chooser.unparent();
    });

    // Show the emoji chooser
    emoji_chooser.popup();
}

/// Show emoji picker using keyboard shortcut (Ctrl+.)
pub fn show_emoji_picker_shortcut(editor: &MarkdownEditor) {
    show_emoji_picker_dialog(editor);
}
