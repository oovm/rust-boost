//! Top-level benchmark suite and reporting.

use crate::{
    config::BenchmarkConfig,
    group::BenchmarkGroup,
    report::{BenchmarkStats, render_text},
};

/// Owns benchmark configuration and aggregated reports.
#[derive(Debug, Default)]
pub struct BenchmarkSuite {
    config: BenchmarkConfig,
    reports: Vec<BenchmarkStats>,
}

impl BenchmarkSuite {
    /// Creates a suite with the default measurement configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a suite with an explicit configuration.
    pub fn with_config(config: BenchmarkConfig) -> Self {
        Self { config, reports: Vec::new() }
    }

    /// Opens a named benchmark group.
    pub fn benchmark_group(&mut self, name: impl Into<String>) -> BenchmarkGroup<'_> {
        BenchmarkGroup::new(name, self.config, self)
    }

    /// Records statistics produced by a finished group.
    pub(crate) fn push_reports(&mut self, reports: Vec<BenchmarkStats>) {
        self.reports.extend(reports);
    }

    /// Returns all collected statistics.
    pub fn reports(&self) -> &[BenchmarkStats] {
        &self.reports
    }

    /// Prints the text report for every collected benchmark.
    pub fn finish(&self) {
        for report in &self.reports {
            print!("{}", render_text(report));
        }
    }
}
