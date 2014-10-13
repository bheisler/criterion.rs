use std::iter::AdditiveIterator;
use std::num;
use std::raw::{Repr, mod};
use std::simd::{f32x4, f64x2};

use Stats;

/// Non accelerated version of Stats::t
pub fn t<A: FloatMath + FromPrimitive>(x: &[A], y: &[A]) -> A {
    use std_test::stats::Stats;

    let (x_bar, y_bar) = (x.mean(), y.mean());
    let (s2_x, s2_y) = (x.var(), y.var());
    let (n_x, n_y) = (num::cast::<_, A>(x.len()).unwrap(), num::cast::<_, A>(y.len()).unwrap());

    let num = x_bar - y_bar;
    let den = (s2_x / n_x + s2_y / n_y).sqrt();

    num / den
}

impl<'a> Stats<f32> for &'a [f32] {
    fn median_abs_dev(self, median: Option<f32>) -> f32 {
        static FACTOR: f32 = 1.4826;

        let median = median.unwrap_or_else(|| self.percentiles().median());
        // NB Although this operation can be SIMD accelerated, the gain is negligible because the
        // bottle neck is the sorting operation which is part of the computation of the median
        let abs_devs = self.iter().map(|&x| (x - median).abs()).collect::<Vec<_>>();

        abs_devs[].percentiles().median() * FACTOR
    }

    fn sum(self) -> f32 {
        let raw::Slice { data, len } = self.repr();

        if len < 8 {
            self.iter().map(|&x| x).sum()
        } else {
            let data = data as *const f32x4;

            let mut sum = unsafe { *data };
            for i in range(1, (len / 4) as int) {
                sum += unsafe { *data.offset(i) };
            }

            let tail = self.iter().rev().take(len % 4).map(|&x| x).sum();

            sum.0 + sum.1 + sum.2 + sum.3 + tail
        }
    }

    fn var(self, mean: Option<f32>) -> f32 {
        let raw::Slice { data, len } = self.repr();

        assert!(len > 1);

        let mean = mean.unwrap_or_else(|| self.mean());
        let squared_deviation = |&x: &f32| {
            let diff = x - mean;
            diff * diff
        };

        let sum = if len < 8 {
            self.iter().map(squared_deviation).sum()
        } else {
            let data = data as *const f32x4;

            let mean4 = f32x4(mean, mean, mean, mean);
            let mut sum = f32x4(0., 0., 0., 0.);
            for i in range(0, (len / 4) as int) {
                let diff = unsafe { *data.offset(i) } - mean4;
                sum += diff * diff;
            }

            let tail = self.iter().rev().take(len % 4).map(squared_deviation).sum();

            sum.0 + sum.1 + sum.2 + sum.3 + tail
        };

        sum / (len - 1) as f32
    }
}

impl<'a> Stats<f64> for &'a [f64] {
    fn median_abs_dev(self, median: Option<f64>) -> f64 {
        static FACTOR: f64 = 1.4826;

        let median = median.unwrap_or_else(|| self.percentiles().median());
        // NB Although this operation can be SIMD accelerated, the gain is negligible because the
        // bottle neck is the sorting operation which is part of the computation of the median
        let abs_devs = self.iter().map(|&x| (x - median).abs()).collect::<Vec<_>>();

        abs_devs[].percentiles().median() * FACTOR
    }

    fn sum(self) -> f64 {
        let raw::Slice { data, len } = self.repr();

        if len < 4 {
            self.iter().map(|&x| x).sum()
        } else {
            let data = data as *const f64x2;

            let mut sum = unsafe { *data };
            for i in range(1, (len / 2) as int) {
                sum += unsafe { *data.offset(i) };
            }

            let tail = self.iter().rev().take(len % 2).map(|&x| x).sum();

            sum.0 + sum.1 + tail
        }
    }

    fn var(self, mean: Option<f64>) -> f64 {
        let raw::Slice { data, len } = self.repr();

        assert!(len > 1);

        let mean = mean.unwrap_or_else(|| self.mean());
        let squared_deviation = |&x: &f64| {
            let diff = x - mean;
            diff * diff
        };

        let sum = if len < 4 {
            self.iter().map(squared_deviation).sum()
        } else {
            let data = data as *const f64x2;

            let mean2 = f64x2(mean, mean);
            let mut sum = f64x2(0., 0.);
            for i in range(0, (len / 2) as int) {
                let diff = unsafe { *data.offset(i) } - mean2;
                sum += diff * diff;
            }

            let tail = self.iter().rev().take(len % 2).map(squared_deviation).sum();

            sum.0 + sum.1 + tail
        };

        sum / (len - 1) as f64
    }
}

