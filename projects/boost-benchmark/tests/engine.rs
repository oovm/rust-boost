use boost_benchmark::{BenchmarkConfig, BenchmarkSuite, black_box};

#[test]
fn iter_collects_samples_and_reports() {
    let mut suite = BenchmarkSuite::with_config(BenchmarkConfig::new(1, 5));
    let mut group = suite.benchmark_group("math");
    let mut value = 0_u64;
    group.bench_function("add_one", |b| {
        b.iter(|| {
            value += 1;
            black_box(value);
        });
    });
    group.finish();

    let reports = suite.reports();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].qualified_name, "math/add_one");
    assert_eq!(reports[0].sample_count, 5);
    assert!(reports[0].mean > std::time::Duration::ZERO);
}

#[test]
fn iter_batched_times_routine_only() {
    let mut suite = BenchmarkSuite::with_config(BenchmarkConfig::new(1, 3));
    let mut group = suite.benchmark_group("batch");
    group.bench_function("double_values", |b| {
        b.iter_batched(|| vec![1_u32, 2, 3], |values| values.into_iter().map(|v| v * 2).sum::<u32>(), 3);
    });
    group.finish();

    assert_eq!(suite.reports().len(), 1);
    assert_eq!(suite.reports()[0].sample_count, 3);
}
