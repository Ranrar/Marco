//! TOC (Table of Contents) sidebar panel for Polo.
//!
//! Displays a resizable panel to the left of the preview WebView.
//! Each row is a clickable heading entry that scrolls the preview to the
//! corresponding heading anchor via `scrollIntoView`.
//!
//! # Layout
//!
//! ```text
//! toc_paned (gtk4::Paned)
//! ├── toc_panel (gtk4::Box)    ← start child (hidden by default)
//! │   ├── header label "Contents"
//! │   └── scrolled list of buttons
//! └── webview widget           ← end child (set by caller)
//! ```

use crate::components::sidebar_coordinator::{
    SharedPanelWidth, DEFAULT_SIDEBAR_WIDTH, MIN_SIDEBAR_WIDTH,
};
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use marco_core::intelligence::toc::TocEntry;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Handle that lets the rest of the app rebuild and toggle the TOC panel.
#[derive(Clone)]
pub struct TocPanelHandle {
    panel_box: gtk4::Box,
    list_box: gtk4::Box,
    paned: gtk4::Paned,
    visible: Rc<Cell<bool>>,
    /// Current maximum heading depth (1–6).
    pub depth: Rc<Cell<u8>>,
    /// Cached entries from the last file load.
    current_entries: Rc<RefCell<Vec<TocEntry>>>,
    /// WebView used to scroll the preview when a heading is clicked.
    webview: crate::components::viewer::platform_webview::PlatformWebView,
    /// Shared with the file-tree panel — see [`SharedPanelWidth`].
    manual_width: SharedPanelWidth,
    /// Guards `paned`'s `notify::position` handler against mistaking our
    /// *own* `set_position` calls (opening to the shared default/manual
    /// width, or collapsing to 0 on hide) for a user drag. Held for the
    /// *entire* show/hide transition — from before `panel_box`'s visibility
    /// even changes through to after the deferred `set_position` call
    /// completes — not just around the `set_position` call itself; see
    /// [`TocPanelHandle::show`]'s doc comment for why the wider window
    /// matters.
    programmatic_resize: Rc<Cell<bool>>,
}

impl TocPanelHandle {
    /// Whether the panel is currently visible.
    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    /// Parse `text` for TOC headings, cache them, and rebuild the list.
    /// If the panel is visible the list is updated immediately.
    ///
    /// Prefer [`update_from_text_async`] for large files to avoid blocking the
    /// GTK main loop.  This synchronous variant is kept for tests and for call
    /// sites that already hold the text string with no latency concern.
    #[allow(dead_code)]
    pub fn update_from_text(&self, text: &str) {
        let entries = marco_shared::cache::global_parser_cache().get_or_compute_toc(text);
        *self.current_entries.borrow_mut() = entries.as_ref().to_vec();
        if self.visible.get() {
            let borrowed = self.current_entries.borrow();
            self.rebuild(&borrowed, self.depth.get());
        }
    }

