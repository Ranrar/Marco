//! File-read helpers that go through the global file cache.
//!
//! Used by [`crate::logic::buffer::DocumentBuffer::read_content`] and
//! `polo/src/components/viewer/rendering.rs`.

use std::io;
use std::path::Path;

/// Read a text file through the global [`crate::cache::FileCache`].
///
/// Returns cached content when the file has not changed on disk since the
/// last read. Otherwise reads from disk, caches the result, and returns it.
pub fn read_to_string(path: &Path) -> Result<String, io::Error> {
    if let Some(content) = super::global_cache().get(path) {
        return Ok(content);
    }

    // Capture mtime BEFORE reading content. If we read content first and then
    // mtime, a concurrent write would let us cache stale content tagged with
    // the post-write mtime, defeating mtime-based invalidation.
    let mtime_before = std::fs::metadata(path).and_then(|m| m.modified())?;
    let content = std::fs::read_to_string(path)?;
    let mtime_after = std::fs::metadata(path).and_then(|m| m.modified()).ok();

    // Only cache if the file was stable across the read. If mtime changed,
    // skip caching this round; the next read will pick up the latest content.
    if mtime_after == Some(mtime_before) {
        super::global_cache().insert(path.to_path_buf(), content.clone(), mtime_before);
    }
    Ok(content)
}
