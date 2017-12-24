#[macro_use]
extern crate criterion;
extern crate criterion_stats as stats;
extern crate rand;

mod common_bench;

use criterion::Criterion;

macro_rules! bench {
    ($ty:ident) => {
        pub mod $ty {
            use stats::univariate::Sample;
            use criterion::Criterion;

            const NRESAMPLES: usize = 100_000;
            const SAMPLE_SIZE: usize = 100;

            pub fn mean(c: &mut Criterion) {
                let v = ::common_bench::vec_sized::<$ty>(SAMPLE_SIZE).unwrap();
                let sample = Sample::new(&v);

                c.bench_function(
                    &format!("univariate_bootstrap_mean_{}", stringify!($ty)),
                    |b| b.iter(|| {
                        sample.bootstrap(NRESAMPLES, |s| (s.mean(),))
                    })
                );
            }
        }
    }
}

mod bench {
    bench!(f32);
    bench!(f64);
}

criterion_group!(
    name = benches;
    config = common_bench::reduced_samples();
    targets = bench::f32::mean, bench::f64::mean);
criterion_main!(benches);