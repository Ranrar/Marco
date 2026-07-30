//! Mutual-exclusion coordinator for Polo's two sidebar panels — Table of
//! Contents and the file-tree browser (`file_tree_panel`). Only one is ever
//! visible at a time.
//!
//! Transition rules, as specified:
//! - Toggling either panel while it's the active one hides it (back to
//!   neither panel shown) — fully symmetric; the file tree does **not**
//!   fall back to TOC when closed, it just closes.
//! - Toggling a panel while the *other* one is active switches to it,
//!   closing whichever was open — only one panel is ever visible at a time.

use crate::components::file_tree_panel::FileTreePanelHandle;
use crate::components::toc_panel::TocPanelHandle;
use std::cell::Cell;
use std::rc::Rc;

/// Single source of truth for sidebar panel width, shared by both
/// `toc_panel.rs` and `file_tree_panel.rs` so the two can never disagree —
/// exactly two knobs:
///
/// - [`DEFAULT_SIDEBAR_WIDTH`]: the width either panel opens to at startup,
///   before the user has ever manually resized either one.
/// - [`SharedPanelWidth`] (`Some(w)`): the width either panel opens to once
///   the user *has* dragged either divider — replaced wholesale on every
///   subsequent manual resize, and in-memory only (not persisted across
///   restarts — "for the application's lifetime", not forever).
///
/// Both panels used to each pick their own default (TOC auto-sized to its
/// widest heading, the file tree used a fixed constant) — switching between
/// them then visibly resized the sidebar even though the user never touched
/// the divider. A single shared default removes that.
///
/// [`MIN_SIDEBAR_WIDTH`] is the floor for both knobs above. It's enforced
/// twice: as the hard GTK minimum (widget `width_request` + CSS `min-width`
/// in both panels, which stops an interactive drag going lower) and again
/// wherever a width is read from or written to `SharedPanelWidth`. The
/// second clamp matters even though the first should make it redundant: if
/// a sub-minimum value ever *does* slip into the cell (e.g. a transient
/// layout-driven `notify::position` firing as if it were a user drag), the
/// unclamped read used to feed that value straight back into
/// `Paned::set_position`, and a subsequent close/open cycle could clamp it
/// down again — a ratchet that made the sidebar collapse a little further
/// on every toggle. Clamping on both ends makes the stored value self-heal
/// back to the floor instead.
pub type SharedPanelWidth = Rc<Cell<Option<i32>>>;

/// See [`SharedPanelWidth`]'s doc comment.
pub const DEFAULT_SIDEBAR_WIDTH: i32 = 240;

/// See [`SharedPanelWidth`]'s doc comment.
pub const MIN_SIDEBAR_WIDTH: i32 = 150;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SidebarMode {
    None,
    Toc,
    Dir,
}

/// Handle shared by the toolbar and the View menu so a click on either the
/// TOC button/item or the file-tree button/item enforces the same
/// mutual-exclusion rules regardless of which one was used.
#[derive(Clone)]
pub struct SidebarCoordinator {
    mode: Rc<Cell<SidebarMode>>,
    toc: TocPanelHandle,
    dir: FileTreePanelHandle,
}

impl SidebarCoordinator {
    pub fn new(toc: TocPanelHandle, dir: FileTreePanelHandle) -> Self {
        Self {
            mode: Rc::new(Cell::new(SidebarMode::None)),
            toc,
            dir,
        }
    }

    /// Handle a click on the TOC toggle button/menu item.
    pub fn toggle_toc(&self) {
        match self.mode.get() {
            SidebarMode::Toc => {
                self.toc.hide();
                self.mode.set(SidebarMode::None);
            }
            SidebarMode::Dir | SidebarMode::None => {
                self.dir.hide();
                self.toc.show();
                self.mode.set(SidebarMode::Toc);
            }
        }
    }

    /// Handle a click on the file-tree toggle button/menu item.
    pub fn toggle_dir(&self) {
        match self.mode.get() {
            SidebarMode::Dir => {
                self.dir.hide();
                self.mode.set(SidebarMode::None);
            }
            SidebarMode::Toc | SidebarMode::None => {
                self.toc.hide();
                self.dir.show();
                self.mode.set(SidebarMode::Dir);
            }
        }
    }
}
