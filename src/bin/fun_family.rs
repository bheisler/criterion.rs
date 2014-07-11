extern crate criterion;

use criterion::{Bencher,Criterion};

fn main() {
    Criterion::default().
        bench_family(
            "from_elem",
            from_elem,
            [10u, 100, 1_000]);
}

fn from_elem(b: &mut Bencher, n: &uint) {
    b.iter(|| Vec::from_elem(*n, 0u))
}
