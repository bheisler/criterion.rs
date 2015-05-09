extern crate criterion;

use criterion::Criterion;

#[test]
fn dealloc() {
    Criterion::default().
        bench("dealloc", |b| {
            b.iter_with_setup(|| {
                (0..1024*1024).collect::<Vec<usize>>()
            }, |v| {
                drop(v);
            })
        });
}
