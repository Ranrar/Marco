//! Core path detection logic for Marco/Polo
//!
//! This module provides the fundamental detection logic that other path modules build upon:
//! - Binary name detection (marco vs polo)
//! - Mode detection (development vs installed)
//! - Asset root finding (where to look for assets)

use std::env;
use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;

use super::platform;

/// Error type for asset path operations
#[derive(Debug, Clone)]
pub enum AssetError {
    /// Failed to get current executable path
    ExePathError(String),
    /// Executable has no parent directory
    ParentMissing,
    /// Asset directory not found at expected locations
    AssetDirMissing(Vec<PathBuf>),
    /// Invalid path conversion
    PathConversionError(String),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::ExePathError(e) => write!(f, "Failed to get executable path: {}", e),
            AssetError::ParentMissing => write!(f, "Executable has no parent directory"),
            AssetError::AssetDirMissing(paths) => {
                writeln!(f, "Asset directory not found. Searched:")?;
                for path in paths {
                    writeln!(f, "  - {}", path.display())?;
                }
                Ok(())
            }
            AssetError::PathConversionError(e) => write!(f, "Path conversion error: {}", e),
        }
    }
}

impl std::error::Error for AssetError {}

/// Get the name of the currently running binary (e.g., "marco", "polo")
///
/// This is cached on first call for performance.
pub fn get_binary_name() -> &'static str {
    static BINARY_NAME: OnceLock<String> = OnceLock::new();
    BINARY_NAME.get_or_init(|| {
        env::current_exe()
            .ok()
            .and_then(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    })
}

/// Detect if running in development mode (from target/ directory)
///
/// Returns true if the binary is running from a `target/` directory,
/// which indicates it's being run via `cargo run` or directly from the build output.
pub fn is_dev_mode() -> bool {
    static IS_DEV: OnceLock<bool> = OnceLock::new();
    *IS_DEV.get_or_init(|| {
        if let Ok(exe) = env::current_exe() {
            // Convert to string and check if path contains "target/"
            let path_str = exe.to_string_lossy();
            // Check for both Unix and Windows path separators
            path_str.contains("target/") || path_str.contains("target\\")
        } else {
            false
        }
    })
}

/// Find the asset root directory, checking multiple locations in order
///
/// Search order:
/// 1. Development mode: `target/{debug|release}/marco_assets/`
///    1b. Windows production: `executable_dir/assets/` (installer layout)
/// 2. User install: `~/.local/share/marco/` (Linux) or `%LOCALAPPDATA%\Marco` (Windows)
/// 3. System local: `/usr/local/share/marco/` (Linux) or `%PROGRAMFILES%\Marco` (Windows)
/// 4. System global: `/usr/share/marco/` (Linux) or `%PROGRAMDATA%\Marco` (Windows)
///
/// Returns the first existing directory found.
pub fn find_asset_root() -> Result<PathBuf, AssetError> {
    static ASSET_ROOT: OnceLock<Result<PathBuf, AssetError>> = OnceLock::new();

    ASSET_ROOT
        .get_or_init(|| {
            let exe_path =
                env::current_exe().map_err(|e| AssetError::ExePathError(e.to_string()))?;
            let parent = exe_path.parent().ok_or(AssetError::ParentMissing)?;

            let mut candidate_paths = Vec::new();

            // 1. Development mode: next to binary (target/{debug|release}/marco_assets)
            let dev_path = parent.join("marco_assets");
            candidate_paths.push(dev_path.clone());

            // 2..n. Platform-specific asset locations.
            candidate_paths.extend(platform::asset_root_candidates(parent));

            // Find first existing directory that actually contains an asset bundle.
            for path in &candidate_paths {
                if path.exists() && path.is_dir() {
                    if platform::is_valid_asset_root(path) {
                        log::debug!("Found asset root: {}", path.display());
                        return Ok(path.clone());
                    }

                    log::debug!(
                        "Asset root candidate exists but does not look like an asset bundle: {}",
                        path.display()
                    );
                }
            }

            // None found
            Err(AssetError::AssetDirMissing(candidate_paths))
        })
        .clone()
}

/// Find the project workspace root by searching for workspace Cargo.toml
///
/// This only works in development mode. Returns None if not in a workspace.
pub fn find_workspace_root() -> Option<PathBuf> {
    if !is_dev_mode() {
        return None;
    }

    let exe_path = env::current_exe().ok()?;
    let mut current = exe_path.parent()?;

    // Search upward for workspace Cargo.toml (contains [workspace])
    while let Some(parent) = current.parent() {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Read the file to check if it's a workspace
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    log::debug!("Found workspace root: {}", current.display());
                    return Some(current.to_path_buf());
                }
            }
        }
        current = parent;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_binary_name() {
        let name = get_binary_name();
        // Should not panic and should return something
        assert!(!name.is_empty());
        println!("Binary name: {}", name);
    }

    #[test]
    fn test_is_dev_mode() {
        // This test itself runs in dev mode
        assert!(is_dev_mode(), "Tests should run in dev mode");
    }

    #[test]
    fn test_find_asset_root() {
        let result = find_asset_root();
        match result {
            Ok(path) => {
                println!("Asset root found: {}", path.display());
                assert!(path.exists(), "Asset root should exist");
            }
            Err(e) => {
                println!("Asset root not found (may be expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_find_workspace_root() {
        if is_dev_mode() {
            let workspace = find_workspace_root();
            if let Some(root) = workspace {
                println!("Workspace root: {}", root.display());
                assert!(root.join("Cargo.toml").exists());
            }
        }
    }
}
