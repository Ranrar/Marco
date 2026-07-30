// Polo icon toolbar
//
//! # Toolbar Module
//!
//! Creates the icon-based toolbar displayed below the titlebar.
//!
//! ## Buttons (left to right)
//!
//! - **Open in Editor** – opens the current file in Marco editor (disabled when
//!   no file is loaded)
//! - **TOC** – toggles the Table-of-Contents side panel
//! - **Files** – toggles the file-tree browser side panel
//! - *separator*
//! - **Light / Dark mode** – toggles between light and dark colour mode
//!
//! TOC and Files are mutually exclusive — see `sidebar_coordinator`, which
//! both buttons go through instead of toggling their panels directly.
//!
//! There is deliberately no "Open file" toolbar button: the File menu's
//! "Open..." entry (`menu.rs`) is the fallback once a document is loaded,
//! and the empty state (`viewer::empty_state`) has its own primary Open File
//! button plus drag-and-drop for the "no file loaded yet" case.
//!
//! ## Icons
//!
//! All icons are inline SVG strings rendered via `rsvg` + `cairo`.  Hover and
//! active states are handled through `EventControllerMotion` / `GestureClick`.

use crate::components::menu::{render_svg_texture, toggle_color_mode};
use crate::components::sidebar_coordinator::SidebarCoordinator;
use crate::components::viewer::find_engine::{self, FindOptions};
use crate::components::viewer::platform_webview::PlatformWebView;
use gtk4::{
    prelude::*, Align, Box as GtkBox, Button, CheckButton, Label, Orientation, Picture, Revealer,
    RevealerTransitionType, SearchEntry, Separator,
};
use marco_shared::logic::swanson::SettingsManager;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

// ── Inline SVG icons ──────────────────────────────────────────────────────

/// Open-file (folder-open) icon - Tabler Icons `icon-tabler-folder-open`.
/// `pub(crate)` (not just used here) — reused verbatim as literal HTML/SVG
/// markup by `viewer::empty_state`'s "Open File" button, same cross-module
/// reuse pattern as `SVG_SUN`/`SVG_MOON` below.
pub(crate) const SVG_OPEN_FILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 19l2.757 -7.351a1 1 0 0 1 .936 -.649h12.307a1 1 0 0 1 .986 1.164l-.996 5.211a2 2 0 0 1 -1.964 1.625h-14.026a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2h4l3 3h7a2 2 0 0 1 2 2v2"/></svg>"#;

/// Print icon - Tabler Icons `icon-tabler-printer`
const SVG_PRINT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M17 17h2a2 2 0 0 0 2 -2v-4a2 2 0 0 0 -2 -2h-14a2 2 0 0 0 -2 2v4a2 2 0 0 0 2 2h2"/><path d="M17 9v-4a2 2 0 0 0 -2 -2h-6a2 2 0 0 0 -2 2v4"/><path d="M7 15a2 2 0 0 1 2 -2h6a2 2 0 0 1 2 2v4a2 2 0 0 1 -2 2h-6a2 2 0 0 1 -2 -2l0 -4"/></svg>"#;

/// Open-in-editor (pencil) icon - Tabler Icons `icon-tabler-pencil`
const SVG_OPEN_EDITOR: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M4 20h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4"/><path d="M13.5 6.5l4 4"/></svg>"#;

/// Table-of-contents (stack-2) icon - Tabler Icons `icon-tabler-stack-2`
const SVG_TOC: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 4l-8 4l8 4l8 -4l-8 -4"/><path d="M4 12l8 4l8 -4"/><path d="M4 16l8 4l8 -4"/></svg>"#;

