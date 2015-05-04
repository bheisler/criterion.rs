extern crate criterion;

use criterion::Criterion;

#[test]
fn alloc() {
    Criterion::default().
        bench("alloc", |b| {
            b.iter_with_large_drop(|| {
                (0..1024*1024).collect::<Vec<usize>>()
            })
        });
}
