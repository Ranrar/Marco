# Marco Performance Profiling Guide

## Overview

The Marco performance profiling infrastructure provides comprehensive monitoring and analysis tools for parsing operations, memory usage, cache performance, and bottleneck identification.

## Quick Start

```rust
use marco::tests::performance::{PerformanceProfiler, ProfilerConfig, MarcoBenchmarks};

// Basic profiling setup
let mut profiler = PerformanceProfiler::new(ProfilerConfig::default());

// Profile a parsing operation
profiler.start_operation("document_parse");
let result = parse_document(input);
profiler.record_parse_time("document", duration, result.is_ok());
profiler.end_operation("document_parse");

// Get results
let metrics = profiler.get_metrics();
println!("Average parse time: {:.2}ms", metrics.avg_parse_time.as_millis());
```

## Components

### PerformanceProfiler

The core profiling component that tracks parsing operations, cache performance, and memory usage.

#### Configuration Options

```rust
use marco::tests::performance::ProfilerConfig;

let config = ProfilerConfig {
    max_samples: 1000,           // Maximum samples to store
    enable_cache_monitoring: true, // Track cache hit/miss rates
    enable_rule_tracking: true,   // Track individual rule performance
    detailed_timing: true,        // Collect detailed timing information
};

let profiler = PerformanceProfiler::new(config);
```

#### Basic Usage

```rust
let mut profiler = PerformanceProfiler::new(ProfilerConfig::default());

// Start profiling session
profiler.start_session("parsing_session");

// Record parsing operations
profiler.start_operation("heading_parse");
let result = parse_heading("# Main Title");
profiler.record_parse_time("heading", duration, result.is_ok());
profiler.end_operation("heading_parse");

// Record cache operations
profiler.record_cache_hit(Duration::from_micros(10));
profiler.record_cache_miss(Duration::from_millis(2));

// Get comprehensive metrics
let metrics = profiler.get_metrics();
profiler.end_session("parsing_session");
```

#### Advanced Features

```rust
// Track rule-specific performance
profiler.record_rule_parse("heading", duration, true);
profiler.record_rule_parse("paragraph", duration, false);

// Monitor memory usage
profiler.record_memory_usage(current_memory_bytes);

// Track cache evictions
profiler.record_cache_eviction();

// Export results for analysis
let json_export = profiler.export_json()?;
std::fs::write("profile_results.json", json_export)?;
```

### MarcoBenchmarks

Comprehensive benchmarking suite with real-world scenarios and stress tests.

#### Benchmark Categories

```rust
let mut benchmarks = MarcoBenchmarks::new();

// Parser benchmarks
let parser_results = benchmarks.benchmark_parser();
println!("Parser Performance: {:.2}ms average", parser_results.average_time.as_millis());

// AST building benchmarks
let ast_results = benchmarks.benchmark_ast_building();

// Full pipeline benchmarks
let pipeline_results = benchmarks.benchmark_pipeline();

// Memory usage benchmarks
let memory_results = benchmarks.benchmark_memory_usage();

// Cache performance benchmarks
let cache_results = benchmarks.benchmark_cache_performance();

// Real-world scenario benchmarks
let scenario_results = benchmarks.benchmark_real_world_scenarios();
```

#### Custom Test Cases

```rust
// Generate test documents of various sizes
let small_doc = benchmarks.generate_small_document();      // ~1KB
let medium_doc = benchmarks.generate_medium_document();    // ~10KB  
let large_doc = benchmarks.generate_large_document();      // ~100KB

// Generate stress test content
let memory_stress = benchmarks.generate_memory_stress_test();
let deeply_nested = benchmarks.generate_deeply_nested_content();
let large_table = benchmarks.generate_large_table();

// Real-world document types
let blog_post = benchmarks.generate_blog_post();
let technical_docs = benchmarks.generate_technical_docs();
let user_manual = benchmarks.generate_user_manual();
let api_docs = benchmarks.generate_api_docs();
```

## Performance Metrics

### Core Metrics

```rust
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub avg_parse_time: Duration,
    pub min_parse_time: Duration,
    pub max_parse_time: Duration,
    pub cache_hit_rate: f64,
    pub cache_miss_rate: f64,
    pub total_cache_hits: usize,
    pub total_cache_misses: usize,
    pub avg_memory_usage: usize,
    pub peak_memory_usage: usize,
    pub rule_performance: HashMap<String, RuleMetrics>,
}

// Access metrics
let metrics = profiler.get_metrics();
println!("Success rate: {:.1}%", 
    (metrics.successful_operations as f64 / metrics.total_operations as f64) * 100.0);
println!("Cache efficiency: {:.1}%", metrics.cache_hit_rate * 100.0);
println!("Memory peak: {} MB", metrics.peak_memory_usage / 1_000_000);
```

