//! Scroll synchronization between editor and preview components
//!
//! This module provides functionality to synchronize scrolling between different
//! ScrolledWindow widgets, particularly the editor and preview panes.
//!
//! # Platform Support
//!
//! WebView synchronization is Linux-only (uses WebKit6).
//! Basic ScrolledWindow synchronization is cross-platform.

use gtk4::prelude::*;
use log::debug;
use std::cell::Cell;
use std::rc::Rc;
#[cfg(target_os = "windows")]
use std::time::Instant;

#[cfg(target_os = "linux")]
use webkit6::prelude::*;

#[cfg(target_os = "windows")]
use crate::components::viewer::wry_platform_webview::PlatformWebView;

/// Core scroll synchronization system with loop prevention and runtime control
pub struct ScrollSynchronizer {
    /// Guard flag to prevent infinite loops during synchronization
    is_syncing: Rc<Cell<bool>>,
    /// Whether synchronization is currently enabled
    enabled: Rc<Cell<bool>>,
}

impl ScrollSynchronizer {
    /// Create a new scroll synchronizer
    pub fn new() -> Self {
        Self {
            is_syncing: Rc::new(Cell::new(false)),
            enabled: Rc::new(Cell::new(true)),
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

    #[cfg(target_os = "windows")]
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
    #[cfg(target_os = "windows")]
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
    #[cfg(target_os = "windows")]
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
    #[cfg(target_os = "windows")]
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
    #[cfg(target_os = "windows")]
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
        let editor_sw_cb = editor_sw.clone();
        let last_host_percent_cb = Rc::clone(&last_host_percent);
        preview_webview.set_scroll_report_callback(move |percentage: f64| {
            if !enabled_cb.get() || is_syncing_cb.get() {
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

    /// Connect ScrolledWindow to WebView using JavaScript scroll events
    ///
    /// # Platform Support
    ///
    /// Linux-only (uses WebKit6 for WebView)
    #[cfg(target_os = "linux")]
    pub fn connect_scrolled_window_to_webview(
        &self,
        source_sw: &gtk4::ScrolledWindow,
        target_webview: &webkit6::WebView,
        label: &str,
    ) {
        // Get vertical adjustment from scrolled window
        let source_adj = source_sw.vadjustment();

        // Clone references for closure
        let is_syncing_clone = Rc::clone(&self.is_syncing);
        let enabled_clone = Rc::clone(&self.enabled);
        let target_webview_clone = target_webview.clone();
        let label_owned = label.to_string();

        // Connect source -> webview synchronization
        source_adj.connect_value_changed(move |source_adj| {
            // Skip if we're already syncing, if sync is disabled, or if debouncing
            if is_syncing_clone.get() || !enabled_clone.get() {
                return;
            }

            // Check debouncing - create a minimal sync checker
            const DEBOUNCE_MS: u64 = 16; // ~60fps
            thread_local! {
                static LAST_SYNC: Cell<Option<std::time::Instant>> = const { Cell::new(None) };
            }

            let should_sync = LAST_SYNC.with(|last| {
                let now = std::time::Instant::now();
                if let Some(last_sync) = last.get() {
                    if now.duration_since(last_sync).as_millis() < DEBOUNCE_MS as u128 {
                        return false;
                    }
                }
                last.set(Some(now));
                true
            });

            if !should_sync {
                return;
            }

            // Set sync guard to prevent feedback loops
            is_syncing_clone.set(true);

            // Calculate scroll percentage in source
            let source_value = source_adj.value();
            let source_upper = source_adj.upper();
            let source_page_size = source_adj.page_size();

            // Avoid division by zero
            let source_range = source_upper - source_page_size;
            if source_range <= 0.0 {
                is_syncing_clone.set(false);
                return;
            }

            let scroll_percentage = (source_value / source_range).clamp(0.0, 1.0);

            // Apply percentage to webview using JavaScript
            let js_code = format!(
                r#"
                (function() {{
                    if (window.__scroll_sync_guard) return;
                    window.__scroll_sync_guard = true;
                    
                    const maxScroll = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
                    const targetScroll = {} * maxScroll;
                    
                    window.scrollTo({{
                        top: targetScroll,
                        behavior: 'auto'
                    }});
                    
                    setTimeout(() => {{
                        window.__scroll_sync_guard = false;
                    }}, 50);
                }})();
                "#,
                scroll_percentage
            );

            target_webview_clone.evaluate_javascript(&js_code, None, None, None::<&gio::Cancellable>, |result| {
                if let Err(e) = result {
                    debug!("JavaScript scroll sync error: {:?}", e);
                }
            });

            debug!(
                "[scroll_sync] {} sync: {:.2}% (SW {:.1})",
                label_owned, scroll_percentage * 100.0, source_value
            );

            // Clear sync guard
            is_syncing_clone.set(false);
        });
    }

    /// Set up bidirectional scroll synchronization between ScrolledWindow and WebView
    #[cfg(target_os = "linux")]
    pub fn connect_scrolled_window_and_webview(
        &self,
        editor_sw: &gtk4::ScrolledWindow,
        preview_webview: &webkit6::WebView,
    ) {
        // Connect editor ScrolledWindow -> WebView
        self.connect_scrolled_window_to_webview(editor_sw, preview_webview, "editor->webview");

        // Setup WebView -> editor ScrolledWindow using title change detection
        self.setup_webview_title_listener(preview_webview, editor_sw, "webview->editor");

        debug!(
            "Bidirectional scroll synchronization established between ScrolledWindow and WebView"
        );
    }

    /// Setup title change listener in WebView to sync back to ScrolledWindow
    #[cfg(target_os = "linux")]
    pub fn setup_webview_title_listener(
        &self,
        source_webview: &webkit6::WebView,
        target_sw: &gtk4::ScrolledWindow,
        label: &str,
    ) {
        // Clone references for the title change handler
        let is_syncing_clone = Rc::clone(&self.is_syncing);
        let enabled_clone = Rc::clone(&self.enabled);
        let target_sw_clone = target_sw.clone();
        let label_owned = label.to_string();

        // Connect to notify::title signal to handle scroll position reports
        source_webview.connect_notify_local(Some("title"), move |webview, _| {
            if !enabled_clone.get() || is_syncing_clone.get() {
                return;
            }

            // Debouncing for webview->editor sync
            const DEBOUNCE_MS: u64 = 16; // ~60fps
            thread_local! {
                static LAST_WEBVIEW_SYNC: Cell<Option<std::time::Instant>> = const { Cell::new(None) };
            }

            let should_sync = LAST_WEBVIEW_SYNC.with(|last| {
                let now = std::time::Instant::now();
                if let Some(last_sync) = last.get() {
                    if now.duration_since(last_sync).as_millis() < DEBOUNCE_MS as u128 {
                        return false;
                    }
                }
                last.set(Some(now));
                true
            });

            if !should_sync {
                return;
            }

            if let Some(title) = webview.title() {
                let title_str = title.as_str();
                if let Some(scroll_data) = title_str.strip_prefix("marco_scroll:") {
                    if let Ok(percentage) = scroll_data.parse::<f64>() {
                        // Set sync guard and update ScrolledWindow
                        is_syncing_clone.set(true);
                        Self::set_scroll_percentage(&target_sw_clone, percentage);

                        debug!(
                            "[scroll_sync] {} sync: {:.2}%",
                            label_owned, percentage * 100.0
                        );

                        is_syncing_clone.set(false);
                    }
                }
            }
        });

        debug!("WebView title-based scroll listener setup complete");
    }
}

impl Default for ScrollSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}
