//! Installation mode path helpers
//!
//! This module provides utilities for detecting and working with installed binaries:
//! - User local installation (~/.local/share/marco/)
//! - System local installation (/usr/local/share/marco/)
//! - System global installation (/usr/share/marco/)
//! - Portable mode (Windows): runs from writable directory (USB drive, user folder)

use std::path::PathBuf;

/// Installation location type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallLocation {
    /// User local installation (~/.local/share/marco/)
    UserLocal,
    /// System local installation (/usr/local/share/marco/)
    SystemLocal,
    /// System global installation (/usr/share/marco/)
    SystemGlobal,
    /// Development mode (not installed)
    Development,
    /// Portable mode (Windows): running from writable directory
    Portable,
}

/// Get user local install directory
/// - Linux: ~/.local/share/marco/
/// - Windows: %LOCALAPPDATA%\Marco
pub fn user_install_dir() -> PathBuf {
    if cfg!(target_os = "linux") {
        return dirs::home_dir()
            .map(|h| h.join(".local").join("share").join("marco"))
            .unwrap_or_else(|| PathBuf::from("/tmp/marco"));
    }

    if cfg!(target_os = "windows") {
        return dirs::data_local_dir()
            .map(|d| d.join("Marco"))
            .unwrap_or_else(|| PathBuf::from("C:\\Temp\\Marco"));
    }

    // Fallback to Linux-style user dir
    dirs::home_dir()
        .map(|h| h.join(".local").join("share").join("marco"))
        .unwrap_or_else(|| PathBuf::from("/tmp/marco"))
}

/// Get system local install directory
/// - Linux: /usr/local/share/marco/
/// - Windows: %PROGRAMFILES%\Marco
pub fn system_local_install_dir() -> PathBuf {
    if cfg!(target_os = "linux") {
        return PathBuf::from("/usr/local/share/marco");
    }

    if cfg!(target_os = "windows") {
        return std::env::var("PROGRAMFILES")
            .map(|p| PathBuf::from(p).join("Marco"))
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files\\Marco"));
    }

    // Fallback
    PathBuf::from("/usr/local/share/marco")
}

/// Check if a directory is writable by attempting to create a test file
/// 
/// Returns true if the directory exists and we can write to it
fn is_directory_writable(dir: &std::path::Path) -> bool {
    use std::fs;
    use std::io::Write;
    
    if !dir.exists() {
        // Try to create the directory
        if fs::create_dir_all(dir).is_err() {
            return false;
        }
    }
    
    // Try to create a test file
    let test_file = dir.join(".marco_write_test");
    let result = fs::File::create(&test_file)
        .and_then(|mut f| {
            f.write_all(b"test")?;
            f.sync_all()?;
            fs::remove_file(&test_file)
        });
    
    result.is_ok()
}

/// Detect if running in portable mode (Windows only)
/// 
/// Portable mode: exe is in a user-writable directory (USB drive, user folder, Downloads, etc.)
/// Returns the portable root directory if detected, None otherwise
#[cfg(target_os = "windows")]
pub fn detect_portable_mode() -> Option<PathBuf> {
    use std::env;
    
    // Get exe directory
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    
    // Check if exe directory is writable
    // If we can write to the exe directory, we're in portable mode
    if is_directory_writable(exe_dir) {
        log::debug!("Portable mode detected: exe directory is writable at {}", exe_dir.display());
        return Some(exe_dir.to_path_buf());
    }
    
    log::debug!("Not in portable mode: exe directory is not writable at {}", exe_dir.display());
    None
}

#[cfg(not(target_os = "windows"))]
pub fn detect_portable_mode() -> Option<PathBuf> {
    // Portable mode is Windows-only concept
    None
}

/// Get system global install directory
/// - Linux: /usr/share/marco/
/// - Windows: %PROGRAMDATA%\Marco
pub fn system_global_install_dir() -> PathBuf {
    if cfg!(target_os = "linux") {
        return PathBuf::from("/usr/share/marco");
    }

    if cfg!(target_os = "windows") {
        return std::env::var("PROGRAMDATA")
            .map(|p| PathBuf::from(p).join("Marco"))
            .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData\\Marco"));
    }

    // Fallback
    PathBuf::from("/usr/share/marco")
}

