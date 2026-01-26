//! Editor component modules
//!
//! This module contains the editor UI components and supporting functionality:
//!
//! - `debounce` - Trailing-edge debouncing for GTK signal handlers
//! - `display_config` - Font configuration and display settings
//! - `editor` - Main editor UI construction with preview integration
//! - `editor_manager` - Editor state management and lifecycle coordination
//! - `footer` - Footer status bar updates and statistics
//! - `lsp_integration` - LSP syntax highlighting integration
//! - `scroll_sync` - Scroll synchronization between editor and preview
//! - `sourceview` - SourceView5 rendering and configuration
//! - `utilities` - Async extension processing (line wrapping, tab conversion, etc.)

pub mod debounce;
pub mod editor_manager;
pub mod editor;
pub mod display_config;
pub mod footer;
pub mod lsp_integration;
pub mod utilities;
pub mod sourceview;
pub mod scroll_sync;
