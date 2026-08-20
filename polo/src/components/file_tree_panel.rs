//! File-tree sidebar panel for Polo — browse a directory tree and open a
//! markdown file from it, without going through the native file-chooser
//! dialog every time.
//!
//! Architecture (see `dir.md` §1–§2 for the full plan this implements):
//!
//! ```text
//! Filesystem (std::fs, via list_tree_entries)
//!   → DirEntry (glib::Object subclass)
//!   → gio::ListStore (one per directory level, built lazily)
//!   → gtk4::TreeListModel (lazy child loading on expand)
//!   → gtk4::FilterListModel (search-as-you-type, name match)
//!   → gtk4::SingleSelection
//!   → gtk4::ListView + SignalListItemFactory (virtualized rows)
//! ```
//!
//! The tree is read-only navigation (dir.md §4): no rename/delete/new-file,
//! no context menu, no drag-and-drop. Only `.md`/`.markdown`/`.mdown`/`.mkd`
//! files and non-hidden directories are shown — this is a markdown viewer's
//! file tree, not a general file browser.
//!
//! Roots (dir.md §3, adjusted after review): two fixed top-level nodes are
//! always present, each a normal collapsed row with its own chevron —
//! "Root" on Linux (`/`) / "This PC" on Windows (enumerated drives via
//! `GetLogicalDrives`) listed first, then "Home" (the user's home/profile
//! directory, labeled with the username on Windows to match Explorer's
//! `%USERPROFILE%` label). They behave as an accordion: expanding one
//! collapses the other if it was open. No separate switcher UI — the two
//! rows *are* the switcher.
//!
//! Icons deliberately deviate from dir.md §7's GResource + `IconTheme`
//! symbolic setup: Polo's established convention is inline SVG constants
//! rendered through `menu::render_svg_texture` (rsvg + cairo) with palette
//! colors, re-rendered when the window's `.marco-theme-*` class flips — the
//! same trigger the toolbar icons use. The SVG sources are dir.md §11's set.
//!
//! Structurally the panel still mirrors `toc_panel.rs` (own `Paned`,
//! `show`/`hide`, self-contained CSS injection) — see that module's doc
//! comment for the layout diagram.

use super::file_tree_search::{
    build_search_visible, effective_active_folder, is_active_folder, reveal_matches,
    update_scope_ui, SearchState,
};
use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
use crate::components::menu::render_svg_texture;
use crate::components::sidebar_coordinator::{
    SharedPanelWidth, DEFAULT_SIDEBAR_WIDTH, MIN_SIDEBAR_WIDTH,
};
use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::time::Duration;

/// Markdown extensions shown as leaf nodes (dir.md §4).
const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown", "mdown", "mkd"];

/// Logical icon size in px; rendered at 2x and downscaled by the `Image`'s
/// pixel size, same supersampling reasoning as `render_svg_texture`.
const ICON_SIZE: f64 = 16.0;

// ── Inline SVG icons (dir.md §11, Tabler-style outline set) ───────────────

const SVG_HOME: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 12l-2 0l9 -9l9 9l-2 0"/><path d="M5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-7"/><path d="M9 21v-6a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v6"/></svg>"#;

const SVG_FOLDER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 4h4l3 3h7a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2"/></svg>"#;

const SVG_FOLDER_OPEN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 19l2.757 -7.351a1 1 0 0 1 .936 -.649h12.307a1 1 0 0 1 .986 1.164l-.996 5.211a2 2 0 0 1 -1.964 1.625h-14.026a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2h4l3 3h7a2 2 0 0 1 2 2v2"/></svg>"#;

const SVG_FILE_TEXT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M14 3v4a1 1 0 0 0 1 1h4"/><path d="M17 21h-10a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2h7l5 5v11a2 2 0 0 1 -2 2"/><path d="M9 9l1 0"/><path d="M9 13l6 0"/><path d="M9 17l6 0"/></svg>"#;

#[cfg(not(target_os = "windows"))]
const SVG_FOLDER_ROOT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M10 13a2 2 0 1 0 4 0a2 2 0 1 0 -4 0"/><path d="M12 15v4"/><path d="M5 4h4l3 3h7a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2"/></svg>"#;

#[cfg(target_os = "windows")]
const SVG_DRIVE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M22 12H2m3.45-6.89L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11M6 16h.01M10 16h.01"/></svg>"#;

/// Which glyph a tree node renders with (dir.md §7's icon-to-node mapping).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(crate) enum IconKind {
    /// The user's home directory, wherever it appears in the tree (dir.md
    /// §3) — not a root node itself anymore, see [`dir_icon_kind`].
    Home,
    /// Linux full-system root node (`/`).
    #[cfg(not(target_os = "windows"))]
    SystemRoot,
    /// Windows "This PC" container node — its children are the enumerated
    /// drives, computed specially in `create_children_model` rather than
    /// via `read_dir` (see [`IconKind::Drive`] for the drives themselves).
    #[cfg(target_os = "windows")]
    ThisPc,
    /// Windows drive node (`C:\`, `D:\`, …), child of the `ThisPc` node.
    #[cfg(target_os = "windows")]
    Drive,
    #[default]
    Folder,
    File,
}

// ── DirEntry: the GObject data model (dir.md §2 Step 1) ───────────────────

mod imp {
    use gtk4::glib;
    use gtk4::glib::subclass::prelude::*;
    use std::cell::{Cell, OnceCell};
    use std::path::PathBuf;

    /// Instance data for [`super::DirEntry`]. Fields are set exactly once at
    /// construction (`OnceCell`) — entries are immutable snapshots of a
    /// directory listing, replaced wholesale on refresh rather than mutated.
    #[derive(Default)]
    pub struct DirEntry {
        pub(super) path: OnceCell<PathBuf>,
        pub(super) display_name: OnceCell<String>,
        pub(super) is_dir: Cell<bool>,
        pub(super) kind: Cell<super::IconKind>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DirEntry {
        const NAME: &'static str = "PoloFileTreeDirEntry";
        type Type = super::DirEntry;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DirEntry {}
}

glib::wrapper! {
    /// One filesystem entry in the tree — the item type stored in every
    /// `gio::ListStore` level of the `TreeListModel`.
    pub struct DirEntry(ObjectSubclass<imp::DirEntry>);
}

impl DirEntry {
    fn new(path: PathBuf, display_name: String, is_dir: bool, kind: IconKind) -> Self {
        use gtk4::glib::subclass::prelude::*;
        let obj: Self = glib::Object::new();
        obj.imp().path.set(path).expect("path set once");
        obj.imp()
            .display_name
            .set(display_name)
            .expect("display_name set once");
        obj.imp().is_dir.set(is_dir);
        obj.imp().kind.set(kind);
        obj
    }

    fn path(&self) -> &Path {
        use gtk4::glib::subclass::prelude::*;
        self.imp().path.get().expect("path set at construction")
    }

    fn display_name(&self) -> &str {
        use gtk4::glib::subclass::prelude::*;
        self.imp()
            .display_name
            .get()
            .expect("display_name set at construction")
    }

    fn is_dir(&self) -> bool {
        use gtk4::glib::subclass::prelude::*;
        self.imp().is_dir.get()
    }

    fn kind(&self) -> IconKind {
        use gtk4::glib::subclass::prelude::*;
        self.imp().kind.get()
    }
}

// ── Roots (dir.md §3) ─────────────────────────────────────────────────────

/// Which top-level view the tree is rooted at. `Default` is the normal
/// state: two fixed accordion nodes, Home and Root/This PC. `Custom` is
/// reached only through [`FileTreePanelHandle::navigate_to`] and replaces
/// both with a single arbitrary directory. `pub(crate)`: also read by
/// `file_tree_search.rs` (search scope is always relative to this root).
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum TreeRoot {
    /// Home + Root/This PC, both always present, accordion-style expansion.
    Default,
    /// An arbitrary directory set programmatically.
    Custom(PathBuf),
}

/// The user's home/profile directory (`dirs` crate — dir.md §3). `pub(crate)`:
/// also read by `file_tree_search.rs` as the search-scope fallback.
pub(crate) fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

/// Enumerate drive letters as `C:\`-style paths via the `GetLogicalDrives`
/// bitmask — dir.md §3's no-`sysinfo` alternative; the `windows` crate is
/// already a dependency.
#[cfg(target_os = "windows")]
fn list_drive_roots() -> Vec<PathBuf> {
    use windows::Win32::Storage::FileSystem::GetLogicalDrives;
    let mask = unsafe { GetLogicalDrives() };
    (0u32..26)
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| PathBuf::from(format!("{}:\\", char::from(b'A' + i as u8))))
        .collect()
}

/// Top-level entries for the current root mode. Always directories; each
/// becomes a lazily-expandable node exactly like any subdirectory.
///
/// `Default` always returns exactly one entry: the single full-system root
/// — "root" (`/`) on Linux, "This PC" on Windows. The user's home directory
/// is no longer a top-level node; it's reached (and auto-expanded to, see
/// [`FileTreePanelHandle::rebuild_top_level`]) as an ordinary descendant of
/// this root, distinguished only by [`IconKind::Home`]'s icon (see
/// [`dir_icon_kind`]). The Windows "This PC" entry's path is an unused empty
/// sentinel; its children are computed specially in
/// [`create_children_model`], never via `read_dir`.
fn build_root_entries(root: &TreeRoot) -> Vec<DirEntry> {
    match root {
        TreeRoot::Default => {
            #[cfg(target_os = "windows")]
            let system_entry = DirEntry::new(
                PathBuf::new(),
                "This PC".to_string(),
                true,
                IconKind::ThisPc,
            );
            #[cfg(not(target_os = "windows"))]
            let system_entry = DirEntry::new(
                PathBuf::from("/"),
                "root".to_string(),
                true,
                IconKind::SystemRoot,
            );

            vec![system_entry]
        }
        TreeRoot::Custom(dir) => {
            let label = dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| dir.display().to_string());
            vec![DirEntry::new(dir.clone(), label, true, IconKind::Folder)]
        }
    }
}

