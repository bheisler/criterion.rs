#[macro_use]
extern crate criterion;
extern crate criterion_stats as stats;
extern crate rand;

mod common_bench;

use criterion::Criterion;

macro_rules! bench {
    ($ty:ident) => {
        pub mod $ty {
            use criterion::Criterion;
            use stats::bivariate::Data;
            use stats::bivariate::regression::{Slope, StraightLine};

            pub fn slope(c: &mut Criterion) {
                let x = ::common_bench::vec::<$ty>();
                let y = ::common_bench::vec();

                c.bench_function(
                    &format!("bivariate_regression_slope_{}", stringify!($ty)),
                    move |b| {
                        let data = Data::new(&x, &y);
                        b.iter(|| Slope::fit(data))
                    },
                );
            }

            pub fn straight_line(c: &mut Criterion) {
                let x = ::common_bench::vec::<$ty>();
                let y = ::common_bench::vec();

                c.bench_function(
                    &format!("bivariate_regression_straight_line_{}", stringify!($ty)),
                    move |b| {
                        let data = Data::new(&x, &y);
                        b.iter(|| StraightLine::fit(data))
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