    /// Async variant of [`update_from_text`]: dispatches the parse + TOC
    /// extraction to a background thread so large files never stall the GTK
    /// main loop.  The panel is rebuilt on the main thread once the result is
    /// ready.
    pub fn update_from_text_async(&self, text: String) {
        let handle = self.clone();
        glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(move || {
                marco_shared::cache::global_parser_cache().get_or_compute_toc(&text)
            })
            .await;
            match result {
                Ok(arc_entries) => {
                    *handle.current_entries.borrow_mut() = arc_entries.as_ref().to_vec();
                    if handle.visible.get() {
                        let borrowed = handle.current_entries.borrow();
                        handle.rebuild(&borrowed, handle.depth.get());
                    }
                }
                Err(e) => log::error!("[polo] TOC compute task panicked: {:?}", e),
            }
        });
    }

    /// Show the panel (rebuilds from the last cached entries) at the shared
    /// default width, or the shared manual width if the user has set one —
    /// see `SharedPanelWidth`.
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
        let borrowed = self.current_entries.borrow();
        self.rebuild(&borrowed, self.depth.get());
        drop(borrowed);

        let width = self
            .manual_width
            .get()
            .unwrap_or(DEFAULT_SIDEBAR_WIDTH)
            .max(MIN_SIDEBAR_WIDTH);
        let paned = self.paned.clone();
        let programmatic = self.programmatic_resize.clone();
        glib::idle_add_local_once(move || {
            paned.set_position(width);
            programmatic.set(false);
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

    /// Rebuild the entry list from a slice of TOC entries.
    pub fn rebuild(&self, entries: &[TocEntry], max_depth: u8) {
        // Clear existing rows.
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let filtered: Vec<&TocEntry> = entries.iter().filter(|e| e.level <= max_depth).collect();

        if filtered.is_empty() {
            let empty_label = gtk4::Label::new(Some("No headings"));
            empty_label.set_halign(gtk4::Align::Start);
            empty_label.add_css_class("toc-panel-empty");
            self.list_box.append(&empty_label);
            return;
        }

        let min_level = filtered.iter().map(|e| e.level).min().unwrap_or(1);

        for entry in filtered {
            let indent_px = ((entry.level - min_level) as i32) * 12;

            let btn = gtk4::Button::new();
            btn.set_has_frame(false);
            btn.set_halign(gtk4::Align::Fill);
            btn.set_hexpand(true);
            btn.add_css_class("toc-panel-entry");
            btn.add_css_class(&format!("toc-depth-{}", entry.level));

            let label = gtk4::Label::new(Some(&entry.text));
            label.set_xalign(0.0);
            label.set_halign(gtk4::Align::Start);
            label.set_hexpand(true);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_margin_start(indent_px);
            btn.set_child(Some(&label));

            let slug = entry.slug.clone();
            let wv = self.webview.clone();
            btn.connect_clicked(move |_| {
                let js = format!(
                    r#"(function(){{
                        var el = document.getElementById({slug:?});
                        if (el) {{ el.scrollIntoView({{behavior:'smooth', block:'start'}}); }}
                    }})();"#,
                    slug = slug,
                );
                wv.evaluate_script(&js);
            });

            self.list_box.append(&btn);
        }
    }
}

/// Create the TOC sidebar paned and return the paned together with the handle.
///
/// `manual_width` is shared with the file-tree panel — see
/// [`SharedPanelWidth`] — so a user-driven resize of either panel overrides
/// both from then on.
///
/// The caller sets the end child:
/// ```ignore
/// let (paned, toc) = create_toc_panel(&webview, manual_width);
/// paned.set_end_child(Some(&webview.widget()));
/// window.set_child(Some(&paned));
/// ```
pub fn create_toc_panel(
    webview: &crate::components::viewer::platform_webview::PlatformWebView,
    manual_width: SharedPanelWidth,
) -> (gtk4::Paned, TocPanelHandle) {
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

    // ── TOC panel (start child) ───────────────────────────────────────────────
    let panel_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    panel_box.set_visible(false);
    panel_box.set_hexpand(false);
    panel_box.set_vexpand(true);
    panel_box.set_width_request(MIN_SIDEBAR_WIDTH);
    panel_box.add_css_class("toc-panel");

    let header = gtk4::Label::new(Some("Contents"));
    header.set_halign(gtk4::Align::Start);
    header.set_margin_start(8);
    header.set_margin_top(6);
    header.set_margin_bottom(4);
    header.add_css_class("toc-panel-header");
    panel_box.append(&header);

    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    panel_box.append(&sep);

    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.add_css_class("toc-panel-scroll");
    scrolled.set_direction(gtk4::TextDirection::Ltr);

    let list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    list_box.set_vexpand(true);
    list_box.set_margin_top(2);
    list_box.set_margin_bottom(2);
    list_box.set_margin_start(2);
    list_box.set_margin_end(2);
    scrolled.set_child(Some(&list_box));
    panel_box.append(&scrolled);

    paned.set_start_child(Some(&panel_box));

    inject_toc_css();

    let handle = TocPanelHandle {
        panel_box,
        list_box,
        paned: paned.clone(),
        visible: Rc::new(Cell::new(false)),
        depth: Rc::new(Cell::new(3)),
        current_entries: Rc::new(RefCell::new(Vec::new())),
        webview: webview.clone(),
        manual_width,
        programmatic_resize,
    };

    (paned, handle)
}

