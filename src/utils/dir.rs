//! Cross-platform resource path resolver for Marco
//!
//! Resolves asset, theme, and language paths using the MARCO_DATA_DIR environment variable if set,
//! otherwise falls back to platform-appropriate user data directories.

use std::env;
use std::path::{PathBuf};

/// Resolves a resource path for Marco, checking MARCO_DATA_DIR first, then falling back to platform default.
/// Example: resolve_resource_path("assets", "syntect.css")
pub fn resolve_resource_path(subdir: &str, filename: &str) -> PathBuf {
    // 1. Check MARCO_DATA_DIR
    if let Ok(base) = env::var("MARCO_DATA_DIR") {
        let mut path = PathBuf::from(base);
        path.push(subdir);
        path.push(filename);
        return path;
    }

    // 2. Platform fallback
    let base = if cfg!(target_os = "windows") {
        // Use %APPDATA%\marco
        env::var("APPDATA").map(|appdata| {
            let mut p = PathBuf::from(appdata);
            p.push("marco");
            p
        }).unwrap_or_else(|_| {
            // Fallback: current dir
            PathBuf::from(".")
        })
    } else {
        // Use ~/.local/share/marco
        env::var("HOME").map(|home| {
            let mut p = PathBuf::from(home);
            p.push(".local");
            p.push("share");
            p.push("marco");
            p
        }).unwrap_or_else(|_| {
            // Fallback: current dir
            PathBuf::from(".")
        })
    };
    let mut path = base;
    path.push(subdir);
    path.push(filename);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_var() {
        env::set_var("MARCO_DATA_DIR", "/tmp/marco_test");
        let p = resolve_resource_path("assets", "foo.txt");
        assert!(p.starts_with("/tmp/marco_test/assets"));
        env::remove_var("MARCO_DATA_DIR");
    }

    #[test]
    fn test_linux_fallback() {
        if cfg!(target_os = "windows") {
            return;
        }
        env::remove_var("MARCO_DATA_DIR");
        let home = env::var("HOME").unwrap_or_else(|_| String::from("/home/testuser"));
        let p = resolve_resource_path("themes", "bar.css");
        assert!(p.starts_with(format!("{}/.local/share/marco/themes", home)) || p.starts_with("./themes"));
    }

    #[test]
    fn test_windows_fallback() {
        if !cfg!(target_os = "windows") {
            return;
        }
        env::remove_var("MARCO_DATA_DIR");
        let appdata = env::var("APPDATA").unwrap_or_else(|_| String::from("C:/Users/testuser/AppData/Roaming"));
        let p = resolve_resource_path("language", "baz.yml");
        assert!(p.starts_with(format!("{}/marco/language", appdata)) || p.starts_with("./language"));
    }
}
