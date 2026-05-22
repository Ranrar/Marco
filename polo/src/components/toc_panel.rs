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

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use marco_core::intelligence::toc::TocEntry;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Minimum pixel width of the TOC panel.
const MIN_PANEL_WIDTH: i32 = 150;
/// Maximum pixel width of the TOC panel.
const MAX_PANEL_WIDTH: i32 = 400;

/// Handle that lets the rest of the app rebuild and toggle the TOC panel.
#[derive(Clone)]
pub struct TocPanelHandle {
    panel_box: gtk4::Box,
    list_box: gtk4::Box,
    paned: gtk4::Paned,
    visible: Rc<Cell<bool>>,
    /// Current maximum heading depth (1–6).
    pub depth: Rc<Cell<u8>>,
    /// Pixel width of the widest entry (text + indent).
    widest_entry_px: Rc<Cell<i32>>,
    /// Cached entries from the last file load.
    current_entries: Rc<RefCell<Vec<TocEntry>>>,
    /// WebView used to scroll the preview when a heading is clicked.
    webview: crate::components::viewer::platform_webview::PlatformWebView,
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

    /// Show the panel (rebuilds from the last cached entries).
    pub fn show(&self) {
        self.panel_box.set_visible(true);
        self.visible.set(true);
        let borrowed = self.current_entries.borrow();
        self.rebuild(&borrowed, self.depth.get());
    }

    /// Hide the panel.
    pub fn hide(&self) {
        self.panel_box.set_visible(false);
        self.visible.set(false);
    }

    /// Toggle visibility.
    pub fn toggle(&self) {
        if self.visible.get() {
            self.hide();
        } else {
            self.show();
        }
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

        // Measure pixel widths with Pango to auto-size the panel.
        let mut widest = 0i32;
        {
            let pango_ctx = self.list_box.pango_context();
            for entry in &filtered {
                let indent_px = ((entry.level - min_level) as i32) * 12;
                let layout = gtk4::pango::Layout::new(&pango_ctx);
                layout.set_text(&entry.text);
                let (text_w, _) = layout.pixel_size();
                widest = widest.max(text_w + indent_px);
            }
        }
        self.widest_entry_px.set(widest);

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

        // Auto-resize the paned position when the panel is visible.
        if self.visible.get() {
            let paned = self.paned.clone();
            let extra = 36; // button padding + panel margins + header + separator
            let width =
                (self.widest_entry_px.get() + extra).clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
            glib::idle_add_local_once(move || {
                paned.set_position(width);
            });
        }
    }
}

/// Create the TOC sidebar paned and return the paned together with the handle.
///
/// The caller sets the end child:
/// ```ignore
/// let (paned, toc) = create_toc_panel(&webview);
/// paned.set_end_child(Some(&webview.widget()));
/// window.set_child(Some(&paned));
/// ```
pub fn create_toc_panel(
    webview: &crate::components::viewer::platform_webview::PlatformWebView,
) -> (gtk4::Paned, TocPanelHandle) {
    let paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(0); // collapsed by default
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned.set_resize_start_child(false);
    paned.set_resize_end_child(true);

    // ── TOC panel (start child) ───────────────────────────────────────────────
    let panel_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    panel_box.set_visible(false);
    panel_box.set_hexpand(false);
    panel_box.set_vexpand(true);
    panel_box.set_width_request(80);
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
        widest_entry_px: Rc::new(Cell::new(0)),
        current_entries: Rc::new(RefCell::new(Vec::new())),
        webview: webview.clone(),
    };

    (paned, handle)
}

fn inject_toc_css() {
    use std::sync::OnceLock;
    static INJECTED: OnceLock<()> = OnceLock::new();
    if INJECTED.set(()).is_err() {
        return;
    }

    let css = r#"
/* TOC Panel */
.toc-panel {
    background: transparent;
    border-right: 1px solid alpha(currentColor, 0.15);
    min-width: 80px;
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
    provider.load_from_data(css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
