//! Structured benchmark results and text projection.

use core::fmt::Write;
use std::time::Duration;

/// One timed sample for a benchmark function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    /// Wall-clock duration for a single iteration or batched routine.
    pub duration: Duration,
}

/// Aggregate statistics for a benchmark function.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkStats {
    /// Qualified benchmark name (`group/function`).
    pub qualified_name: String,
    /// Number of collected samples.
    pub sample_count: usize,
    /// Total time spent in timed samples.
    pub total: Duration,
    /// Minimum sample duration.
    pub min: Duration,
    /// Median sample duration.
    pub median: Duration,
    /// Mean sample duration.
    pub mean: Duration,
    /// Population standard deviation; `None` when fewer than two samples exist.
    pub std_dev: Option<Duration>,
    /// 95th percentile; `None` when fewer than twenty samples exist.
    pub p95: Option<Duration>,
    /// 99th percentile; `None` when fewer than one hundred samples exist.
    pub p99: Option<Duration>,
    /// Raw sample durations in collection order.
    pub samples: Vec<Duration>,
}

impl BenchmarkStats {
    /// Builds statistics from a qualified name and sample durations.
    pub fn from_samples(qualified_name: impl Into<String>, samples: Vec<Duration>) -> Self {
        let sample_count = samples.len();
        let total = samples.iter().copied().fold(Duration::ZERO, |acc, sample| acc + sample);
        let min = samples.iter().copied().min().unwrap_or(Duration::ZERO);
        let mean = if sample_count == 0 { Duration::ZERO } else { total / sample_count as u32 };
        let median = median_duration(&samples);
        let std_dev = std_dev_duration(&samples, mean);
        let p95 = percentile_duration(&samples, 0.95);
        let p99 = percentile_duration(&samples, 0.99);

        Self { qualified_name: qualified_name.into(), sample_count, total, min, median, mean, std_dev, p95, p99, samples }
    }
}

/// Renders a stable text report for one benchmark function.
pub fn render_text(stats: &BenchmarkStats) -> String {
    let mut text = String::new();
    let _ =
        writeln!(text, "bench {} ... {} samples, total {:.3} ms", stats.qualified_name, stats.sample_count, duration_to_millis(stats.total));
    let _ = writeln!(
        text,
        "  min {:.3} ms | median {:.3} ms | mean {:.3} ms",
        duration_to_millis(stats.min),
        duration_to_millis(stats.median),
        duration_to_millis(stats.mean)
    );
    match stats.std_dev {
        Some(std_dev) => {
            let _ = writeln!(text, "  std dev {:.3} ms", duration_to_millis(std_dev));
        }
        None => {
            let _ = writeln!(text, "  std dev unavailable");
        }
    }
    match (stats.p95, stats.p99) {
        (Some(p95), Some(p99)) => {
            let _ = writeln!(text, "  p95 {:.3} ms | p99 {:.3} ms", duration_to_millis(p95), duration_to_millis(p99));
        }
        _ => {
            let _ = writeln!(text, "  tail percentiles unavailable");
        }
    }
    text
}

fn duration_to_millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn median_duration(samples: &[Duration]) -> Duration {
    if samples.is_empty() {
        return Duration::ZERO;
    }
    let mut sorted = samples.to_vec();
    sorted.sort();
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) { (sorted[mid - 1] + sorted[mid]) / 2 } else { sorted[mid] }
}

fn std_dev_duration(samples: &[Duration], mean: Duration) -> Option<Duration> {
    if samples.len() < 2 {
        return None;
    }
    let mean_nanos = mean.as_nanos() as f64;
    let variance = samples
        .iter()
        .map(|sample| {
            let delta = sample.as_nanos() as f64 - mean_nanos;
            delta * delta
        })
        .sum::<f64>()
        / samples.len() as f64;
    Some(Duration::from_nanos(variance.sqrt() as u64))
}

fn percentile_duration(samples: &[Duration], percentile: f64) -> Option<Duration> {
    if samples.is_empty() {
        return None;
    }
    let min_samples = if percentile >= 0.99 {
        100
    }
    else if percentile >= 0.95 {
        20
    }
    else {
        1
    };
    if samples.len() < min_samples {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort();
    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    Some(sorted[index])
}
