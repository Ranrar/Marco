//! Comprehensive path management system for Marco, Polo, and core.
//!
//! This module provides a structured approach to managing asset paths across:
//! - Different binaries (marco vs polo)
//! - Different modes (development vs installed)
//! - Different asset types (fonts, themes, config, etc.)
//!
//! # Architecture
//!
//! - **core.rs**: Binary detection, mode detection, asset root finding
//! - **shared.rs**: Assets shared between marco and polo (fonts, icons, language)
//! - **marco.rs**: Marco-specific paths (editor themes, UI CSS)
//! - **polo.rs**: Polo-specific paths
//! - **dev.rs**: Development mode helpers (test assets, workspace root)
//! - **install.rs**: Installation mode helpers (system paths)
//!
//! # Usage
//!
//! ```rust
//! use core::components::paths::{MarcoPaths, PoloPaths, SharedPaths};
//!
//! // Get paths for the appropriate binary
//! let marco_paths = MarcoPaths::new().expect("Failed to initialize paths");
//! let font_path = marco_paths.shared().font("ui_menu.ttf");
//! let theme_path = marco_paths.editor_theme("dark");
//! ```

pub mod core;
pub mod shared;
pub mod marco;
pub mod polo;
pub mod dev;
pub mod install;

// Re-export main types and functions
pub use core::{AssetError, get_binary_name, is_dev_mode, find_asset_root};
pub use shared::SharedPaths;
pub use marco::MarcoPaths;
pub use polo::PoloPaths;
pub use dev::{workspace_root, test_assets_dir, source_assets_dir};
pub use install::{InstallLocation, detect_install_location};

/// Trait for path providers - allows polymorphic path access
pub trait PathProvider {
    /// Get the shared paths accessor
    fn shared(&self) -> &SharedPaths;
    
    /// Get the asset root directory
    fn asset_root(&self) -> &std::path::PathBuf;
    
    /// Check if running in development mode
    fn is_dev_mode(&self) -> bool;
}
