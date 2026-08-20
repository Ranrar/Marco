use std::env;
use std::path::{Path, PathBuf};

use crate::paths::InstallLocation;

pub(crate) fn asset_root_candidates(exe_parent: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Portable / dev layout: assets folder next to executable.
    candidates.push(exe_parent.join("assets"));

    // macOS app bundle: <exe_dir>/../Resources/assets
    candidates.push(exe_parent.join("..").join("Resources").join("assets"));

    // System-wide app bundles in /Applications.
    candidates.push(PathBuf::from("/Applications/Marco.app/Contents/Resources/assets"));
    candidates.push(PathBuf::from("/Applications/Polo.app/Contents/Resources/assets"));

    // User-local asset install.
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join("Library/Application Support/Marco"));
    }

    // Homebrew prefix installs (Apple Silicon and Intel).
    candidates.push(PathBuf::from("/opt/homebrew/share/marco"));
    candidates.push(PathBuf::from("/usr/local/share/marco"));

    candidates
}

pub(crate) fn config_dir() -> PathBuf {
    // Portable mode: keep config next to exe.
    if let Some(portable_root) = detect_portable_mode() {
        return portable_root.join("config");
    }

    dirs::config_dir()
        .map(|c| c.join("marco"))
        .or_else(|| dirs::home_dir().map(|h| h.join(".config").join("marco")))
        .unwrap_or_else(|| PathBuf::from("/tmp/marco/config"))
}

pub(crate) fn user_data_dir() -> PathBuf {
    // Portable mode: keep data next to exe.
    if let Some(portable_root) = detect_portable_mode() {
        return portable_root.join("data");
    }

    dirs::data_local_dir()
        .map(|d| d.join("marco"))
        .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share").join("marco")))
        .unwrap_or_else(|| PathBuf::from("/tmp/marco/data"))
}

pub(crate) fn detect_portable_mode() -> Option<PathBuf> {
    // Avoid treating dev builds (executed from `target/`) as portable.
    if crate::paths::core::is_dev_mode() {
        return None;
    }

    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    // Never treat an .app bundle's MacOS directory as a portable root —
    // settings must live in the user's Library, not inside the bundle.
    if exe_dir
        .to_string_lossy()
        .contains(".app/Contents/MacOS")
    {
        return None;
    }

    detect_portable_mode_from_exe_dir(exe_dir)
}

fn is_directory_writable(dir: &Path) -> bool {
    use std::fs;
    use std::io::Write;

    if !dir.exists() {
        return false;
    }

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
    // GUI apps launched from Finder have no LC_*/LANG environment, so ask
    // the system preferences directly. `AppleLocale` looks like `en_US`,
    // `zh-Hans_CN`, etc. — the caller normalizes it to a BCP-47-ish tag.
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        None
    } else {
        Some(raw)
    }
}

pub(crate) fn detect_install_location_from_asset_root(asset_root: &Path) -> InstallLocation {
    // Portable mode has priority.
    if let Some(portable_root) = detect_portable_mode() {
        if asset_root.starts_with(&portable_root) {
            return InstallLocation::Portable;
        }
    }

    if asset_root.starts_with(Path::new("/Applications")) {
        return InstallLocation::SystemGlobal;
    }

    if asset_root.starts_with(Path::new("/opt/homebrew/share/marco"))
        || asset_root.starts_with(Path::new("/usr/local/share/marco"))
    {
        return InstallLocation::SystemLocal;
    }

    InstallLocation::UserLocal
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let mut perms = std::fs::metadata(dir.path())
            .expect("metadata")
            .permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(dir.path(), perms).expect("set permissions");

        assert!(!is_directory_writable(dir.path()));
        assert_eq!(detect_portable_mode_from_exe_dir(dir.path()), None);

        let mut perms = std::fs::metadata(dir.path())
            .expect("metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dir.path(), perms).expect("restore permissions");
    }

    #[test]
    fn smoke_test_detect_locale_from_platform_does_not_panic() {
        let _ = detect_locale_from_platform();
    }
}