/// File-tree icon - Tabler Icons `icon-tabler-folder-tree`.
/// Deliberately distinct from `SVG_OPEN_FILE`'s folder-*open* glyph above —
/// that one triggers the native file-chooser dialog; this one toggles the
/// in-window file-tree sidebar panel.
const SVG_FILE_TREE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M2.2 2.76 h1.76 l1.32 1.32 h3.08 a0.88 0.88 0 0 1 0.88 0.88 v3.52 a0.88 0.88 0 0 1 -0.88 0.88 h-6.16 a0.88 0.88 0 0 1 -0.88 -0.88 v-4.84 a0.88 0.88 0 0 1 0.88 -0.88"/><path d="M5.28 9.36 V11.44 A0.56 0.56 0 0 0 5.84 12 H11.34"/><path d="M5.28 9.36 V17.44 A0.56 0.56 0 0 0 5.84 18 H11.34"/><path d="M11.9 10.12 h1.12 l0.84 0.84 h1.96 a0.56 0.56 0 0 1 0.56 0.56 v2.24 a0.56 0.56 0 0 1 -0.56 0.56 h-3.92 a0.56 0.56 0 0 1 -0.56 -0.56 v-3.08 a0.56 0.56 0 0 1 0.56 -0.56"/><path d="M11.9 16.12 h1.12 l0.84 0.84 h1.96 a0.56 0.56 0 0 1 0.56 0.56 v2.24 a0.56 0.56 0 0 1 -0.56 0.56 h-3.92 a0.56 0.56 0 0 1 -0.56 -0.56 v-3.08 a0.56 0.56 0 0 1 0.56 -0.56"/></svg>"#;

/// Sun icon (light mode) - Tabler Icons `icon-tabler-sun`
pub(crate) const SVG_SUN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M8 12a4 4 0 1 0 8 0a4 4 0 1 0 -8 0"/><path d="M3 12h1m8 -9v1m8 8h1m-9 8v1m-6.4 -15.4l.7 .7m12.1 -.7l-.7 .7m0 11.4l.7 .7m-12.1 -.7l-.7 .7"/></svg>"#;

/// Moon icon (dark mode) - Tabler Icons `icon-tabler-moon`
pub(crate) const SVG_MOON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454l0 .008"/></svg>"#;

/// Search (magnifying glass) icon - Tabler Icons `icon-tabler-search`
const SVG_SEARCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M10 10m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0"/><path d="M21 21l-6 -6"/></svg>"#;

/// Chevron-up icon (previous match) - Tabler Icons `icon-tabler-chevron-up`
const SVG_CHEVRON_UP: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M6 15l6 -6l6 6"/></svg>"#;

/// Chevron-down icon (next match) - Tabler Icons `icon-tabler-chevron-down`
const SVG_CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M6 9l6 6l6 -6"/></svg>"#;

/// Logical icon size in pixels (will be rendered at 2× for HiDPI)
const TOOLBAR_ICON_SIZE: f64 = 12.0;

// ── Public types ──────────────────────────────────────────────────────────

/// State returned from `create_polo_toolbar`.
pub struct PoloToolbarState {
    /// The toolbar widget to insert into the window layout.
    pub toolbar: GtkBox,
    /// The "Open in Editor" button; enable it when a file is loaded.
    pub open_editor_btn: Button,
    /// Reveals the find-in-page bar, focuses its entry, and re-runs the
    /// search if a query is already present — the same thing clicking the
    /// toolbar's search icon does when the bar is closed. Wired to `Ctrl+F`
    /// by `main.rs`/`menu.rs` (`win.polo-find`).
    pub open_find_bar: Rc<dyn Fn()>,
}

// ── Public constructor ────────────────────────────────────────────────────

