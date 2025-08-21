
//! Utilities for using a bundled IcoMoon TTF icon font with GTK/Pango.
//!
//! Font family: `icomoon` (glyphs U+31..U+39 map to the app icons):
//! - U+31: split_scene_left (layout)
//! - U+32: only_preview
//! - U+33: only_editor
//! - U+34: minimize
//! - U+35: fullscreen_exit (restore)
//! - U+36: fullscreen (maximize)
//! - U+37: editor_preview (split)
//! - U+38: detatch
//! - U+39: close
//!
//! Call `set_local_font_dir` before initializing GTK so Fontconfig can find the bundled TTF.

use std::env;

/// Sets the environment variable so Fontconfig/Pango can find fonts in the given directory.
/// Call this before initializing GTK.
pub fn set_local_font_dir(font_dir: &str) {
    // XDG_DATA_HOME is respected by Fontconfig on Linux
    env::set_var("XDG_DATA_HOME", font_dir);
}