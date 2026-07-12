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

/// Detect the system locale as a BCP-47-ish tag: a bare ISO 639-1 language
/// code (`en`), or that code plus an ISO 3166-1 region subtag (`zh-CN`) when
/// the OS/environment reports one. Used to auto-select region-qualified
/// locale files such as `zh-CN.toml` / `zh-TW.toml`, as well as plain ones.
///
/// - Linux: reads `LC_ALL`, `LC_MESSAGES`, `LANG` (in that order).
/// - Windows: uses `GetUserDefaultLocaleName` and falls back to environment vars.
///
/// Returns `None` if no useful locale can be detected.
pub(crate) fn detect_system_locale_bcp47() -> Option<String> {
    // Prefer explicit language from environment variables if present.
    // This is particularly useful in dev environments and in WSL/MSYS.
    if let Some(from_env) = detect_from_env() {
        return Some(from_env);
    }

    // Platform-specific fallback.
    detect_from_platform().and_then(|raw| normalize_to_bcp47(&raw))
}

fn detect_from_env() -> Option<String> {
    // Common order used by many apps.
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"].into_iter() {
        if let Ok(val) = std::env::var(key) {
            if let Some(code) = normalize_to_bcp47(&val) {
                return Some(code);
            }
        }
    }

    None
}

/// Platform-specific fallback, returning the *raw* locale string (not yet
/// normalized) so callers can extract as much of it (language, region) as
/// they need.
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

/// Split a raw locale string (like `en_US.UTF-8`, `zh-Hans-CN`, `en`) into
/// its language and, if present, region parts: a lowercase ISO 639-1 code,
/// and an uppercase ISO 3166-1 region code.
///
/// The region is taken as the first remaining 2-letter alphabetic subtag, so
/// a 4-letter script subtag some platforms report for Chinese (`Hans`/`Hant`
/// in `zh-Hans-CN`) is skipped rather than mistaken for a region. Locales
/// that use a script subtag *without* a following region (e.g. bare
/// `zh-Hans`) are treated as having no region.
fn normalize_locale_parts(raw: &str) -> Option<(String, Option<String>)> {
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
    let mut parts = without_modifier.split(['_', '-']);

    let lang_part = parts.next().unwrap_or(without_modifier).trim();
    if lang_part.len() != 2 || !lang_part.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let lang = lang_part.to_ascii_lowercase();

    let region = parts
        .find(|part| {
            let part = part.trim();
            part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic())
        })
        .map(|part| part.trim().to_ascii_uppercase());

    Some((lang, region))
}

/// Normalize a locale string to a BCP-47-ish tag: `xx` or `xx-YY` when a
/// region subtag is present (e.g. `zh_CN.UTF-8` -> `zh-CN`).
fn normalize_to_bcp47(raw: &str) -> Option<String> {
    normalize_locale_parts(raw).map(|(lang, region)| match region {
        Some(region) => format!("{lang}-{region}"),
        None => lang,
    })
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
    fn smoke_test_normalize_to_bcp47() {
        // No region present.
        assert_eq!(normalize_to_bcp47("en"), Some("en".to_string()));
        assert_eq!(normalize_to_bcp47("EN"), Some("en".to_string()));

        assert_eq!(normalize_to_bcp47(""), None);
        assert_eq!(normalize_to_bcp47("C"), None);
        assert_eq!(normalize_to_bcp47("POSIX"), None);
        assert_eq!(normalize_to_bcp47("e"), None);
        assert_eq!(normalize_to_bcp47("eng"), None);
        assert_eq!(normalize_to_bcp47("1n"), None);

        // Region preserved (this is the whole point of the BCP-47 variant).
        assert_eq!(normalize_to_bcp47("en_US"), Some("en-US".to_string()));
        assert_eq!(normalize_to_bcp47("en-US"), Some("en-US".to_string()));
        assert_eq!(normalize_to_bcp47("da_DK.UTF-8"), Some("da-DK".to_string()));
        assert_eq!(normalize_to_bcp47("de-DE@euro"), Some("de-DE".to_string()));
        assert_eq!(normalize_to_bcp47("zh_CN.UTF-8"), Some("zh-CN".to_string()));
        assert_eq!(normalize_to_bcp47("zh_TW.UTF-8"), Some("zh-TW".to_string()));

        // A 4-letter script subtag (Windows sometimes reports `zh-Hans-CN` /
        // `zh-Hant-TW`) is skipped in favor of the following 2-letter region.
        assert_eq!(normalize_to_bcp47("zh-Hans-CN"), Some("zh-CN".to_string()));
        assert_eq!(normalize_to_bcp47("zh-Hant-TW"), Some("zh-TW".to_string()));

        // A script subtag with no region falls back to the bare language.
        assert_eq!(normalize_to_bcp47("zh-Hans"), Some("zh".to_string()));
    }
}
