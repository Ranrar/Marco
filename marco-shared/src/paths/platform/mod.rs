//! Platform-specific path implementations.
//!
//! This module centralizes all OS-specific filesystem conventions for Marco/Polo.
//! Public `core::paths` APIs delegate to the functions exposed here.

use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

// --------------------------------------------------------------------------
// Locale detection (shared helper; platform-specific fallbacks live in OS files)
// --------------------------------------------------------------------------

/// Detect the system locale and return an ISO 639-1 (two-letter) language code.
///
/// - Linux: reads `LC_ALL`, `LC_MESSAGES`, `LANG` (in that order).
/// - Windows: uses `GetUserDefaultLocaleName` and falls back to environment vars.
///
/// Returns `None` if no useful locale can be detected.
pub(crate) fn detect_system_locale_iso639_1() -> Option<String> {
    // Prefer explicit language from environment variables if present.
    // This is particularly useful in dev environments and in WSL/MSYS.
    if let Some(from_env) = detect_from_env() {
        return Some(from_env);
    }

    // Platform-specific fallback.
    detect_from_platform()
}

fn detect_from_env() -> Option<String> {
    // Common order used by many apps.
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"].into_iter() {
        if let Ok(val) = std::env::var(key) {
            if let Some(code) = normalize_to_iso639_1(&val) {
                return Some(code);
            }
        }
    }

    None
}

fn detect_from_platform() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        linux::detect_locale_from_platform()
    }

    #[cfg(target_os = "windows")]
    {
        windows::detect_locale_from_platform()
    }
}

/// Normalize a locale string (like `en_US.UTF-8`, `de-DE`, `en`) to `en`, `de`, …
fn normalize_to_iso639_1(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Strip encoding and modifiers (e.g. `.UTF-8`, `@euro`).
    let without_encoding = trimmed.split('.').next().unwrap_or(trimmed);
    let without_modifier = without_encoding
        .split('@')
        .next()
        .unwrap_or(without_encoding);

    // Locale may be `en_US`, `en-US`, or just `en`.
    let lang_part = without_modifier
        .split(['_', '-'])
        .next()
        .unwrap_or(without_modifier)
        .trim();

    if lang_part.len() != 2 {
        return None;
    }

    if !lang_part.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }

    Some(lang_part.to_ascii_lowercase())
}

/// Candidate locations (in priority order) where an *asset bundle root* could exist.
#[cfg(target_os = "linux")]
pub(crate) fn asset_root_candidates(exe_parent: &Path) -> Vec<PathBuf> {
    linux::asset_root_candidates(exe_parent)
}

#[cfg(target_os = "windows")]
pub(crate) fn asset_root_candidates(exe_parent: &Path) -> Vec<PathBuf> {
    windows::asset_root_candidates(exe_parent)
}

/// Return true if `path` looks like a real Marco/Polo asset bundle root.
///
/// This is important because user-data directories may exist even when no bundled
/// assets are present; accepting an arbitrary directory can shadow system assets
/// (notably in the Linux .deb layout).
pub(crate) fn is_valid_asset_root(path: &Path) -> bool {
    // Keep this intentionally minimal and aligned with the actual `assets/` layout.
    // Icon-font support was removed, so a `fonts/` directory is no longer required.
    path.join("icons").is_dir() && path.join("themes").is_dir() && path.join("language").is_dir()
}

#[cfg(target_os = "linux")]
pub(crate) fn config_dir() -> PathBuf {
    linux::config_dir()
}

#[cfg(target_os = "windows")]
pub(crate) fn config_dir() -> PathBuf {
    windows::config_dir()
}

#[cfg(target_os = "linux")]
pub(crate) fn user_data_dir() -> PathBuf {
    linux::user_data_dir()
}

#[cfg(target_os = "windows")]
pub(crate) fn user_data_dir() -> PathBuf {
    windows::user_data_dir()
}

#[cfg(target_os = "linux")]
pub(crate) fn detect_portable_mode() -> Option<PathBuf> {
    linux::detect_portable_mode()
}

#[cfg(target_os = "windows")]
pub(crate) fn detect_portable_mode() -> Option<PathBuf> {
    windows::detect_portable_mode()
}

#[cfg(target_os = "linux")]
pub(crate) fn detect_install_location_from_asset_root(asset_root: &Path) -> super::InstallLocation {
    linux::detect_install_location_from_asset_root(asset_root)
}

#[cfg(target_os = "windows")]
pub(crate) fn detect_install_location_from_asset_root(asset_root: &Path) -> super::InstallLocation {
    windows::detect_install_location_from_asset_root(asset_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_normalize_to_iso639_1() {
        assert_eq!(normalize_to_iso639_1("en"), Some("en".to_string()));
        assert_eq!(normalize_to_iso639_1("EN"), Some("en".to_string()));
        assert_eq!(normalize_to_iso639_1("en_US"), Some("en".to_string()));
        assert_eq!(normalize_to_iso639_1("en-US"), Some("en".to_string()));
        assert_eq!(normalize_to_iso639_1("da_DK.UTF-8"), Some("da".to_string()));
        assert_eq!(normalize_to_iso639_1("de-DE@euro"), Some("de".to_string()));

        assert_eq!(normalize_to_iso639_1(""), None);
        assert_eq!(normalize_to_iso639_1("C"), None);
        assert_eq!(normalize_to_iso639_1("POSIX"), None);
        assert_eq!(normalize_to_iso639_1("e"), None);
        assert_eq!(normalize_to_iso639_1("eng"), None);
        assert_eq!(normalize_to_iso639_1("1n"), None);
    }
}
