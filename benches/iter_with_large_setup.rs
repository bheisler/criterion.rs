use std::mem;

use criterion::Criterion;
use std::time::Duration;

const SIZE: usize = 1024 * 1024;

fn dealloc(c: &mut Criterion) {
    c.bench_function("large_dealloc", |b| {
        b.iter_with_large_setup(|| (0..SIZE).map(|_| 0u8).collect::<Vec<_>>(), mem::drop);
    });
}

fn short_warmup() -> Criterion {
    Criterion::default().warm_up_time(Duration::new(1, 0))
}

criterion_group!{
    name = benches;
    config = short_warmup();
    targets = dealloc
}
