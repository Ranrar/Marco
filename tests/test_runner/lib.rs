//! Marco Test Runner Library
//!
//! This library provides testing functionality for the Marco markdown engine.
//! It can be used both as a standalone CLI tool and integrated into other test suites.

pub mod spec;
pub mod runner;
pub mod diff;
pub mod interactive;
pub mod cli;

// Re-export main types for external use
pub use spec::{TestCase, TestSpec, TestResult, TestSummary};
pub use runner::{TestRunner, RunnerConfig};
pub use diff::{DiffConfig, create_unified_diff, create_compact_diff, create_side_by_side_diff};