//! TOC (Table of Contents) sidebar panel.
//!
//! Displays a resizable panel to the left of the entire editor+preview area.
//! Each row is a clickable heading entry. Clicking scrolls:
//! - the source editor when in `EditorOnly` layout
//! - the live HTML preview (via JS `scrollIntoView`) in all other layouts
//!
//! Because the `toc_paned` wraps the *whole* inner editor/preview split, the
//! panel is independent of the `SplitController` lock and remains accessible
//! in `ViewOnly` mode as well.
//!
//! # Layout
//!
//! ```text
//! toc_paned (gtk4::Paned, outermost)   ← returned as EditorReturn element 10
//! ├── toc_panel (gtk4::Box)            ← start child (hidden by default)
//! │   ├── header label "Contents"
//! │   └── scrolled list of buttons
//! └── overlay (gtk4::Overlay)          ← end child; wraps editor+preview paned
//!     └── paned (gtk4::Paned)          ← inner editor / preview split
//! ```
//!
/// The `toc_paned` position (= panel width) is auto-sized to fit the widest
/// TOC entry whenever the panel is shown or rebuilt.
use core::intelligence::toc::TocEntry;
#[cfg(target_os = "linux")]
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
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
    /// Current maximum heading depth (1-6).  Updated when the settings change.
    pub(crate) depth: Rc<Cell<u8>>,
    /// Pixel width of the widest entry (text + indent), updated on rebuild.
    widest_entry_px: Rc<Cell<i32>>,
    /// Saved buffer reference and view reference for navigation callbacks.
    source_view: sourceview5::View,
}

impl TocPanelHandle {
    /// Returns whether the TOC panel is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    /// Update the heading depth used by future rebuilds.
    pub fn set_depth(&self, depth: u8) {
        let clamped = depth.clamp(1, 6);
        self.depth.set(clamped);
        // Immediately re-render if the panel is open.
        if self.visible.get() {
            self.rebuild_from_buffer();
        }
    }

    /// Show the panel with auto-sized width based on content,
    /// and populate the list from the current buffer contents.
    pub fn show(&self) {
        self.panel_box.set_visible(true);
        self.visible.set(true);
        self.rebuild_from_buffer();
    }

    /// Return an ideal paned position derived from the widest entry,
    /// clamped between [`MIN_PANEL_WIDTH`] and [`MAX_PANEL_WIDTH`].
    fn ideal_width(&self) -> i32 {
        // widest text+indent, plus button padding (6px * 2) + panel margins
        // (2+2) + header margin (8) + separator + a little breathing room.
        let extra = 36;
        (self.widest_entry_px.get() + extra).clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH)
    }

    /// Read the source-view buffer, parse markdown, and rebuild the list.
    /// If the panel is visible, also re-sizes the paned to fit content.
    fn rebuild_from_buffer(&self) {
        use gtk4::prelude::TextBufferExt;
        let buffer = self.source_view.buffer();
        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), false)
            .to_string();
        let depth = self.depth.get();
        match core::parser::parse(&text) {
            Ok(doc) => {
                let entries = core::intelligence::toc::extract_toc(&doc);
                self.rebuild(&entries, depth);
            }
            Err(e) => {
                log::warn!("[TOC] parser error during rebuild: {e}");
                self.rebuild(&[], depth);
            }
        }
        // `rebuild()` already schedules a deferred paned resize via
        // idle_add_local_once, so no additional set_position call is needed here.
    }

    /// Hide the panel.
    pub fn hide(&self) {
        self.panel_box.set_visible(false);
        self.visible.set(false);
    }

    /// Toggle visibility, resetting width when showing.
    pub fn toggle(&self) {
        if self.visible.get() {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Rebuild the entry list from a new slice of TOC entries filtered to `max_depth`.
    pub fn rebuild(&self, entries: &[TocEntry], max_depth: u8) {
        // Remove all existing children from the list box.
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

        // Measure the pixel width of each entry text (+ indent) with Pango.
        // Use list_box's own Pango context directly — appending/removing a
        // temporary probe widget to a visible list_box can trigger GTK to
        // snapshot the paned separator before it has an allocation, producing
        // the "Trying to snapshot GtkGizmo without a current allocation" warning.
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

            let label = gtk4::Label::new(Some(&entry.text));
            label.set_xalign(0.0);
            label.set_halign(gtk4::Align::Start);
            label.set_hexpand(true);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_margin_start(indent_px);
            btn.set_child(Some(&label));

            // depth-specific CSS class for optional styling
            btn.add_css_class(&format!("toc-depth-{}", entry.level));

            let slug = entry.slug.clone();
            let line = entry.line;
            let sv = self.source_view.clone();
            btn.connect_clicked(move |_| {
                use core::logic::layoutstate::LayoutState;
                let layout =
                    crate::components::editor::editor_manager::get_current_layout_state();

                match layout {
                    // Editor visible, preview hidden → scroll the source view.
                    LayoutState::EditorOnly => {
                        if line == 0 {
                            return;
                        }
                        let buffer = sv.buffer();
                        use gtk4::prelude::{TextBufferExt, TextViewExt};
                        // GTK lines are 0-based; span lines are 1-based.
                        let gtk_line = (line as i32).saturating_sub(1);
                        if let Some(mut iter) = buffer.iter_at_line(gtk_line) {
                            let mut line_end = iter;
                            line_end.forward_to_line_end();
                            buffer.select_range(&iter, &line_end);
                            sv.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.3);
                            sv.grab_focus();
                        }
                    }
                    // Preview visible (DualView, ViewOnly, EditorAndViewSeparate) → scroll preview.
                    LayoutState::DualView
                    | LayoutState::ViewOnly
                    | LayoutState::EditorAndViewSeparate => {
                        #[cfg(target_os = "linux")]
                        crate::components::editor::editor_manager::with_primary_preview_webview(
                            |wv| {
                                let js = format!(
                                    r#"(function(){{
                                        var el = document.getElementById({slug:?});
                                        if (el) {{ el.scrollIntoView({{behavior:'smooth', block:'start'}}); }}
                                    }})();"#,
                                    slug = slug,
                                );
                                use webkit6::prelude::WebViewExt as _;
                                wv.evaluate_javascript(
                                    &js,
                                    None,
                                    None,
                                    None::<&gio::Cancellable>,
                                    |_| {},
                                );
                            },
                        );
                    }
                }
            });

            self.list_box.append(&btn);
        }

        // After content changes, resize the paned to fit the widest entry.
        // Defer to idle so GTK has completed any pending layout passes first,
        // avoiding the "GtkGizmo without current allocation" snapshot warning.
        if self.visible.get() {
            let paned = self.paned.clone();
            let width = self.ideal_width();
            glib::idle_add_local_once(move || {
                paned.set_position(width);
            });
        }
    }
}

