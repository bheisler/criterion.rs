use std::ptr::Unique;
use std::{cmp, mem};

use cast;
use floaty::Floaty;
use num_cpus;
use thread_scoped as thread;

use tuple::{Tuple, TupledDistributions};
use univariate::Percentiles;
use univariate::resamples::Resamples;

/// A collection of data points drawn from a population
///
/// Invariants:
///
/// - The sample contains at least 2 data points
/// - The sample contains no `NaN`s
pub struct Sample<A>([A]);

impl<A> Sample<A> {
    /// Returns a slice that contains all the data points
    ///
    /// NOTE: This will be removed in favor of a `Deref<Target=[A]>` implementation once
    /// [rust-lang/rust#22257] is fixed.
    ///
    /// [rust-lang/rust#22257]: https://github.com/rust-lang/rust/issues/22257
    pub fn as_slice(&self) -> &[A] {
        unsafe {
            mem::transmute(self)
        }
    }

}

// TODO(rust-lang/rfcs#735) move this `impl` into a private percentiles module
impl<A> Sample<A> where A: Floaty {
    /// Creates a new sample from an existing slice
    ///
    /// # Panics
    ///
    /// Panics if `slice` contains any `NaN` or if `slice` has less than two elements
    pub fn new(slice: &[A]) -> &Sample<A> {
        assert!(
            slice.len() > 1 &&
            slice.iter().all(|x| !x.is_nan())
        );

        unsafe {
            mem::transmute(slice)
        }
    }

    /// Returns the biggest element in the sample
    ///
    /// - Time: `O(length)`
    pub fn max(&self) -> A {
        let mut elems = self.as_slice().iter();

        match elems.next() {
            Some(&head) => elems.fold(head, |a, &b| a.max(b)),
            // NB `unreachable!` because `Sample` is guaranteed to have at least one data point
            None => unreachable!(),
        }
    }

    /// Returns the arithmetic average of the sample
    ///
    /// - Time: `O(length)`
    pub fn mean(&self) -> A {
        let n = self.as_slice().len();

        self.sum() / A::cast(n)
    }

    /// Returns the median absolute deviation
    ///
    /// The `median` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    pub fn median_abs_dev(&self, median: Option<A>) -> A where
        usize: cast::From<A, Output=Result<usize, cast::Error>>,
    {
        let median = median.unwrap_or_else(|| self.percentiles().median());

        // NB Although this operation can be SIMD accelerated, the gain is negligible because the
        // bottle neck is the sorting operation which is part of the computation of the median
        let abs_devs = self.as_slice().iter().map(|&x| (x - median).abs()).collect::<Vec<_>>();

        let abs_devs: &Sample<A> = unsafe {
            mem::transmute(&*abs_devs)
        };

        abs_devs.percentiles().median() * A::cast(1.4826)
    }

    /// Returns the median absolute deviation as a percentage of the median
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    pub fn median_abs_dev_pct(&self) -> A where
        usize: cast::From<A, Output=Result<usize, cast::Error>>,
    {
        let _100 = A::cast(100);
        let median = self.percentiles().median();
        let mad = self.median_abs_dev(Some(median));

        (mad / median) * _100
    }

    /// Returns the smallest element in the sample
    ///
    /// - Time: `O(length)`
    pub fn min(&self) -> A {
        let mut elems = self.as_slice().iter();

        match elems.next() {
            Some(&elem) => elems.fold(elem, |a, &b| a.min(b)),
            // NB `unreachable!` because `Sample` is guaranteed to have at least one data point
            None => unreachable!(),
        }
    }

    /// Returns a "view" into the percentiles of the sample
    ///
    /// This "view" makes consecutive computations of percentiles much faster (`O(1)`)
    ///
    /// - Time: `O(N log N) where N = length`
    /// - Memory: `O(length)`
    pub fn percentiles(&self) -> Percentiles<A> where
        usize: cast::From<A, Output=Result<usize, cast::Error>>,
    {
        use std::cmp::Ordering;

        // NB This function assumes that there are no `NaN`s in the sample
        fn cmp<T>(a: &T, b: &T) -> Ordering where T: PartialOrd {
            if a < b {
                Ordering::Less
            } else if a == b {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }

        let mut v = self.as_slice().to_vec().into_boxed_slice();
        v.sort_by(cmp);

        // NB :-1: to intra-crate privacy rules
        unsafe {
            mem::transmute(v)
        }
    }

    /// Returns the standard deviation of the sample
    ///
    /// The `mean` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    pub fn std_dev(&self, mean: Option<A>) -> A {
        self.var(mean).sqrt()
    }