/// Build the Polo icon toolbar.
///
/// The returned `PoloToolbarState.open_editor_btn` should be passed to
/// `create_custom_titlebar` so that the File → Open action can enable it
/// when a file is successfully opened.
#[allow(clippy::type_complexity)]
pub fn create_polo_toolbar(
    window: &gtk4::ApplicationWindow,
    webview: PlatformWebView,
    settings_manager: Arc<SettingsManager>,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
    sidebar: SidebarCoordinator,
) -> PoloToolbarState {
    let toolbar = GtkBox::new(Orientation::Horizontal, 0);
    toolbar.add_css_class("polo-toolbar");
    toolbar.set_valign(Align::Center);

    // ── Open in Editor ────────────────────────────────────────────────
    let open_editor_btn = make_icon_btn(
        window,
        SVG_OPEN_EDITOR,
        "Open in Marco editor",
        TOOLBAR_ICON_SIZE,
    );
    let has_file = current_file_path
        .read()
        .ok()
        .and_then(|g| g.as_ref().cloned())
        .is_some();
    open_editor_btn.set_sensitive(has_file);

    {
        let win_weak = window.downgrade();
        let cfp = current_file_path.clone();
        open_editor_btn.connect_clicked(move |_| {
            if let Some(w) = win_weak.upgrade() {
                if let Ok(guard) = cfp.read() {
                    if let Some(ref path) = *guard {
                        crate::components::dialog::show_open_in_editor_dialog(&w, path);
                    }
                }
            }
        });
    }

    // ── TOC ────────────────────────────────────────────────────────────
    let toc_btn = make_icon_btn(
        window,
        SVG_TOC,
        "Toggle Table of Contents",
        TOOLBAR_ICON_SIZE,
    );
    {
        let sidebar = sidebar.clone();
        toc_btn.connect_clicked(move |_| sidebar.toggle_toc());
    }

    // ── Files (file-tree browser) ───────────────────────────────────────
    // Mutually exclusive with TOC — see `sidebar_coordinator`.
    let files_btn = make_icon_btn(window, SVG_FILE_TREE, "Toggle File Tree", TOOLBAR_ICON_SIZE);
    {
        let sidebar = sidebar.clone();
        files_btn.connect_clicked(move |_| sidebar.toggle_dir());
    }

    // ── Separator ──────────────────────────────────────────────────────
    let sep2 = make_separator();

    // ── Print ─────────────────────────────────────────────────────────
    let print_btn = make_icon_btn(window, SVG_PRINT, "Print (Ctrl+P)", TOOLBAR_ICON_SIZE);
    {
        let wv_print = webview.clone();
        let win_weak = window.downgrade();
        print_btn.connect_clicked(move |_| {
            let parent = win_weak.upgrade();
            wv_print.print(parent.as_ref().map(|w| w.upcast_ref()));
        });
    }

    // ── Separator ──────────────────────────────────────────────────────
    let sep3 = make_separator();

    // ── Light / Dark mode ─────────────────────────────────────────────
    let mode_btn = make_mode_btn(
        window,
        settings_manager.clone(),
        webview.clone(),
        current_file_path.clone(),
        asset_root,
    );

    // ── Separator ──────────────────────────────────────────────────────
    let sep4 = make_separator();

    // ── Find in page ─────────────────────────────────────────────────
    let (search_btn, search_revealer, open_find_bar) = make_find_bar(window, webview.clone());

    // ── Assemble toolbar ───────────────────────────────────────────────
    // "Open in Marco editor" sits directly left of Print, not at the very
    // start of the toolbar — grouped with the action it's most related to
    // (both act on the currently-loaded file) rather than with the TOC/Files
    // sidebar toggles.
    toolbar.append(&toc_btn);
    toolbar.append(&files_btn);
    toolbar.append(&sep2);
    toolbar.append(&open_editor_btn);
    toolbar.append(&print_btn);
    toolbar.append(&sep3);
    toolbar.append(&mode_btn);
    toolbar.append(&sep4);
    toolbar.append(&search_btn);
    toolbar.append(&search_revealer);

    PoloToolbarState {
        toolbar,
        open_editor_btn,
        open_find_bar,
    }
}

// ── Mode toggle button ────────────────────────────────────────────────────

fn make_mode_btn(
    window: &gtk4::ApplicationWindow,
    settings_manager: Arc<SettingsManager>,
    webview: PlatformWebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
) -> Button {
    use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};

    // Determine initial mode
    let is_dark = {
        let s = settings_manager.get_settings();
        s.appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.contains("dark"))
            .unwrap_or(false)
    };

    let icon_color = if is_dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    };
    let icon_svg = if is_dark { SVG_SUN } else { SVG_MOON };
    let tooltip = if is_dark {
        "Switch to Light Mode"
    } else {
        "Switch to Dark Mode"
    };

    // Build the button through the shared helper so it gets the same
    // hover/active mouseover styling as every other toolbar icon.
    // `current_svg` lets the click handler below repoint those re-renders at
    // the new icon shape once the mode has toggled.
    let (mode_btn, current_svg) =
        make_icon_btn_from_svg(window, icon_svg, tooltip, TOOLBAR_ICON_SIZE);

    // Override the initial color using the foreground palette color.
    if let Some(pic) = mode_btn.child().and_then(|c| c.downcast::<Picture>().ok()) {
        let t = render_svg_texture(icon_svg, icon_color, TOOLBAR_ICON_SIZE);
        pic.set_paintable(Some(&t));
    }

    // Retrieve the Picture child so we can update it on click
    let pic = mode_btn
        .child()
        .and_then(|c| c.downcast::<Picture>().ok())
        .expect("mode button has Picture child");

    let sm = settings_manager.clone();
    let wv = webview.clone();
    let cfp = current_file_path.clone();
    let asset = asset_root.to_path_buf();
    let window_clone = window.clone();
    let pic_clone = pic.clone();

    mode_btn.connect_clicked(move |_| {
        toggle_color_mode(
            &window_clone,
            sm.clone(),
            wv.clone(),
            cfp.clone(),
            &asset,
            Some((&pic_clone, TOOLBAR_ICON_SIZE)),
        );
        // Update tooltip to reflect new state
        // (toggle_color_mode already updated the picture)

        // Keep the shared hover/active re-render path (see
        // make_icon_btn_from_svg) in sync with the new icon shape.
        let now_dark = sm
            .get_settings()
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.contains("dark"))
            .unwrap_or(false);
        current_svg.set(if now_dark { SVG_SUN } else { SVG_MOON });
    });

    mode_btn
}

