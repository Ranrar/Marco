use std::env;
use std::path::{Path, PathBuf};

use crate::paths::InstallLocation;

pub(crate) fn asset_root_candidates(exe_parent: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Production install: assets folder next to executable.
    // (Installer puts assets in something like: C:\Program Files\Marco-suite\assets\)
    candidates.push(exe_parent.join("assets"));

    // User-local install: %LOCALAPPDATA%\Marco
    if let Some(local_app_data) = dirs::data_local_dir() {
        candidates.push(local_app_data.join("Marco"));
    }

    // System-local: %PROGRAMFILES%\Marco
    if let Ok(program_files) = env::var("PROGRAMFILES") {
        candidates.push(PathBuf::from(program_files).join("Marco"));
    }

    // System-global: %PROGRAMDATA%\Marco
    if let Ok(program_data) = env::var("PROGRAMDATA") {
        candidates.push(PathBuf::from(program_data).join("Marco"));
    }

    candidates
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

pub(crate) fn detect_portable_mode() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    // Prefer the explicit portable layout created by our packaging scripts:
    //   <exe_dir>\config\
    //   <exe_dir>\data\
    // Some environments (for example, tightened ACLs on the exe directory) can
    // make the parent directory non-writable even though the `config/` folder
    // itself is writable. Checking the config directory first makes portable
    // detection more reliable.
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

pub(crate) fn config_dir() -> PathBuf {
    // Portable mode: keep config next to exe.
    if let Some(portable_root) = detect_portable_mode() {
        return portable_root.join("config");
    }

    // Normal installation.
    dirs::config_dir()
        .map(|c| c.join("marco"))
        .or_else(|| dirs::data_local_dir().map(|d| d.join("Marco").join("config")))
        .unwrap_or_else(|| PathBuf::from("C:\\Temp\\marco\\config"))
}

pub(crate) fn user_data_dir() -> PathBuf {
    // Portable mode: keep data next to exe.
    if let Some(portable_root) = detect_portable_mode() {
        return portable_root.join("data");
    }

    dirs::data_local_dir()
        .map(|d| d.join("marco"))
        .or_else(|| dirs::home_dir().map(|h| h.join("AppData").join("Local").join("marco")))
        .unwrap_or_else(|| PathBuf::from("C:\\Temp\\marco\\data"))
}

pub(crate) fn detect_install_location_from_asset_root(asset_root: &Path) -> InstallLocation {
    // Portable mode has priority.
    if let Some(portable_root) = detect_portable_mode() {
        if asset_root.starts_with(&portable_root) {
            return InstallLocation::Portable;
        }
    }

    if let Some(local_app_data) = dirs::data_local_dir() {
        let user_local_assets = local_app_data.join("Marco");
        if asset_root.starts_with(&user_local_assets) {
            return InstallLocation::UserLocal;
        }
    }

    if let Ok(program_files) = env::var("PROGRAMFILES") {
        let system_local = PathBuf::from(program_files).join("Marco");
        if asset_root.starts_with(&system_local) {
            return InstallLocation::SystemLocal;
        }
    }

    if let Ok(program_data) = env::var("PROGRAMDATA") {
        let system_global = PathBuf::from(program_data).join("Marco");
        if asset_root.starts_with(&system_global) {
            return InstallLocation::SystemGlobal;
        }
    }

    InstallLocation::UserLocal
}

pub(crate) fn detect_locale_from_platform() -> Option<String> {
    // LOCALE_NAME_MAX_LENGTH is 85.
    const LOCALE_NAME_MAX_LENGTH: usize = 85;

    // SAFETY: Windows API writes into our provided UTF-16 buffer.
    unsafe {
        let mut buf: [u16; LOCALE_NAME_MAX_LENGTH] = [0; LOCALE_NAME_MAX_LENGTH];
        let len = GetUserDefaultLocaleName(buf.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH as i32);
        if len <= 0 {
            return None;
        }

        // `len` includes the terminating NUL.
        let len = len as usize;
        let slice = if len > 0 && buf[len.saturating_sub(1)] == 0 {
            &buf[..len - 1]
        } else {
            &buf[..len]
        };

        let locale = String::from_utf16_lossy(slice);
        super::normalize_to_iso639_1(&locale)
    }
}

#[link(name = "Kernel32")]
extern "system" {
    fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
}
