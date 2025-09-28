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

    cli::main()
}

#[cfg(not(feature = "integration-tests"))]
fn main() -> Result<()> {
    eprintln!("This binary requires the 'integration-tests' feature to be enabled.");
    eprintln!("Run with: cargo run --bin marco-test --features integration-tests");
    std::process::exit(1);
}
