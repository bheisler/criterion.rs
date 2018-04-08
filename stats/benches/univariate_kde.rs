#[macro_use]
extern crate criterion;
extern crate criterion_stats as stats;
extern crate itertools_num;
extern crate rand;

mod common_bench;

macro_rules! bench {
    ($ty:ident) => {
        pub mod $ty {
            use criterion::Criterion;
            use stats::univariate::Sample;
            use stats::univariate::kde::kernel::Gaussian;
            use stats::univariate::kde::{Bandwidth, Kde};

            const KDE_POINTS: usize = 100;
            const SAMPLE_SIZE: usize = 100_000;

            fn call(c: &mut Criterion) {
                let data = ::common_bench::vec::<$ty>();

                c.bench_function(
                    &format!("univariate_kde_call_{}", stringify!($ty)),
                    move |b| {
                        let kde = Kde::new(Sample::new(&data), Gaussian, Bandwidth::Silverman);
                        let x = Sample::new(&data).mean();
                        b.iter(|| kde.estimate(x))
                    },
                );
            }

            fn map(c: &mut Criterion) {
                let data = ::common_bench::vec_sized(SAMPLE_SIZE).unwrap();
                let xs: Vec<_> = ::itertools_num::linspace::<$ty>(0., 1., KDE_POINTS).collect();

                c.bench_function(
                    &format!("univariate_kde_map_{}", stringify!($ty)),
                    move |b| {
                        let kde = Kde::new(Sample::new(&data), Gaussian, Bandwidth::Silverman);
                        b.iter(|| kde.map(&xs))
                    },
                );
            }

            criterion_group!{
                name = benches;
                config = ::common_bench::reduced_samples();
                targets = call, map
            }
        }
    };
}

mod bench {
    bench!(f32);
    bench!(f64);
}

criterion_main!(bench::f32::benches, bench::f64::benches);
