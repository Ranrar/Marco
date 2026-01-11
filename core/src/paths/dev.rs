//! Development mode path helpers
//!
//! This module provides utilities for development mode, including:
//! - Workspace root detection
//! - Test assets access
//! - Source assets access

use super::core::find_workspace_root;
use std::path::PathBuf;

/// Get the workspace root directory
///
/// Only works in development mode. Returns None if not in a workspace or not in dev mode.
pub fn workspace_root() -> Option<PathBuf> {
    find_workspace_root()
}

/// Get the test assets directory (tests/markdown_showcase/)
///
/// Returns None if not in development mode or workspace root not found.
pub fn test_assets_dir() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("tests").join("markdown_showcase"))
}

/// Get the test specs directory (tests/spec/)
///
/// Returns None if not in development mode or workspace root not found.
pub fn test_specs_dir() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("tests").join("spec"))
}

/// Get the source assets directory (workspace assets/)
///
/// This is the original assets directory in the workspace root,
/// not the copied assets in target/*/marco_assets/
///
/// Returns None if not in development mode or workspace root not found.
pub fn source_assets_dir() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("assets"))
}

/// Get the test settings file (tests/settings/settings.ron)
pub fn test_settings_file() -> Option<PathBuf> {
    workspace_root().map(|root| root.join("tests").join("settings").join("settings.ron"))
}

#[cfg(test)]
mod tests {
    use super::super::core::is_dev_mode;
    use super::*;

    #[test]
    fn test_workspace_root() {
        if is_dev_mode() {
            let root = workspace_root();
            assert!(root.is_some(), "Should find workspace root in dev mode");
            if let Some(root) = root {
                println!("Workspace root: {}", root.display());
                assert!(root.join("Cargo.toml").exists());
            }
        }
    }

    #[test]
    fn test_test_assets_dir() {
        if is_dev_mode() {
            if let Some(dir) = test_assets_dir() {
                println!("Test assets dir: {}", dir.display());
                // Should contain markdown test files
            }
        }
    }

    #[test]
    fn test_source_assets_dir() {
        if is_dev_mode() {
            if let Some(dir) = source_assets_dir() {
                println!("Source assets dir: {}", dir.display());
                assert!(dir.ends_with("assets"));
            }
        }
    }
}
