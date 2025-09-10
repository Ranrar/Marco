use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance profiling infrastructure for Marco parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerConfig {
    /// Enable/disable profiling
    pub enabled: bool,
    /// Collect memory usage statistics
    pub track_memory: bool,
    /// Track parser cache performance
    pub track_cache: bool,
    /// Track individual rule performance
    pub track_rules: bool,
    /// Maximum number of samples to keep per metric
    pub max_samples: usize,
    /// Minimum duration to record (filters out noise)
    pub min_duration_ns: u64,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for production
            track_memory: true,
            track_cache: true,
            track_rules: true,
            max_samples: 10_000,
            min_duration_ns: 1_000, // 1 microsecond
        }
    }
}

impl ProfilerConfig {
    /// Development configuration with detailed profiling
    pub fn development() -> Self {
        Self {
            enabled: true,
            track_memory: true,
            track_cache: true,
            track_rules: true,
            max_samples: 50_000,
            min_duration_ns: 100, // 100 nanoseconds
        }
    }

    /// Performance testing configuration
    pub fn performance_testing() -> Self {
        Self {
            enabled: true,
            track_memory: true,
            track_cache: true,
            track_rules: true,
            max_samples: 100_000,
            min_duration_ns: 0, // Record everything
        }
    }

    /// Production configuration with minimal overhead
    pub fn production() -> Self {
        Self {
            enabled: false,
            track_memory: false,
            track_cache: false,
            track_rules: false,
            max_samples: 1_000,
            min_duration_ns: 10_000, // 10 microseconds
        }
    }
}

/// Individual performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    pub duration: Duration,
    pub memory_bytes: Option<usize>,
    pub timestamp: std::time::SystemTime,
    pub context: String,
}

/// Performance metrics for a specific operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub total_calls: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub memory_usage: Option<MemoryMetrics>,
    pub recent_samples: Vec<PerformanceSample>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_bytes: usize,
    pub avg_bytes: usize,
    pub total_allocations: u64,
    pub current_bytes: usize,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f64,
    pub avg_lookup_time: Duration,
    pub memory_used: usize,
    pub max_memory: usize,
}

/// Grammar rule performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetrics {
    pub rule_name: String,
    pub parse_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub success_rate: f64,
    pub avg_parse_time: Duration,
    pub total_parse_time: Duration,
    pub max_parse_time: Duration,
}

/// Comprehensive performance profiler
pub struct PerformanceProfiler {
    config: ProfilerConfig,
    metrics: HashMap<String, PerformanceMetrics>,
    cache_metrics: CacheMetrics,
    rule_metrics: HashMap<String, RuleMetrics>,
    global_start_time: Instant,
}

impl PerformanceProfiler {
    pub fn new(config: ProfilerConfig) -> Self {
        Self {
            config,
            metrics: HashMap::new(),
            cache_metrics: CacheMetrics {
                hits: 0,
                misses: 0,
                evictions: 0,
                hit_rate: 0.0,
                avg_lookup_time: Duration::from_nanos(0),
                memory_used: 0,
                max_memory: 0,
            },
            rule_metrics: HashMap::new(),
            global_start_time: Instant::now(),
        }
    }

    /// Start timing an operation
    pub fn start_timer(&self, operation: &str) -> Option<ProfileTimer> {
        if !self.config.enabled {
            return None;
        }

        Some(ProfileTimer {
            operation: operation.to_string(),
            start_time: Instant::now(),
            start_memory: if self.config.track_memory {
                Some(self.get_current_memory_usage())
            } else {
                None
            },
        })
    }

    /// Record a completed operation
    pub fn record_operation(&mut self, timer: ProfileTimer) {
        if !self.config.enabled {
            return;
        }

        let duration = timer.start_time.elapsed();

        // Skip if duration is below threshold
        if duration.as_nanos() < self.config.min_duration_ns as u128 {
            return;
        }

        let memory_usage = if let Some(start_mem) = timer.start_memory {
            let current_mem = self.get_current_memory_usage();
            Some(current_mem.saturating_sub(start_mem))
        } else {
            None
        };

        let sample = PerformanceSample {
            duration,
            memory_bytes: memory_usage,
            timestamp: std::time::SystemTime::now(),
            context: timer.operation.clone(),
        };

        self.update_metrics(&timer.operation, sample);
    }