// ── Filesystem listing (unchanged logic, extended extension list) ─────────

/// Read `dir` and return its markdown-viewer-relevant entries: non-hidden
/// subdirectories and markdown files (see [`MARKDOWN_EXTENSIONS`]), sorted
/// directories-first then alphabetically (case-insensitive) within each
/// group. Pure filesystem logic, no GTK — kept separate so it's
/// unit-testable without a live display (this project's test suite is
/// deliberately headless-safe). `pub(crate)`: also the inclusion-rule
/// source `file_tree_search.rs`'s recursive walk reuses (§3 Approach D).
pub(crate) fn list_tree_entries(dir: &Path) -> std::io::Result<Vec<(PathBuf, String, bool)>> {
    let mut entries: Vec<(PathBuf, String, bool)> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None; // hidden
            }
            let is_dir = path.is_dir();
            if !is_dir {
                let is_markdown =
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| {
                            MARKDOWN_EXTENSIONS
                                .iter()
                                .any(|m| ext.eq_ignore_ascii_case(m))
                        });
                if !is_markdown {
                    return None;
                }
            }
            Some((path, name, is_dir))
        })
        .collect();

    entries.sort_by(
        |(_, a_name, a_is_dir), (_, b_name, b_is_dir)| match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a_name.to_lowercase().cmp(&b_name.to_lowercase()),
        },
    );

    Ok(entries)
}

/// The `TreeListModel` lazy-loading closure (dir.md §2 Step 3): called the
/// first time a row needs its children. Returns `None` for files, for
/// unreadable directories (permission denied → the row simply shows no
/// expander, dir.md §4), and for directories with nothing to show.
fn create_children_model(item: &glib::Object) -> Option<gio::ListModel> {
    let entry = item.downcast_ref::<DirEntry>()?;
    if !entry.is_dir() {
        return None;
    }

    // Windows "This PC" node: its children are the enumerated drives, not
    // a `read_dir` of its (unused, empty) path.
    #[cfg(target_os = "windows")]
    if entry.kind() == IconKind::ThisPc {
        let drives = list_drive_roots();
        if drives.is_empty() {
            return None;
        }
        let store = gio::ListStore::new::<DirEntry>();
        for path in drives {
            let name = path.display().to_string();
            store.append(&DirEntry::new(path, name, true, IconKind::Drive));
        }
        return Some(store.upcast());
    }

    let children = match list_tree_entries(entry.path()) {
        Ok(c) => c,
        Err(e) => {
            log::warn!(
                "[polo] file tree: cannot read {}: {}",
                entry.path().display(),
                e
            );
            return None;
        }
    };
    if children.is_empty() {
        return None;
    }
    let home = home_dir();
    let store = gio::ListStore::new::<DirEntry>();
    for (path, name, is_dir) in children {
        let kind = if is_dir {
            dir_icon_kind(&path, &home)
        } else {
            IconKind::File
        };
        store.append(&DirEntry::new(path, name, is_dir, kind));
    }
    Some(store.upcast())
}

/// Icon kind for a plain (non-root) directory entry: the distinguished home
/// icon (dir.md §3) if `path` is the user's home directory, otherwise the
/// regular folder icon. Applied wherever the home directory shows up in the
/// tree — not just at startup — so it stays visually identifiable no matter
/// where the user has navigated to since (see [`create_children_model`]).
fn dir_icon_kind(path: &Path, home: &Path) -> IconKind {
    if path == home {
        IconKind::Home
    } else {
        IconKind::Folder
    }
}

// ── Icon rendering ────────────────────────────────────────────────────────

/// Lazily-rendered texture per (glyph, theme) pair — rendering an SVG
/// through rsvg per bind would be wasteful for a virtualized list that
/// rebinds rows constantly while scrolling.
#[derive(Clone, Default)]
struct IconCache(Rc<RefCell<HashMap<(usize, bool), gdk::MemoryTexture>>>);

impl IconCache {
    fn get(&self, svg: &'static str, is_dark: bool) -> gdk::MemoryTexture {
        let key = (svg.as_ptr() as usize, is_dark);
        if let Some(t) = self.0.borrow().get(&key) {
            return t.clone();
        }
        let color = if is_dark {
            DARK_PALETTE.control_icon
        } else {
            LIGHT_PALETTE.control_icon
        };
        let texture = render_svg_texture(svg, color, ICON_SIZE);
        self.0.borrow_mut().insert(key, texture.clone());
        texture
    }
}

/// Pick and apply the right glyph for a row (folder rows swap between
/// closed/open with the row's expansion state — dir.md §7's mapping).
fn set_row_icon(
    icon: &gtk4::Image,
    entry: &DirEntry,
    expanded: bool,
    is_dark: bool,
    cache: &IconCache,
) {
    let svg: &'static str = match entry.kind() {
        IconKind::Home => SVG_HOME,
        #[cfg(not(target_os = "windows"))]
        IconKind::SystemRoot => SVG_FOLDER_ROOT,
        #[cfg(target_os = "windows")]
        IconKind::ThisPc | IconKind::Drive => SVG_DRIVE,
        IconKind::Folder => {
            if expanded {
                SVG_FOLDER_OPEN
            } else {
                SVG_FOLDER
            }
        }
        IconKind::File => SVG_FILE_TEXT,
    };
    icon.set_paintable(Some(&cache.get(svg, is_dark)));
}

// ── Panel handle & construction ───────────────────────────────────────────

/// Callback invoked with the path of a markdown file the user activated in
/// the tree.
pub type OpenFileCallback = Rc<dyn Fn(&Path)>;

/// Late-bound slot for [`OpenFileCallback`] — same reasoning as
/// `title_label_for_header` in `main.rs`: the panel (and the sidebar
/// mutual-exclusion coordinator built on top of it) needs to exist *before*
/// the toolbar/titlebar are constructed, but the callback itself needs
/// `open_editor_btn` and the real title `Label`, which only exist *after*
/// them. Filled in once, later in `main.rs::build_ui`.
type OpenFileCallbackSlot = Rc<RefCell<Option<OpenFileCallback>>>;

/// Handle that lets the rest of the app show/hide the file-tree panel and
/// change its root directory.
#[derive(Clone)]
pub struct FileTreePanelHandle {
    panel_box: gtk4::Box,
    paned: gtk4::Paned,
    visible: Rc<Cell<bool>>,
    root: Rc<RefCell<TreeRoot>>,
    open_file_cb: OpenFileCallbackSlot,
    /// Shared with the TOC panel — see [`SharedPanelWidth`].
    manual_width: SharedPanelWidth,
    /// Guards `paned`'s `notify::position` handler against mistaking our
    /// *own* `set_position` calls (opening to the shared default/manual
    /// width, or collapsing to 0 on hide) for a user drag. Held for the
    /// *entire* show/hide transition — from before `panel_box`'s visibility
    /// even changes through to after the deferred `set_position` call
    /// completes — not just around the `set_position` call itself; see
    /// [`FileTreePanelHandle::show`]'s doc comment for why the wider window
    /// matters.
    programmatic_resize: Rc<Cell<bool>>,
    list_view: gtk4::ListView,
    scrolled: gtk4::ScrolledWindow,
    empty_label: gtk4::Label,
    filter: gtk4::CustomFilter,
    search: gtk4::SearchEntry,
    /// The currently-open file, if any — shared with the row factory's bind
    /// closure (persistent `.open-file` marker, dir.md §9) and the
    /// activate handler (tree clicks), and read by [`Self::reveal_open_path`].
    open_path: Rc<RefCell<Option<PathBuf>>>,
    /// The last folder the user clicked in the tree, if any — the "where
    /// you are" blue marker (POLO_TREE_SEARCH_HIGHLIGHTING.md §2, revised)
    /// follows this, falling back to Home/the custom root when `None` (no
    /// folder clicked yet this session, or just re-rooted — see
    /// [`Self::navigate_to`]). Deliberately independent of `open_path`:
    /// clicking a folder and opening a file are two separate "where you
    /// are" signals that can both be true on-screen at once.
    active_folder: Rc<RefCell<Option<PathBuf>>>,
    /// The live `TreeListModel` backing the view, kept around so
    /// [`Self::reveal_open_path`] can expand/search it directly — it sits
    /// upstream of the search `FilterListModel`, so walking it is unaffected
    /// by whatever the user last typed into the search box.
    tree_model: Rc<RefCell<Option<gtk4::TreeListModel>>>,
    /// Current theme, read at bind time for icon colors; kept in sync with
    /// the window's `.marco-theme-*` CSS class.
    is_dark: Rc<Cell<bool>>,
    /// Currently-bound row widgets by position, shared with the row
    /// factory — see the field's own doc comment where it's declared in
    /// [`create_file_tree_panel`]. Read by [`Self::reveal_open_path`] to
    /// center a programmatically-revealed row.
    row_widgets: Rc<RefCell<HashMap<u32, gtk4::Box>>>,
    /// Generation counter for in-flight centering animations — see
    /// [`schedule_center_row`].
    h_scroll_gen: Rc<Cell<u64>>,
}

impl FileTreePanelHandle {
    /// Whether the panel is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    /// Fill the late-bound open-file callback. See [`OpenFileCallbackSlot`].
    pub fn set_open_file_callback(&self, cb: OpenFileCallback) {
        *self.open_file_cb.borrow_mut() = Some(cb);
    }

