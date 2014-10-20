extern crate criterion;

use criterion::Criterion;

#[test]
fn alloc() {
    Criterion::default().
        bench("alloc", |b| {
            b.iter_with_large_drop(|| {
                Vec::from_elem(1024 * 1024, 0u8)
            })
        });
}
