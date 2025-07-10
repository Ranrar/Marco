// Rayon parallelism utilities for Marco
//
// This module provides a global Rayon thread pool for CPU-bound parallel processing.
// Use `run_in_pool` to execute closures on the thread pool. See:
// https://docs.rs/rayon/latest/rayon/index.html#how-to-use-rayon
//
// - Only use Rayon for CPU-bound work, never for GTK UI updates.
// - Always send results back to the GTK main thread using glib channels or idle_add_local.

use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use once_cell::sync::Lazy;

/// Global Rayon thread pool for parallel processing
pub static GLOBAL_THREADPOOL: Lazy<ThreadPool> = Lazy::new(|| {
    ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .expect("Failed to build global rayon thread pool")
});

/// Run a closure on the global thread pool
///
/// # Example
/// ```rust
/// let result = run_in_pool(|| heavy_computation());
/// ```
pub fn run_in_pool<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    GLOBAL_THREADPOOL.install(f)
}