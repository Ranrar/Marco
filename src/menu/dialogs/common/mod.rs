// Common dialog utilities and components
// This module contains shared functionality used across different dialog types

pub mod builders;
pub mod validation;
pub mod preview;

// Re-export commonly used imports for dialog modules
pub use gtk4::prelude::*;
pub use gtk4::{
    Dialog, Grid, Entry, ResponseType, Orientation, Label, SpinButton, Adjustment,
    ScrolledWindow, TextView, Button
};

// Re-export builder functions
pub use builders::*;
