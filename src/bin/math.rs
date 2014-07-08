extern crate criterion;
extern crate test;

use criterion::Criterion;

fn main() {
    Criterion::default().
        bench("exp", |b| {
            let mut x: f64 = 2.0;
            test::black_box(&mut x);

            b.iter(|| x.exp())
        }).
        bench("ln", |b| {
            let mut x: f64 = 2.0;
            test::black_box(&mut x);

            b.iter(|| x.ln())
        }).
        bench("sqrt", |b| {
            let mut x: f64 = 2.0;
            test::black_box(&mut x);

            b.iter(|| x.sqrt())
        });
}
