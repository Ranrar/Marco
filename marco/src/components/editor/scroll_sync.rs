//! Scroll synchronization between editor and preview components
//!
//! This module provides functionality to synchronize scrolling between different
//! ScrolledWindow widgets, particularly the editor and preview panes.
//!
//! WebView synchronization runs over the unified wry-based
//! `PlatformWebView` (JS scroll events + `marco_scroll:` IPC reports) on all
//! platforms; basic ScrolledWindow synchronization is pure GTK.

use gtk4::prelude::*;
use log::debug;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use crate::components::viewer::wry_platform_webview::PlatformWebView;

/// Core scroll synchronization system with loop prevention and runtime control
pub struct ScrollSynchronizer {
    /// Guard flag to prevent infinite loops during synchronization
    is_syncing: Rc<Cell<bool>>,
    /// Whether synchronization is currently enabled
    enabled: Rc<Cell<bool>>,
    /// Counter-based suppression for WebView -> editor sync callbacks.
    ///
    /// This is used for programmatic jumps (e.g. bookmark navigation) where we
    /// want to ignore transient preview reports such as `marco_scroll:0.0` after
    /// preview reloads in large documents.
    suppress_preview_to_editor_sync: Rc<Cell<u32>>,
}

impl ScrollSynchronizer {
    /// Create a new scroll synchronizer
    pub fn new() -> Self {
        Self {
            is_syncing: Rc::new(Cell::new(false)),
            enabled: Rc::new(Cell::new(true)),
            suppress_preview_to_editor_sync: Rc::new(Cell::new(0)),
        }
    }

    /// Enable or disable scroll synchronization
    pub fn set_enabled(&self, enabled: bool) {
        debug!("Scroll sync enabled: {}", enabled);
        self.enabled.set(enabled);
    }

    /// Check if scroll synchronization is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Temporarily suppress WebView -> editor sync reports.
    ///
    /// Nested calls are supported; each `suspend` must be paired with `resume`.
    pub fn suspend_preview_to_editor_sync(&self) {
        let depth = self.suppress_preview_to_editor_sync.get();
        self.suppress_preview_to_editor_sync
            .set(depth.saturating_add(1));
    }

    /// Resume WebView -> editor sync reports after a previous suspension.
    pub fn resume_preview_to_editor_sync(&self) {
        let depth = self.suppress_preview_to_editor_sync.get();
        self.suppress_preview_to_editor_sync
            .set(depth.saturating_sub(1));
    }

    fn scroll_percentage(sw: &gtk4::ScrolledWindow) -> Option<f64> {
        let adj = sw.vadjustment();
        let upper = adj.upper();
        let page_size = adj.page_size();
        let range = upper - page_size;
        if range <= 0.0 {
            return None;
        }
        Some((adj.value() / range).clamp(0.0, 1.0))
    }

    /// Check if a widget has proper allocation for rendering
    fn has_valid_allocation(widget: &impl IsA<gtk4::Widget>) -> bool {
        let allocation = widget.allocation();
        allocation.width() > 0 && allocation.height() > 0
    }

    /// Set the scroll percentage of a ScrolledWindow with allocation check
    pub fn set_scroll_percentage(sw: &gtk4::ScrolledWindow, percentage: f64) {
        // Check if the ScrolledWindow has proper allocation before scrolling
        if !Self::has_valid_allocation(sw) {
            debug!("Skipping scroll operation - ScrolledWindow has no allocation");
            return;
        }

        let adj = sw.vadjustment();
        let upper = adj.upper();
        let page_size = adj.page_size();
        let range = upper - page_size;

        if range > 0.0 {
            let target_value = percentage.clamp(0.0, 1.0) * range;
            adj.set_value(target_value);
        }
    }

    /// Connect two ScrolledWindow widgets so scrolling the source updates the target.
    ///
    /// This is cross-platform and intended for syncing the editor pane with other
    /// GTK scrollable panes (for example the HTML code view TextView).
    pub fn connect_scrolled_window_to_scrolled_window(
        &self,
        source_sw: &gtk4::ScrolledWindow,
        target_sw: &gtk4::ScrolledWindow,
        label: &str,
    ) {
        let source_adj = source_sw.vadjustment();

        let is_syncing_clone = Rc::clone(&self.is_syncing);
        let enabled_clone = Rc::clone(&self.enabled);
        let source_sw_clone = source_sw.clone();
        let target_sw_clone = target_sw.clone();
        let label_owned = label.to_string();
        let last_sync = Rc::new(Cell::new(None::<Instant>));
        let last_sync_cb = Rc::clone(&last_sync);

        source_adj.connect_value_changed(move |_source_adj| {
            if is_syncing_clone.get() || !enabled_clone.get() {
                return;
            }

            const DEBOUNCE_MS: u64 = 16; // ~60fps

            let should_sync = {
                let now = Instant::now();
                if let Some(prev) = last_sync_cb.get() {
                    if now.duration_since(prev).as_millis() < DEBOUNCE_MS as u128 {
                        false
                    } else {
                        last_sync_cb.set(Some(now));
                        true
                    }
                } else {
                    last_sync_cb.set(Some(now));
                    true
                }
            };

            if !should_sync {
                return;
            }

            if let Some(percentage) = Self::scroll_percentage(&source_sw_clone) {
                is_syncing_clone.set(true);
                Self::set_scroll_percentage(&target_sw_clone, percentage);
                debug!(
                    "[scroll_sync] {} sync: {:.2}%",
                    label_owned,
                    percentage * 100.0
                );
                is_syncing_clone.set(false);
            }
        });
    }

