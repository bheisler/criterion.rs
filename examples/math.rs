extern crate criterion;

use criterion::Bencher;

fn main() {
    let mut b = Bencher::new();

    b.bench("exp", || 2.0_f64.exp());
    b.bench("ln", || 2.0_f64.ln());
    b.bench("sqrt", || 2.0_f64.sqrt());
}
