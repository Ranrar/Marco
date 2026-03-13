//! Editor component modules
//!
//! This module contains the editor UI components and supporting functionality:
//!
//! - `debounce` - Trailing-edge debouncing for GTK signal handlers
//! - `display_config` - Font configuration and display settings
//! - `ui` - Main editor UI construction with preview integration
//! - `editor_manager` - Editor state management and lifecycle coordination
//! - `footer` - Footer status bar updates and statistics
//! - `intelligence_integration` - Markdown intelligence highlight integration
//! - `scroll_sync` - Scroll synchronization between editor and preview
//! - `sourceview` - SourceView5 rendering and configuration
//! - `utilities` - Async extension processing (line wrapping, tab conversion, etc.)

pub mod contextmenu;
pub mod debounce;
pub mod display_config;
pub mod editor_manager;
pub mod footer;
pub mod hover_provider;
pub mod intelligence;
pub mod scroll_sync;
pub mod sourceview;
pub mod table_edit;
pub mod ui;
pub mod utilities;
