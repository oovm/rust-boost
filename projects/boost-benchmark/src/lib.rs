#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

pub mod bencher;
pub mod config;
pub mod group;
pub mod report;
pub mod suite;

pub use bencher::Bencher;
pub use config::BenchmarkConfig;
pub use group::BenchmarkGroup;
pub use report::{BenchmarkStats, Sample, render_text};
pub use suite::BenchmarkSuite;

/// Prevents the compiler from optimizing away benchmark inputs and outputs.
pub use core::hint::black_box;

/// Declares one or more benchmark functions that share a suite entry point.
///
/// ```ignore
/// fn bench_example(c: &mut BenchmarkSuite) { /* ... */ }
/// benchmark_group!(benches, bench_example);
/// benchmark_main!(benches);
/// ```
#[macro_export]
macro_rules! benchmark_group {
    ($name:ident, $($function:ident),*) => {
        fn $name(c: &mut $crate::BenchmarkSuite) {
            $($function(c);)*
        }
    };
}

/// Runs declared benchmark groups and prints structured reports.
#[macro_export]
macro_rules! benchmark_main {
    ($($group:ident),*) => {
        fn main() {
            let mut suite = $crate::BenchmarkSuite::new();
            $($group(&mut suite);)*
            suite.finish();
        }
    };
}