    /// Returns the standard deviation as a percentage of the mean
    ///
    /// - Time: `O(length)`
    pub fn std_dev_pct(&self) -> A {
        let _100 = A::cast(100);
        let mean = self.mean();
        let std_dev = self.std_dev(Some(mean));

        (std_dev / mean) * _100
    }

    /// Returns the sum of all the elements of the sample
    ///
    /// - Time: `O(length)`
    pub fn sum(&self) -> A {
        ::sum(self.as_slice())
    }

    /// Returns the t score between these two samples
    ///
    /// - Time: `O(length)`
    pub fn t(&self, other: &Sample<A>) -> A {
        let (x_bar, y_bar) = (self.mean(), other.mean());
        let (s2_x, s2_y) = (self.var(Some(x_bar)), other.var(Some(y_bar)));
        let n_x = A::cast(self.as_slice().len());
        let n_y = A::cast(other.as_slice().len());
        let num = x_bar - y_bar;
        let den = (s2_x / n_x + s2_y / n_y).sqrt();

        num / den
    }

    /// Returns the variance of the sample
    ///
    /// The `mean` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    pub fn var(&self, mean: Option<A>) -> A {
        use std::ops::Add;

        let mean = mean.unwrap_or_else(|| self.mean());
        let slice = self.as_slice();

        let sum = slice.iter().map(|&x| (x - mean).powi(2)).fold(A::cast(0), Add::add);

        sum / A::cast(slice.len() - 1)
    }

    // TODO Remove the `T` parameter in favor of `S::Output`
    /// Returns the bootstrap distributions of the parameters estimated by the 1-sample statistic
    ///
    /// - Multi-threaded
    /// - Time: `O(nresamples)`
    /// - Memory: `O(nresamples)`
    pub fn bootstrap<T, S>(
        &self,
        nresamples: usize,
        statistic: S,
    ) -> T::Distributions where
        S: Fn(&Sample<A>) -> T,
        S: Sync,
        T: Tuple,
        T::Distributions: Send,
    {
        let ncpus = num_cpus::get();

        unsafe {
            // TODO need some sensible threshold to trigger the multi-threaded path
            if ncpus > 1 && nresamples > self.as_slice().len() {
                let granularity = nresamples / ncpus + 1;
                let statistic = &statistic;
                let mut distributions: T::Distributions =
                    TupledDistributions::uninitialized(nresamples);

                let _ = (0..ncpus).map(|i| {
                    // NB Can't implement `chunks_mut` for the tupled distributions without HKT,
                    // for now I'll make do with aliasing and careful non-overlapping indexing
                    let mut ptr = Unique::new_unchecked(&mut distributions);
                    let mut resamples = Resamples::new(self);
                    let offset = i * granularity;

                    thread::scoped(move || {
                        let distributions: &mut T::Distributions = ptr.as_mut();

                        for i in offset..cmp::min(offset + granularity, nresamples) {
                            distributions.set_unchecked(i, statistic(resamples.next()))
                        }
                    })
                }).collect::<Vec<_>>();

                distributions
            } else {
                let mut distributions: T::Distributions =
                    TupledDistributions::uninitialized(nresamples);
                let mut resamples = Resamples::new(self);

                for i in 0..nresamples {
                    distributions.set_unchecked(i, statistic(resamples.next()));
                }

                distributions
            }
        }
    }

    #[cfg(test)]
    pub fn iqr(&self) -> A where
        usize: cast::From<A, Output=Result<usize, cast::Error>>,
    {
        self.percentiles().iqr()
    }

    #[cfg(test)]
    pub fn median(&self) -> A where
        usize: cast::From<A, Output=Result<usize, cast::Error>>,
    {
        self.percentiles().median()
    }
}

// FIXME(rust-lang/rust#22257) Using this generates ICEs
//impl<A> Deref for Sample<A> where A: Data {
    //type Target = [A];

    //fn deref(&self) -> &[A] {
        //unsafe {
            //mem::transmute(self)
        //}
    //}
//}

