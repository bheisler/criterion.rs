extern crate criterion;

use criterion::{Bencher,Criterion};

fn main() {
    let mut b = Criterion::new();

    b.bench("box", |b| b.iter(|| box 0.0_f64));
    b.bench_group("from_elem", &[1u, 100, 10_000, 1_000_000], from_elem);
}

fn from_elem(b: &mut Bencher, n: uint) {
    b.iter(|| {
        Vec::from_elem(n, 0.0_f64)
    })
}
