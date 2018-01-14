use criterion::Criterion;
use std::time::Duration;

const SIZE: usize = 1024 * 1024;

fn alloc(c: &mut Criterion) {
    c.bench_function("alloc", |b| {
            b.iter_with_large_drop(|| (0..SIZE).map(|_| 0u8).collect::<Vec<_>>())
        });
}

fn short_warmup() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::new(1, 0))
}

criterion_group!{
    name = benches;
    config = short_warmup();
    targets = alloc
}