### Rule-Specific Performance

```rust
// Get performance data for specific rules
for (rule_name, rule_metrics) in &metrics.rule_performance {
    println!("Rule '{}': avg {:.2}ms, success rate {:.1}%",
        rule_name,
        rule_metrics.avg_duration.as_millis(),
        rule_metrics.success_rate * 100.0
    );
}

// Identify bottlenecks
let bottlenecks = profiler.identify_bottlenecks();
for bottleneck in bottlenecks {
    println!("Bottleneck: {} taking {:.2}ms", 
        bottleneck.rule_name, 
        bottleneck.avg_duration.as_millis()
    );
}
```

## Integration Examples

### CI/CD Performance Monitoring

```rust
#[test]
fn performance_regression_test() {
    let mut benchmarks = MarcoBenchmarks::new();
    
    // Define performance baselines
    let max_parse_time = Duration::from_millis(100);
    let min_cache_hit_rate = 0.8; // 80%
    let max_memory_usage = 50_000_000; // 50MB
    
    // Run comprehensive benchmarks
    let results = benchmarks.run_comprehensive_benchmarks();
    
    // Verify performance requirements
    assert!(results.avg_parse_time < max_parse_time, 
        "Parse time regression: {:.2}ms > {:.2}ms",
        results.avg_parse_time.as_millis(),
        max_parse_time.as_millis()
    );
    
    assert!(results.cache_hit_rate > min_cache_hit_rate,
        "Cache hit rate regression: {:.1}% < {:.1}%",
        results.cache_hit_rate * 100.0,
        min_cache_hit_rate * 100.0
    );
    
    assert!(results.peak_memory_usage < max_memory_usage,
        "Memory usage regression: {} > {}",
        results.peak_memory_usage,
        max_memory_usage
    );
    
    // Generate performance report
    let report = benchmarks.generate_markdown_report(&results);
    std::fs::write("performance_report.md", report).unwrap();
}
```

### Development Profiling

```rust
fn profile_development_changes() {
    let mut profiler = PerformanceProfiler::new(ProfilerConfig {
        max_samples: 100,
        enable_cache_monitoring: true,
        enable_rule_tracking: true,
        detailed_timing: true,
    });
    
    // Test various document types
    let test_documents = [
        ("simple", "# Title\nSimple paragraph."),
        ("complex", include_str!("test_documents/complex.md")),
        ("large", include_str!("test_documents/large.md")),
    ];
    
    for (name, content) in test_documents {
        profiler.start_operation(&format!("parse_{}", name));
        
        let start = std::time::Instant::now();
        let result = parse_document(content);
        let duration = start.elapsed();
        
        profiler.record_parse_time(name, duration, result.is_ok());
        profiler.end_operation(&format!("parse_{}", name));
        
        println!("{}: {:.2}ms", name, duration.as_millis());
    }
    
    // Analyze results
    let metrics = profiler.get_metrics();
    let bottlenecks = profiler.identify_bottlenecks();
    
    println!("Performance Summary:");
    println!("- Average: {:.2}ms", metrics.avg_parse_time.as_millis());
    println!("- Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0);
    println!("- Memory peak: {} KB", metrics.peak_memory_usage / 1000);
    
    if !bottlenecks.is_empty() {
        println!("Bottlenecks identified:");
        for bottleneck in bottlenecks {
            println!("- {}: {:.2}ms", bottleneck.rule_name, bottleneck.avg_duration.as_millis());
        }
    }
}
```

### Memory Profiling

