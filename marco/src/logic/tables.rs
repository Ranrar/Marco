//! Global state and helpers for table editing behaviour.
//!
//! The `table_auto_align` flag controls whether Tab/Enter inside a GFM table
//! triggers automatic column re-alignment.  When enabled, the key-press
//! handler delegates to `components::editor::table_edit` which re-formats the
//! whole table block in-place before moving the cursor.

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether Tab/Enter inside a table should auto-align columns.
///
/// Loaded from `editor.table_auto_align` in `settings.ron` at startup.
/// Updated live when the user toggles the setting in Preferences.
static TABLE_AUTO_ALIGN: AtomicBool = AtomicBool::new(true);

/// `true` while a Tab/Enter navigation operation is actively modifying the
/// buffer.  The cursor-leave handler checks this and skips reformatting so
/// that navigation-triggered `cursor-position` signals cannot recursively
/// trigger a second reformat.
static TABLE_NAVIGATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Returns `true` if table auto-alignment is currently enabled.
#[inline]
pub fn is_table_auto_align_enabled() -> bool {
    TABLE_AUTO_ALIGN.load(Ordering::Relaxed)
}

/// Set the table auto-alignment flag.  Called at startup and whenever the
/// setting changes in the Preferences dialog.
pub fn set_table_auto_align(enabled: bool) {
    TABLE_AUTO_ALIGN.store(enabled, Ordering::Relaxed);
    log::debug!("[tables] auto-align = {enabled}");
}

/// Returns `true` while a key-press navigation is modifying the buffer.
#[inline]
pub fn is_table_navigation_in_progress() -> bool {
    TABLE_NAVIGATION_IN_PROGRESS.load(Ordering::Relaxed)
}

/// RAII guard: sets `TABLE_NAVIGATION_IN_PROGRESS` on creation and clears it
/// on drop.  Wrap any navigation that modifies the buffer in this guard so
/// the cursor-leave handler ignores spurious `cursor-position` signals fired
/// by the navigation's own buffer writes.
pub struct NavigationGuard;

impl NavigationGuard {
    pub fn new() -> Self {
        TABLE_NAVIGATION_IN_PROGRESS.store(true, Ordering::Relaxed);
        NavigationGuard
    }
}

impl Default for NavigationGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NavigationGuard {
    fn drop(&mut self) {
        TABLE_NAVIGATION_IN_PROGRESS.store(false, Ordering::Relaxed);
    }
}
