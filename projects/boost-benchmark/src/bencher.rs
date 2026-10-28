//! Per-function measurement harness.

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use crate::{config::BenchmarkConfig, report::BenchmarkStats};

/// Measures one benchmark function within a group.
pub struct Bencher {
    qualified_name: String,
    config: BenchmarkConfig,
    samples: Vec<Duration>,
}

impl Bencher {
    /// Creates a bencher for `qualified_name` using `config`.
    pub(crate) fn new(qualified_name: String, config: BenchmarkConfig) -> Self {
        Self { qualified_name, config, samples: Vec::with_capacity(config.sample_size) }
    }

    /// Runs `routine` for warmup, then records `sample_size` timed iterations.
    pub fn iter<R, F>(&mut self, mut routine: F)
    where
        F: FnMut() -> R,
    {
        for _ in 0..self.config.warmup_iterations {
            black_box(routine());
        }

        self.samples.clear();
        self.samples.reserve(self.config.sample_size);
        for _ in 0..self.config.sample_size {
            let start = Instant::now();
            black_box(routine());
            self.samples.push(start.elapsed());
        }
    }

    /// Runs `setup` outside the timed section, then times `routine` over `batch_size` inputs.
    pub fn iter_batched<S, R, FS, FR>(&mut self, mut setup: FS, mut routine: FR, batch_size: usize)
    where
        FS: FnMut() -> S,
        FR: FnMut(S) -> R,
    {
        for _ in 0..self.config.warmup_iterations {
            let input = setup();
            black_box(routine(input));
        }

        self.samples.clear();
        self.samples.reserve(self.config.sample_size);
        for _ in 0..self.config.sample_size {
            let mut batch = Vec::with_capacity(batch_size);
            for _ in 0..batch_size {
                batch.push(setup());
            }

            let start = Instant::now();
            for input in batch {
                black_box(routine(input));
            }
            self.samples.push(start.elapsed());
        }
    }

    /// Finalizes collected samples into structured statistics.
    pub(crate) fn finish(self) -> BenchmarkStats {
        BenchmarkStats::from_samples(self.qualified_name, self.samples)
    }
}