    /// Show the panel. Builds the tree from the current root the first time
    /// this is called; otherwise reuses whatever was already built. Either
    /// way, always reveals and centers whatever file is currently open
    /// (dir.md §9) — the caller doesn't need to know whether that file was
    /// opened through the tree itself or some other route (dialog,
    /// drag-and-drop, Open Recent, startup).
    pub fn show(&self) {
        // Guard starts *before* the visibility flip below, not just around
        // the `set_position` call further down: making a previously-hidden,
        // 0-width start child visible forces GTK to immediately reconcile
        // that against the panel's enforced minimum width, which can itself
        // fire a `notify::position` for a transient, non-final value before
        // our own deferred `set_position` call ever runs. If that transient
        // notify isn't guarded too, it gets mistaken for a user drag and
        // persisted into `SharedPanelWidth` — silently replacing the
        // "standard" default with whatever that transient value was, so
        // every *subsequent* open used it instead of the real default. That
        // was the cause of a close/open cycle appearing to shrink the panel
        // even though nothing was ever dragged.
        self.programmatic_resize.set(true);
        self.panel_box.set_visible(true);
        self.visible.set(true);
        if self.list_view.model().is_none() {
            self.rebuild_top_level();
        }
        // Width: the user's manual override if they've set one (shared with
        // the TOC panel — see `SharedPanelWidth`), otherwise the shared
        // default. Either way clamped to the shared floor, in case a
        // sub-minimum value ever made it into `manual_width` (see
        // `SharedPanelWidth`'s doc comment).
        let width = self
            .manual_width
            .get()
            .unwrap_or(DEFAULT_SIDEBAR_WIDTH)
            .max(MIN_SIDEBAR_WIDTH);
        let paned = self.paned.clone();
        let programmatic = self.programmatic_resize.clone();
        let handle = self.clone();
        glib::idle_add_local_once(move || {
            paned.set_position(width);
            programmatic.set(false);
            // Deferred to the same idle turn as the width/position set above
            // so the panel has had a layout pass (needed for the centering
            // math in `reveal_open_path`, which reads the scrolled window's
            // adjustment).
            handle.reveal_open_path();
        });
    }

    /// Hide the panel.
    pub fn hide(&self) {
        self.programmatic_resize.set(true);
        self.panel_box.set_visible(false);
        self.visible.set(false);
        self.paned.set_position(0);
        self.programmatic_resize.set(false);
    }

    /// Record `path` as the file currently open in the viewer, regardless of
    /// how it got opened (tree click, File→Open, Open Recent, drag-and-drop,
    /// or the file loaded at startup) — every one of those call sites should
    /// call this so the tree's `.open-file` marker (dir.md §9) and
    /// auto-reveal stay accurate no matter which route the user took.
    ///
    /// Always refreshes the persistent marker; additionally reveals and
    /// centers the row immediately if the panel is already visible (so it
    /// doesn't go stale while the user has it open across several file
    /// opens). If the panel is hidden, the reveal happens lazily the next
    /// time [`Self::show`] is called instead.
    pub fn set_open_path(&self, path: Option<PathBuf>) {
        *self.open_path.borrow_mut() = path;
        refresh_visible_rows(&self.list_view);
        if self.visible.get() {
            self.reveal_open_path();
        }
    }

    /// Expand every ancestor of the currently-open file, select it, and
    /// scroll it to the vertical center of the panel (dir.md §9's
    /// auto-reveal, extended per feature request to center rather than just
    /// scroll-into-view). No-op if nothing is open, the tree hasn't been
    /// built yet, or the open file isn't reachable from the current root
    /// (see [`ancestor_chain`]).
    fn reveal_open_path(&self) {
        let Some(target) = self.open_path.borrow().clone() else {
            return;
        };
        let Some(tree_model) = self.tree_model.borrow().clone() else {
            return;
        };
        // Clear any active search first: `ancestor_chain`/`child_row` below
        // walk the raw `TreeListModel`, but the row's *displayed* position
        // (used for selection and centering) comes from the search
        // `FilterListModel` downstream of it — the two only line up 1:1
        // when the filter is passing everything through.
        self.search.set_text("");

        let root = self.root.borrow().clone();
        let Some(row) = expand_to(&tree_model, &root, &target) else {
            return;
        };

        let position = row.position();
        if let Some(selection) = self
            .list_view
            .model()
            .and_downcast::<gtk4::SingleSelection>()
        {
            selection.select_item(position, true);
        }
        // Rough pre-scroll: the target row may be far outside the
        // currently-realized range of this virtualized `ListView`, so
        // nothing downstream (`schedule_center_row`) can measure its real
        // geometry yet — this gets it realized/bound and roughly into view.
        // See `schedule_rough_center_row`'s doc comment for why this goes
        // through `GtkListView::scroll_to` rather than the vertical
        // adjustment directly.
        schedule_rough_center_row(&self.list_view, position);
        // Precise refinement pass: once the row above's rough scroll has
        // gotten the target realized, this measures its *actual* rendered
        // bounds and centers both axes exactly from that — `scroll_to` only
        // guarantees the row is in view, not that it's centered.
        schedule_center_row(
            &self.scrolled,
            &self.row_widgets,
            &self.h_scroll_gen,
            position,
        );
    }

    /// Re-root the whole panel at `dir` (a "custom" root, replacing the
    /// fixed Home/Root/This-PC accordion nodes) and rebuild the top-level
    /// listing. Does nothing if `dir` isn't an existing directory.
    ///
    /// Currently has no caller (the old ".. (up)" row was the only one);
    /// kept as the intended entry point for future "browse the opened
    /// file's folder"-style features.
    #[allow(dead_code)]
    pub fn navigate_to(&self, dir: PathBuf) {
        if !dir.is_dir() {
            log::warn!(
                "[polo] file tree: cannot navigate to non-directory {}",
                dir.display()
            );
            return;
        }
        *self.root.borrow_mut() = TreeRoot::Custom(dir);
        // Whatever folder was "where you are" under the old root has no
        // bearing on the new one — reset to the fallback (§ effective_
        // active_folder), which immediately becomes the new custom root
        // itself once `rebuild_top_level` re-binds every row below.
        *self.active_folder.borrow_mut() = None;
        self.rebuild_top_level();
    }

    /// Rebuild the whole model chain from the current root: fresh root
    /// `ListStore` → `TreeListModel` → `FilterListModel` → `SingleSelection`
    /// (dir.md §3: swapping roots rebuilds the chain — cheap, children are
    /// lazy). Either way there's exactly one top-level row now, and it's
    /// always auto-expanded: for a `Custom` root so its contents are
    /// immediately visible, for the `Default` root all the way down to the
    /// user's home directory (dir.md §3) so the user doesn't have to
    /// manually drill down to it on every launch.
    fn rebuild_top_level(&self) {
        // Sync theme state before any row binds — the panel may be built
        // before the first theme class change ever fires.
        if let Some(window) = self.panel_box.root() {
            let widget = window.upcast::<gtk4::Widget>();
            self.is_dark.set(widget.has_css_class("marco-theme-dark"));
        }

        let root = self.root.borrow().clone();
        // Scope indicators (POLO_TREE_SEARCH_DESIGN.md §4.2, highlighting §2
        // revised): named in the search entry's placeholder text before the
        // user types a character, and as a full-path tooltip on hover.
        // `active_folder` is reset to `None` by `navigate_to` right before
        // this runs, so a re-root immediately shows the new root itself
        // here, exactly like startup.
        update_scope_ui(&self.search, &root, &self.active_folder.borrow());
        let entries = build_root_entries(&root);
        if entries.is_empty() {
            self.empty_label.set_visible(true);
            self.scrolled.set_visible(false);
            self.list_view.set_model(None::<&gtk4::SingleSelection>);
            *self.tree_model.borrow_mut() = None;
            return;
        }
        self.empty_label.set_visible(false);
        self.scrolled.set_visible(true);

        let store = gio::ListStore::new::<DirEntry>();
        for e in &entries {
            store.append(e);
        }

        let tree_model = gtk4::TreeListModel::new(store, false, false, create_children_model);
        *self.tree_model.borrow_mut() = Some(tree_model.clone());
        match &root {
            TreeRoot::Custom(_) => {
                if let Some(row) = tree_model.item(0).and_downcast::<gtk4::TreeListRow>() {
                    row.set_expanded(true);
                }
            }
            TreeRoot::Default => {
                // `expand_to` only expands *ancestors* of its target (right
                // for revealing a file, which has nothing to show open) —
                // the home directory itself also needs `set_expanded(true)`
                // so its own contents are visible immediately on startup,
                // not just the folder itself.
                if let Some(row) = expand_to(&tree_model, &root, &home_dir()) {
                    row.set_expanded(true);
                }
            }
        }
        let filter_model = gtk4::FilterListModel::new(Some(tree_model), Some(self.filter.clone()));
        let selection = gtk4::SingleSelection::new(Some(filter_model));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);
        self.list_view.set_model(Some(&selection));
    }
}

/// Detach and reattach the view's model so every visible row re-binds —
/// used after a theme flip (icon textures must re-render in the new color)
/// or an open-file change. The selection/expansion state lives in the model
/// objects, which are kept, so nothing collapses or deselects. `pub(crate)`:
/// also called by `file_tree_search.rs`'s `SearchState::restart` to clear
/// stale markers immediately on every keystroke/folder click.
pub(crate) fn refresh_visible_rows(list_view: &gtk4::ListView) {
    if let Some(model) = list_view.model() {
        list_view.set_model(None::<&gtk4::SingleSelection>);
        list_view.set_model(Some(&model));
    }
}

// ── Auto-reveal (dir.md §9) ─────────────────────────────────────────────

