// Debug test to investigate LSP highlighting positions
use core::parser::parse;
use core::lsp::compute_highlights;

pub fn debug_lsp_positions() {
    // Simple test case from user's report
    let markdown = "# Test Document for LSP Syntax Highlighting\n\nThis document tests various markdown elements.";
    
    println!("\n=== Debug LSP Highlighting Positions ===\n");
    println!("Input markdown (byte positions shown):");
    for (i, ch) in markdown.chars().enumerate() {
        if ch == '\n' {
            println!("  byte {}: \\n", markdown[..i+1].len() - 1);
        }
    }
    println!("{}\n", markdown);
    
    // Show line breakdown
    println!("Lines:");
    for (line_num, line) in markdown.lines().enumerate() {
        println!("  Line {}: {:?} ({} bytes, {} chars)", 
            line_num + 1, line, line.len(), line.chars().count());
    }
    println!();
    
    // Parse the document
    match parse(markdown) {
        Ok(document) => {
            println!("✓ Parsed successfully\n");
            
            // Compute highlights
            let highlights = compute_highlights(&document);
            println!("=== Generated {} highlights ===\n", highlights.len());
            
            for (i, highlight) in highlights.iter().enumerate() {
                let start = &highlight.span.start;
                let end = &highlight.span.end;
                
                // Extract the text for this span
                let lines: Vec<&str> = markdown.lines().collect();
                let text = if start.line == end.line {
                    // Single line - column is 1-based byte offset
                    if let Some(line) = lines.get((start.line - 1) as usize) {
                        let start_byte = (start.column - 1) as usize;
                        let end_byte = (end.column - 1) as usize;
                        if start_byte < line.len() && end_byte <= line.len() {
                            &line[start_byte..end_byte]
                        } else {
                            "<out of bounds>"
                        }
                    } else {
                        "<line out of bounds>"
                    }
                } else {
                    "<multi-line>"
                };
                
                println!("{}. {:?}", i + 1, highlight.tag);
                println!("   Position: [{}:{} to {}:{}]", 
                    start.line, start.column, end.line, end.column);
                println!("   Text: {:?}", text);
                
                // Calculate what GTK would receive
                let gtk_start_line = start.line - 1;  // 1-based -> 0-based
                let gtk_end_line = end.line - 1;
                
                // For single-line, calculate char offsets
                if start.line == end.line {
                    if let Some(line) = lines.get(gtk_start_line as usize) {
                        let start_byte_idx = (start.column - 1) as usize;
                        let end_byte_idx = (end.column - 1) as usize;
                        
                        let start_char_offset: usize = line.char_indices()
                            .take_while(|(byte_idx, _)| *byte_idx < start_byte_idx)
                            .count();
                        
                        let end_char_offset: usize = line.char_indices()
                            .take_while(|(byte_idx, _)| *byte_idx < end_byte_idx)
                            .count();
                        
                        println!("   GTK: line {} chars {}-{}", 
                            gtk_start_line, start_char_offset, end_char_offset);
                    }
                }
                println!();
            }
            
            // Check for overlapping highlights
            println!("\n=== Checking for overlaps ===");
            for i in 0..highlights.len() {
                for j in (i+1)..highlights.len() {
                    let h1 = &highlights[i];
                    let h2 = &highlights[j];
                    
                    // Check if same line and overlapping columns
                    if h1.span.start.line == h2.span.start.line 
                        && h1.span.start.line == h1.span.end.line
                        && h2.span.start.line == h2.span.end.line {
                        
                        let h1_start = h1.span.start.column;
                        let h1_end = h1.span.end.column;
                        let h2_start = h2.span.start.column;
                        let h2_end = h2.span.end.column;
                        
                        // Check overlap
                        if (h1_start < h2_end && h1_end > h2_start) {
                            println!("⚠️  OVERLAP DETECTED:");
                            println!("    {:?} [{}:{} to {}:{}]", 
                                h1.tag, h1.span.start.line, h1_start, h1.span.end.line, h1_end);
                            println!("    {:?} [{}:{} to {}:{}]",
                                h2.tag, h2.span.start.line, h2_start, h2.span.end.line, h2_end);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Parse failed: {}", e);
            panic!("Parse should succeed");
        }
    }
}

pub fn debug_simple_heading() {
    let markdown = "# Test";
    
    println!("\n=== Debug Simple Heading ===");
    println!("Input: {:?}", markdown);
    println!("Bytes: {:?}", markdown.as_bytes());
    println!("Chars: {:?}", markdown.chars().collect::<Vec<_>>());
    
    match parse(markdown) {
        Ok(document) => {
            let highlights = compute_highlights(&document);
            println!("Highlights: {}", highlights.len());
            
            for h in highlights {
                println!("  {:?}: [{}:{} to {}:{}] = {:?}",
                    h.tag,
                    h.span.start.line, h.span.start.column,
                    h.span.end.line, h.span.end.column,
                    &markdown[(h.span.start.column-1) as usize..(h.span.end.column-1) as usize]
                );
            }
        }
        Err(e) => panic!("Parse failed: {}", e)
    }
}