fn inject_toc_css() {
    use std::sync::OnceLock;
    static INJECTED: OnceLock<()> = OnceLock::new();
    if INJECTED.set(()).is_err() {
        return;
    }

    // `.toc-panel`'s `min-width` below must match `MIN_SIDEBAR_WIDTH` — kept
    // as a literal (not `format!`) since this whole block is a plain CSS
    // string with no other substitutions.
    let css = r#"
/* TOC Panel */
.toc-panel {
    background: transparent;
    border-right: 1px solid alpha(currentColor, 0.15);
    min-width: 150px;
}
.marco-theme-light .toc-panel {
    background-color: #f0f2f4;
    border-right: 1px solid #d0d3d8;
}
.marco-theme-dark .toc-panel {
    background-color: #1e2025;
    border-right: 1px solid #3a3d44;
}
.toc-panel-header {
    font-weight: bold;
    font-size: 0.85em;
    opacity: 0.65;
    letter-spacing: 0.06em;
    text-transform: uppercase;
}
.marco-theme-light .toc-panel-header { color: #2c3e50; }
.marco-theme-dark  .toc-panel-header { color: #c8cdd6; }
.toc-panel-empty {
    font-size: 0.85em;
    opacity: 0.5;
    margin: 8px;
}
.toc-panel-entry {
    border-radius: 4px;
    padding: 1px 6px;
    min-height: 22px;
}
.marco-theme-light .toc-panel-entry { color: #2c3e50; }
.marco-theme-dark  .toc-panel-entry { color: #c8cdd6; }
.marco-theme-light .toc-panel-entry:hover { background-color: rgba(0,0,0,0.07); }
.marco-theme-dark  .toc-panel-entry:hover { background-color: rgba(255,255,255,0.07); }

/* Bold for H1/H2, lighter for deeper levels */
.toc-depth-1 label { font-weight: bold; }
.toc-depth-2 label { font-weight: 600; }

/* Scrollbar — base (overlay-style, thin, no decoration) */
scrolledwindow.toc-panel-scroll scrollbar {
    -gtk-icon-transform: none;
    min-width: 12px;
    min-height: 12px;
    background: transparent;
    border: none;
    box-shadow: none;
    padding: 0;
    margin: 0;
}
scrolledwindow.toc-panel-scroll scrollbar trough {
    border: none;
    box-shadow: none;
    background-image: none;
    min-width: 12px;
    min-height: 12px;
    padding: 0;
    margin: 0;
}
scrolledwindow.toc-panel-scroll scrollbar slider {
    border-radius: 0px;
    border: none;
    box-shadow: none;
    background-image: none;
    min-width: 12px;
    min-height: 12px;
    margin: 0;
    padding: 0;
}

/* Scrollbar — Light theme */
.marco-theme-light scrolledwindow.toc-panel-scroll scrollbar trough {
    background-color: #F0F0F0;
}
.marco-theme-light scrolledwindow.toc-panel-scroll scrollbar slider {
    background-color: #D0D4D8;
}
.marco-theme-light scrolledwindow.toc-panel-scroll scrollbar slider:hover {
    background-color: #C2C7CC;
}

/* Scrollbar — Dark theme */
.marco-theme-dark scrolledwindow.toc-panel-scroll scrollbar trough {
    background-color: #252526;
}
.marco-theme-dark scrolledwindow.toc-panel-scroll scrollbar slider {
    background-color: #3A3F44;
}
.marco-theme-dark scrolledwindow.toc-panel-scroll scrollbar slider:hover {
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
