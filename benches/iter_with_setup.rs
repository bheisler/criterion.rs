use std::mem;

use criterion::Criterion;

const SIZE: usize = 1024 * 1024;

fn dealloc(c: &mut Criterion) {
    c.bench_function("dealloc", |b| {
        b.iter_with_setup(|| (0..SIZE).map(|_| 0u8).collect::<Vec<_>>(), mem::drop)
    });
}

criterion_group!(benches, dealloc);