    /// Record cache operation
    pub fn record_cache_hit(&mut self, lookup_time: Duration) {
        if !self.config.track_cache {
            return;
        }

        self.cache_metrics.hits += 1;
        self.update_cache_avg_time(lookup_time);
        self.update_cache_hit_rate();
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self, lookup_time: Duration) {
        if !self.config.track_cache {
            return;
        }

        self.cache_metrics.misses += 1;
        self.update_cache_avg_time(lookup_time);
        self.update_cache_hit_rate();
    }

    /// Record cache eviction
    pub fn record_cache_eviction(&mut self) {
        if !self.config.track_cache {
            return;
        }

        self.cache_metrics.evictions += 1;
    }

    /// Record grammar rule parsing
    pub fn record_rule_parse(&mut self, rule_name: &str, duration: Duration, success: bool) {
        if !self.config.track_rules {
            return;
        }

        let metrics = self
            .rule_metrics
            .entry(rule_name.to_string())
            .or_insert_with(|| RuleMetrics {
                rule_name: rule_name.to_string(),
                parse_count: 0,
                success_count: 0,
                failure_count: 0,
                success_rate: 0.0,
                avg_parse_time: Duration::from_nanos(0),
                total_parse_time: Duration::from_nanos(0),
                max_parse_time: Duration::from_nanos(0),
            });

        metrics.parse_count += 1;
        if success {
            metrics.success_count += 1;
        } else {
            metrics.failure_count += 1;
        }

        metrics.success_rate = metrics.success_count as f64 / metrics.parse_count as f64;
        metrics.total_parse_time += duration;
        metrics.avg_parse_time =
            Duration::from_nanos(metrics.total_parse_time.as_nanos() as u64 / metrics.parse_count);

        if duration > metrics.max_parse_time {
            metrics.max_parse_time = duration;
        }
    }