/// Create the TOC sidebar paned.  Returns:
/// - the `gtk4::Paned` (`toc_paned`) — the outermost container that wraps the
///   TOC sidebar and, as its end-child, the pre-existing split indicator
///   overlay (set by the caller via `toc_paned.set_end_child(Some(&overlay))`)
/// - a [`TocPanelHandle`] for runtime control
pub fn create_toc_panel(source_view: &sourceview5::View) -> (gtk4::Paned, TocPanelHandle) {
    // Outer paned – start = TOC panel, end = split overlay (set by caller)
    let paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    // Start at position 0 (start child collapsed) because the panel is hidden by
    // default. Setting a non-zero position on an unallocated Paned with a hidden
    // start child places the separator GtkGizmo at that offset before any layout
    // pass, which triggers "Trying to snapshot GtkGizmo without a current
    // allocation" at startup. show() sets the position to DEFAULT_PANEL_WIDTH.
    paned.set_position(0);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    // The TOC sidebar is a fixed-width panel: keep its width when the window is
    // resized. Only the end child (the editor+preview area) should stretch.
    paned.set_resize_start_child(false);
    paned.set_resize_end_child(true);

    // ── TOC panel (start child) ───────────────────────────────────────────────
    let panel_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    panel_box.set_visible(false); // hidden by default
    panel_box.set_hexpand(false);
    panel_box.set_vexpand(true);
    panel_box.set_width_request(80); // minimum usable width
    panel_box.add_css_class("toc-panel");

    // Header
    let header = gtk4::Label::new(Some("Contents"));
    header.set_halign(gtk4::Align::Start);
    header.set_margin_start(8);
    header.set_margin_top(6);
    header.set_margin_bottom(4);
    header.add_css_class("toc-panel-header");
    panel_box.append(&header);

    // Separator
    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    panel_box.append(&sep);

    // Scrolled list
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.add_css_class("toc-panel-scroll");
    // Pin scrollbar to the physical-right edge regardless of application text
    // direction.  Without this, activating RTL causes a double-mirror: the Paned
    // container allocates the panel from the right AND the ScrolledWindow's own
    // RTL layout also flips, placing the scrollbar off-screen.  Children of this
    // ScrolledWindow carry TextDirection::None so they still inherit RTL from the
    // global default (set_default_direction) and render text correctly.
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

    // CSS for the panel is injected globally once
    inject_toc_css();

    let handle = TocPanelHandle {
        panel_box,
        list_box,
        paned: paned.clone(),
        visible: Rc::new(Cell::new(false)),
        depth: Rc::new(Cell::new(3)),
        widest_entry_px: Rc::new(Cell::new(0)),
        source_view: source_view.clone(),
    };

    (paned, handle)
}

fn inject_toc_css() {
    // Guard: only register once per process
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
