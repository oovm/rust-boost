//! Benchmark measurement configuration.

/// Controls warmup, sample collection, and optional time budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkConfig {
    /// Iterations executed before samples are recorded.
    pub warmup_iterations: usize,
    /// Number of timed samples collected per benchmark function.
    pub sample_size: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 3,
            sample_size: 100,
        }
    }
}

impl BenchmarkConfig {
    /// Creates a configuration with explicit warmup and sample sizes.
    pub const fn new(warmup_iterations: usize, sample_size: usize) -> Self {
        Self {
            warmup_iterations,
            sample_size,
        }
    }
}
