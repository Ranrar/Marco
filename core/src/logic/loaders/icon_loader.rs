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
//!
//! # Platform Differences
//!
//! - **Linux**: Uses `XDG_DATA_HOME` environment variable for Fontconfig
//! - **Windows**: Uses `FONTCONFIG_PATH` environment variable to point to fonts directory

use std::env;

/// Sets the environment variable so Fontconfig/Pango can find fonts in the given directory.
/// Call this before initializing GTK.
///
/// # Platform Behavior
///
/// - **Linux**: Sets `XDG_DATA_HOME` to the parent directory (Fontconfig looks in `$XDG_DATA_HOME/fonts/`)
/// - **Windows**: Sets `FONTCONFIG_PATH` to point directly to the fonts directory
#[cfg(target_os = "linux")]
pub fn set_local_font_dir(font_dir: &str) {
    // XDG_DATA_HOME is respected by Fontconfig on Linux
    // Fontconfig will look for fonts in $XDG_DATA_HOME/fonts/
    env::set_var("XDG_DATA_HOME", font_dir);
}

#[cfg(target_os = "windows")]
pub fn set_local_font_dir(font_dir: &str) {
    // On Windows, FONTCONFIG_PATH should point to the fonts directory
    // Construct the full path to the fonts subdirectory
    let fonts_path = std::path::Path::new(font_dir).join("fonts");
    if let Some(fonts_str) = fonts_path.to_str() {
        env::set_var("FONTCONFIG_PATH", fonts_str);
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]  
pub fn set_local_font_dir(_font_dir: &str) {
    // Unsupported platform - no-op
    // Font loading may not work correctly on this platform
}
