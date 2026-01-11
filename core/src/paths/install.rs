//! Installation mode path helpers
//!
//! This module provides utilities for detecting and working with installed binaries:
//! - User local installation (~/.local/share/marco/)
//! - System local installation (/usr/local/share/marco/)
//! - System global installation (/usr/share/marco/)

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
}

/// Get user local install directory (~/.local/share/marco/)
pub fn user_install_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".local/share/marco"))
        .unwrap_or_else(|| PathBuf::from("/tmp/marco"))
}

/// Get system local install directory (/usr/local/share/marco/)
pub fn system_local_install_dir() -> PathBuf {
    PathBuf::from("/usr/local/share/marco")
}

/// Get system global install directory (/usr/share/marco/)
pub fn system_global_install_dir() -> PathBuf {
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

        if asset_str.contains(".local/share/marco") {
            return InstallLocation::UserLocal;
        } else if asset_str.contains("/usr/local/share/marco") {
            return InstallLocation::SystemLocal;
        } else if asset_str.contains("/usr/share/marco") {
            return InstallLocation::SystemGlobal;
        }
    }

    // Default to user local
    InstallLocation::UserLocal
}

/// Get the config directory for the current install location
///
/// - User local: ~/.config/marco/
/// - System: /etc/marco/
pub fn config_dir() -> PathBuf {
    match detect_install_location() {
        InstallLocation::UserLocal | InstallLocation::Development => dirs::config_dir()
            .map(|c| c.join("marco"))
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".config/marco"))
                    .unwrap_or_else(|| PathBuf::from("/tmp/marco/config"))
            }),
        InstallLocation::SystemLocal | InstallLocation::SystemGlobal => PathBuf::from("/etc/marco"),
    }
}

/// Get the user data directory (for storing user-specific data like recent files)
///
/// - User: ~/.local/share/marco/
/// - System: Falls back to /tmp/marco/
pub fn user_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|d| d.join("marco"))
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share/marco")))
        .unwrap_or_else(|| PathBuf::from("/tmp/marco/data"))
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