// ── Find bar ──────────────────────────────────────────────────────────────

/// Widgets driving the "Highlight All" / "Match Case" / "Match Diacritics" /
/// "Whole Word" checkboxes — bundled so the search/re-search closures below
/// don't need four separate clones each.
#[derive(Clone)]
struct FindOptionWidgets {
    highlight_all: CheckButton,
    match_case: CheckButton,
    match_diacritics: CheckButton,
    whole_word: CheckButton,
}

impl FindOptionWidgets {
    fn current(&self) -> FindOptions {
        FindOptions {
            case_sensitive: self.match_case.is_active(),
            whole_word: self.whole_word.is_active(),
            highlight_all: self.highlight_all.is_active(),
            match_diacritics: self.match_diacritics.is_active(),
        }
    }
}

/// Re-run (or clear) the in-page search from the entry's current text and
/// the checkboxes' current state. Shared by the entry's `search-changed` and
/// every checkbox's `toggled` handler so toggling an option re-searches
/// in-place instead of waiting for the next keystroke.
fn run_find(webview: &PlatformWebView, entry: &SearchEntry, opts: &FindOptionWidgets) {
    let query = entry.text();
    if query.is_empty() {
        find_engine::clear(webview);
    } else {
        find_engine::install(webview);
        find_engine::search(webview, &query, opts.current());
    }
}

