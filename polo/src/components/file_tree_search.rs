//! Recursive tree search for the file-tree panel (`file_tree_panel.rs`) —
//! everything specific to *searching* the tree, split out from everything
//! specific to *being* the tree (data model, widget construction,
//! rendering) so each file stays legible on its own.
//!
//! Design references: `POLO_TREE_SEARCH_DESIGN.md` (the recursive-walk
//! design, §4 in particular) and `POLO_TREE_SEARCH_HIGHLIGHTING.md` (the
//! "where you are" blue marker and search-match gold marker, §2/§3 —
//! revised since that document was written: the blue marker and the
//! search scope root are now the same thing, driven by whichever folder
//! was last clicked, see [`effective_active_folder`]).
//!
//! This module owns no widgets and builds no UI itself — [`SearchState`]
//! is plain shared state, and every function here is either pure
//! (`search_walk`, `build_search_visible`, `scope_label`, …) or a small
//! side effect on state `file_tree_panel.rs` already owns (`reveal_matches`,
//! `update_scope_ui`). `file_tree_panel.rs::create_file_tree_panel` is what
//! actually wires these into the `SearchEntry`/`CustomFilter`/`ListView`.

use super::file_tree_panel::{
    ancestor_chain, expand_to, home_dir, list_tree_entries, refresh_visible_rows, TreeRoot,
};
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

/// The directory eager/recursive search walks from: Home for the default
/// root, or the active custom root (POLO_TREE_SEARCH_DESIGN.md §4.2).
fn search_scope_root(root: &TreeRoot) -> PathBuf {
    match root {
        TreeRoot::Default => home_dir(),
        TreeRoot::Custom(dir) => dir.clone(),
    }
}

/// The folder the blue "where you are" marker (POLO_TREE_SEARCH_HIGHLIGHTING.md
/// §2, revised) currently sits on — and, since the two were unified after
/// real usage showed a marker that lied about search scope was worse than
/// no marker at all, also the folder recursive search actually walks from.
/// `active_folder` if the user has clicked a folder since the panel was
/// built, otherwise a fallback: Home for the default root, or the active
/// custom root.
pub(crate) fn effective_active_folder(root: &TreeRoot, active_folder: &Option<PathBuf>) -> PathBuf {
    active_folder
        .clone()
        .unwrap_or_else(|| search_scope_root(root))
}

/// Whether `path` is the current "where you are"/search-scope folder — same
/// shape of check as `dir_icon_kind`'s `path == home` in `file_tree_panel.rs`.
pub(crate) fn is_active_folder(
    root: &TreeRoot,
    active_folder: &Option<PathBuf>,
    path: &Path,
) -> bool {
    path == effective_active_folder(root, active_folder)
}

