use std::iter::AdditiveIterator;
use std::num::FromPrimitive;
use std::cmp::Ordering::{self, Equal, Greater, Less};

use {Percentiles, Simd, Stats};

static EMPTY_MSG: &'static str = "sample is empty";

impl<T> Stats<T> for [T] where T: Simd {
    fn max(&self) -> T {
        let mut elems = self.iter();

        match elems.next() {
            Some(&head) => elems.fold(head, |a, &b| a.max(b)),
            None => panic!(EMPTY_MSG),
        }
    }

    fn mean(&self) -> T {
        let n = self.len();

        assert!(n > 0);

        self.sum() / FromPrimitive::from_uint(n).unwrap()
    }

    fn median_abs_dev(&self, median: Option<T>) -> T {
        let median = median.unwrap_or_else(|| self.percentiles().median());
        // NB Although this operation can be SIMD accelerated, the gain is negligible because the
        // bottle neck is the sorting operation which is part of the computation of the median
        let abs_devs = self.iter().map(|&x| (x - median).abs()).collect::<Vec<_>>();

        abs_devs[].percentiles().median() * FromPrimitive::from_f64(1.4826).unwrap()
    }

    fn median_abs_dev_pct(&self) -> T {
        let hundred = FromPrimitive::from_uint(100).unwrap();
        let median = self.percentiles().median();
        let mad = self.median_abs_dev(Some(median));

        (mad / median) * hundred
    }

    fn min(&self) -> T {
        let mut elems = self.iter();

        match elems.next() {
            Some(&elem) => elems.fold(elem, |a, &b| a.min(b)),
            None => panic!(EMPTY_MSG),
        }
    }

    fn percentiles(&self) -> Percentiles<T> {
        // NB This function assumes that there are no NaNs in the sample
        fn cmp<T>(a: &T, b: &T) -> Ordering where T: PartialOrd {
            if a < b {
                Less
            } else if a == b {
                Equal
            } else {
                Greater
            }
        }

        assert!(self.len() > 0 && !self.iter().any(|x| x.is_nan()));

        let mut v = self.to_vec();
        v.sort_by(|a, b| cmp(a, b));
        Percentiles(v)
    }

    fn std_dev(&self, mean: Option<T>) -> T {
        self.var(mean).sqrt()
    }

    fn std_dev_pct(&self) -> T {
        let hundred = FromPrimitive::from_uint(100).unwrap();
        let mean = self.mean();
        let std_dev = self.std_dev(Some(mean));

        (std_dev / mean) * hundred
    }

    fn sum(&self) -> T {
        Simd::sum(self)
    }

    fn t(&self, other: &[T]) -> T {
        let (x_bar, y_bar) = (self.mean(), other.mean());
        let (s2_x, s2_y) = (self.var(Some(x_bar)), other.var(Some(y_bar)));
        let n_x = FromPrimitive::from_uint(self.len()).unwrap();
        let n_y = FromPrimitive::from_uint(other.len()).unwrap();
        let num = x_bar - y_bar;
        let den = (s2_x / n_x + s2_y / n_y).sqrt();

        num / den
    }

    fn var(&self, mean: Option<T>) -> T {
        Simd::var(self, mean)
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
                        std_dev_pct, sum);
                stat_none!($ty <- median_abs_dev, std_dev, var);

                mod fast {
                    extern crate test;

                    use quickcheck::TestResult;

                    use test::ApproxEq;

                    fast_stat!($ty <- (median_abs_dev, median), (std_dev, mean), (var, mean));
                }
            }
      )+}
    }

    test!(f32, f64);
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
                        std_dev_pct, sum);
                stat_none!($ty <- median_abs_dev, std_dev, var);

                mod fast {
                    use std_test::Bencher;

                    use test::BENCH_SIZE;
                    use Stats;

                    fast_stat!($ty <- (median_abs_dev, median), (std_dev, mean), (var, mean));
                }

                mod std {
                    use std_test::Bencher;

                    use test::BENCH_SIZE;
                    use std_test::stats::Stats;

                    stat!($ty <- iqr, max, mean, median, median_abs_dev, median_abs_dev_pct, min,
                            quartiles, std_dev, std_dev_pct, sum, var);
                }
            }
       )+}
    }

    bench!(f32, f64);
}