/// Detect the current installation location
///
/// This checks which install directory exists and is being used.
pub fn detect_install_location() -> InstallLocation {
    use super::core::{find_asset_root, is_dev_mode};

    // Check if in development mode first
    if is_dev_mode() {
        return InstallLocation::Development;
    }

    // Try to determine from asset root
    if let Ok(asset_root) = find_asset_root() {
        let asset_str = asset_root.to_string_lossy();

        // Linux paths
        if asset_str.contains(".local/share/marco") || asset_str.contains("\\Marco") && asset_str.contains("Local") {
            return InstallLocation::UserLocal;
        } else if asset_str.contains("/usr/local/share/marco") || asset_str.contains("Program Files\\Marco") {
            return InstallLocation::SystemLocal;
        } else if asset_str.contains("/usr/share/marco") || asset_str.contains("ProgramData\\Marco") {
            return InstallLocation::SystemGlobal;
        }
    }

    // Default to user local
    InstallLocation::UserLocal
}

/// Get the user configuration directory.
///
/// For GUI apps like Marco/Polo, settings must be writable for the *current user*.
/// 
/// **Windows Portable Mode**: If exe directory is writable, uses `{exe_dir}\config\`
/// **Windows Installed Mode**: Uses `%LOCALAPPDATA%\Marco\config\`
/// **Linux**: Uses `~/.config/marco/`
pub fn config_dir() -> PathBuf {
    // Windows: Check for portable mode first
    #[cfg(target_os = "windows")]
    if let Some(portable_root) = detect_portable_mode() {
        log::debug!("Using portable mode config directory");
        return portable_root.join("config");
    }
    
    // Normal installation paths
    dirs::config_dir()
        .map(|c| c.join("marco"))
        .or_else(|| {
            if cfg!(target_os = "linux") {
                dirs::home_dir().map(|h| h.join(".config").join("marco"))
            } else if cfg!(target_os = "windows") {
                dirs::data_local_dir().map(|d| d.join("Marco").join("config"))
            } else {
                dirs::home_dir().map(|h| h.join(".config").join("marco"))
            }
        })
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                PathBuf::from("C:\\Temp\\marco\\config")
            } else {
                PathBuf::from("/tmp/marco/config")
            }
        })
}

/// Get the user data directory (for storing user-specific data like recent files)
///
/// **Windows Portable Mode**: If exe directory is writable, uses `{exe_dir}\data\`
/// **Windows Installed Mode**: Uses `%LOCALAPPDATA%\Marco\data\`
/// **Linux**: Uses `~/.local/share/marco/`
pub fn user_data_dir() -> PathBuf {
    // Windows: Check for portable mode first
    #[cfg(target_os = "windows")]
    if let Some(portable_root) = detect_portable_mode() {
        log::debug!("Using portable mode data directory");
        return portable_root.join("data");
    }
    
    // Normal installation paths
    dirs::data_local_dir()
        .map(|d| d.join("marco"))
        .or_else(|| {
            if cfg!(target_os = "linux") {
                dirs::home_dir().map(|h| h.join(".local").join("share").join("marco"))
            } else if cfg!(target_os = "windows") {
                dirs::home_dir().map(|h| h.join("AppData").join("Local").join("Marco"))
            } else {
                dirs::home_dir().map(|h| h.join(".local").join("share").join("marco"))
            }
        })
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                PathBuf::from("C:\\Temp\\marco\\data")
            } else {
                PathBuf::from("/tmp/marco/data")
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_install_location() {
        let location = detect_install_location();
        println!("Install location: {:?}", location);
        // Should not panic
    }

    #[test]
    fn test_install_dirs() {
        println!("User local: {}", user_install_dir().display());
        println!("System local: {}", system_local_install_dir().display());
        println!("System global: {}", system_global_install_dir().display());
        println!("Config dir: {}", config_dir().display());
        println!("User data dir: {}", user_data_dir().display());
    }
}
