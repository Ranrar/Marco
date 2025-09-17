// Simple debug program to understand parsing
use std::fs;

fn main() {
    // Read the grammar file content
    let grammar_content = fs::read_to_string("src/components/marco_engine/marco_grammar.pest")
        .expect("Failed to read grammar file");

    println!(
        "Grammar file exists and contains {} characters",
        grammar_content.len()
    );
    println!("First few lines:");
    for (i, line) in grammar_content.lines().take(10).enumerate() {
        println!("{}: {}", i + 1, line);
    }
}
