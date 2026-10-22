//! Named benchmark groups.

use crate::bencher::Bencher;
use crate::config::BenchmarkConfig;
use crate::report::BenchmarkStats;
use crate::suite::BenchmarkSuite;

/// A logical group of related benchmark functions.
pub struct BenchmarkGroup<'a> {
    group_name: String,
    config: BenchmarkConfig,
    reports: Vec<BenchmarkStats>,
    suite: &'a mut BenchmarkSuite,
}

impl<'a> BenchmarkGroup<'a> {
    /// Creates a group named `group_name` attached to `suite`.
    pub(crate) fn new(
        group_name: impl Into<String>,
        config: BenchmarkConfig,
        suite: &'a mut BenchmarkSuite,
    ) -> Self {
        Self {
            group_name: group_name.into(),
            config,
            reports: Vec::new(),
            suite,
        }
    }

    /// Measures one function and records its statistics under this group.
    pub fn bench_function<F>(&mut self, name: impl AsRef<str>, mut benchmark: F)
    where
        F: FnMut(&mut Bencher),
    {
        let qualified_name = format!("{}/{}", self.group_name, name.as_ref());
        let mut bencher = Bencher::new(qualified_name, self.config);
        benchmark(&mut bencher);
        self.reports.push(bencher.finish());
    }

    /// Finalizes this group and records its statistics in the parent suite.
    pub fn finish(self) {
        self.suite.push_reports(self.reports);
    }
}
