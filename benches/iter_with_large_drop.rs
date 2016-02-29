extern crate criterion;

use criterion::Criterion;

const SIZE: usize = 1024 * 1024;

#[test]
fn alloc() {
    Criterion::default()
        .bench_function("alloc", |b| {
            b.iter_with_large_drop(|| (0..SIZE).map(|_| 0u8).collect::<Vec<_>>())
        });
}
