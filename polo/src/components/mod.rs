// Components module - Re-exports all Polo components
//
//! # Polo Components Module
//!
//! This module organizes all of Polo's UI and functional components:
//!
//! - **css**: Styling management - loads Marco's menu.css and Polo-specific styles
//! - **dialog**: File picker and Marco editor integration dialogs
//! - **menu**: Custom titlebar with theme dropdown, window controls, and action buttons
//! - **utils**: Utility functions for color parsing, theme management, and GTK settings
//! - **viewer**: Markdown rendering orchestration - loads files and renders HTML to WebView
//!
//! ## Component Organization
//!
//! Each component is self-contained and follows single responsibility principle:
//!
//! ```text
//! components/
//! ├── css/              # Styling (menu.css + polo.css)
//! │   ├── mod.rs
//! │   ├── polo_styles.rs
//! │   └── theme.rs
//! ├── dialog.rs         # File picker, "Open in Marco" dialogs
//! ├── menu.rs           # Custom titlebar and controls
//! ├── utils.rs          # Helper functions
//! └── viewer/           # Markdown rendering
//!     ├── mod.rs
//!     ├── empty_state.rs
//!     └── rendering.rs
//! ```

pub mod css;
pub mod dialog;
pub mod menu;
pub mod utils;
pub mod viewer;