/// The directories between `start` (exclusive) and `target` (inclusive), in
/// root-to-leaf order — e.g. `start=/home/kim`, `target=/home/kim/docs/a.md`
/// → `[/home/kim/docs, /home/kim/docs/a.md]`. Empty if `target` isn't under
/// `start` at all.
fn chain_under(start: &Path, target: &Path) -> Vec<PathBuf> {
    let Ok(rel) = target.strip_prefix(start) else {
        return Vec::new();
    };
    let mut cur = start.to_path_buf();
    let mut chain = Vec::new();
    for comp in rel.components() {
        cur.push(comp);
        chain.push(cur.clone());
    }
    chain
}

/// The chain of directory levels to expand from the tree's single top-level
/// row (position 0 — dir.md §3's redesign leaves exactly one root, so
/// there's no longer any "which root" to work out) down to (and including)
/// `target` itself — the pure-logic half of dir.md §9's auto-reveal. Returns
/// `None` if `target` isn't reachable from the root at all: outside a
/// `Custom` root, or (Windows) on a drive `list_drive_roots` didn't
/// enumerate — dir.md §9's documented edge case, where reveal is simply
/// skipped and the persistent `.open-file` marker is the only indicator.
/// `pub(crate)`: also used by `file_tree_search.rs`'s `build_search_visible`
/// to keep the path down to the search scope root visible.
pub(crate) fn ancestor_chain(root: &TreeRoot, target: &Path) -> Option<Vec<PathBuf>> {
    match root {
        TreeRoot::Custom(dir) => {
            if target == dir.as_path() || !target.starts_with(dir) {
                return None;
            }
            Some(chain_under(dir, target))
        }
        TreeRoot::Default => {
            #[cfg(not(target_os = "windows"))]
            {
                let root_path = Path::new("/");
                if target.starts_with(root_path) {
                    return Some(chain_under(root_path, target));
                }
                None
            }
            #[cfg(target_os = "windows")]
            {
                for drive in list_drive_roots() {
                    if target.starts_with(&drive) {
                        let mut chain = vec![drive.clone()];
                        chain.extend(chain_under(&drive, target));
                        return Some(chain);
                    }
                }
                None
            }
        }
    }
}

/// Find `target` among a `TreeListRow`'s already-loaded children (as
/// returned by `TreeListRow::children` after `set_expanded(true)`) and
/// return its index within that child list, for `TreeListRow::child_row`.
fn find_child_index(children: &gio::ListModel, target: &Path) -> Option<u32> {
    (0..children.n_items()).find(|&i| {
        children
            .item(i)
            .and_downcast::<DirEntry>()
            .is_some_and(|e| e.path() == target)
    })
}

/// Expand every ancestor of `target` (see [`ancestor_chain`]), from
/// `tree_model`'s single top-level row down to and including `target`
/// itself, returning the resulting `TreeListRow` — or `None` if `target`
/// isn't reachable. Shared by [`FileTreePanelHandle::reveal_open_path`]
/// (which goes on to select and center the row) and
/// [`FileTreePanelHandle::rebuild_top_level`]'s startup auto-expand to the
/// user's home directory (dir.md §3), which only needs the expansion side
/// effect and ignores the returned row. `pub(crate)`: also used by
/// `file_tree_search.rs`'s `reveal_matches` to lazily materialize and
/// expand ancestors of each search result.
pub(crate) fn expand_to(
    tree_model: &gtk4::TreeListModel,
    root: &TreeRoot,
    target: &Path,
) -> Option<gtk4::TreeListRow> {
    let chain = ancestor_chain(root, target)?;
    let mut row = tree_model.item(0).and_downcast::<gtk4::TreeListRow>()?;
    for ancestor in &chain {
        row.set_expanded(true);
        let children = row.children()?;
        let idx = find_child_index(&children, ancestor)?;
        row = row.child_row(idx)?;
    }
    Some(row)
}

/// Rough pre-scroll so the row at `position` (in the *unfiltered*
/// `TreeListModel`'s flat index space — callers must clear any active search
/// first, see `FileTreePanelHandle::reveal_open_path`) is realized/bound,
/// close enough to the viewport for [`schedule_center_row`]'s precise pass
/// to then measure it and animate the exact centering.
///
/// Goes through `GtkListView::scroll_to` (GTK 4.12+) rather than computing a
/// pixel offset and poking `scrolled`'s vertical `GtkAdjustment` directly —
/// an earlier version did the latter (via a `ROW_HEIGHT`-based formula,
/// mirroring how `schedule_center_row` itself has to fall back to measured
/// widget bounds instead of that same formula — see its doc comment) and it
/// raced `GtkListView`'s own virtualization bookkeeping: called right after
/// `refresh_visible_rows`'s detach/reattach (which forces every visible row
/// to rebind, so the target row was often *already* bound, just not yet
/// laid out), setting the adjustment value directly made GtkListView
/// immediately unbind that just-bound row as part of its own internal
/// recompute — and on this GTK4/Linux combination that unbind was never
/// followed by a rebind within any reasonable frame budget, so
/// `schedule_center_row`'s measurement poll simply never found the row
/// again and silently gave up. `scroll_to` is GTK's own sanctioned API for
/// "make this row of a virtualized list realized," so it drives the same
/// internal bookkeeping instead of fighting it.
fn schedule_rough_center_row(list_view: &gtk4::ListView, position: u32) {
    list_view.scroll_to(position, gtk4::ListScrollFlags::empty(), None);
}

// ── Centering on click / reveal (feature request: keep the active item
//    centered as deep nesting pushes it wide of the sidebar) ──────────────

/// Duration of the re-center animation. Quick and subtle per the feature
/// request — long enough to read as motion, short enough to not feel laggy
/// when the user is clicking through several rows quickly.
const H_SCROLL_ANIMATION_MS: f64 = 160.0;

/// Safety cap on frames spent waiting for the target row's layout to settle
/// before giving up (see `schedule_center_row`) — normally resolves within
/// 1-2 frames; this just bounds the worst case.
const H_SCROLL_MAX_WAIT_FRAMES: u32 = 30;

/// Schedule centering on the row at `position`, driven by a per-frame tick
/// callback with two phases: wait for the row's widget to report a real
/// (non-zero-area) layout, then animate the requested adjustment(s) to
/// center it — using `compute_bounds`'s actually-measured geometry rather
/// than any assumed row size. That matters on both axes: horizontally,
/// there's no way to assume a row's width up front at all (it depends on
/// depth and label text); vertically, an earlier version of this code used
/// a `ROW_HEIGHT` constant matching this app's own CSS `min-height: 24px`
/// — correct on Linux, where the row renders at exactly that, but a user
/// reported the reveal landing off-center on Windows, where GTK4's Win32
/// backend evidently doesn't render the row at quite that height (font
/// metrics/DPI scaling differ enough that a `position * ROW_HEIGHT` formula
/// drifts by a few px per row, compounding over a deep reveal into a
/// visible offset). Measuring the real widget sidesteps needing that
/// assumption to hold on every platform.
///
/// Polling frame-by-frame (rather than a single deferred
/// `glib::idle_add_local_once` lookup) is deliberate: a file click routes
/// through `FileTreePanelHandle::set_open_path` → `refresh_visible_rows`,
/// which detaches and reattaches the `ListView`'s model to force the
/// `.open-file` CSS marker onto the new row — and while the model is
/// momentarily unset, `GtkListView` has no content, which collapses its
/// adjustments' `upper` and clamps `value` to 0 as an ordinary
/// `GtkAdjustment` side effect. That collapse-and-clamp, and the rebind
/// that follows it, don't necessarily complete within a single idle turn,
/// so a one-shot lookup can read stale (zero-area) bounds and center on
/// nothing. Polling for real bounds sidesteps needing to know exactly how
/// many turns GTK's internal resize queue takes.
///
/// `h_scroll_gen` is bumped up front (not just when the animation phase
/// starts) so a newer call — another click, or the same click's *own*
/// `reveal_open_path`-driven follow-up — makes any older in-flight
/// wait-or-animate tick callback stop touching the adjustment(s) on its
/// next frame, rather than the two fighting over the value.
///
/// Every click — file, folder, or expand/collapse chevron alike — centers
/// both axes: even a folder toggle can leave the clicked row off-center
/// vertically if it wasn't already centered (e.g. it was sitting near the
/// top/bottom edge of the viewport when clicked), so there's no click type
/// that only needs one axis.
///
/// No-ops gracefully if the row's widget never turns up within
/// [`H_SCROLL_MAX_WAIT_FRAMES`] (e.g. scrolled back out of the
/// virtualization window before its layout settled, or simply never
/// scrolled close enough vertically to get realized at all; see
/// [`FileTreePanelHandle::reveal_open_path`]'s formula-based pre-scroll for
/// why that shouldn't happen in practice) — whatever selection/rough
/// positioning already happened stays, so this is a pure-polish miss, not a
/// broken state.
fn schedule_center_row(
    scrolled: &gtk4::ScrolledWindow,
    row_widgets: &Rc<RefCell<HashMap<u32, gtk4::Box>>>,
    h_scroll_gen: &Rc<Cell<u64>>,
    position: u32,
) {
    let my_gen = h_scroll_gen.get() + 1;
    h_scroll_gen.set(my_gen);
    let h_scroll_gen = h_scroll_gen.clone();
    let row_widgets = row_widgets.clone();
    let frames_waited = Rc::new(Cell::new(0u32));
    // (start_value, target_value) per axis, `None` when that axis isn't
    // being animated this call; shared start frame time for both.
    type AxisAnim = Option<(f64, f64)>;
    let animation: Rc<Cell<Option<(AxisAnim, AxisAnim, i64)>>> = Rc::new(Cell::new(None));
    scrolled.add_tick_callback(move |scrolled, clock| {
        if h_scroll_gen.get() != my_gen {
            return glib::ControlFlow::Break; // superseded by a newer call
        }
        if let Some((h, v, t0)) = animation.get() {
            let elapsed_ms = (clock.frame_time() - t0) as f64 / 1000.0;
            let progress = (elapsed_ms / H_SCROLL_ANIMATION_MS).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - progress).powi(3); // ease-out cubic
            if let Some((start, target)) = h {
                scrolled
                    .hadjustment()
                    .set_value(start + (target - start) * eased);
            }
            if let Some((start, target)) = v {
                scrolled
                    .vadjustment()
                    .set_value(start + (target - start) * eased);
            }
            return if progress >= 1.0 {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            };
        }

        // Still waiting for the row's layout to settle.
        let widget = row_widgets.borrow().get(&position).cloned();
        let bounds = widget
            .as_ref()
            .and_then(|w| w.compute_bounds(scrolled))
            .filter(|b| b.width() > 0.0 && b.height() > 0.0);
        let Some(bounds) = bounds else {
            let n = frames_waited.get() + 1;
            frames_waited.set(n);
            return if n < H_SCROLL_MAX_WAIT_FRAMES {
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            };
        };

        // Centers `adj` on `center` (in `adj`'s own coordinate space, i.e.
        // already offset by the current scroll value — see the `bounds.x()`/
        // `bounds.y()` reasoning below), clamped to the scrollable range.
        let axis_target = |adj: &gtk4::Adjustment, center: f64| -> Option<(f64, f64)> {
            let page_size = adj.page_size();
            if page_size <= 0.0 {
                return None;
            }
            let max_value = (adj.upper() - page_size).max(0.0);
            let target = (center - page_size / 2.0).clamp(0.0, max_value);
            let start = adj.value();
            if (target - start).abs() >= 1.0 {
                Some((start, target))
            } else {
                adj.set_value(target);
                None
            }
        };

        let hadj = scrolled.hadjustment();
        let vadj = scrolled.vadjustment();
        let h_center = hadj.value() + bounds.x() as f64 + bounds.width() as f64 / 2.0;
        let v_center = vadj.value() + bounds.y() as f64 + bounds.height() as f64 / 2.0;
        let h = axis_target(&hadj, h_center);
        let v = axis_target(&vadj, v_center);

        if h.is_none() && v.is_none() {
            return glib::ControlFlow::Break; // already centered (or no room to scroll)
        }
        animation.set(Some((h, v, clock.frame_time())));
        glib::ControlFlow::Continue
    });
}

