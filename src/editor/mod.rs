// Editor module
pub mod editor;

pub use editor::render_editor;

// Function to create basic editor structure for main.rs
pub fn create_editor_structure() -> gtk4::Widget {
    let editor = render_editor::new();
    editor.widget().clone()
}