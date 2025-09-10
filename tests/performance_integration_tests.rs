//! Test for performance modules
//! Verifies that the moved profiler and benchmarks modules work correctly

mod performance;

use performance::{MarcoBenchmarks, PerformanceProfiler, ProfilerConfig};

#[test]
fn test_profiler_configuration() {
    let config = ProfilerConfig::development();
    assert!(config.enabled);
    assert!(config.track_memory);
    assert!(config.track_cache);
    assert!(config.track_rules);

    let profiler = PerformanceProfiler::new(config);
    // Verify profiler was created successfully
    let timer = profiler.start_timer("test_operation");
    assert!(timer.is_some());
}

#[test]
fn test_performance_profiling() {
    let config = ProfilerConfig::performance_testing();
    let mut profiler = PerformanceProfiler::new(config);

    // Start and record a mock operation
    if let Some(timer) = profiler.start_timer("test_parse") {
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));
        profiler.record_operation(timer);
    }

    let report = profiler.generate_report();
    assert!(report.profiling_duration > std::time::Duration::from_nanos(0));
}

#[test]
fn test_benchmarks_creation() {
    // Test that we can create a benchmarks instance
    // Note: This test doesn't run the full benchmark suite to avoid long test times
    let _benchmarks = MarcoBenchmarks::new();

    // If we get here without panicking, the modules are properly connected
    assert!(true);
}

#[test]
fn test_profiler_configurations() {
    let dev_config = ProfilerConfig::development();
    let prod_config = ProfilerConfig::production();
    let test_config = ProfilerConfig::performance_testing();

    assert!(dev_config.enabled);
    assert!(!prod_config.enabled);
    assert!(test_config.enabled);

    assert!(test_config.max_samples >= dev_config.max_samples);
    assert_eq!(test_config.min_duration_ns, 0);
}

#[test]
fn test_profiler_reset() {
    let config = ProfilerConfig::development();
    let mut profiler = PerformanceProfiler::new(config);

    // Add some mock data
    if let Some(timer) = profiler.start_timer("test_operation") {
        profiler.record_operation(timer);
    }

    // Reset and verify
    profiler.reset();
    let report = profiler.generate_report();
    assert_eq!(report.operations.len(), 0);
}
