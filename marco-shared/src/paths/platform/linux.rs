use std::env;
use std::path::{Path, PathBuf};

use crate::paths::InstallLocation;

pub(crate) fn asset_root_candidates(exe_parent: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Portable / self-contained bundle: assets folder next to executable.
    candidates.push(exe_parent.join("assets"));

    // User-local asset install (not the same as config; may or may not exist).
    if let Some(home) = dirs::home_dir() {
        candidates.push(
            home.join(".local")
                .join("share")
                .join(crate::paths::APP_DIR_NAME),
        );
    }

    // System-local install
    candidates.push(PathBuf::from("/usr/local/share").join(crate::paths::APP_DIR_NAME));

    // System-global install (Debian package layout)
    candidates.push(PathBuf::from("/usr/share").join(crate::paths::APP_DIR_NAME));

    // Prefix-relative fallback: <exe_dir>/../share/<APP_DIR_NAME>. Covers any
    // install prefix the literals above miss -- an /opt tree, a staged install,
    // anything rooted somewhere other than /usr.
    //
    // Appended last on purpose. Under the Debian package the executable is
    // /usr/bin/<name>, so this resolves to /usr/share/<APP_DIR_NAME> -- the
    // same directory the literal above already covers. Placing it earlier
    // would promote the system-global tree over a user-local one and silently
    // change resolution order for .deb installs.
    if let Some(prefix) = exe_parent.parent() {
        candidates.push(prefix.join("share").join(crate::paths::APP_DIR_NAME));
    }

    candidates
}

pub(crate) fn config_dir() -> PathBuf {
    // Portable mode: keep config next to exe.
    if let Some(portable_root) = detect_portable_mode() {
        return portable_root.join("config");
    }

    dirs::config_dir()
        .map(|c| c.join(crate::paths::APP_DIR_NAME))
        .or_else(|| dirs::home_dir().map(|h| h.join(".config").join(crate::paths::APP_DIR_NAME)))
        .unwrap_or_else(|| {
            PathBuf::from("/tmp")
                .join(crate::paths::APP_DIR_NAME)
                .join("config")
        })
}

pub(crate) fn user_data_dir() -> PathBuf {
    // Portable mode: keep data next to exe.
    if let Some(portable_root) = detect_portable_mode() {
        return portable_root.join("data");
    }

    dirs::data_local_dir()
        .map(|d| d.join(crate::paths::APP_DIR_NAME))
        .or_else(|| {
            dirs::home_dir().map(|h| {
                h.join(".local")
                    .join("share")
                    .join(crate::paths::APP_DIR_NAME)
            })
        })
        .unwrap_or_else(|| {
            PathBuf::from("/tmp")
                .join(crate::paths::APP_DIR_NAME)
                .join("data")
        })
}

pub(crate) fn detect_portable_mode() -> Option<PathBuf> {
    // Avoid treating dev builds (executed from `target/`) as portable.
    // Dev mode uses workspace-local settings/test assets and should remain stable.
    if crate::paths::core::is_dev_mode() {
        return None;
    }

    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    detect_portable_mode_from_exe_dir(exe_dir)
}

fn is_directory_writable(dir: &Path) -> bool {
    use std::fs;
    use std::io::Write;

    if !dir.exists() {
        return false;
    }

    // Try to create a small test file.
    // This is a best-effort check; failures simply mean "not writable".
    let test_file = dir.join(".marco_write_test");
    let result = fs::File::create(&test_file).and_then(|mut f| {
        f.write_all(b"test")?;
        f.sync_all()?;
        fs::remove_file(&test_file)
    });

    result.is_ok()
}

fn detect_portable_mode_from_exe_dir(exe_dir: &Path) -> Option<PathBuf> {
    // Prefer the explicit portable layout (mirrors the Windows portable build):
    //   <exe_dir>/config/
    //   <exe_dir>/data/
    let portable_config = exe_dir.join("config");
    if is_directory_writable(&portable_config) {
        log::debug!(
            "Portable mode detected: config directory is writable at {}",
            portable_config.display()
        );
        return Some(exe_dir.to_path_buf());
    }

    // Fallback: if the executable directory itself is writable, we can keep
    // configuration and data next to the binary.
    if is_directory_writable(exe_dir) {
        log::debug!(
            "Portable mode detected: exe directory is writable at {}",
            exe_dir.display()
        );
        return Some(exe_dir.to_path_buf());
    }

    None
}

