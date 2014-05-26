extern crate criterion;

use criterion::Bencher;

fn main() {
    let mut b = Bencher::new();

    b.bench("box", || box 0.0f64);
    b.bench_group("from_elem", &[1u, 100, 10_000, 1_000_000], |n| {
        Vec::from_elem(n, 0.0f64)
    });
}