    /// Set up bidirectional sync between two ScrolledWindow widgets.
    pub fn connect_scrolled_windows_bidirectional(
        &self,
        a: &gtk4::ScrolledWindow,
        b: &gtk4::ScrolledWindow,
    ) {
        self.connect_scrolled_window_to_scrolled_window(a, b, "scrolledwindow a->b");
        self.connect_scrolled_window_to_scrolled_window(b, a, "scrolledwindow b->a");
        debug!("Bidirectional scroll synchronization established between ScrolledWindows");
    }

    /// Connect ScrolledWindow to a Windows `PlatformWebView` (wry/WebView2)
    /// using JavaScript scrolling.
    ///
    /// This is Windows-only.
    pub fn connect_scrolled_window_to_platform_webview(
        &self,
        source_sw: &gtk4::ScrolledWindow,
        target_webview: &PlatformWebView,
        label: &str,
        last_host_percent: Rc<Cell<f64>>,
    ) {
        let source_adj = source_sw.vadjustment();

        let is_syncing_clone = Rc::clone(&self.is_syncing);
        let enabled_clone = Rc::clone(&self.enabled);
        let source_sw_clone = source_sw.clone();
        let target_webview_clone = target_webview.clone();
        let label_owned = label.to_string();
        let last_host_percent_for_closure = Rc::clone(&last_host_percent);
        let last_sync = Rc::new(Cell::new(None::<Instant>));
        let last_sync_cb = Rc::clone(&last_sync);

        source_adj.connect_value_changed(move |_source_adj| {
            if is_syncing_clone.get() || !enabled_clone.get() {
                return;
            }

            const DEBOUNCE_MS: u64 = 16; // ~60fps

            let should_sync = {
                let now = Instant::now();
                if let Some(prev) = last_sync_cb.get() {
                    if now.duration_since(prev).as_millis() < DEBOUNCE_MS as u128 {
                        false
                    } else {
                        last_sync_cb.set(Some(now));
                        true
                    }
                } else {
                    last_sync_cb.set(Some(now));
                    true
                }
            };

            if !should_sync {
                return;
            }

            let Some(scroll_percentage) = Self::scroll_percentage(&source_sw_clone) else {
                return;
            };

            last_host_percent_for_closure.set(scroll_percentage);
            is_syncing_clone.set(true);

            // Apply percentage to webview using JavaScript. Guard prevents feedback.
            let js_code = format!(
                r#"
                (function() {{
                    try {{
                        if (window.__scroll_sync_guard) return;
                        window.__scroll_sync_guard = true;

                        const maxScroll = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
                        const targetScroll = {p} * maxScroll;

                        window.scrollTo({{ top: targetScroll, behavior: 'auto' }});

                        setTimeout(() => {{ window.__scroll_sync_guard = false; }}, 50);
                    }} catch (e) {{
                    }}
                }})();
                "#,
                p = scroll_percentage
            );

            // Best-effort: if the webview isn't ready yet, this is a no-op.
            target_webview_clone.evaluate_script(&js_code);

            debug!(
                "[scroll_sync] {} sync: {:.2}%",
                label_owned,
                scroll_percentage * 100.0
            );

            is_syncing_clone.set(false);
        });
    }

    /// Bidirectional editor<->preview scroll sync for Windows wry/WebView2.
    pub fn connect_scrolled_window_and_platform_webview(
        &self,
        editor_sw: &gtk4::ScrolledWindow,
        preview_webview: &PlatformWebView,
    ) {
        let last_host_percent = Rc::new(Cell::new(-1.0f64));

        self.connect_scrolled_window_to_platform_webview(
            editor_sw,
            preview_webview,
            "editor->wry",
            Rc::clone(&last_host_percent),
        );

        // WebView -> editor sync via IPC messages (see SCROLL_REPORT_JS).
        let is_syncing_cb = Rc::clone(&self.is_syncing);
        let enabled_cb = Rc::clone(&self.enabled);
        let suppress_preview_to_editor_sync_cb = Rc::clone(&self.suppress_preview_to_editor_sync);
        let editor_sw_cb = editor_sw.clone();
        let last_host_percent_cb = Rc::clone(&last_host_percent);
        preview_webview.set_scroll_report_callback(move |percentage: f64| {
            if !enabled_cb.get() || is_syncing_cb.get() {
                return;
            }
            if suppress_preview_to_editor_sync_cb.get() > 0 {
                return;
            }
            if (percentage - last_host_percent_cb.get()).abs() < 0.0005 {
                return;
            }
            is_syncing_cb.set(true);
            Self::set_scroll_percentage(&editor_sw_cb, percentage);
            is_syncing_cb.set(false);
        });

        debug!(
            "Bidirectional scroll synchronization established between ScrolledWindow and PlatformWebView"
        );
    }

}

impl Default for ScrollSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}
