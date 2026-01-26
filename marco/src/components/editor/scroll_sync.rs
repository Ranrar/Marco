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

#[cfg(target_os = "linux")]
use webkit6::prelude::*;

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
