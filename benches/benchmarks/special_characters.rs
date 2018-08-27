use criterion::{Benchmark, Criterion};

fn some_benchmark(c: &mut Criterion) {
    c.bench(
        "\"*group/\"",
        Benchmark::new("\"*benchmark/\"", |b| b.iter(|| 1 + 1)),
    );
}

criterion_group!(benches, some_benchmark);
