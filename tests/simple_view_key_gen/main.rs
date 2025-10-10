//! Simple View Key Generator
//! 
//! Generates a temporary key for Polo's simple view mode.
//! This tool will later be integrated into Marco for launching Polo.

use marco_core::logic::simple_view_key;

fn main() {
    let key = simple_view_key::generate_simple_view_key();
    println!("=== Simple View Key Generator ===");
    println!("Generated Key: {}", key);
    println!("\nUsage:");
    println!("  polo --simple-view {} <file.md>", key);
    println!("\nNote: This key is valid for 5 minutes.");
}
