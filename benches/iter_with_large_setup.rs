extern crate criterion;

use criterion::Criterion;

#[test]
fn dealloc() {
    Criterion::default().
        bench("large_dealloc", |b| {
            b.iter_with_large_setup(|| {
                Vec::from_elem(1024 * 1024, 0u8)
            }, |v| {
                drop(v);
            })
        });
}