#[cfg(test)]
mod test {
    macro_rules! stat {
        ($ty:ident <- $($stat:ident),+) => {
            $(
                #[quickcheck]
                fn $stat(size: usize, start: usize) -> TestResult {
                    if let Some(v) = ::test::vec::<$ty>(size, start) {
                        let slice = &v[start..];
                        let lhs = ::univariate::Sample::new(slice).$stat();
                        let rhs = slice.$stat();

                        TestResult::from_bool(relative_eq!(lhs, rhs, max_relative = 1e-5))
                    } else {
                        TestResult::discard()
                    }
                }
            )+
        }
    }

    macro_rules! stat_none {
        ($ty:ident <- $($stat:ident),+) => {
            $(
                #[quickcheck]
                fn $stat(size: usize, start: usize) -> TestResult {
                    if let Some(v) = ::test::vec::<$ty>(size, start) {
                        let slice = &v[start..];
                        let lhs = ::univariate::Sample::new(slice).$stat(None);
                        let rhs = slice.$stat();

                        TestResult::from_bool(relative_eq!(lhs, rhs, max_relative = 1e-5))
                    } else {
                        TestResult::discard()
                    }
                }
            )+
        }
    }

    macro_rules! fast_stat {
        ($ty:ident <- $(($stat:ident, $aux_stat:ident)),+) => {
            $(
                #[quickcheck]
                fn $stat(size: usize, start: usize) -> TestResult {
                    if let Some(v) = ::test::vec::<$ty>(size, start) {
                        let slice = &v[start..];
                        let lhs = {
                            let s = ::univariate::Sample::new(slice);

                            s.$stat(Some(s.$aux_stat()))
                        };
                        let rhs = slice.$stat();

                        TestResult::from_bool(relative_eq!(lhs, rhs, max_relative = 1e-5))
                    } else {
                        TestResult::discard()
                    }
                }
            )+
        }
    }

    macro_rules! test {
        ($ty:ident) => {
            mod $ty {
                use stdtest::stats::Stats;

                use quickcheck::TestResult;

                stat!($ty <- iqr, max, mean, median, median_abs_dev_pct, min,
                        std_dev_pct, sum);
                stat_none!($ty <- median_abs_dev, std_dev, var);

                mod fast {
                    use stdtest::stats::Stats;

                    use quickcheck::TestResult;

                    fast_stat!($ty <- (median_abs_dev, median), (std_dev, mean), (var, mean));
                }
            }
        }
    }

    test!(f64);
}

#[cfg(test)]
mod bench {
    macro_rules! std_stat {
        ($ty:ident <- $($stat:ident),+) => {
            $(
                #[bench]
                fn $stat(b: &mut Bencher) {
                    let v = ::bench::vec::<$ty>();

                    b.iter(|| v.$stat());
                }
            )+
        }
    }

    macro_rules! stat {
        ($ty:ident <- $($stat:ident),+) => {
            $(
                #[bench]
                fn $stat(b: &mut Bencher) {
                    let v = ::bench::vec::<$ty>();
                    let s = ::univariate::Sample::new(&v);

                    b.iter(|| s.$stat());
                }
            )+
        }
    }

    macro_rules! stat_none {
        ($ty:ident <- $($stat:ident),+) => {
            $(
                #[bench]
                fn $stat(b: &mut Bencher) {
                    let v = ::bench::vec::<$ty>();
                    let s = ::univariate::Sample::new(&v);

                    b.iter(|| s.$stat(None));
                }
            )+
        }
    }

    macro_rules! fast_stat {
        ($ty:ident <- $(($stat:ident, $aux_stat:ident)),+) => {
            $(
                #[bench]
                fn $stat(b: &mut Bencher) {
                    let v = ::bench::vec::<$ty>();
                    let s = ::univariate::Sample::new(&v);
                    let aux = Some(s.$aux_stat());

                    b.iter(|| s.$stat(aux));
                }
            )+
        }
    }

    macro_rules! bench {
        ($ty:ident) => {
            mod $ty {
                use stdtest::Bencher;

                stat!($ty <- iqr, max, mean, median, median_abs_dev_pct, min, std_dev_pct, sum);
                stat_none!($ty <- median_abs_dev, std_dev, var);

                mod fast {
                    use stdtest::Bencher;

                    fast_stat!($ty <- (median_abs_dev, median), (std_dev, mean), (var, mean));
                }

                mod std {
                    use stdtest::Bencher;
                    use stdtest::stats::Stats;

                    std_stat!($ty <- iqr, max, mean, median, median_abs_dev, median_abs_dev_pct,
                              min, std_dev, std_dev_pct, sum, var);
                }
            }
        }
    }

    bench!(f64);
}
