//! Platform-specific path implementations.
//!
//! This module centralizes all OS-specific filesystem conventions for Marco/Polo.
//! Public `core::paths` APIs delegate to the functions exposed here.

use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

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
    path.join("fonts").is_dir()
        && path.join("icons").is_dir()
        && path.join("themes").is_dir()
        && path.join("language").is_dir()
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
