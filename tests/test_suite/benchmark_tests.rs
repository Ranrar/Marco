// Performance benchmark tests for Marco parser

use core::parser;
use core::render;
use std::time::{Duration, Instant};
use super::utils::print_header;

/// Statistics for a single benchmark run
#[derive(Debug, Clone)]
struct BenchmarkStats {
    min: Duration,
    max: Duration,
    mean: Duration,
    median: Duration,
    p95: Duration,
    p99: Duration,
}

impl BenchmarkStats {
    fn from_durations(mut durations: Vec<Duration>) -> Self {
        durations.sort();
        let len = durations.len();
        
        let sum: Duration = durations.iter().sum();
        let mean = sum / len as u32;
        
        let median = durations[len / 2];
        let p95 = durations[(len as f64 * 0.95) as usize];
        let p99 = durations[(len as f64 * 0.99) as usize];
        
        BenchmarkStats {
            min: durations[0],
            max: durations[len - 1],
            mean,
            median,
            p95,
            p99,
        }
    }
    
    fn format_duration(d: Duration) -> String {
        let micros = d.as_micros();
        if micros < 1000 {
            format!("{}μs", micros)
        } else {
            format!("{:.2}ms", micros as f64 / 1000.0)
        }
    }
    
    fn print_report(&self, name: &str) {
        println!("  {}:", name);
        println!("    Min:    {}", Self::format_duration(self.min));
        println!("    Mean:   {}", Self::format_duration(self.mean));
        println!("    Median: {}", Self::format_duration(self.median));
        println!("    P95:    {}", Self::format_duration(self.p95));
        println!("    P99:    {}", Self::format_duration(self.p99));
        println!("    Max:    {}", Self::format_duration(self.max));
    }
}

/// Run a benchmark with multiple iterations
fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkStats
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..5 {
        f();
    }
    
    // Actual benchmark
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        let duration = start.elapsed();
        durations.push(duration);
    }
    
    let stats = BenchmarkStats::from_durations(durations);
    stats.print_report(name);
    stats
}

/// Benchmark a single document with parse, render, and full pipeline tests
fn benchmark_document(content: &str, name: &str, iterations: usize) -> (BenchmarkStats, BenchmarkStats, BenchmarkStats) {
    println!("┌─ {} ({} bytes) ────────────────────", name, content.len());
    
    let parse_stats = benchmark("Parse", iterations, || {
        let _ = parser::parse(content);
    });
    
    let ast = parser::parse(content).unwrap();
    let render_stats = benchmark("Render", iterations, || {
        let _ = render::render(&ast, &render::RenderOptions::default());
    });
    
    let full_stats = benchmark("Parse + Render", iterations, || {
        if let Ok(ast) = parser::parse(content) {
            let _ = render::render(&ast, &render::RenderOptions::default());
        }
    });
    println!();
    
    (parse_stats, render_stats, full_stats)
}

pub fn run_performance_benchmarks(iterations: usize, custom_file: Option<&str>) {
    print_header("Performance Benchmarks");
    
    // If custom file is provided, only benchmark that file
    if let Some(file_path) = custom_file {
        println!("\nBenchmarking custom file: {}", file_path);
        println!("Running {} iterations per benchmark...\n", iterations);
        
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                eprintln!("Current directory: {:?}", std::env::current_dir().unwrap());
                eprintln!("Try using an absolute path or a path relative to the workspace root.");
                return;
            }
        };
        
        let (parse_stats, render_stats, full_stats) = benchmark_document(&content, file_path, iterations);
        
        // Summary for custom file
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║                    Performance Summary                    ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!();
        println!("File: {}", file_path);
        println!("Size: {} bytes", content.len());
        println!();
        println!("Parse:          {}", BenchmarkStats::format_duration(parse_stats.median));
        println!("Render:         {}", BenchmarkStats::format_duration(render_stats.median));
        println!("Parse + Render: {}", BenchmarkStats::format_duration(full_stats.median));
        println!();
        println!("Throughput: {} MB/s", 
            (content.len() as f64 / full_stats.median.as_secs_f64() / 1_000_000.0) as u32
        );
        println!();
        println!("─────────────────────────────────────────────────────────");
        println!("✓ Custom file benchmark complete");
        println!("─────────────────────────────────────────────────────────");
        return;
    }
    
    // Default: benchmark built-in test documents
    println!("\nRunning {} iterations per benchmark...\n", iterations);
    
    // Test documents
    let small_doc = "# Hello World\n\nThis is a **simple** document with *emphasis* and a [link](url).";
    let medium_doc = include_str!("benchmark/medium_document.md");
    let large_doc = include_str!("benchmark/large_document.md");
    
    let (small_parse, small_render, small_full) = benchmark_document(small_doc, "Small Document", iterations);
    let (medium_parse, medium_render, medium_full) = benchmark_document(medium_doc, "Medium Document", iterations);
    let (large_parse, large_render, large_full) = benchmark_document(large_doc, "Large Document", iterations);
    
    // Summary
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    Performance Summary                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Throughput (based on median times):");
    println!("  Small  ({:>6} bytes): {:>8} MB/s", 
        small_doc.len(), 
        (small_doc.len() as f64 / small_full.median.as_secs_f64() / 1_000_000.0) as u32
    );
    println!("  Medium ({:>6} bytes): {:>8} MB/s", 
        medium_doc.len(), 
        (medium_doc.len() as f64 / medium_full.median.as_secs_f64() / 1_000_000.0) as u32
    );
    println!("  Large  ({:>6} bytes): {:>8} MB/s", 
        large_doc.len(), 
        (large_doc.len() as f64 / large_full.median.as_secs_f64() / 1_000_000.0) as u32
    );
    println!();
    
    println!("Parse Performance:");
    println!("  Small:  {}/iteration", BenchmarkStats::format_duration(small_parse.median));
    println!("  Medium: {}/iteration", BenchmarkStats::format_duration(medium_parse.median));
    println!("  Large:  {}/iteration", BenchmarkStats::format_duration(large_parse.median));
    println!();
    
    println!("Render Performance:");
    println!("  Small:  {}/iteration", BenchmarkStats::format_duration(small_render.median));
    println!("  Medium: {}/iteration", BenchmarkStats::format_duration(medium_render.median));
    println!("  Large:  {}/iteration", BenchmarkStats::format_duration(large_render.median));
    println!();
    
    println!("─────────────────────────────────────────────────────────");
    println!("✓ Performance benchmarks complete");
    println!("─────────────────────────────────────────────────────────");
}
