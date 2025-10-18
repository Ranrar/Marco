// Shared utilities for test suite

use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

pub fn print_header(title: &str) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  {:^56}  ║", title);
    println!("╚════════════════════════════════════════════════════════════╝\n");
}

pub fn print_section(title: &str) {
    println!("\n┌─ {} {}", title, "─".repeat(60 - title.len() - 3));
}