    /// Get comprehensive performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let mut operations: Vec<_> = self.metrics.values().cloned().collect();
        operations.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));

        let mut rules: Vec<_> = self.rule_metrics.values().cloned().collect();
        rules.sort_by(|a, b| b.total_parse_time.cmp(&a.total_parse_time));

        PerformanceReport {
            profiling_duration: self.global_start_time.elapsed(),
            operations,
            cache_metrics: self.cache_metrics.clone(),
            rule_metrics: rules,
            top_bottlenecks: self.identify_bottlenecks(),
            memory_summary: self.get_memory_summary(),
        }
    }

    /// Export report as JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let report = self.generate_report();
        serde_json::to_string_pretty(&report)
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.metrics.clear();
        self.rule_metrics.clear();
        self.cache_metrics = CacheMetrics {
            hits: 0,
            misses: 0,
            evictions: 0,
            hit_rate: 0.0,
            avg_lookup_time: Duration::from_nanos(0),
            memory_used: 0,
            max_memory: 0,
        };
        self.global_start_time = Instant::now();
    }

    // Private helper methods
    fn update_metrics(&mut self, operation: &str, sample: PerformanceSample) {
        let metrics = self
            .metrics
            .entry(operation.to_string())
            .or_insert_with(|| PerformanceMetrics {
                operation_name: operation.to_string(),
                total_calls: 0,
                total_duration: Duration::from_nanos(0),
                min_duration: Duration::from_secs(u64::MAX),
                max_duration: Duration::from_nanos(0),
                avg_duration: Duration::from_nanos(0),
                p50_duration: Duration::from_nanos(0),
                p95_duration: Duration::from_nanos(0),
                p99_duration: Duration::from_nanos(0),
                memory_usage: None,
                recent_samples: Vec::new(),
            });

        metrics.total_calls += 1;
        metrics.total_duration += sample.duration;

        if sample.duration < metrics.min_duration {
            metrics.min_duration = sample.duration;
        }
        if sample.duration > metrics.max_duration {
            metrics.max_duration = sample.duration;
        }

        metrics.avg_duration =
            Duration::from_nanos(metrics.total_duration.as_nanos() as u64 / metrics.total_calls);

        // Keep recent samples for percentile calculations
        metrics.recent_samples.push(sample);
        if metrics.recent_samples.len() > self.config.max_samples {
            metrics.recent_samples.remove(0);
        }

        // Update percentiles inline to avoid borrowing issues
        if !metrics.recent_samples.is_empty() {
            let mut durations: Vec<Duration> =
                metrics.recent_samples.iter().map(|s| s.duration).collect();
            durations.sort();

            let len = durations.len();
            metrics.p50_duration = durations[len / 2];
            metrics.p95_duration = durations[(len as f64 * 0.95) as usize];
            metrics.p99_duration = durations[(len as f64 * 0.99) as usize];
        }
    }

    fn update_cache_avg_time(&mut self, lookup_time: Duration) {
        let total_operations = self.cache_metrics.hits + self.cache_metrics.misses;
        if total_operations > 0 {
            let total_time =
                self.cache_metrics.avg_lookup_time * (total_operations as u32 - 1) + lookup_time;
            self.cache_metrics.avg_lookup_time = total_time / total_operations as u32;
        } else {
            self.cache_metrics.avg_lookup_time = lookup_time;
        }
    }

    fn update_cache_hit_rate(&mut self) {
        let total = self.cache_metrics.hits + self.cache_metrics.misses;
        if total > 0 {
            self.cache_metrics.hit_rate = self.cache_metrics.hits as f64 / total as f64;
        }
    }

    fn get_current_memory_usage(&self) -> usize {
        // Placeholder - in a real implementation, this would use a memory profiler
        // or system calls to get actual memory usage
        0
    }

    fn identify_bottlenecks(&self) -> Vec<BottleneckReport> {
        let mut bottlenecks = Vec::new();

        // Identify slow operations
        for metrics in self.metrics.values() {
            if metrics.avg_duration.as_millis() > 10 {
                bottlenecks.push(BottleneckReport {
                    operation: metrics.operation_name.clone(),
                    issue_type: "Slow Operation".to_string(),
                    severity: if metrics.avg_duration.as_millis() > 100 {
                        "High"
                    } else {
                        "Medium"
                    }
                    .to_string(),
                    description: format!(
                        "Average duration: {}ms, Total calls: {}",
                        metrics.avg_duration.as_millis(),
                        metrics.total_calls
                    ),
                    recommendation: "Consider optimizing this operation or caching results"
                        .to_string(),
                });
            }
        }

        // Identify cache performance issues
        if self.cache_metrics.hit_rate < 0.8 {
            bottlenecks.push(BottleneckReport {
                operation: "Parser Cache".to_string(),
                issue_type: "Low Cache Hit Rate".to_string(),
                severity: "Medium".to_string(),
                description: format!("Hit rate: {:.2}%", self.cache_metrics.hit_rate * 100.0),
                recommendation: "Consider increasing cache size or improving cache key strategy"
                    .to_string(),
            });
        }

        bottlenecks
    }

    fn get_memory_summary(&self) -> MemorySummary {
        // Placeholder implementation
        MemorySummary {
            peak_usage: 0,
            current_usage: 0,
            total_allocations: 0,
            avg_allocation_size: 0,
        }
    }
}

/// Timer for measuring operation duration
pub struct ProfileTimer {
    operation: String,
    start_time: Instant,
    start_memory: Option<usize>,
}

/// Complete performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub profiling_duration: Duration,
    pub operations: Vec<PerformanceMetrics>,
    pub cache_metrics: CacheMetrics,
    pub rule_metrics: Vec<RuleMetrics>,
    pub top_bottlenecks: Vec<BottleneckReport>,
    pub memory_summary: MemorySummary,
}

/// Performance bottleneck identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckReport {
    pub operation: String,
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
}

/// Memory usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub peak_usage: usize,
    pub current_usage: usize,
    pub total_allocations: usize,
    pub avg_allocation_size: usize,
}

/// Macro for easy profiling
#[macro_export]
macro_rules! profile_operation {
    ($profiler:expr, $operation:expr, $code:block) => {{
        let timer = $profiler.start_timer($operation);
        let result = $code;
        if let Some(timer) = timer {
            $profiler.record_operation(timer);
        }
        result
    }};
}

/// Global profiler instance for convenience
static mut GLOBAL_PROFILER: Option<PerformanceProfiler> = None;
static PROFILER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize global profiler
pub fn init_global_profiler(config: ProfilerConfig) {
    PROFILER_INIT.call_once(|| unsafe {
        GLOBAL_PROFILER = Some(PerformanceProfiler::new(config));
    });
}

/// Get reference to global profiler
pub fn global_profiler() -> Option<&'static mut PerformanceProfiler> {
    unsafe { GLOBAL_PROFILER.as_mut() }
}
