use criterion::{BatchSize, Benchmark, Criterion};

fn some_benchmark(c: &mut Criterion) {
    c.bench(
        "overhead",
        Benchmark::new("iter", |b| b.iter(|| 1))
            .with_function("iter_with_setup", |b| b.iter_with_setup(|| (), |_| 1))
            .with_function("iter_with_large_setup", |b| {
                b.iter_with_large_setup(|| (), |_| 1)
            })
            .with_function("iter_with_large_drop", |b| b.iter_with_large_drop(|| 1))
            .with_function("iter_batched_small_input", |b| {
                b.iter_batched(|| (), |_| 1, BatchSize::SmallInput)
            })
            .with_function("iter_batched_large_input", |b| {
                b.iter_batched(|| (), |_| 1, BatchSize::LargeInput)
            })
            .with_function("iter_batched_per_iteration", |b| {
                b.iter_batched(|| (), |_| 1, BatchSize::PerIteration)
            })
            .with_function("iter_batched_ref_small_input", |b| {
                b.iter_batched_ref(|| (), |_| 1, BatchSize::SmallInput)
            })
            .with_function("iter_batched_ref_large_input", |b| {
                b.iter_batched_ref(|| (), |_| 1, BatchSize::LargeInput)
            })
            .with_function("iter_batched_ref_per_iteration", |b| {
                b.iter_batched_ref(|| (), |_| 1, BatchSize::PerIteration)
            }),
    );
}

criterion_group!(benches, some_benchmark);
