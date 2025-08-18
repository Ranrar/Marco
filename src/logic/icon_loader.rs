
//! Utility for loading a local TTF icon font for GTK4 widgets using Fontconfig/Pango workaround.
//! # IcoMoon Font Details
//!
//! Font Family: "icomoon"
//! Glyphs: Unicode 0x31–0x39 (characters '1'–'9')
//!
//! | Unicode | Icon Name                | Description           |
//! |---------|--------------------------|----------------------|
//! | 0x31    | marco-split_scene_left   | Show layout options   |
//! | 0x32    | marco-only_preview       | Only preview          |
//! | 0x33    | marco-only_editor        | Only editor           |
//! | 0x34    | marco-minimize           | Minimize              |
//! | 0x35    | marco-fullscreen_exit    | Exit maximize         |
//! | 0x36    | marco-fullscreen         | Maximize              |
//! | 0x37    | marco-editor_preview     | Standard layout       |
//! | 0x38    | marco-detatch            | Detatch               |
//! | 0x39    | marco-close              | Close                 |
//!
//! Usage Example:
//! ```rust
//! use pango::FontDescription;
//! let font_desc = FontDescription::from_string("icomoon 16");
//! label.set_font_desc(&font_desc);
//! label.set_text("\u{31}"); // marco-split_scene_left
//! ```
//!
//! This allows you to use a TTF icon font without installing it system-wide.
//!
//! Make sure to call these functions before initializing GTK.
//!
//! For more info, see: https://docs.rs/pango/latest/pango/struct.FontDescription.html
//!
//! Future helpers: font name extraction, reload, etc.
//! For each font, specify its purpose, icon set, or usage notes. You can reference each by its name in `FontDescription::from_string("<font_name> <size>")`.
//!
//! Example for using font 3:
//! ```rust
//! let font_desc = FontDescription::from_string("ui_menu3 16");
//! label.set_font_desc(&font_desc);
//! ```
//! ```
//!
//! This allows you to use a TTF icon font without installing it system-wide.
//!
//! Make sure to call these functions before initializing GTK.
//!
//! For more info, see: https://docs.rs/pango/latest/pango/struct.FontDescription.html
//!
//! Future helpers: font name extraction, reload, etc.
// ...existing code...

use std::env;

/// Sets the environment variable so Fontconfig/Pango can find fonts in the given directory.
/// Call this before initializing GTK.
pub fn set_local_font_dir(font_dir: &str) {
    // XDG_DATA_HOME is respected by Fontconfig on Linux
    env::set_var("XDG_DATA_HOME", font_dir);
}