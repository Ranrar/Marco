//! Marco Test Runner - Command Line Interface
//!
//! This is the main entry point for the Marco test runner application.
//! It provides a comprehensive testing framework for the Marco markdown engine.

#[cfg(feature = "integration-tests")]
mod cli;
#[cfg(feature = "integration-tests")]
mod diff;
#[cfg(feature = "integration-tests")]
mod interactive;
#[cfg(feature = "integration-tests")]
mod runner;
#[cfg(feature = "integration-tests")]
mod spec;

use anyhow::Result;

#[cfg(feature = "integration-tests")]
fn main() -> Result<()> {
    // Initialize colored output based on terminal capabilities
    if atty::is(atty::Stream::Stdout) {
        colored::control::set_override(true);
    } else {
        colored::control::set_override(false);
    }

    // Initialize settings manager for shared settings
    let _settings_manager = match marco_core::logic::paths::get_settings_path() {
        Ok(settings_path) => {
            match marco_core::logic::swanson::SettingsManager::initialize(settings_path) {
                Ok(manager) => {
                    eprintln!("Settings initialized for test runner");
                    Some(manager)
                },
                Err(e) => {
                    eprintln!("Warning: Failed to initialize settings: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            eprintln!("Warning: Failed to get settings path: {}", e);
            None
        }
    };

    cli::main()
}

#[cfg(not(feature = "integration-tests"))]
fn main() -> Result<()> {
    eprintln!("This binary requires the 'integration-tests' feature to be enabled.");
    eprintln!("Run with: cargo run --bin marco-test --features integration-tests");
    std::process::exit(1);
}