/// Human-readable name for the current search-scope folder (§4.2,
/// highlighting §2 revised), used in the search entry's placeholder text
/// and tooltip: "Home" for the fallback default-root case, otherwise the
/// clicked folder's (or custom root's) own name.
fn scope_label(root: &TreeRoot, active_folder: &Option<PathBuf>) -> String {
    let path = effective_active_folder(root, active_folder);
    if path == home_dir() {
        return "Home".to_string();
    }
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Sync the search entry's placeholder text and tooltip with the current
/// "where you are"/search-scope folder — called whenever `root` or
/// `active_folder` changes.
pub(crate) fn update_scope_ui(
    search: &gtk4::SearchEntry,
    root: &TreeRoot,
    active_folder: &Option<PathBuf>,
) {
    let label = scope_label(root, active_folder);
    search.set_placeholder_text(Some(&format!("Search {}…", label)));
    let path = effective_active_folder(root, active_folder);
    search.set_tooltip_text(Some(&path.display().to_string()));
}

/// Recursively search `scope_root` for markdown files whose name contains
/// `query` (case-insensitive), reusing `list_tree_entries`'s inclusion rules
/// (hidden entries skipped, markdown-only, dir.md's own filesystem rules —
/// §3 Approach D). Runs off the main thread (§4.1 Tier 2); checks
/// `current_gen` against `my_gen` before reading each directory so a
/// superseded walk (a newer keystroke already in flight) stops early
/// instead of finishing a search whose result would just be discarded
/// (§4.1's generation-counter cancellation, mirroring `h_scroll_gen`).
fn search_walk(
    scope_root: &Path,
    query: &str,
    my_gen: u64,
    current_gen: &Arc<AtomicU64>,
) -> HashSet<PathBuf> {
    let mut matches = HashSet::new();
    let mut stack = vec![scope_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if current_gen.load(Ordering::Relaxed) != my_gen {
            return HashSet::new();
        }
        let Ok(entries) = list_tree_entries(&dir) else {
            continue;
        };
        for (path, name, is_dir) in entries {
            if is_dir {
                stack.push(path);
            } else if name.to_lowercase().contains(query) {
                matches.insert(path);
            }
        }
    }
    matches
}

/// `matches` plus every ancestor directory between them and `scope_root`
/// (inclusive of `scope_root` itself, and — via `ancestor_chain` — the path
/// down to it from the tree's literal top-level row, so Home/the custom
/// root never disappears from view once this set is applied). The filter's
/// directory-visibility set once Tier 2 results exist (§4.4); computed once
/// per completed walk, not per row.
pub(crate) fn build_search_visible(
    root: &TreeRoot,
    scope_root: &Path,
    matches: &HashSet<PathBuf>,
) -> HashSet<PathBuf> {
    let mut visible: HashSet<PathBuf> = matches.clone();
    if let Some(chain) = ancestor_chain(root, scope_root) {
        visible.extend(chain);
    }
    visible.insert(scope_root.to_path_buf());
    for m in matches {
        let mut cur = m.parent();
        while let Some(dir) = cur {
            if dir == scope_root || !visible.insert(dir.to_path_buf()) {
                break;
            }
            cur = dir.parent();
        }
    }
    visible
}

/// Expand every ancestor of each match so it becomes a materialized,
/// visible row (§4.3) — reuses `expand_to` exactly as
/// `FileTreePanelHandle::reveal_open_path` does, just without the
/// selection/centering that's specific to "the currently-open file."
pub(crate) fn reveal_matches(
    tree_model: &gtk4::TreeListModel,
    root: &TreeRoot,
    matches: &HashSet<PathBuf>,
) {
    for m in matches {
        expand_to(tree_model, root, m);
    }
}

/// Bundled, cheaply-`Clone`able search state (just a handful of `Rc`s/an
/// `Arc` behind one clone) shared across the filter predicate, the
/// search-changed handler, the folder-click handler, and the Tier 2
/// results poll loop in `file_tree_panel.rs::create_file_tree_panel` — the
/// same "small `Clone` struct of shared state, methods instead of free
/// functions with a dozen parameters" shape `menu.rs`'s `PoloMenuState`
/// already uses for its own debounce.
#[derive(Clone)]
pub(crate) struct SearchState {
    /// Bumped on every keystroke *and* every folder click (mirrors
    /// `h_scroll_gen`'s cancellation idiom, §4.1) — `Arc` rather than `Rc`
    /// because it has to cross the background-thread boundary, doubling as
    /// both "which walk is current" on the main thread and "should I stop
    /// walking" inside the thread.
    pub(crate) gen: Arc<AtomicU64>,
    /// Raw file matches from the most recent completed (non-superseded)
    /// Tier 2 walk — drives the `.search-match` row marker (highlighting §3).
    pub(crate) matches: Rc<RefCell<HashSet<PathBuf>>>,
    /// `matches` plus ancestor directories (§4.4) — drives the filter's
    /// directory-visibility rule once `ready` is set.
    pub(crate) visible: Rc<RefCell<HashSet<PathBuf>>>,
    /// Whether Tier 2 has produced (and applied) results for the *current*
    /// query/scope yet. While false, the filter falls back to "directories
    /// always pass" so nothing flashes hidden during the debounce window.
    pub(crate) ready: Rc<Cell<bool>>,
    /// The pending debounce timer, if any.
    debounce: Rc<RefCell<Option<glib::SourceId>>>,
    /// Background walks report results back over this channel rather than
    /// touching GTK state directly — a walk runs on a plain `std::thread`,
    /// and none of the `Rc`-based fields above are `Send`, so a `Sender`
    /// (`Send`) paired with a receiver polled from a main-thread-only timer
    /// is the only thread-safe way to hand results back (§4.1: "no async
    /// runtime… stay in the plain-callback idiom").
    tx: mpsc::Sender<(u64, HashSet<PathBuf>)>,
}

impl SearchState {
    /// Builds the state and its channel together, returning the receiver
    /// half separately — the caller (the Tier 2 results poll loop) owns and
    /// polls it directly, so `file_tree_panel.rs` never needs to name
    /// `mpsc`/`HashSet` itself just to hold onto it.
    pub(crate) fn new() -> (Self, mpsc::Receiver<(u64, HashSet<PathBuf>)>) {
        let (tx, rx) = mpsc::channel();
        (
            Self {
                gen: Arc::new(AtomicU64::new(0)),
                matches: Rc::new(RefCell::new(HashSet::new())),
                visible: Rc::new(RefCell::new(HashSet::new())),
                ready: Rc::new(Cell::new(false)),
                debounce: Rc::new(RefCell::new(None)),
                tx,
            },
            rx,
        )
    }

    /// Cancel any pending/in-flight Tier 2 walk, reset to "no results yet"
    /// (so the filter falls back to "directories always pass" until a new
    /// walk lands) and refresh visible rows so stale `.search-match`/
    /// `.scope-root` markers clear immediately — then, if `query` is
    /// non-empty, schedule a fresh debounced walk scoped to `scope_root`
    /// (§4.1). Called on every keystroke *and* every folder click, since
    /// both can change what "the current search" means: a new query, or
    /// the same query against a newly-clicked scope.
    pub(crate) fn restart(&self, query: String, scope_root: PathBuf, list_view: &gtk4::ListView) {
        if let Some(id) = self.debounce.borrow_mut().take() {
            id.remove();
        }
        let my_gen = self.gen.fetch_add(1, Ordering::Relaxed) + 1;
        self.ready.set(false);
        self.matches.borrow_mut().clear();
        self.visible.borrow_mut().clear();
        refresh_visible_rows(list_view);

        if query.is_empty() {
            return;
        }

        let thread_gen = self.gen.clone();
        let tx = self.tx.clone();
        let debounce_fired = self.debounce.clone();
        let id = glib::timeout_add_local_once(Duration::from_millis(200), move || {
            // A `_once` source removes itself from the main context the
            // moment it fires — clear our own record of it *first*, so a
            // later restart's cancel-and-reschedule never tries to
            // `SourceId::remove()` an ID GLib has already torn down (that
            // call panics — "Source ID N was not found" — from inside a
            // signal trampoline that can't unwind). Same idiom `menu.rs`'s
            // `schedule_hover_switch` uses.
            debounce_fired.borrow_mut().take();
            let thread_gen = thread_gen.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                let matches = search_walk(&scope_root, &query, my_gen, &thread_gen);
                let _ = tx.send((my_gen, matches));
            });
        });
        *self.debounce.borrow_mut() = Some(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Manually-managed temp directory (no `tempfile` dependency needed for
    /// a couple of tests) under `std::env::temp_dir()`, unique per test run
    /// via the process ID, removed on drop. Duplicated from
    /// `file_tree_panel.rs`'s own test module rather than shared — small
    /// enough that a `pub(crate)` test-utility export isn't worth it for
    /// two files.
    struct TempDir(PathBuf);
    impl TempDir {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "polo_file_tree_search_test_{}_{}",
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
    fn is_active_folder_falls_back_to_home_or_custom_root_when_nothing_clicked() {
        let home = home_dir();
        assert!(is_active_folder(&TreeRoot::Default, &None, &home));
        assert!(!is_active_folder(
            &TreeRoot::Default,
            &None,
            &home.join("docs")
        ));

        let tmp = TempDir::new("is_active_folder_custom");
        let custom = TreeRoot::Custom(tmp.path().to_path_buf());
        assert!(is_active_folder(&custom, &None, tmp.path()));
        assert!(!is_active_folder(&custom, &None, &home));
    }

    #[test]
    fn is_active_folder_prefers_the_clicked_folder_over_the_fallback() {
        let tmp = TempDir::new("is_active_folder_clicked");
        let clicked = tmp.path().join("some_subdir");
        let active = Some(clicked.clone());

        // Even under the default root (whose fallback is Home), a clicked
        // folder elsewhere takes over as "where you are".
        assert!(is_active_folder(&TreeRoot::Default, &active, &clicked));
        assert!(!is_active_folder(&TreeRoot::Default, &active, &home_dir()));
    }

    #[test]
    fn scope_label_is_home_for_default_and_dir_name_for_custom() {
        assert_eq!(scope_label(&TreeRoot::Default, &None), "Home");
        let tmp = TempDir::new("scope_label_custom");
        let sub = tmp.path().join("my_notes");
        std::fs::create_dir_all(&sub).unwrap();
        assert_eq!(scope_label(&TreeRoot::Custom(sub), &None), "my_notes");
    }

    #[test]
    fn scope_label_follows_the_clicked_folder() {
        let tmp = TempDir::new("scope_label_active");
        let clicked = tmp.path().join("project");
        std::fs::create_dir_all(&clicked).unwrap();
        assert_eq!(scope_label(&TreeRoot::Default, &Some(clicked)), "project");
    }

    #[test]
    fn search_walk_finds_matches_several_levels_deep_and_skips_non_matches() {
        let tmp = TempDir::new("search_walk_deep");
        let deep = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("target_note.md"), "# hit").unwrap();
        std::fs::write(deep.join("other.md"), "# miss").unwrap();
        std::fs::write(tmp.path().join("target_root.md"), "# hit too").unwrap();

        let gen = Arc::new(AtomicU64::new(1));
        let matches = search_walk(tmp.path(), "target", 1, &gen);

        assert_eq!(
            matches,
            HashSet::from([
                deep.join("target_note.md"),
                tmp.path().join("target_root.md")
            ])
        );
    }

    #[test]
    fn search_walk_returns_empty_once_superseded() {
        let tmp = TempDir::new("search_walk_superseded");
        std::fs::write(tmp.path().join("target.md"), "# hit").unwrap();

        // Generation already moved on to 2 by the time the walk checks —
        // simulates a newer keystroke arriving mid-walk (§4.1).
        let gen = Arc::new(AtomicU64::new(2));
        let matches = search_walk(tmp.path(), "target", 1, &gen);
        assert!(matches.is_empty());
    }

    #[test]
    fn build_search_visible_includes_match_ancestors_and_scope_root() {
        let tmp = TempDir::new("build_search_visible");
        let deep = tmp.path().join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        let match_path = deep.join("hit.md");

        let root = TreeRoot::Custom(tmp.path().to_path_buf());
        let matches = HashSet::from([match_path.clone()]);
        let visible = build_search_visible(&root, tmp.path(), &matches);

        assert!(visible.contains(&match_path));
        assert!(visible.contains(&deep)); // immediate parent
        assert!(visible.contains(&tmp.path().join("a"))); // grandparent
        assert!(visible.contains(tmp.path())); // scope root itself
    }

    #[test]
    fn build_search_visible_excludes_unrelated_siblings() {
        let tmp = TempDir::new("build_search_visible_siblings");
        let matching_dir = tmp.path().join("matches_here");
        let sibling_dir = tmp.path().join("no_matches_here");
        std::fs::create_dir_all(&matching_dir).unwrap();
        std::fs::create_dir_all(&sibling_dir).unwrap();
        let match_path = matching_dir.join("hit.md");

        let root = TreeRoot::Custom(tmp.path().to_path_buf());
        let matches = HashSet::from([match_path]);
        let visible = build_search_visible(&root, tmp.path(), &matches);

        assert!(!visible.contains(&sibling_dir));
    }
}
