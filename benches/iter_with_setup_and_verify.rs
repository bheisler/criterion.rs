extern crate criterion;

use criterion::Criterion;

#[test]
fn dealloc() {
    let len = 1024*1024;
    Criterion::default().
        bench("dealloc", |b| {
            b.iter_with_setup_and_verify(|| {
                (0..len).collect::<Vec<usize>>()
            }, |mut v| {
                v[0] = 99;
                v
            }, |v| {
                assert_eq!(99, v[0]);
                assert_eq!(len, v.len());
                drop(v);
            })
        });
}