#[cfg(test)]
mod test {
    macro_rules! stat {
        ($ty:ident <- $($stat:ident),+) => {$(
            #[quickcheck]
            fn $stat(size: uint) -> TestResult {
                if let Some(v) = ::test::vec::<$ty>(size) {
                    let lhs = {
                        use Stats;

                        v[].$stat()
                    };
                    let rhs = {
                        use std_test::stats::Stats;

                        v[].$stat()
                    };

                    TestResult::from_bool(lhs.approx_eq(rhs))
                } else {
                    TestResult::discard()
                }
            }
       )+}
    }

    macro_rules! stat_none {
        ($ty:ident <- $($stat:ident),+) => {$(
            #[quickcheck]
            fn $stat(size: uint) -> TestResult {
                if let Some(v) = ::test::vec::<$ty>(size) {
                    let lhs = {
                        use Stats;

                        v[].$stat(None)
                    };
                    let rhs = {
                        use std_test::stats::Stats;

                        v[].$stat()
                    };

                    TestResult::from_bool(lhs.approx_eq(rhs))
                } else {
                    TestResult::discard()
                }
            }
       )+}
    }

    macro_rules! fast_stat {
        ($ty:ident <- $(($stat:ident, $aux_stat:ident)),+) => {$(
            #[quickcheck]
            fn $stat(size: uint) -> TestResult {
                if let Some(v) = ::test::vec::<$ty>(size) {
                    let lhs = {
                        use Stats;

                        v[].$stat(Some(v[].$aux_stat()))
                    };
                    let rhs = {
                        use std_test::stats::Stats;

                        v[].$stat()
                    };

                    TestResult::from_bool(lhs.approx_eq(rhs))
                } else {
                    TestResult::discard()
                }
            }
       )+}
    }

    macro_rules! test {
        ($($ty:ident),+) => {$(
            mod $ty {
                extern crate test;

                use quickcheck::TestResult;

                use test::ApproxEq;

                stat!($ty <- iqr, max, mean, median, median_abs_dev_pct, min, quartiles,
                        std_dev_pct, sum)
                stat_none!($ty <- median_abs_dev, std_dev, var)

                mod fast {
                    extern crate test;

                    use quickcheck::TestResult;

                    use test::ApproxEq;

                    fast_stat!($ty <- (median_abs_dev, median), (std_dev, mean), (var, mean))
                }
            }
      )+}
    }

    test!(f32, f64)
}

#[cfg(test)]
mod bench {
    macro_rules! stat {
        ($ty:ident <- $($stat:ident),+) => {$(
            #[bench]
            fn $stat(b: &mut Bencher) {
                let v = ::test::vec::<$ty>(BENCH_SIZE).unwrap();
                let s = v[];

                b.iter(|| s.$stat());
            })+
        }
    }

    macro_rules! stat_none {
        ($ty:ident <- $($stat:ident),+) => {$(
            #[bench]
            fn $stat(b: &mut Bencher) {
                let v = ::test::vec::<$ty>(BENCH_SIZE).unwrap();
                let s = v[];

                b.iter(|| s.$stat(None));
            })+
        }
    }

    macro_rules! fast_stat {
        ($ty:ident <- $(($stat:ident, $aux_stat:ident)),+) => {$(
            #[bench]
            fn $stat(b: &mut Bencher) {
                let v = ::test::vec::<$ty>(BENCH_SIZE).unwrap();
                let s = v[];
                let aux = Some(s.$aux_stat());

                b.iter(|| s.$stat(aux));
            })+
        }
    }

    macro_rules! bench {
        ($($ty:ident),+) => {$(
            mod $ty {
                use std_test::Bencher;

                use test::BENCH_SIZE;
                use Stats;

                stat!($ty <- iqr, max, mean, median, median_abs_dev_pct, min, quartiles,
                        std_dev_pct, sum)
                stat_none!($ty <- median_abs_dev, std_dev, var)

                mod fast {
                    use std_test::Bencher;

                    use test::BENCH_SIZE;
                    use Stats;

                    fast_stat!($ty <- (median_abs_dev, median), (std_dev, mean), (var, mean))
                }

                mod std {
                    use std_test::Bencher;

                    use test::BENCH_SIZE;
                    use std_test::stats::Stats;

                    stat!($ty <- iqr, max, mean, median, median_abs_dev, median_abs_dev_pct, min,
                            quartiles, std_dev, std_dev_pct, sum, var)
                }
            }
       )+}
    }

    bench!(f32, f64)
}