/// Create the file-tree sidebar paned and return it together with the handle.
///
/// `initial_root`: `None` starts with the two fixed accordion roots, Home
/// and Root/This PC (dir.md §3); `Some(dir)` starts at that directory as a
/// single custom root instead.
///
/// `manual_width` is shared with the TOC panel — see [`SharedPanelWidth`] —
/// so a user-driven resize of either panel overrides both from then on.
///
/// The caller sets the end child, exactly like [`crate::components::toc_panel::create_toc_panel`],
/// and fills the open-file callback once it's available (see
/// [`FileTreePanelHandle::set_open_file_callback`]).
pub fn create_file_tree_panel(
    initial_root: Option<PathBuf>,
    manual_width: SharedPanelWidth,
) -> (gtk4::Paned, FileTreePanelHandle) {
    let paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(0); // collapsed by default
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned.set_resize_start_child(false);
    paned.set_resize_end_child(true);

    let programmatic_resize = Rc::new(Cell::new(false));
    // Detect a genuine user drag (vs. our own programmatic sets above/below,
    // all guarded by `programmatic_resize`) and remember it as the new
    // shared manual width. Registered *after* the initial `set_position(0)`
    // above so that call isn't mistaken for a user resize.
    {
        let manual_width = manual_width.clone();
        let programmatic = programmatic_resize.clone();
        paned.connect_notify_local(Some("position"), move |p, _| {
            if programmatic.get() {
                return;
            }
            let pos = p.position();
            // Collapsed, or below the enforced floor — the latter should be
            // unreachable via a real user drag (`shrink_start_child(false)`
            // plus the panel's own width_request/CSS min-width), but a
            // transient layout-driven notify has been observed to fire a
            // sub-minimum value here; see `SharedPanelWidth`'s doc comment
            // for why that must never get persisted.
            if pos < MIN_SIDEBAR_WIDTH {
                return;
            }
            manual_width.set(Some(pos));
        });
    }

    // ── File-tree panel (start child) ──────────────────────────────────
    let panel_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    panel_box.set_visible(false);
    panel_box.set_hexpand(false);
    panel_box.set_vexpand(true);
    panel_box.set_width_request(MIN_SIDEBAR_WIDTH);
    panel_box.add_css_class("file-tree-panel");

    let header = gtk4::Label::new(Some("Files"));
    header.set_halign(gtk4::Align::Start);
    header.set_margin_start(8);
    header.set_margin_top(6);
    header.set_margin_bottom(4);
    header.add_css_class("file-tree-panel-header");
    panel_box.append(&header);

    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    panel_box.append(&sep);

    // No separate root switcher: the tree has a single top-level row (dir.md
    // §3, revised) — "root" (Linux) / "This PC" (Windows) — auto-expanded
    // down to the user's home directory on first build (see
    // `rebuild_top_level`) rather than requiring the user to drill down.

    // Search-as-you-type filter (dir.md §6). Matches only what's already
    // loaded/expanded — the sanctioned v1 tradeoff for a lazy tree.
    let search = gtk4::SearchEntry::new();
    search.set_margin_start(6);
    search.set_margin_end(6);
    search.set_margin_bottom(4);
    search.add_css_class("file-tree-search");
    panel_box.append(&search);

    let empty_label = gtk4::Label::new(Some("Nothing to show"));
    empty_label.set_halign(gtk4::Align::Start);
    empty_label.add_css_class("file-tree-empty");
    empty_label.set_visible(false);
    panel_box.append(&empty_label);

    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);
    scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    // GTK4's default "overlay scrolling" draws the scrollbar on top of the
    // content and auto-hides it after a moment of inactivity — fine for a
    // trackpad-driven scroll, but easy to miss/hard to grab in a narrow
    // sidebar. Disabling it gives the vertical scrollbar its own reserved
    // column that stays put whenever the tree overflows.
    scrolled.set_overlay_scrolling(false);
    scrolled.add_css_class("file-tree-panel-scroll");
    scrolled.set_direction(gtk4::TextDirection::Ltr);

    // Shared bits captured by the factory/filter/activate closures.
    let open_file_cb: OpenFileCallbackSlot = Rc::new(RefCell::new(None));
    let is_dark = Rc::new(Cell::new(false));
    let icon_cache = IconCache::default();
    let open_path: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let active_folder: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let search_text: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    // Hoisted here (rather than built inline in the `FileTreePanelHandle`
    // literal below, as before) so the search closures below — which need
    // to know the current root and reach the live `TreeListModel` — can
    // share the exact same `Rc<RefCell<_>>` cells the handle ends up
    // holding, not a second, disconnected copy.
    let initial = match initial_root {
        Some(dir) => TreeRoot::Custom(dir),
        None => TreeRoot::Default,
    };
    let root: Rc<RefCell<TreeRoot>> = Rc::new(RefCell::new(initial));
    let tree_model: Rc<RefCell<Option<gtk4::TreeListModel>>> = Rc::new(RefCell::new(None));

    // Recursive search state (`file_tree_search.rs`, POLO_TREE_SEARCH_DESIGN.md
    // §4) — see `SearchState`'s own doc comment for what each field is for
    // and why it's bundled rather than several separate `Rc`s threaded
    // individually through every closure below.
    let (search_state, search_rx) = SearchState::new();

    // Filter: while Tier 2 hasn't produced results yet for the current
    // query, directories always pass (today's/Tier 1's behavior, §4.1) so
    // nothing flashes hidden while the background walk is still running.
    // Once Tier 2 is ready, only directories on the path to an actual match
    // stay visible (§4.4) — files always match on their own name either way.
    let filter = gtk4::CustomFilter::new({
        let search_text = search_text.clone();
        let search_state = search_state.clone();
        move |obj| {
            let query = search_text.borrow();
            if query.is_empty() {
                return true;
            }
            let Some(row) = obj.downcast_ref::<gtk4::TreeListRow>() else {
                return true;
            };
            let Some(entry) = row.item().and_downcast::<DirEntry>() else {
                return true;
            };
            if entry.is_dir() {
                return if search_state.ready.get() {
                    search_state.visible.borrow().contains(entry.path())
                } else {
                    true
                };
            }
            entry.display_name().to_lowercase().contains(query.as_str())
        }
    });
    // Row factory (dir.md §2 Step 5): TreeExpander → [icon, label].
    // Folder rows additionally watch their TreeListRow's `expanded` property
    // to swap the closed/open glyph; those per-row handlers are tracked here
    // and disconnected on unbind so recycled rows don't leak connections.
    let expanded_handlers: Rc<RefCell<HashMap<usize, (gtk4::TreeListRow, glib::SignalHandlerId)>>> =
        Rc::new(RefCell::new(HashMap::new()));

    // Currently-bound row content (icon+label box, excluding the
    // TreeExpander's leading indent) keyed by `ListItem::position` — lets
    // click/reveal handlers below look up a row's actual on-screen widget
    // (needed for `compute_bounds`-based horizontal centering, since GTK
    // gives no public "row N's pixel position" query) without holding a
    // stale reference across recycling.
    let row_widgets: Rc<RefCell<HashMap<u32, gtk4::Box>>> = Rc::new(RefCell::new(HashMap::new()));
    // Monotonic generation counter so a fresh horizontal-centering animation
    // (new click) cancels whatever animation was already in flight, instead
    // of the two fighting over `hadjustment`'s value frame-by-frame.
    let h_scroll_gen: Rc<Cell<u64>> = Rc::new(Cell::new(0));

    let factory = gtk4::SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        let item = item
            .downcast_ref::<gtk4::ListItem>()
            .expect("factory item is a ListItem");
        let expander = gtk4::TreeExpander::new();
        let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        row_box.add_css_class("file-tree-row");
        let icon = gtk4::Image::new();
        icon.set_pixel_size(ICON_SIZE as i32);
        // No hexpand, no ellipsize: the label is left at its natural
        // (full-text) width so a deeply-indented row's minimum width can
        // exceed the viewport, which is what makes the horizontal scrollbar
        // GtkScrolledWindow's Automatic policy shows for a MINIMUM-scroll-
        // policy child (GtkListView's default) actually kick in — see
        // inject_file_tree_css's scrollbar rules, styled for both axes.
        let label = gtk4::Label::new(None);
        label.set_xalign(0.0);
        label.set_halign(gtk4::Align::Start);
        row_box.append(&icon);
        row_box.append(&label);
        expander.set_child(Some(&row_box));
        item.set_child(Some(&expander));
    });
    {
        let is_dark = is_dark.clone();
        let icon_cache = icon_cache.clone();
        let open_path = open_path.clone();
        let root = root.clone();
        let active_folder = active_folder.clone();
        let search_state = search_state.clone();
        let expanded_handlers = expanded_handlers.clone();
        let row_widgets = row_widgets.clone();
        factory.connect_bind(move |_, item| {
            let item = item
                .downcast_ref::<gtk4::ListItem>()
                .expect("factory item is a ListItem");
            let Some(row) = item.item().and_downcast::<gtk4::TreeListRow>() else {
                return;
            };
            let Some(entry) = row.item().and_downcast::<DirEntry>() else {
                return;
            };
            let expander = item
                .child()
                .and_downcast::<gtk4::TreeExpander>()
                .expect("row child is a TreeExpander");
            expander.set_list_row(Some(&row));
            let row_box = expander
                .child()
                .and_downcast::<gtk4::Box>()
                .expect("expander child is a Box");
            let icon = row_box
                .first_child()
                .and_downcast::<gtk4::Image>()
                .expect("first row widget is the icon");
            let label = icon
                .next_sibling()
                .and_downcast::<gtk4::Label>()
                .expect("second row widget is the label");

            label.set_text(entry.display_name());
            if open_path.borrow().as_deref() == Some(entry.path()) {
                row_box.add_css_class("open-file");
            } else {
                row_box.remove_css_class("open-file");
            }
            // "Where you are" marker (POLO_TREE_SEARCH_HIGHLIGHTING.md §2,
            // revised): the last folder clicked, or Home/the custom root
            // as a fallback before anything's been clicked — same blue as
            // the open-file marker above, independent of it.
            if is_active_folder(&root.borrow(), &active_folder.borrow(), entry.path()) {
                row_box.add_css_class("scope-root");
            } else {
                row_box.remove_css_class("scope-root");
            }
            // Search-match marker (§3): only ever a file, never a directory
            // — ancestor directories stay default-colored even though
            // they're kept visible for path context (§4.4).
            if !entry.is_dir() && search_state.matches.borrow().contains(entry.path()) {
                row_box.add_css_class("search-match");
            } else {
                row_box.remove_css_class("search-match");
            }
            set_row_icon(&icon, &entry, row.is_expanded(), is_dark.get(), &icon_cache);

            row_widgets
                .borrow_mut()
                .insert(item.position(), row_box.clone());

            if entry.is_dir() {
                let handler = row.connect_expanded_notify({
                    let icon = icon.clone();
                    let entry = entry.clone();
                    let is_dark = is_dark.clone();
                    let icon_cache = icon_cache.clone();
                    move |row| {
                        set_row_icon(&icon, &entry, row.is_expanded(), is_dark.get(), &icon_cache);
                    }
                });
                expanded_handlers
                    .borrow_mut()
                    .insert(item.as_ptr() as usize, (row, handler));
            }
        });
    }
    {
        let expanded_handlers = expanded_handlers.clone();
        let row_widgets = row_widgets.clone();
        factory.connect_unbind(move |_, item| {
            let item = item
                .downcast_ref::<gtk4::ListItem>()
                .expect("factory item is a ListItem");
            if let Some((row, handler)) = expanded_handlers
                .borrow_mut()
                .remove(&(item.as_ptr() as usize))
            {
                row.disconnect(handler);
            }
            row_widgets.borrow_mut().remove(&item.position());
            if let Some(expander) = item.child().and_downcast::<gtk4::TreeExpander>() {
                expander.set_list_row(None);
            }
        });
    }

    let list_view = gtk4::ListView::new(None::<gtk4::SingleSelection>, Some(factory));
    list_view.set_vexpand(true);
    // Single click activates — preserves the old panel's one-click-to-open
    // behavior (and one-click expand/collapse for folders).
    list_view.set_single_click_activate(true);
    list_view.add_css_class("file-tree-view");

    {
        let search_text = search_text.clone();
        let filter = filter.clone();
        let search_state = search_state.clone();
        let root = root.clone();
        let active_folder = active_folder.clone();
        let list_view = list_view.clone();
        search.connect_search_changed(move |entry| {
            *search_text.borrow_mut() = entry.text().to_lowercase();
            // Tier 1: unchanged, instant reaction on every keystroke — see
            // the filter's own `ready` fallback above for why this alone
            // is still correct while Tier 2 hasn't caught up yet.
            filter.changed(gtk4::FilterChange::Different);

            let query = search_text.borrow().clone();
            let scope_root = effective_active_folder(&root.borrow(), &active_folder.borrow());
            search_state.restart(query, scope_root, &list_view);
        });
    }
    // Drain completed Tier 2 walks. Polled rather than woken directly from
    // the background thread because the closure a background thread could
    // hand to a thread-safe scheduler (`glib::idle_add_once`) must be
    // `Send`, and every piece of state it would need to touch here (the
    // filter, the tree model, the row widgets) is `Rc`-based and therefore
    // not `Send` — so the thread only ever sends plain (`u64`,
    // `HashSet<PathBuf>`) data, and this main-thread-only timer is what
    // actually applies it. 50ms is imperceptible next to the 200ms debounce
    // above and cheap to poll (an empty-channel `try_recv` is just an
    // atomic check).
    {
        let search_state = search_state.clone();
        let filter = filter.clone();
        let list_view = list_view.clone();
        let tree_model = tree_model.clone();
        let root = root.clone();
        let active_folder = active_folder.clone();
        glib::timeout_add_local(Duration::from_millis(50), move || {
            while let Ok((gen, matches)) = search_rx.try_recv() {
                if gen != search_state.gen.load(Ordering::Relaxed) {
                    continue; // superseded by a newer keystroke/folder click — discard
                }
                let root_ref = root.borrow().clone();
                let scope_root = effective_active_folder(&root_ref, &active_folder.borrow());
                if let Some(tree_model) = tree_model.borrow().as_ref() {
                    reveal_matches(tree_model, &root_ref, &matches);
                }
                let visible = build_search_visible(&root_ref, &scope_root, &matches);
                *search_state.matches.borrow_mut() = matches;
                *search_state.visible.borrow_mut() = visible;
                search_state.ready.set(true);
                filter.changed(gtk4::FilterChange::Different);
                refresh_visible_rows(&list_view);
            }
            glib::ControlFlow::Continue
        });
    }
    // Escape: clear the query and hand focus back to the tree (§4.7) —
    // `GtkSearchEntry` already fires `stop-search` on Escape by its own
    // built-in keybinding, so this only needs to react to it, not bind a
    // key itself.
    {
        let list_view = list_view.clone();
        search.connect_stop_search(move |entry| {
            entry.set_text("");
            list_view.grab_focus();
        });
    }
    // Enter: hand off from the search box to the results list, selecting
    // the first visible row (§4.7) — the standard `SearchEntry` → results
    // handoff Nautilus/VS Code's filter boxes use.
    {
        let list_view = list_view.clone();
        search.connect_activate(move |_| {
            let Some(model) = list_view.model() else {
                return;
            };
            if model.n_items() == 0 {
                return;
            }
            if let Some(selection) = model.downcast_ref::<gtk4::SingleSelection>() {
                selection.select_item(0, true);
            }
            list_view.grab_focus();
        });
    }

    {
        let open_file_cb = open_file_cb.clone();
        let open_path = open_path.clone();
        let active_folder = active_folder.clone();
        let root = root.clone();
        let search = search.clone();
        let search_text = search_text.clone();
        let search_state = search_state.clone();
        let scrolled_for_activate = scrolled.clone();
        let row_widgets = row_widgets.clone();
        let h_scroll_gen = h_scroll_gen.clone();
        list_view.connect_activate(move |lv, position| {
            let Some(model) = lv.model() else { return };
            let Some(row) = model.item(position).and_downcast::<gtk4::TreeListRow>() else {
                return;
            };
            let Some(entry) = row.item().and_downcast::<DirEntry>() else {
                return;
            };
            if entry.is_dir() {
                let now_expanded = !row.is_expanded();
                row.set_expanded(now_expanded);
                // Every click re-centers both axes on the clicked item,
                // folder or file alike (feature request point 2) — deferred
                // since expanding/collapsing just changed the tree's width
                // (and may have changed its height too, if collapsing
                // pulled the row up toward the top of the viewport).
                schedule_center_row(
                    &scrolled_for_activate,
                    &row_widgets,
                    &h_scroll_gen,
                    position,
                );
                // Clicking a folder makes it the "where you are" marker —
                // *and* the search scope (highlighting §2, revised: these
                // used to be decoupled, but a scope the blue marker lied
                // about was worse than no marker at all). Both the rebind
                // that moves the blue color and the search restart are
                // deferred exactly like `set_open_path`'s below — calling
                // `refresh_visible_rows` (which detaches/reattaches
                // `list_view`'s own model, and `SearchState::restart`
                // always does this too) synchronously from inside
                // `list_view`'s own `activate` handler is the same
                // reentrancy this file already hit once on Windows (see
                // the doc comment further down, on the file-open path).
                let clicked = entry.path().to_path_buf();
                *active_folder.borrow_mut() = Some(clicked);
                update_scope_ui(&search, &root.borrow(), &active_folder.borrow());
                let lv = lv.clone();
                let root = root.clone();
                let active_folder = active_folder.clone();
                let search_text = search_text.clone();
                let search_state = search_state.clone();
                glib::idle_add_local_once(move || {
                    let query = search_text.borrow().clone();
                    let scope_root =
                        effective_active_folder(&root.borrow(), &active_folder.borrow());
                    search_state.restart(query, scope_root, &lv);
                });
                return;
            }
            let path = entry.path().to_path_buf();
            *open_path.borrow_mut() = Some(path.clone());
            // Opening a file always routes through `open_file_cb` →
            // `FileTreePanelHandle::set_open_path`, the single source of
            // truth the doc comment on `set_open_path` establishes for "a
            // file just opened" regardless of entry point — it refreshes
            // the `.open-file` marker and (via `Self::reveal_open_path`)
            // re-centers both axes. An earlier version called `cb`
            // synchronously right here, which — unlike every *other* call
            // site of `set_open_path` (File→Open, Open Recent,
            // drag-and-drop, startup), all of which call in from outside
            // any `list_view` signal — runs from inside `list_view`'s own
            // `activate` signal handler. `set_open_path` cascades into
            // `refresh_visible_rows`, which detaches and reattaches
            // `list_view`'s own model: mutating a widget's model from
            // inside its own signal callback is reentrant, and on Windows
            // this reliably left the tree's scroll reset to the top after
            // the file opened — reproducing only via a click in the tree
            // itself, never via the other entry points, exactly matching
            // that structural difference. (A previously *deferred* second
            // call used to exist for an unrelated reason and was removed
            // for racing the centering animation — see git history — but
            // deferring the *only* call, as done here, sidesteps the
            // reentrancy instead of racing anything.) One idle turn is
            // enough: this just needs to run after the activate signal has
            // unwound, not to wait on any layout, unlike
            // `schedule_rough_center_row`'s polling.
            let open_file_cb = open_file_cb.clone();
            glib::idle_add_local_once(move || {
                let cb = open_file_cb.borrow().clone();
                if let Some(cb) = cb {
                    cb(&path);
                } else {
                    log::warn!(
                        "[polo] file tree: open-file callback not yet wired; ignoring click"
                    );
                }
            });
        });
    }
    scrolled.set_child(Some(&list_view));
    panel_box.append(&scrolled);

    paned.set_start_child(Some(&panel_box));

    inject_file_tree_css();

    let handle = FileTreePanelHandle {
        panel_box: panel_box.clone(),
        paned: paned.clone(),
        visible: Rc::new(Cell::new(false)),
        root,
        open_file_cb,
        manual_width,
        programmatic_resize,
        list_view: list_view.clone(),
        scrolled,
        empty_label,
        filter,
        search,
        open_path,
        active_folder,
        tree_model,
        is_dark: is_dark.clone(),
        row_widgets,
        h_scroll_gen,
    };

    // Keep `is_dark` in sync with Polo's own theme toggle: the
    // `.marco-theme-*` class lives on the top-level window, so watch its
    // `css-classes` property once the panel is mapped into a window — the
    // same notify the toolbar icons re-render from.
    {
        let hooked = Rc::new(Cell::new(false));
        let is_dark = is_dark.clone();
        let list_view = list_view.clone();
        panel_box.connect_map(move |pb| {
            if hooked.replace(true) {
                return;
            }
            let Some(window) = pb.root() else {
                return;
            };
            let widget = window.upcast::<gtk4::Widget>();
            let apply = {
                let is_dark = is_dark.clone();
                let list_view = list_view.clone();
                move |w: &gtk4::Widget| {
                    let dark = w.has_css_class("marco-theme-dark");
                    if is_dark.replace(dark) != dark {
                        refresh_visible_rows(&list_view);
                    }
                }
            };
            apply(&widget);
            widget.connect_notify_local(Some("css-classes"), move |w, _| apply(w));
        });
    }

    (paned, handle)
}

