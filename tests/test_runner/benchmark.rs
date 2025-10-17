//! Performance benchmarking utilities for Marco
//!
//! This module provides tools to measure and analyze the performance
//! of the Marco markdown engine.

use colored::*;
use marco_core::parse_and_render;
use std::time::{Duration, Instant};

/// Benchmark result for a single test
pub struct BenchmarkResult {
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

impl BenchmarkResult {
    /// Calculate throughput (iterations per second)
    pub fn throughput(&self) -> f64 {
        self.iterations as f64 / self.total_duration.as_secs_f64()
    }
    
    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "{} iterations in {:?}\n\
             Average: {:?} per iteration\n\
             Min: {:?}, Max: {:?}\n\
             Throughput: {:.2} iterations/sec",
            self.iterations,
            self.total_duration,
            self.avg_duration,
            self.min_duration,
            self.max_duration,
            self.throughput()
        )
    }
}

/// Run performance benchmark on markdown input
pub fn benchmark_markdown(markdown: &str, iterations: usize) -> BenchmarkResult {
    let mut durations = Vec::with_capacity(iterations);
    
    // Warmup
    for _ in 0..5 {
        let _ = parse_and_render(markdown, Default::default());
    }
    
    // Actual benchmark
    let total_start = Instant::now();
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = parse_and_render(markdown, Default::default());
        durations.push(start.elapsed());
    }
    
    let total_duration = total_start.elapsed();
    
    let min_duration = *durations.iter().min().unwrap();
    let max_duration = *durations.iter().max().unwrap();
    let avg_duration = total_duration / iterations as u32;
    
    BenchmarkResult {
        iterations,
        total_duration,
        avg_duration,
        min_duration,
        max_duration,
    }
}

/// Run benchmark suite with various markdown samples
pub fn run_benchmark_suite() -> String {
    let mut output = String::new();
    
    output.push_str(&format!("\n{}\n", "=== Marco Performance Benchmark Suite ===".blue().bold()));
    output.push_str(&format!("{}\n\n", "Running warmup and benchmark iterations...".yellow()));
    
    let test_cases = vec![
        ("Simple text", "Hello, world!", 1000),
        ("Headers", "# H1\n## H2\n### H3", 1000),
        ("Bold and italic", "**bold** and *italic* text", 1000),
        ("Lists", "- Item 1\n- Item 2\n  - Nested\n- Item 3", 500),
        ("Code blocks", "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```", 500),
        ("Links", "[Example](https://example.com) and [Another](https://test.com)", 500),
        ("Complex document", "# Sample Document\n\nThis is a **complex** document with *various* elements.\n\n- Lists\n- Code\n- Links", 100),
    ];
    
    for (name, markdown, iterations) in test_cases {
        output.push_str(&format!("\n{}\n", format!("📊 Benchmark: {}", name).cyan().bold()));
        output.push_str(&format!("   Input size: {} bytes\n", markdown.len()));
        
        let result = benchmark_markdown(markdown, iterations);
        
        output.push_str(&format!("   {}\n", result.format().replace('\n', "\n   ")));
        
        // Performance assessment
        let avg_micros = result.avg_duration.as_micros();
        let assessment = if avg_micros < 100 {
            "⚡ Excellent".green()
        } else if avg_micros < 500 {
            "✓ Good".green()
        } else if avg_micros < 2000 {
            "⚠ Acceptable".yellow()
        } else {
            "❌ Slow".red()
        };
        
        output.push_str(&format!("   Performance: {}\n", assessment));
    }
    
    output.push_str(&format!("\n{}\n", "=== Benchmark Complete ===".green().bold()));
    
    output
}

/// Test parser cache performance
pub fn benchmark_parser_cache(markdown: &str, iterations: usize) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("\n{}\n", "=== Parser Cache Performance Test ===".blue().bold()));
    output.push_str(&format!("📝 Input: {:?}\n", markdown));
    output.push_str(&format!("🔄 Iterations: {}\n\n", iterations));
    
    // First parse (cache miss)
    let start1 = Instant::now();
    for _ in 0..iterations {
        let _ = parse_and_render(markdown, Default::default());
    }
    let duration1 = start1.elapsed();
    
    output.push_str(&format!("First run: {:?} ({:?} per iteration)\n", 
        duration1, duration1 / iterations as u32));
    
    // Second parse (should be faster due to caching)
    let start2 = Instant::now();
    for _ in 0..iterations {
        let _ = parse_and_render(markdown, Default::default());
    }
    let duration2 = start2.elapsed();
    
    output.push_str(&format!("Second run: {:?} ({:?} per iteration)\n\n", 
        duration2, duration2 / iterations as u32));
    
    // Calculate speedup
    if duration2 < duration1 {
        let speedup = duration1.as_secs_f64() / duration2.as_secs_f64();
        output.push_str(&format!("{} {:.2}x speedup from caching\n", 
            "✓".green(), speedup));
    } else {
        output.push_str(&format!("{}\n", 
            "⚠ No significant speedup detected (cache may not be effective)".yellow()));
    }
    
    output
}

/// Quick performance check
pub fn quick_benchmark(markdown: &str) -> String {
    let result = benchmark_markdown(markdown, 100);
    
    format!(
        "\n{}\n{}\n",
        "Quick Performance Check:".blue().bold(),
        result.format()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_benchmark() {
        let result = benchmark_markdown("# Test\n\nParagraph.", 10);
        assert_eq!(result.iterations, 10);
        assert!(result.total_duration.as_micros() > 0);
        assert!(result.throughput() > 0.0);
    }

    #[test]
    fn smoke_test_benchmark_suite() {
        let output = run_benchmark_suite();
        assert!(!output.is_empty());
        assert!(output.contains("Benchmark Suite"));
    }

    #[test]
    fn smoke_test_cache_benchmark() {
        let output = benchmark_parser_cache("**Bold** text", 50);
        assert!(!output.is_empty());
        assert!(output.contains("Cache Performance"));
    }

    #[test]
    fn smoke_test_quick_benchmark() {
        let output = quick_benchmark("*Italic*");
        assert!(!output.is_empty());
        assert!(output.contains("Performance Check"));
    }
}
