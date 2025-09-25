#!/usr/bin/env cargo
//! ```cargo
//! [dependencies]
//! marco = { path = "../" }
//! tempfile = "3.0"
//! ```

use marco::logic::buffer::DocumentBuffer;
use std::time::Instant;
use std::fs;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Marco DocumentBuffer Optimization Demo");
    println!("=========================================");
    
    // Create a temporary directory and file for testing
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test_document.md");
    let test_content = "# Large Document\n\nThis is a test document with multiple paragraphs.\n\n".repeat(1000);
    
    // Write test content to file
    fs::write(&test_file, &test_content)?;
    println!("Test content size: {} bytes", test_content.len());
    
    // Test the traditional way (separate operations)
    println!("\n📊 Testing traditional file operations:");
    let mut buffer = DocumentBuffer::new_from_file(&test_file)?;
    let start = Instant::now();
    let content1 = buffer.read_content()?;
    buffer.set_baseline(&content1);
    let traditional_time = start.elapsed();
    println!("  Traditional read_content + set_baseline: {:?}", traditional_time);
    
    // Get stats before optimization
    let stats_before = buffer.get_document_stats();
    println!("  Stats: baseline_size: {}, modified: {}", 
             stats_before.baseline_size, 
             stats_before.is_modified);
    
    // Test the optimized way
    println!("\n⚡ Testing optimized file operations:");
    let mut buffer2 = DocumentBuffer::new_from_file(&test_file)?;
    let start = Instant::now();
    let content2 = buffer2.load_and_set_baseline()?;
    let optimized_time = start.elapsed();
    println!("  Optimized load_and_set_baseline: {:?}", optimized_time);
    
    // Get stats after optimization
    let stats_after = buffer2.get_document_stats();
    println!("  Stats: baseline_size: {}, modified: {}", 
             stats_after.baseline_size, 
             stats_after.is_modified);
    
    // Verify content is the same
    assert_eq!(content1, content2);
    println!("  ✅ Content matches between methods");
    
    // Test batch update
    println!("\n🔄 Testing batch update operations:");
    let modified_content = format!("{}\n\n## Added Section\nThis is new content!", test_content);
    let start = Instant::now();
    buffer2.update_baseline_and_state(&modified_content, true);
    let batch_time = start.elapsed();
    println!("  Batch update with baseline: {:?}", batch_time);
    
    let final_stats = buffer2.get_document_stats();
    println!("  Final stats: baseline_size: {}, modified: {}", 
             final_stats.baseline_size, 
             final_stats.is_modified);
    
    // Test save with optimized method
    println!("\n💾 Testing optimized save operations:");
    let start = Instant::now();
    buffer2.save_content(&modified_content)?;
    let save_time = start.elapsed();
    println!("  Optimized save_content: {:?}", save_time);
    
    // Log document state
    println!("\n📝 Document state logging:");
    buffer2.log_document_state("demo_optimization");
    
    // Performance comparison
    println!("\n📈 Performance Summary:");
    println!("  Traditional approach: {:?}", traditional_time);
    println!("  Optimized approach:   {:?}", optimized_time);
    if optimized_time < traditional_time {
        let improvement = traditional_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
        println!("  🚀 Optimized is {:.1}x faster!", improvement);
    }
    
    println!("\n✅ Optimization demo complete!");
    println!("The new methods provide:");
    println!("  • Smart allocation detection");
    println!("  • Batch operations to reduce clones");
    println!("  • Enhanced logging and statistics");
    println!("  • Memory-conscious string handling");
    println!("  • Integrated file operations");
    
    Ok(())
}