extern crate criterion;

use criterion::Criterion;

#[test]
fn alloc() {
    Criterion::default().
        bench("alloc+dealloc", |b| {
            b.iter(|| {
                (0..1024*1024).collect::<Vec<usize>>()
            })
        });
}