/// Build the toolbar's search toggle button and the `Revealer` housing the
/// inline find-in-page bar (entry, prev/next, option checkboxes, match-count
/// label). Returns `(toggle_button, revealer)` — both get appended to the
/// toolbar by the caller.
///
/// The bar is a permanent widget that's shown/hidden (`Revealer`), not a
/// popover or a separate window: closing it (Esc, via `SearchEntry`'s
/// built-in `stop-search` signal, or clicking the toggle button again) only
/// hides it and clears the preview's highlights — it does not clear the
/// query text, matching Firefox's find bar (reopening shows the last query,
/// re-highlighted immediately since the text is still there).
#[allow(clippy::type_complexity)]
fn make_find_bar(
    window: &gtk4::ApplicationWindow,
    webview: PlatformWebView,
) -> (Button, Revealer, Rc<dyn Fn()>) {
    let search_btn = make_icon_btn(
        window,
        SVG_SEARCH,
        "Find in Page (Esc to close)",
        TOOLBAR_ICON_SIZE,
    );

    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Find in page"));
    search_entry.add_css_class("polo-search-entry");
    search_entry.set_valign(Align::Center);

    let match_count_label = Label::new(None);
    match_count_label.add_css_class("polo-search-match-label");
    match_count_label.set_valign(Align::Center);

    let prev_btn = make_icon_btn(window, SVG_CHEVRON_UP, "Previous Match", TOOLBAR_ICON_SIZE);
    let next_btn = make_icon_btn(window, SVG_CHEVRON_DOWN, "Next Match", TOOLBAR_ICON_SIZE);

    let highlight_all_cb = CheckButton::with_label("Highlight All");
    highlight_all_cb.set_active(true);
    highlight_all_cb.add_css_class("polo-search-checkbox");
    let match_case_cb = CheckButton::with_label("Match Case");
    match_case_cb.add_css_class("polo-search-checkbox");
    let match_diacritics_cb = CheckButton::with_label("Match Diacritics");
    match_diacritics_cb.add_css_class("polo-search-checkbox");
    let whole_word_cb = CheckButton::with_label("Whole Word");
    whole_word_cb.add_css_class("polo-search-checkbox");

    let opt_widgets = FindOptionWidgets {
        highlight_all: highlight_all_cb.clone(),
        match_case: match_case_cb.clone(),
        match_diacritics: match_diacritics_cb.clone(),
        whole_word: whole_word_cb.clone(),
    };

    let search_bar = GtkBox::new(Orientation::Horizontal, 6);
    search_bar.add_css_class("polo-search-bar");
    search_bar.set_valign(Align::Center);
    search_bar.append(&search_entry);
    search_bar.append(&match_count_label);
    search_bar.append(&prev_btn);
    search_bar.append(&next_btn);
    search_bar.append(&make_separator());
    search_bar.append(&highlight_all_cb);
    search_bar.append(&match_case_cb);
    search_bar.append(&match_diacritics_cb);
    search_bar.append(&whole_word_cb);

    let revealer = Revealer::new();
    revealer.set_transition_type(RevealerTransitionType::SlideRight);
    revealer.set_transition_duration(180);
    revealer.set_child(Some(&search_bar));
    revealer.set_reveal_child(false);

    // Live match-count indicator ("K / N" / "No results") — set once; this
    // is the only feature driving PlatformWebView's find-report callback.
    {
        let label = match_count_label.clone();
        webview.set_find_report_callback(move |report| {
            if report.total == 0 {
                label.set_text("");
            } else {
                label.set_text(&format!("{}/{}", report.active, report.total));
            }
        });
    }

    // Typing — `SearchEntry::search-changed` is already internally debounced
    // by GTK, so no extra Rust-side timer (and none of the stale-timer
    // re-highlight bugs that come with hand-rolling one) is needed.
    {
        let webview = webview.clone();
        let opts = opt_widgets.clone();
        search_entry.connect_search_changed(move |entry| {
            run_find(&webview, entry, &opts);
        });
    }

    // Enter → next match (mirrors the toolbar's prev/next buttons; Shift+Enter
    // for "previous" is not wired — the chevron buttons cover that case).
    {
        let webview = webview.clone();
        search_entry.connect_activate(move |_| {
            find_engine::next(&webview);
        });
    }

    prev_btn.connect_clicked({
        let webview = webview.clone();
        move |_| find_engine::prev(&webview)
    });
    next_btn.connect_clicked({
        let webview = webview.clone();
        move |_| find_engine::next(&webview)
    });

    // Toggling an option re-searches immediately with the current query
    // rather than waiting for the next keystroke.
    for cb in [
        &highlight_all_cb,
        &match_case_cb,
        &match_diacritics_cb,
        &whole_word_cb,
    ] {
        let webview = webview.clone();
        let entry = search_entry.clone();
        let opts = opt_widgets.clone();
        cb.connect_toggled(move |_| run_find(&webview, &entry, &opts));
    }

    // Closing (Esc via SearchEntry's built-in `stop-search`, or the toggle
    // button again) only hides the bar and clears highlights — the query
    // text is kept, matching Firefox's find bar.
    let close_find_bar = {
        let webview = webview.clone();
        let revealer = revealer.clone();
        move || {
            find_engine::clear(&webview);
            revealer.set_reveal_child(false);
        }
    };
    {
        let close_find_bar = close_find_bar.clone();
        search_entry.connect_stop_search(move |_| close_find_bar());
    }

    // Shared "open" logic — clicking the toolbar icon while closed, and the
    // `Ctrl+F` accelerator (`win.polo-find`, wired in `menu.rs`), both just
    // call this rather than toggling: Ctrl+F in a browser always focuses an
    // already-open find bar rather than closing it.
    let open_find_bar: Rc<dyn Fn()> = {
        let revealer = revealer.clone();
        let search_entry = search_entry.clone();
        let webview = webview.clone();
        let opts = opt_widgets.clone();
        Rc::new(move || {
            revealer.set_reveal_child(true);
            search_entry.grab_focus();
            // Select any leftover query so retyping immediately replaces it,
            // matching Firefox/Chrome's Ctrl+F behavior.
            search_entry.select_region(0, -1);
            // Reopening with a leftover query re-highlights immediately
            // instead of showing an empty preview until the next keystroke.
            run_find(&webview, &search_entry, &opts);
        })
    };

    search_btn.connect_clicked({
        let revealer = revealer.clone();
        let open_find_bar = open_find_bar.clone();
        move |_| {
            if revealer.reveals_child() {
                close_find_bar();
            } else {
                open_find_bar();
            }
        }
    });

    (search_btn, revealer, open_find_bar)
}

// ── Private helpers ────────────────────────────────────────────────────────

