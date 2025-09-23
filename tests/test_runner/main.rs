//! Marco Test Runner - Command Line Interface
//!
//! This is the main entry point for the Marco test runner application.
//! It provides a comprehensive testing framework for the Marco markdown engine.

mod spec;
mod runner;
mod diff;
mod interactive;
mod cli;

use anyhow::Result;

fn main() -> Result<()> {
    // Initialize colored output based on terminal capabilities
    if atty::is(atty::Stream::Stdout) {
        colored::control::set_override(true);
    } else {
        colored::control::set_override(false);
    }
    
    cli::main()
}