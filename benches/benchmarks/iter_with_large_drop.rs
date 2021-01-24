#![allow(deprecated)]

use criterion::{criterion_group, Benchmark, Criterion, Throughput};
use std::time::Duration;

const SIZE: usize = 1024 * 1024;

fn large_drop(c: &mut Criterion) {
    c.bench(
        "iter_with_large_drop",
        Benchmark::new("large_drop", |b| {
            let v: Vec<_> = (0..SIZE).map(|i| i as u8).collect();
            b.iter_with_large_drop(|| v.clone());
        })
        .throughput(Throughput::Bytes(SIZE as u64)),
    );
}

fn small_drop(c: &mut Criterion) {
    c.bench(
        "iter_with_large_drop",
        Benchmark::new("small_drop", |b| {
            b.iter_with_large_drop(|| SIZE);
        }),
    );
}

fn short_warmup() -> Criterion {
    Criterion::default().warm_up_time(Duration::new(1, 0))
}

criterion_group! {
    name = benches;
    config = short_warmup();
    targets = large_drop, small_drop
}
