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
//! Call `set_local_font_dir` before initializing GTK so the bundled TTF is discoverable.

use crate::logic::crossplatforms::{detect_platform, Platform};
use std::{env, fs, path::Path, sync::OnceLock};

static FONTCONFIG_FILE: OnceLock<String> = OnceLock::new();

/// Inline SVG definitions for window control icons. Colors can be applied by replacing
/// `currentColor` in the returned string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowIcon {
    Close,
    Minimize,
    Maximize,
    Restore,
    Sun,
    Moon,
}

/// Get the inline SVG string for a window control icon with non-scaling strokes for crisp rendering.
pub fn window_icon_svg(icon: WindowIcon) -> &'static str {
    match icon {
        WindowIcon::Close => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M18 6l-12 12' vector-effect='non-scaling-stroke'/><path d='M6 6l12 12' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Minimize => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M5 12h14' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Maximize => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M5 7a2 2 0 0 1 2 -2h10a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2l0 -10' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Restore => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M8 6a2 2 0 0 1 2 -2h8a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2l0 -8' vector-effect='non-scaling-stroke'/><path d='M16 16v2a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2h2' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Sun => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><circle cx='12' cy='12' r='4' vector-effect='non-scaling-stroke'/><path d='M3 12h1m8 -9v1m8 8h1m-9 8v1m-6.4 -15.4l.7 .7m12.1 -.7l-.7 .7m0 11.4l.7 .7m-12.1 -.7l-.7 .7' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Moon => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454' vector-effect='non-scaling-stroke'/></svg>"#,
    }
}

/// Sets environment so Fontconfig/Pango can find fonts in the given directory.
/// Call this before initializing GTK.
pub fn set_local_font_dir(font_dir: &str) {
    let font_dir_path = Path::new(font_dir).join("fonts");
    log::info!(
        "icon font setup: asset_root={}, fonts_dir={}",
        font_dir,
        font_dir_path.display()
    );

    if !font_dir_path.exists() {
        log::warn!("icon font directory not found: {}", font_dir_path.display());
        return;
    }

    // Normalize separators for env vars to avoid mixed slash handling on Windows.
    let font_dir_normalized = Path::new(font_dir)
        .to_string_lossy()
        .replace('\\', "/");

    // XDG_DATA_HOME is respected by Fontconfig on Linux and by the fontconfig backend on Windows.
    env::set_var("XDG_DATA_HOME", &font_dir_normalized);
    env::set_var("FC_NO_CACHE", "1");

    if detect_platform() == Platform::Windows {
        // Force Pango to use the Fontconfig backend so FONTCONFIG_* env vars take effect.
        env::set_var("PANGOCAIRO_BACKEND", "fontconfig");

        if let Err(err) = configure_fontconfig(&font_dir_path) {
            log::warn!("failed to configure fontconfig for icons: {}", err);
        }
    }
}

/// Generate a minimal fontconfig file that points to the bundled fonts dir.
fn configure_fontconfig(font_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let font_dir = font_dir.canonicalize()?;

    let config_path = FONTCONFIG_FILE.get_or_init(|| {
        let mut path = std::env::temp_dir();
        path.push("marco_fontconfig.conf");
        path.to_string_lossy().to_string()
    });

    let font_dir_normalized = font_dir
        .to_string_lossy()
        .replace('\\', "/");

    let config_contents = format!(
        "<?xml version=\"1.0\"?>\n<!DOCTYPE fontconfig SYSTEM \"fonts.dtd\">\n<fontconfig>\n    <dir>{}</dir>\n</fontconfig>\n",
        font_dir_normalized
    );

    fs::write(config_path, config_contents)?;

    log::info!(
        "icon font fontconfig configured: FONTCONFIG_FILE={}, FONTCONFIG_PATH={}",
        config_path,
        Path::new(config_path)
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(none)".to_string())
    );

    if let Some(parent) = Path::new(config_path).parent() {
        env::set_var("FONTCONFIG_PATH", parent);
    }
    env::set_var("FONTCONFIG_FILE", config_path);
    Ok(())
}
