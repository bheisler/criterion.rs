extern crate criterion;

use criterion::Criterion;

#[test]
fn alloc() {
    Criterion::default().
        bench("alloc+dealloc", |b| {
            b.iter(|| {
                Vec::from_elem(1024 * 1024, 0u8)
            })
        });
}