/// Determine toolbar button color based on GTK state flags and current theme.
fn toolbar_color_for_flags(btn: &gtk4::Button, flags: gtk4::StateFlags) -> &'static str {
    use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
    let is_dark = btn
        .root()
        .and_then(|r| r.downcast::<gtk4::Window>().ok())
        .map(|w| w.has_css_class("marco-theme-dark"))
        .unwrap_or(false);
    if flags.contains(gtk4::StateFlags::ACTIVE) {
        if is_dark {
            DARK_PALETTE.control_icon_active
        } else {
            LIGHT_PALETTE.control_icon_active
        }
    } else if flags.contains(gtk4::StateFlags::PRELIGHT) {
        if is_dark {
            DARK_PALETTE.control_icon_hover
        } else {
            LIGHT_PALETTE.control_icon_hover
        }
    } else if is_dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    }
}

/// Create an icon button from an SVG string using Marco's state-flags approach.
///
/// Icon color updates automatically on hover/active/normal via
/// `connect_state_flags_changed` and on theme changes via `connect_map`.
fn make_icon_btn(
    window: &gtk4::ApplicationWindow,
    svg: &'static str,
    tooltip: &str,
    size: f64,
) -> Button {
    make_icon_btn_from_svg(window, svg, tooltip, size).0
}

/// Like [`make_icon_btn`], but also returns a `Cell` holding the icon's
/// current SVG source. Buttons whose icon shape can change after creation
/// (the mode toggle) can update this cell so the hover/active/map re-renders
/// below keep drawing the right shape instead of the one it was built with.
fn make_icon_btn_from_svg(
    window: &gtk4::ApplicationWindow,
    svg: &'static str,
    tooltip: &str,
    size: f64,
) -> (Button, Rc<Cell<&'static str>>) {
    use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};

    let is_dark = window.has_css_class("marco-theme-dark");
    let initial_color = if is_dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    };

    let pic = Picture::new();
    let texture = render_svg_texture(svg, initial_color, size);
    pic.set_paintable(Some(&texture));
    pic.set_size_request(size as i32, size as i32);
    pic.set_can_shrink(false);
    pic.set_halign(Align::Center);
    pic.set_valign(Align::Center);

    let btn = Button::new();
    btn.set_child(Some(&pic));
    btn.set_tooltip_text(Some(tooltip));
    btn.set_valign(Align::Center);
    btn.set_focusable(false);
    btn.set_can_focus(false);
    btn.set_has_frame(false);
    btn.set_width_request((size + 2.0) as i32);
    btn.set_height_request((size + 2.0) as i32);
    btn.add_css_class("polo-toolbar-btn");

    let current_svg = Rc::new(Cell::new(svg));

    // Recompute icon color whenever button state changes (hover / active / normal).
    // Guard with is_mapped() to avoid snapshotting before first allocation.
    let pic_state = pic.clone();
    let svg_state = current_svg.clone();
    let btn_ref = btn.clone();
    btn.connect_state_flags_changed(move |btn, _| {
        if btn.is_mapped() {
            let flags = btn.state_flags();
            let color = toolbar_color_for_flags(&btn_ref, flags);
            let t = render_svg_texture(svg_state.get(), color, size);
            pic_state.set_paintable(Some(&t));
        }
    });

    // Re-render after map so the root window's theme class is available.
    let pic_map = pic.clone();
    let svg_map = current_svg.clone();
    let btn_ref2 = btn.clone();
    btn.connect_map(move |_| {
        let flags = btn_ref2.state_flags();
        let color = toolbar_color_for_flags(&btn_ref2, flags);
        let t = render_svg_texture(svg_map.get(), color, size);
        pic_map.set_paintable(Some(&t));
    });

    // Also sync after click activation.
    let pic_click = pic.clone();
    let svg_click = current_svg.clone();
    let btn_ref3 = btn.clone();
    btn.connect_clicked(move |_| {
        let flags = btn_ref3.state_flags();
        let color = toolbar_color_for_flags(&btn_ref3, flags);
        let t = render_svg_texture(svg_click.get(), color, size);
        pic_click.set_paintable(Some(&t));
    });

    (btn, current_svg)
}

/// Create a styled vertical separator for the toolbar.
fn make_separator() -> Separator {
    let sep = Separator::new(Orientation::Vertical);
    sep.add_css_class("polo-toolbar-separator");
    sep
}
