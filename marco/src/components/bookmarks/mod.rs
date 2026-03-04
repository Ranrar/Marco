use core::logic::swanson::{BookmarkEntry, SettingsManager};
use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

/// Bookmarks manager backed by centralized settings persistence.
///
/// Line numbers are 0-based internally.
pub struct BookmarkManager {
    settings_manager: Arc<SettingsManager>,
    changed_callbacks: RefCell<Vec<Box<dyn Fn()>>>,
}

impl BookmarkManager {
    pub fn new(settings_manager: Arc<SettingsManager>) -> Self {
        Self {
            settings_manager,
            changed_callbacks: RefCell::new(Vec::new()),
        }
    }

    pub fn register_changed_callback<F: Fn() + 'static>(&self, cb: F) {
        self.changed_callbacks.borrow_mut().push(Box::new(cb));
    }

    pub fn get_all(&self) -> Vec<BookmarkEntry> {
        let mut entries = self.settings_manager.get_settings().get_bookmarks();
        normalize_bookmarks(&mut entries);
        entries
    }

    pub fn get_for_file<P: AsRef<Path>>(&self, file_path: P) -> Vec<u32> {
        let file_path = file_path.as_ref();
        let mut lines: Vec<u32> = self
            .get_all()
            .into_iter()
            .filter(|entry| entry.file_path == file_path)
            .map(|entry| entry.line)
            .collect();
        lines.sort_unstable();
        lines.dedup();
        lines
    }

    pub fn is_bookmarked<P: AsRef<Path>>(&self, file_path: P, line: u32) -> bool {
        let file_path = file_path.as_ref();
        self.get_all()
            .iter()
            .any(|entry| entry.file_path == file_path && entry.line == line)
    }

    pub fn add<P: AsRef<Path>>(&self, file_path: P, line: u32) -> bool {
        let file_path = file_path.as_ref().to_path_buf();
        self.mutate_bookmarks(|entries| {
            if entries
                .iter()
                .any(|entry| entry.file_path == file_path && entry.line == line)
            {
                return false;
            }
            entries.push(BookmarkEntry { file_path, line });
            true
        })
    }

    pub fn remove<P: AsRef<Path>>(&self, file_path: P, line: u32) -> bool {
        let file_path = file_path.as_ref().to_path_buf();
        self.mutate_bookmarks(|entries| {
            let before = entries.len();
            entries.retain(|entry| !(entry.file_path == file_path && entry.line == line));
            entries.len() != before
        })
    }

    pub fn toggle<P: AsRef<Path>>(&self, file_path: P, line: u32) -> bool {
        let file_path = file_path.as_ref().to_path_buf();
        self.mutate_bookmarks(|entries| {
            let before = entries.len();
            entries.retain(|entry| !(entry.file_path == file_path && entry.line == line));
            if entries.len() == before {
                entries.push(BookmarkEntry { file_path, line });
            }
            true
        })
    }

    /// Shift bookmark lines after a text insertion.
    ///
    /// `at_line` and `inserted_lines` are 0-based and count full line inserts.
    pub fn shift_after_insert<P: AsRef<Path>>(
        &self,
        file_path: P,
        at_line: u32,
        inserted_lines: u32,
    ) -> bool {
        if inserted_lines == 0 {
            return false;
        }
        let file_path = file_path.as_ref().to_path_buf();
        self.mutate_bookmarks(|entries| {
            let mut changed = false;
            for entry in entries.iter_mut() {
                if entry.file_path == file_path && entry.line >= at_line {
                    entry.line = entry.line.saturating_add(inserted_lines);
                    changed = true;
                }
            }
            changed
        })
    }

    /// Apply line deletion semantics.
    ///
    /// Removes bookmarks inside the deleted range and shifts trailing bookmarks upward.
    pub fn apply_delete_range<P: AsRef<Path>>(
        &self,
        file_path: P,
        start_line: u32,
        deleted_lines: u32,
    ) -> bool {
        if deleted_lines == 0 {
            return false;
        }

        let file_path = file_path.as_ref().to_path_buf();
        let end_line_exclusive = start_line.saturating_add(deleted_lines);

        self.mutate_bookmarks(|entries| {
            let mut changed = false;

            let before = entries.len();
            entries.retain(|entry| {
                if entry.file_path != file_path {
                    return true;
                }
                let deleted = entry.line >= start_line && entry.line < end_line_exclusive;
                if deleted {
                    changed = true;
                    return false;
                }
                true
            });
            if entries.len() != before {
                changed = true;
            }

            for entry in entries.iter_mut() {
                if entry.file_path == file_path && entry.line >= end_line_exclusive {
                    entry.line = entry.line.saturating_sub(deleted_lines);
                    changed = true;
                }
            }

            changed
        })
    }

    /// Replace all bookmarks for `file_path` with `lines`.
    ///
    /// Input line numbers are 0-based.
    pub fn replace_for_file<P: AsRef<Path>>(&self, file_path: P, lines: &[u32]) -> bool {
        let file_path = file_path.as_ref().to_path_buf();
        let mut desired = lines.to_vec();
        desired.sort_unstable();
        desired.dedup();

        self.mutate_bookmarks(|entries| {
            let mut current: Vec<u32> = entries
                .iter()
                .filter(|entry| entry.file_path == file_path)
                .map(|entry| entry.line)
                .collect();
            current.sort_unstable();
            current.dedup();

            if current == desired {
                return false;
            }

            entries.retain(|entry| entry.file_path != file_path);
            for &line in &desired {
                entries.push(BookmarkEntry {
                    file_path: file_path.clone(),
                    line,
                });
            }
            true
        })
    }

    pub fn grouped_by_current(
        &self,
        current_file: Option<&Path>,
    ) -> (Vec<BookmarkEntry>, Vec<BookmarkEntry>) {
        let mut current = Vec::new();
        let mut other = Vec::new();

        for entry in self.get_all() {
            if current_file.is_some_and(|path| path == entry.file_path.as_path()) {
                current.push(entry);
            } else {
                other.push(entry);
            }
        }

        sort_bookmarks_for_menu(&mut current);
        sort_bookmarks_for_menu(&mut other);
        (current, other)
    }

    pub fn cleanup_missing_files(&self) -> bool {
        self.mutate_bookmarks(|entries| {
            let before = entries.len();
            entries.retain(|entry| entry.file_path.exists());
            before != entries.len()
        })
    }

    fn mutate_bookmarks<F>(&self, mutator: F) -> bool
    where
        F: FnOnce(&mut Vec<BookmarkEntry>) -> bool,
    {
        let mut changed = false;
        let result = self.settings_manager.update_settings(|settings| {
            let mut entries = settings.get_bookmarks();
            changed = mutator(&mut entries);
            if changed {
                normalize_bookmarks(&mut entries);
                settings.set_bookmarks(entries);
            }
        });

        if let Err(e) = result {
            log::warn!("Failed to update bookmarks: {}", e);
            return false;
        }

        if changed {
            self.notify_changed();
        }
        changed
    }

    fn notify_changed(&self) {
        for cb in self.changed_callbacks.borrow().iter() {
            cb();
        }
    }
}

fn normalize_bookmarks(entries: &mut Vec<BookmarkEntry>) {
    entries.retain(|entry| !entry.file_path.as_os_str().is_empty());
    entries.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.line.cmp(&b.line))
    });
    entries.dedup_by(|a, b| a.file_path == b.file_path && a.line == b.line);
}

fn sort_bookmarks_for_menu(entries: &mut [BookmarkEntry]) {
    entries.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.line.cmp(&b.line))
    });
}

pub fn bookmark_menu_label_with_path(entry: &BookmarkEntry) -> String {
    let file_name = entry
        .file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown");
    format!(
        "{}:{} — {}",
        file_name.replace('_', "__"),
        entry.line + 1,
        entry.file_path.display()
    )
}