pub(crate) fn detect_locale_from_platform() -> Option<String> {
    // Environment variables are the canonical source on Linux.
    // (Other sources like /etc/locale.conf vary by distro and are not reliably present.)
    None
}

pub(crate) fn detect_install_location_from_asset_root(asset_root: &Path) -> InstallLocation {
    // Portable mode has priority.
    if let Some(portable_root) = detect_portable_mode() {
        if asset_root.starts_with(&portable_root) {
            return InstallLocation::Portable;
        }
    }

    if let Some(home) = dirs::home_dir() {
        let user_local = home
            .join(".local")
            .join("share")
            .join(crate::paths::APP_DIR_NAME);
        if asset_root.starts_with(&user_local) {
            return InstallLocation::UserLocal;
        }
    }

    if asset_root.starts_with(Path::new("/usr/local/share").join(crate::paths::APP_DIR_NAME)) {
        return InstallLocation::SystemLocal;
    }

    if asset_root.starts_with(Path::new("/usr/share").join(crate::paths::APP_DIR_NAME)) {
        return InstallLocation::SystemGlobal;
    }

    InstallLocation::UserLocal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_asset_root_candidates_cover_custom_prefix() {
        // An install under a prefix like /opt matches none of the literal
        // candidates; without the prefix-relative fallback the app cannot find
        // its assets and exits at startup.
        let candidates = asset_root_candidates(Path::new("/opt/marco/bin"));

        assert!(
            candidates
                .contains(&PathBuf::from("/opt/marco/share").join(crate::paths::APP_DIR_NAME)),
            "expected /opt/marco/share/{} among {:?}",
            crate::paths::APP_DIR_NAME,
            candidates
        );
    }

    #[test]
    fn smoke_test_asset_root_candidates_keep_deb_resolution_order() {
        // The .deb installs the executable to /usr/bin, where the
        // prefix-relative fallback resolves to the same /usr/share directory as
        // the system-global literal. It must stay last so a user-local asset
        // tree keeps winning over the system-global one.
        let candidates = asset_root_candidates(Path::new("/usr/bin"));
        let system_global = PathBuf::from("/usr/share").join(crate::paths::APP_DIR_NAME);

        let system_global_idx = candidates
            .iter()
            .position(|p| *p == system_global)
            .expect("system-global candidate should be present");

        if let Some(home) = dirs::home_dir() {
            let user_local = home
                .join(".local")
                .join("share")
                .join(crate::paths::APP_DIR_NAME);
            let user_local_idx = candidates
                .iter()
                .position(|p| *p == user_local)
                .expect("user-local candidate should be present");

            assert!(
                user_local_idx < system_global_idx,
                "user-local must be searched before system-global: {:?}",
                candidates
            );
        }

        assert_eq!(
            candidates.last(),
            Some(&system_global),
            "prefix-relative fallback must be appended last: {:?}",
            candidates
        );
    }

    #[test]
    fn smoke_test_detect_portable_mode_from_exe_dir_prefers_config_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("config")).expect("create config dir");

        let detected = detect_portable_mode_from_exe_dir(dir.path());
        assert_eq!(detected, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn smoke_test_detect_portable_mode_from_exe_dir_falls_back_to_exe_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let detected = detect_portable_mode_from_exe_dir(dir.path());
        assert_eq!(detected, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn smoke_test_is_directory_writable_respects_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");

        // Remove write bits.
        let mut perms = std::fs::metadata(dir.path())
            .expect("metadata")
            .permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(dir.path(), perms).expect("set permissions");

        assert!(!is_directory_writable(dir.path()));
        assert_eq!(detect_portable_mode_from_exe_dir(dir.path()), None);

        // Restore write bits so tempfile cleanup works reliably.
        let mut perms = std::fs::metadata(dir.path())
            .expect("metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dir.path(), perms).expect("restore permissions");
    }
}
