// Editor module
pub mod editor;

pub use editor::MarkdownEditor;

// Function to create basic editor structure for main.rs
pub fn create_editor_structure() -> gtk4::Widget {
    let editor = MarkdownEditor::new();
    editor.widget().clone()
}