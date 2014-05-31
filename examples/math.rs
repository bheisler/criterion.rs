extern crate criterion;
extern crate test;

use criterion::Criterion;

fn main() {
    let mut b = Criterion::new();

    b.bench("exp", |b| {
        let mut x: f64 = 2.0;
        test::black_box(&mut x);

        b.iter(|| x.exp())
    });

    b.bench("ln", |b| {
        let mut x: f64 = 2.0;
        test::black_box(&mut x);

        b.iter(|| x.ln())
    });

    b.bench("sqrt", |b| {
        let mut x: f64 = 2.0;
        test::black_box(&mut x);

        b.iter(|| x.sqrt())
    });
}
