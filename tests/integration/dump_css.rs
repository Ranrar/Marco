// Temporary test to dump generated CSS and find the error
// Run with: cargo test --test dump_css -- --nocapture

use marco::ui::css::generate_marco_css;

#[test]
fn dump_generated_css() {
    let css = generate_marco_css();
    
    let lines: Vec<&str> = css.lines().collect();
    
    println!("\n=== CSS Generation Report ===");
    println!("Total lines: {}", lines.len());
    println!("Total bytes: {}", css.len());
    
    if lines.len() >= 429 {
        println!("\n=== Around Line 429 (GTK Error Location) ===");
        for i in 425..=432 {
            if i < lines.len() {
                println!("Line {}: {}", i + 1, lines[i]);
            }
        }
        
        // Show columns 33-39 of line 429
        let line_429 = lines[428];
        println!("\n=== Line 429 Details ===");
        println!("Full line: {}", line_429);
        println!("Length: {}", line_429.len());
        if line_429.len() >= 39 {
            println!("Columns 33-39: '{}'", &line_429[32..39]);
            println!("Columns 30-45: '{}'", &line_429[29..std::cmp::min(45, line_429.len())]);
        }
    }
    
    // Look for :empty pseudo-class
    let empty_count = css.matches(":empty").count();
    println!("\n=== Pseudo-class Usage ===");
    println!("Uses of :empty: {}", empty_count);
    
    // Find all lines with :empty
    println!("\n=== Lines with :empty ===");
    for (i, line) in lines.iter().enumerate() {
        if line.contains(":empty") {
            println!("Line {}: {}", i + 1, line);
        }
    }
}
