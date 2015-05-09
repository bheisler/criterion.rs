extern crate criterion;

use criterion::Criterion;

#[test]
fn dealloc() {
    Criterion::default().
        bench("large_dealloc", |b| {
            b.iter_with_large_setup(|| {
                (0..1024*1024).collect::<Vec<usize>>()
            }, |v| {
                drop(v);
            })
        });
}
