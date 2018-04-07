use criterion::Benchmark;
use criterion::Criterion;
use criterion::Throughput;
use std::time::Duration;

const SIZE: usize = 1024 * 1024;

fn alloc(c: &mut Criterion) {
    c.bench(
        "alloc",
        Benchmark::new("alloc", |b| {
            b.iter_with_large_drop(|| (0..SIZE).map(|_| 0u8).collect::<Vec<_>>())
        }).warm_up_time(Duration::new(1, 0))
            .throughput(Throughput::Bytes(SIZE as u32)),
    );
}

criterion_group!{benches, alloc}
