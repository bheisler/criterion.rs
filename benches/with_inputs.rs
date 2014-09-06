extern crate criterion;

use criterion::{Bencher, Criterion};

#[test]
fn from_elem() {
    Criterion::default().bench_with_inputs("from_elem", |b, &size| {
        b.iter(|| Vec::from_elem(size, 0u8));
    }, [1024, 2048, 4096]);
}
