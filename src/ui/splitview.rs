// Type alias for clarity: MarkdownEditorView is just a View
type MarkdownEditorView = sourceview5::View;
use gtk4::{Orientation, Paned, ScrolledWindow, Stack};

/// Create a configured Paned (split view) widget for the editor
pub fn create_editor_split_pane() -> Paned {
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_position(400); // Initial position
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned
}

/// Set the split ratio for a Paned widget
pub fn set_split_ratio(paned: &Paned, total_width: i32) {
    paned.set_position(total_width / 2);
}
use gtk4::prelude::*;
use sourceview5::prelude::*;
use sourceview5::{Buffer};
use crate::view::{MarkdownHtmlView, MarkdownCodeView};

pub struct SplitViewWidgets {
    pub paned: Paned,
    pub editor_data_buffer: Buffer,
    pub editor_view: MarkdownEditorView,
    pub source_scroll: ScrolledWindow,
    pub view_stack: Stack,
    pub html_view: MarkdownHtmlView,
    pub code_view: MarkdownCodeView,
}

/// Create all widgets for the split view UI and return them as a struct
pub fn create_splitview_ui() -> SplitViewWidgets {
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_position(400);
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    let source_buffer = Buffer::new(None);
    let editor_view = MarkdownEditorView::with_buffer(&source_buffer);
    editor_view.set_show_line_numbers(true);
    editor_view.set_highlight_current_line(true);
    editor_view.set_tab_width(4);
    editor_view.set_insert_spaces_instead_of_tabs(true);
    editor_view.set_auto_indent(true);

    let html_view = MarkdownHtmlView::new();
    let code_view = MarkdownCodeView::new();

    let view_stack = Stack::new();
    view_stack.set_vexpand(true);
    view_stack.add_named(html_view.widget(), Some("html"));
    view_stack.add_named(code_view.widget(), Some("code"));
    view_stack.set_visible_child_name("html");

    let source_scroll = ScrolledWindow::new();
    source_scroll.set_child(Some(&editor_view));
    source_scroll.set_vexpand(true);
    source_scroll.set_size_request(200, -1);

    paned.set_start_child(Some(&source_scroll));
    paned.set_end_child(Some(&view_stack));

    SplitViewWidgets {
        paned,
        editor_data_buffer: source_buffer,
        editor_view,
        source_scroll,
        view_stack,
        html_view,
        code_view,
    }
}
