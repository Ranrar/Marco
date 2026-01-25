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

/// Get user local install directory
/// - Linux: ~/.local/share/marco/
/// - Windows: %LOCALAPPDATA%\Marco
pub fn user_install_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .map(|h| h.join(".local/share/marco"))
            .unwrap_or_else(|| PathBuf::from("/tmp/marco"))
    }
    #[cfg(windows)]
    {
        dirs::data_local_dir()
            .map(|d| d.join("Marco"))
            .unwrap_or_else(|| PathBuf::from("C:\\Temp\\Marco"))
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        PathBuf::from("/tmp/marco")
    }
}

/// Get system local install directory
/// - Linux: /usr/local/share/marco/
/// - Windows: %PROGRAMFILES%\Marco
pub fn system_local_install_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/usr/local/share/marco")
    }
    #[cfg(windows)]
    {
        std::env::var("PROGRAMFILES")
            .map(|p| PathBuf::from(p).join("Marco"))
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files\\Marco"))
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        PathBuf::from("/usr/local/share/marco")
    }
}

/// Get system global install directory
/// - Linux: /usr/share/marco/
/// - Windows: %PROGRAMDATA%\Marco
pub fn system_global_install_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/usr/share/marco")
    }
    #[cfg(windows)]
    {
        std::env::var("PROGRAMDATA")
            .map(|p| PathBuf::from(p).join("Marco"))
            .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData\\Marco"))
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        PathBuf::from("/usr/share/marco")
    }
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
/// System-wide defaults can live under /usr/share/marco/, but persisted user changes
/// should go under XDG config.
///
/// Default: $XDG_CONFIG_HOME/marco/ (usually ~/.config/marco/)
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|c| c.join("marco"))
        .or_else(|| {
            #[cfg(target_os = "linux")]
            {
                dirs::home_dir().map(|h| h.join(".config/marco"))
            }
            #[cfg(windows)]
            {
                dirs::data_local_dir().map(|d| d.join("Marco\\config"))
            }
            #[cfg(not(any(target_os = "linux", windows)))]
            {
                dirs::home_dir().map(|h| h.join(".config/marco"))
            }
        })
        .unwrap_or_else(|| {
            #[cfg(windows)]
            {
                PathBuf::from("C:\\Temp\\marco\\config")
            }
            #[cfg(not(windows))]
            {
                PathBuf::from("/tmp/marco/config")
            }
        })
}

/// Get the user data directory (for storing user-specific data like recent files)
///
/// - User: ~/.local/share/marco/
/// - System: Falls back to /tmp/marco/
pub fn user_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|d| d.join("marco"))
        .or_else(|| {
            #[cfg(target_os = "linux")]
            {
                dirs::home_dir().map(|h| h.join(".local/share/marco"))
            }
            #[cfg(windows)]
            {
                dirs::home_dir().map(|h| h.join("AppData\\Local\\Marco"))
            }
            #[cfg(not(any(target_os = "linux", windows)))]
            {
                dirs::home_dir().map(|h| h.join(".local/share/marco"))
            }
        })
        .unwrap_or_else(|| {
            #[cfg(windows)]
            {
                PathBuf::from("C:\\Temp\\marco\\data")
            }
            #[cfg(not(windows))]
            {
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
