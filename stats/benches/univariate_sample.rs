extern crate cast;
#[macro_use]
extern crate criterion;
extern crate criterion_stats as stats;
extern crate rand;

mod common_bench;

macro_rules! stat {
    ($ty:ident <- $($stat:ident),+) => {
        $(
            fn $stat(c: &mut Criterion) {
                let v = ::common_bench::vec::<$ty>();

                c.bench_function(
                    &format!("stat_{}_{}", stringify!($ty), stringify!($stat)),
                    move |b| {
                        let s = ::stats::univariate::Sample::new(&v);
                        b.iter(|| s.$stat())
                    });
            }
        )+
    }
}

macro_rules! stat_none {
    ($ty:ident <- $($stat:ident),+) => {
        $(
            fn $stat(c: &mut Criterion) {
                let v = ::common_bench::vec::<$ty>();

                c.bench_function(
                    &format!("stat_none_{}_{}", stringify!($ty), stringify!($stat)),
                    move |b| {
                        let s = ::stats::univariate::Sample::new(&v);
                        b.iter(|| s.$stat(None))
                    });
            }
        )+
    }
}

macro_rules! fast_stat {
    ($ty:ident <- $(($stat:ident, $aux_stat:ident)),+) => {
        $(
            fn $stat(c: &mut Criterion) {
                let v = ::common_bench::vec::<$ty>();

                c.bench_function(
                    &format!("fast_stat_{}_{}", stringify!($ty), stringify!($stat)),
                    move |b| {
                        let s = ::stats::univariate::Sample::new(&v);
                        let aux = Some(s.$aux_stat());
                        b.iter(|| s.$stat(aux))
                    });
            }
        )+
    }
}

macro_rules! bench {
    ($ty:ident) => {
        pub mod $ty {
            pub trait SampleExt {
                fn base_percentiles(&self) -> ::stats::univariate::Percentiles<$ty>
                where
                    usize: ::cast::From<$ty, Output = Result<usize, ::cast::Error>>;

                fn iqr(&self) -> $ty
                where
                    usize: ::cast::From<$ty, Output = Result<usize, ::cast::Error>>,
                {
                    self.base_percentiles().iqr()
                }

                fn median(&self) -> $ty
                where
                    usize: ::cast::From<$ty, Output = Result<usize, ::cast::Error>>,
                {
                    self.base_percentiles().median()
                }
            }
            impl SampleExt for ::stats::univariate::Sample<$ty> {
                fn base_percentiles(&self) -> ::stats::univariate::Percentiles<$ty>
                where
                    usize: ::cast::From<$ty, Output = Result<usize, ::cast::Error>>,
                {
                    self.percentiles()
                }
            }

            use criterion::Criterion;

            stat!(
                $ty <- iqr,
                max,
                mean,
                median,
                median_abs_dev_pct,
                min,
                std_dev_pct,
                sum
            );
            stat_none!($ty <- median_abs_dev, std_dev, var);

            criterion_group!{
                name = benches;
                config = ::common_bench::reduced_samples();
                targets = iqr, max, mean, median, median_abs_dev_pct, min,
                            std_dev_pct, sum, median_abs_dev, std_dev, var
            }

            pub mod fast {
                use super::SampleExt;
                use criterion::Criterion;

                fast_stat!(
                    $ty <- (median_abs_dev, median),
                    (std_dev, mean),
                    (var, mean)
                );
                criterion_group!{
                    name = benches;
                    config = ::common_bench::reduced_samples();
                    targets = median_abs_dev, std_dev, var
                }
            }
        }
    };
}

bench!(f64);

criterion_main!(f64::benches, f64::fast::benches);
