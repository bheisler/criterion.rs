#[macro_use]
extern crate criterion;
extern crate criterion_stats as stats;
extern crate rand;

use criterion::Criterion;

mod common_bench;

macro_rules! bench {
    ($ty:ident) => {
        pub mod $ty {
            use criterion::Criterion;
            use stats::bivariate::Data;
            use stats::bivariate::regression::{Slope, StraightLine};

            const NRESAMPLES: usize = 100_000;
            const SAMPLE_SIZE: usize = 100;

            pub fn straight_line(c: &mut Criterion) {
                let x = ::common_bench::vec_sized::<f64>(SAMPLE_SIZE).unwrap();
                let y = ::common_bench::vec_sized::<f64>(SAMPLE_SIZE).unwrap();

                c.bench_function(
                    &format!("bivariate_bootstrap_straight_line_{}", stringify!($ty)),
                    move |b| {
                        let data = Data::new(&x, &y);
                        b.iter(|| data.bootstrap(NRESAMPLES, |d| (StraightLine::fit(d),)))
                    },
                );
            }

            pub fn slope(c: &mut Criterion) {
                let x = ::common_bench::vec_sized::<f64>(SAMPLE_SIZE).unwrap();
                let y = ::common_bench::vec_sized::<f64>(SAMPLE_SIZE).unwrap();

                c.bench_function(
                    &format!("bivariate_bootstrap_slope_{}", stringify!($ty)),
                    move |b| {
                        let data = Data::new(&x, &y);
                        b.iter(|| data.bootstrap(NRESAMPLES, |d| (Slope::fit(d),)))
                    },
                );
            }
        }
    };
}

mod bench {
    bench!(f32);
    bench!(f64);
}

criterion_group!(
    name = benches;
    config = common_bench::reduced_samples();
    targets = bench::f32::slope, bench::f32::straight_line,
              bench::f64::slope, bench::f64::straight_line);
criterion_main!(benches);