// ── CSS (dir.md §7: Carbon-style flat rows, theme-split via Polo's own
//    .marco-theme-* classes — never OS-level @accent_color) ────────────────

fn inject_file_tree_css() {
    use std::sync::OnceLock;
    static INJECTED: OnceLock<()> = OnceLock::new();
    if INJECTED.set(()).is_err() {
        return;
    }

    // Same visual language as `toc_panel.rs`'s `inject_toc_css` — same
    // colors, same scrollbar treatment — just a different class prefix
    // (`file-tree-*` instead of `toc-*`) so both panels read as one family
    // without one overriding the other's rules. Row states use
    // `alpha(currentColor, …)` (Carbon-style neutral grays) so they stay
    // correct in both themes without extra theme-split rules.
    // `.file-tree-panel`'s `min-width` below must match `MIN_SIDEBAR_WIDTH`
    // — kept as a literal (not `format!`) since this whole block is a plain
    // CSS string with no other substitutions.
    let css = r#"
/* File-tree panel */
.file-tree-panel {
    background: transparent;
    border-right: 1px solid alpha(currentColor, 0.15);
    min-width: 150px;
}
.marco-theme-light .file-tree-panel {
    background-color: #f0f2f4;
    border-right: 1px solid #d0d3d8;
}
.marco-theme-dark .file-tree-panel {
    background-color: #1e2025;
    border-right: 1px solid #3a3d44;
}
.file-tree-panel-header {
    font-weight: bold;
    font-size: 0.85em;
    opacity: 0.65;
    letter-spacing: 0.06em;
    text-transform: uppercase;
}
.marco-theme-light .file-tree-panel-header { color: #2c3e50; }
.marco-theme-dark  .file-tree-panel-header { color: #c8cdd6; }
.file-tree-empty {
    font-size: 0.85em;
    opacity: 0.5;
    margin: 8px;
}

/* Search entry */
entry.file-tree-search {
    min-height: 24px;
    border-radius: 4px;
}
.marco-theme-light entry.file-tree-search {
    background-color: #ffffff;
    color: #2c3e50;
}
.marco-theme-dark entry.file-tree-search {
    background-color: #2a2d33;
    color: #c8cdd6;
    caret-color: #c8cdd6;
}

/* Tree rows — Carbon-style: flat rectangles, compact height. No hover or
   selected background — the persistent state indicator is the `.open-file`
   text color — but a pressed row (mouse button down, `:active`) gets a
   soft, theme-adaptive tint so a click/hold reads as a deliberate, gentle
   press rather than nothing (or, previously, a stray theme artifact — see
   below) happening at all. */
listview.file-tree-view {
    background: transparent;
    padding: 2px;
}
/* TreeExpander's own CSS node tree (see gtk4 docs) always inserts either an
   `expander` node (real chevron, for a row with children) or an `indent`
   placeholder node (same slot, for a leaf row) — one such node per depth
   level, plus this final one for the row's own expand state. Neither is
   sized to match the other by default, so a leaf row's icon starts one
   column left of an expandable sibling's. Forcing both to the same
   min-width makes every row's icon land in the same column at a given
   depth, real chevron or not — pure CSS, no widget property needed. */
treeexpander indent,
treeexpander expander {
    min-width: 16px;
}
listview.file-tree-view > row {
    padding: 0 4px;
    min-height: 24px;
    border-radius: 0;
    background: transparent;
    /* Symmetric fade for the `:active` tint below — smooths both the
       press-in and the release back to transparent, so the highlight
       reads as a soft pulse rather than a hard on/off flash. */
    transition: background-color 100ms ease-out;
}
.marco-theme-light listview.file-tree-view > row { color: #2c3e50; }
.marco-theme-dark  listview.file-tree-view > row { color: #c8cdd6; }
/* Hover and selected states stay fully transparent — this row design has no
   persistent hover/selection background, only the `.open-file` text color.
   (Previously written as `background: transparent !important`, which is
   not valid GTK CSS — GTK's parser has no `!important` support, so every
   declaration in that block was silently dropped, `Gtk-WARNING: Junk at end
   of value for background`, and the base theme's own default hover/active
   shading showed through as a visual glitch. Application-priority
   providers already outrank the theme provider regardless of specificity —
   see `STYLE_PROVIDER_PRIORITY_APPLICATION` below — so plain, valid
   declarations are all that was ever needed.) */
listview.file-tree-view > row:hover,
listview.file-tree-view > row:selected,
listview.file-tree-view > row:hover:selected {
    background-color: transparent;
    background-image: none;
    box-shadow: none;
    outline: none;
}
/* Pressed state (`:active` — mouse button down, on a row or its chevron):
   a soft wash of the row's own text color at low alpha. `alpha(currentColor,
   …)` rather than a hardcoded color is what makes this theme-correct for
   free — `currentColor` resolves to the light-theme navy or dark-theme pale
   gray set above, so the tint is automatically a soft dark-on-light wash in
   light mode and a soft light-on-dark wash in dark mode, never a fixed
   color that only reads right against one background. */
listview.file-tree-view > row:active,
listview.file-tree-view > row:hover:active,
listview.file-tree-view > row:selected:active,
listview.file-tree-view > row:hover:selected:active {
    background-color: alpha(currentColor, 0.10);
    background-image: none;
    box-shadow: none;
    outline: none;
}

/* Currently-open file marker (dir.md §9): persistent, the sole visual
   state indicator on rows — color only, no background, no bold. */
.marco-theme-light .file-tree-row.open-file label { color: #2563eb; }
.marco-theme-dark  .file-tree-row.open-file label { color: #6cb2f7; }

/* "Where you are" marker (POLO_TREE_SEARCH_HIGHLIGHTING.md §2, revised):
   the last folder the user clicked, or Home/the active custom root as a
   fallback before anything's been clicked (see `effective_active_folder`).
   Deliberately the same blue as .open-file above, and deliberately
   independent of it — a clicked folder and an open file are two separate
   "where you are" signals that can both be true on-screen at once. Not
   the same thing as the search scope root (§4.2): clicking around moves
   this marker without changing where recursive search actually walks
   from. */
.marco-theme-light .file-tree-row.scope-root label { color: #2563eb; }
.marco-theme-dark  .file-tree-row.scope-root label { color: #6cb2f7; }

/* Search-match marker (§3): a *file* that matched the current query,
   gold/amber — the same colors VS Code uses for its modified-file
   decoration. Never applied to a directory row (see the `!entry.is_dir()`
   guard where this class is toggled) — ancestor folders shown for path
   context stay default-colored. */
.marco-theme-light .file-tree-row.search-match label { color: #895503; }
.marco-theme-dark  .file-tree-row.search-match label { color: #e2c08d; }

/* Scrollbar — matches toc-panel-scroll exactly */
scrolledwindow.file-tree-panel-scroll scrollbar {
    -gtk-icon-transform: none;
    min-width: 12px;
    min-height: 12px;
    background: transparent;
    border: none;
    box-shadow: none;
    padding: 0;
    margin: 0;
}
scrolledwindow.file-tree-panel-scroll scrollbar trough {
    border: none;
    box-shadow: none;
    background-image: none;
    min-width: 12px;
    min-height: 12px;
    padding: 0;
    margin: 0;
}
scrolledwindow.file-tree-panel-scroll scrollbar slider {
    border-radius: 0px;
    border: none;
    box-shadow: none;
    background-image: none;
    min-width: 12px;
    min-height: 12px;
    margin: 0;
    padding: 0;
}
.marco-theme-light scrolledwindow.file-tree-panel-scroll scrollbar trough {
    background-color: #F0F0F0;
}
.marco-theme-light scrolledwindow.file-tree-panel-scroll scrollbar slider {
    background-color: #D0D4D8;
}
.marco-theme-light scrolledwindow.file-tree-panel-scroll scrollbar slider:hover {
    background-color: #C2C7CC;
}
.marco-theme-dark scrolledwindow.file-tree-panel-scroll scrollbar trough {
    background-color: #252526;
}
.marco-theme-dark scrolledwindow.file-tree-panel-scroll scrollbar slider {
    background-color: #3A3F44;
}
.marco-theme-dark scrolledwindow.file-tree-panel-scroll scrollbar slider:hover {
    background-color: #4A4F55;
}
"#;

    let provider = gtk4::CssProvider::new();
    provider.load_from_string(css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Manually-managed temp directory (no `tempfile` dependency needed for
    /// one test) under `std::env::temp_dir()`, unique per test run via the
    /// process ID, removed on drop.
    struct TempDir(PathBuf);
    impl TempDir {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "polo_file_tree_test_{}_{}",
                label,
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir); // clean up any leftovers from a crashed prior run
            std::fs::create_dir_all(&dir).expect("create temp test dir");
            Self(dir)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn smoke_test_list_tree_entries_filters_and_sorts() {
        let tmp = TempDir::new("filters_and_sorts");
        let root = tmp.path();

        // Non-markdown file — must be excluded.
        std::fs::write(root.join("notes.txt"), "not markdown").unwrap();
        // Markdown files, deliberately out of alphabetical order and mixed
        // case, covering all four accepted extensions (dir.md §4).
        std::fs::write(root.join("zebra.md"), "# Z").unwrap();
        std::fs::write(root.join("Apple.MD"), "# A").unwrap(); // uppercase extension must still count
        std::fs::write(root.join("banana.markdown"), "# B").unwrap();
        std::fs::write(root.join("cherry.mdown"), "# C").unwrap();
        std::fs::write(root.join("date.mkd"), "# D").unwrap();
        // Hidden directory and hidden markdown file — must be excluded.
        std::fs::create_dir_all(root.join(".hidden_dir")).unwrap();
        std::fs::write(root.join(".hidden.md"), "# hidden").unwrap();
        // Regular subdirectories — included regardless of contents, sorted
        // before files, alphabetically among themselves.
        std::fs::create_dir_all(root.join("zzz_subdir")).unwrap();
        std::fs::create_dir_all(root.join("aaa_subdir")).unwrap();

        let entries = list_tree_entries(root).expect("read temp dir");
        let names: Vec<&str> = entries.iter().map(|(_, name, _)| name.as_str()).collect();

        // Directories first (alphabetical), then files (alphabetical,
        // case-insensitive) — hidden entries and the .txt file excluded.
        assert_eq!(
            names,
            vec![
                "aaa_subdir",
                "zzz_subdir",
                "Apple.MD",
                "banana.markdown",
                "cherry.mdown",
                "date.mkd",
                "zebra.md"
            ]
        );

        let dir_flags: Vec<bool> = entries.iter().map(|(_, _, is_dir)| *is_dir).collect();
        assert_eq!(
            dir_flags,
            vec![true, true, false, false, false, false, false]
        );
    }

    #[test]
    fn smoke_test_list_tree_entries_empty_dir() {
        let tmp = TempDir::new("empty_dir");
        let entries = list_tree_entries(tmp.path()).expect("read empty temp dir");
        assert!(entries.is_empty());
    }

    #[test]
    fn smoke_test_list_tree_entries_nonexistent_dir_errors() {
        let tmp = TempDir::new("nonexistent_parent");
        let missing = tmp.path().join("does_not_exist");
        assert!(list_tree_entries(&missing).is_err());
    }

    /// `DirEntry` is a plain `glib::Object` subclass — constructing it needs
    /// GObject type registration but no display, so this stays headless-safe
    /// like the rest of the suite.
    #[test]
    fn smoke_test_default_root_is_a_single_root_node() {
        let entries = build_root_entries(&TreeRoot::Default);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_dir());

        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(entries[0].display_name(), "root");
            assert_eq!(entries[0].path(), Path::new("/"));
        }
        #[cfg(target_os = "windows")]
        assert_eq!(entries[0].display_name(), "This PC");
    }

    #[test]
    fn dir_icon_kind_marks_only_the_home_directory() {
        let home = PathBuf::from("/home/alice");
        assert_eq!(dir_icon_kind(&home, &home), IconKind::Home);
        assert_eq!(dir_icon_kind(&home.join("docs"), &home), IconKind::Folder);
        assert_eq!(
            dir_icon_kind(Path::new("/home/bob"), &home),
            IconKind::Folder
        );
    }

    #[test]
    fn smoke_test_custom_root_labeled_by_dir_name() {
        let tmp = TempDir::new("custom_root");
        let sub = tmp.path().join("my_notes");
        std::fs::create_dir_all(&sub).unwrap();
        let entries = build_root_entries(&TreeRoot::Custom(sub.clone()));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].display_name(), "my_notes");
        assert_eq!(entries[0].path(), sub.as_path());
    }

    // ── Auto-reveal: ancestor_chain (pure path logic) ──────────────────────

    #[test]
    fn ancestor_chain_custom_root_walks_down_to_the_file() {
        let tmp = TempDir::new("ancestor_chain_custom");
        let sub = tmp.path().join("docs");
        std::fs::create_dir_all(&sub).unwrap();
        let target = sub.join("a.md");

        let root = TreeRoot::Custom(tmp.path().to_path_buf());
        let chain = ancestor_chain(&root, &target).expect("target is under root");
        assert_eq!(chain, vec![sub, target]);
    }

    #[test]
    fn ancestor_chain_custom_root_rejects_target_outside_root() {
        let tmp = TempDir::new("ancestor_chain_outside");
        let root = TreeRoot::Custom(tmp.path().join("a"));
        let target = tmp.path().join("b").join("file.md");
        assert!(ancestor_chain(&root, &target).is_none());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn ancestor_chain_default_root_walks_through_home_to_the_file() {
        let home = home_dir();
        let target = home.join("notes").join("a.md");
        let chain =
            ancestor_chain(&TreeRoot::Default, &target).expect("target is under / (and home)");
        // The single root is "/" now (dir.md §3, revised) — no separate Home
        // row to route through — but the chain still passes through `home`
        // itself as an ordinary intermediate ancestor, which is what lets
        // `dir_icon_kind` single it out for the home icon along the way.
        assert_eq!(chain, chain_under(Path::new("/"), &target));
        assert!(chain.contains(&home));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn ancestor_chain_default_root_walks_down_from_root() {
        let home = home_dir();
        let target = PathBuf::from("/etc/hosts");
        assert!(!target.starts_with(&home), "test assumes HOME isn't /etc");
        let chain = ancestor_chain(&TreeRoot::Default, &target).expect("target is under /");
        assert_eq!(chain, vec![PathBuf::from("/etc"), target]);
    }
}
