use gtk4::prelude::*;
use gtk4::{Orientation, Paned};

/// Create a basic split view structure
pub fn create_split_view() -> Paned {
    let paned = Paned::new(Orientation::Horizontal);
    paned.add_css_class("split-view");
    paned.set_position(400); // Initial position
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned
}