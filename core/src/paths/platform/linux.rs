use std::path::{Path, PathBuf};

use crate::paths::InstallLocation;

pub(crate) fn asset_root_candidates(_exe_parent: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // User-local asset install (not the same as config; may or may not exist).
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".local/share/marco"));
    }

    // System-local install
    candidates.push(PathBuf::from("/usr/local/share/marco"));

    // System-global install (Debian package layout)
    candidates.push(PathBuf::from("/usr/share/marco"));

    candidates
}

pub(crate) fn config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|c| c.join("marco"))
        .or_else(|| dirs::home_dir().map(|h| h.join(".config").join("marco")))
        .unwrap_or_else(|| PathBuf::from("/tmp/marco/config"))
}

pub(crate) fn user_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|d| d.join("marco"))
        .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share").join("marco")))
        .unwrap_or_else(|| PathBuf::from("/tmp/marco/data"))
}

pub(crate) fn detect_portable_mode() -> Option<PathBuf> {
    // Portable mode is a Windows-only concept.
    None
}

pub(crate) fn detect_install_location_from_asset_root(asset_root: &Path) -> InstallLocation {
    if let Some(home) = dirs::home_dir() {
        let user_local = home.join(".local/share/marco");
        if asset_root.starts_with(&user_local) {
            return InstallLocation::UserLocal;
        }
    }

    if asset_root.starts_with(Path::new("/usr/local/share/marco")) {
        return InstallLocation::SystemLocal;
    }

    if asset_root.starts_with(Path::new("/usr/share/marco")) {
        return InstallLocation::SystemGlobal;
    }

    InstallLocation::UserLocal
}