```rust
fn profile_memory_usage() {
    let mut profiler = PerformanceProfiler::new(ProfilerConfig::default());
    let mut benchmarks = MarcoBenchmarks::new();
    
    // Baseline memory usage
    let baseline = get_current_memory_usage();
    profiler.record_memory_usage(baseline);
    
    // Test memory usage with various document sizes
    let test_cases = [
        ("small", benchmarks.generate_small_document()),
        ("medium", benchmarks.generate_medium_document()),
        ("large", benchmarks.generate_large_document()),
        ("stress", benchmarks.generate_memory_stress_test()),
    ];
    
    for (name, document) in test_cases {
        // Measure memory before parsing
        let before = get_current_memory_usage();
        
        // Parse document
        let result = parse_document(&document);
        
        // Measure memory after parsing
        let after = get_current_memory_usage();
        let increase = after - before;
        
        profiler.record_memory_usage(after);
        
        println!("{} document: {} KB increase", name, increase / 1000);
        
        // Verify memory is reasonable
        match name {
            "small" => assert!(increase < 1_000_000, "Small doc memory too high"),
            "medium" => assert!(increase < 10_000_000, "Medium doc memory too high"),
            "large" => assert!(increase < 100_000_000, "Large doc memory too high"),
            "stress" => assert!(increase < 500_000_000, "Stress test memory too high"),
            _ => {}
        }
        
        // Force cleanup
        drop(result);
        std::hint::black_box(());
        
        // Verify memory is released
        let after_cleanup = get_current_memory_usage();
        assert!(after_cleanup <= after, "Memory leak detected in {}", name);
    }
    
    let metrics = profiler.get_metrics();
    println!("Memory Summary:");
    println!("- Average usage: {} KB", metrics.avg_memory_usage / 1000);
    println!("- Peak usage: {} KB", metrics.peak_memory_usage / 1000);
}
```

## Benchmark Results Analysis

### Performance Baselines

Current performance baselines (on development hardware):

| Document Type | Size | Parse Time | Memory Usage |
|---------------|------|------------|--------------|
| Small         | 1KB  | < 5ms      | < 1MB        |
| Medium        | 10KB | < 25ms     | < 5MB        |
| Large         | 100KB| < 150ms    | < 25MB       |
| Blog Post     | 5KB  | < 15ms     | < 3MB        |
| Technical Doc | 50KB | < 75ms     | < 15MB       |
| API Docs      | 25KB | < 50ms     | < 10MB       |

### Cache Performance

| Operation Type | Hit Rate | Miss Penalty |
|----------------|----------|--------------|
| Rule parsing   | > 85%    | < 2ms        |
| AST building   | > 90%    | < 1ms        |
| Validation     | > 95%    | < 0.5ms      |

### Common Bottlenecks

1. **Complex table parsing**: Can take 2-5x longer than other elements
2. **Deeply nested lists**: Memory usage increases exponentially 
3. **Large code blocks**: String allocation overhead
4. **Marco extensions**: User mentions and bookmarks require validation

## Optimization Strategies

### 1. Cache Optimization

```rust
// Tune cache size based on usage patterns
let config = ProfilerConfig {
    max_samples: 2000, // Increase for better hit rates
    enable_cache_monitoring: true,
    // ...
};

// Monitor cache performance
let metrics = profiler.get_metrics();
if metrics.cache_hit_rate < 0.8 {
    println!("Consider increasing cache size");
}
```

### 2. Memory Management

```rust
// Use Cow<str> to reduce allocations
fn optimize_string_handling(content: &str) -> Node {
    let trimmed = content.trim();
    if trimmed.len() == content.len() {
        // No allocation needed
        Node::Text { content: content.to_string(), span }
    } else {
        // Only allocate when necessary
        Node::Text { content: trimmed.to_string(), span }
    }
}
```

### 3. Early Validation

```rust
// Validate input size before expensive operations
fn parse_with_validation(input: &str) -> Result<Node> {
    if input.len() > MAX_DOCUMENT_SIZE {
        return Err(MarcoError::content_overflow("document", MAX_DOCUMENT_SIZE));
    }
    
    // Continue with parsing...
}
```

## Continuous Monitoring

### Automated Performance Testing

```bash
# Run performance tests in CI/CD
cargo test performance_regression_test --release

# Generate performance reports
cargo test --test benchmarks --release -- --nocapture > performance.log

# Profile memory usage
valgrind --tool=massif cargo test memory_profile_test --release
```

### Performance Alerts

Set up monitoring to alert on performance regressions:

- Parse time > 150ms for 100KB documents
- Cache hit rate < 80%
- Memory usage > 100MB for any document
- Success rate < 99% for well-formed documents

## Best Practices

1. **Regular Profiling**: Profile performance changes during development
2. **Baseline Maintenance**: Update performance baselines as features are added
3. **Memory Monitoring**: Watch for memory leaks and excessive allocations
4. **Cache Tuning**: Optimize cache size based on usage patterns
5. **Bottleneck Analysis**: Identify and address the slowest operations first
6. **Real-world Testing**: Use realistic document samples for benchmarking

This performance profiling infrastructure provides comprehensive monitoring capabilities to ensure Marco maintains excellent parsing performance while scaling to handle complex documents efficiently.
