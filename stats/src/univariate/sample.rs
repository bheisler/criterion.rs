use std::{cmp, mem, ops};

use cast;
use float::Float;
use num_cpus;
use thread_scoped as thread;

use tuple::{Tuple, TupledDistributionsBuilder};
use univariate::resamples::Resamples;
use univariate::Percentiles;

/// A collection of data points drawn from a population
///
/// Invariants:
///
/// - The sample contains at least 2 data points
/// - The sample contains no `NaN`s
pub struct Sample<A>([A]);

// TODO(rust-lang/rfcs#735) move this `impl` into a private percentiles module
impl<A> Sample<A>
where
    A: Float,
{
    /// Creates a new sample from an existing slice
    ///
    /// # Panics
    ///
    /// Panics if `slice` contains any `NaN` or if `slice` has less than two elements
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::new_ret_no_self))]
    pub fn new(slice: &[A]) -> &Sample<A> {
        assert!(slice.len() > 1 && slice.iter().all(|x| !x.is_nan()));

        unsafe { mem::transmute(slice) }
    }

    /// Returns the biggest element in the sample
    ///
    /// - Time: `O(length)`
    pub fn max(&self) -> A {
        let mut elems = self.iter();

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
        let n = self.len();

        self.sum() / A::cast(n)
    }

    /// Returns the median absolute deviation
    ///
    /// The `median` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    pub fn median_abs_dev(&self, median: Option<A>) -> A
    where
        usize: cast::From<A, Output = Result<usize, cast::Error>>,
    {
        let median = median.unwrap_or_else(|| self.percentiles().median());

        // NB Although this operation can be SIMD accelerated, the gain is negligible because the
        // bottle neck is the sorting operation which is part of the computation of the median
        let abs_devs = self.iter().map(|&x| (x - median).abs()).collect::<Vec<_>>();

        let abs_devs: &Self = Self::new(&abs_devs);

        abs_devs.percentiles().median() * A::cast(1.4826)
    }

    /// Returns the median absolute deviation as a percentage of the median
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    pub fn median_abs_dev_pct(&self) -> A
    where
        usize: cast::From<A, Output = Result<usize, cast::Error>>,
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
        let mut elems = self.iter();

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
    pub fn percentiles(&self) -> Percentiles<A>
    where
        usize: cast::From<A, Output = Result<usize, cast::Error>>,
    {
        use std::cmp::Ordering;

        // NB This function assumes that there are no `NaN`s in the sample
        fn cmp<T>(a: &T, b: &T) -> Ordering
        where
            T: PartialOrd,
        {
            if a < b {
                Ordering::Less
            } else if a == b {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }

        let mut v = self.to_vec().into_boxed_slice();
        v.sort_by(cmp);

        // NB :-1: to intra-crate privacy rules
        unsafe { mem::transmute(v) }
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
        ::sum(self)
    }

    /// Returns the t score between these two samples
    ///
    /// - Time: `O(length)`
    pub fn t(&self, other: &Sample<A>) -> A {
        let (x_bar, y_bar) = (self.mean(), other.mean());
        let (s2_x, s2_y) = (self.var(Some(x_bar)), other.var(Some(y_bar)));
        let n_x = A::cast(self.len());
        let n_y = A::cast(other.len());
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
        let slice = self;

        let sum = slice
            .iter()
            .map(|&x| (x - mean).powi(2))
            .fold(A::cast(0), Add::add);

        sum / A::cast(slice.len() - 1)
    }

    // TODO Remove the `T` parameter in favor of `S::Output`
    /// Returns the bootstrap distributions of the parameters estimated by the 1-sample statistic
    ///
    /// - Multi-threaded
    /// - Time: `O(nresamples)`
    /// - Memory: `O(nresamples)`
    pub fn bootstrap<T, S>(&self, nresamples: usize, statistic: S) -> T::Distributions
    where
        S: Fn(&Sample<A>) -> T,
        S: Sync,
        T: Tuple,
        T: Send,
        T::Distributions: Send,
        T::Builder: Send,
    {
        let ncpus = num_cpus::get();

        unsafe {
            // TODO need some sensible threshold to trigger the multi-threaded path
            if ncpus > 1 && nresamples > self.len() {
                let granularity = nresamples / ncpus + 1;
                let statistic = &statistic;

                let chunks = (0..ncpus)
                    .map(|i| {
                        // for now I'll make do with aliasing and careful non-overlapping indexing
                        let mut sub_distributions: T::Builder =
                            TupledDistributionsBuilder::new(granularity);
                        let mut resamples = Resamples::new(self);
                        let offset = i * granularity;

                        thread::scoped(move || {
                            for _ in offset..cmp::min(offset + granularity, nresamples) {
                                sub_distributions.push(statistic(resamples.next()))
                            }
                            sub_distributions
                        })
                    })
                    .collect::<Vec<_>>();

                let mut builder: T::Builder = TupledDistributionsBuilder::new(nresamples);
                for chunk in chunks {
                    builder.extend(&mut (chunk.join()));
                }
                builder.complete()
            } else {
                let mut builder: T::Builder = TupledDistributionsBuilder::new(nresamples);
                let mut resamples = Resamples::new(self);

                for _ in 0..nresamples {
                    builder.push(statistic(resamples.next()));
                }

                builder.complete()
            }
        }
    }

    #[cfg(test)]
    pub fn iqr(&self) -> A
    where
        usize: cast::From<A, Output = Result<usize, cast::Error>>,
    {
        self.percentiles().iqr()
    }

    #[cfg(test)]
    pub fn median(&self) -> A
    where
        usize: cast::From<A, Output = Result<usize, cast::Error>>,
    {
        self.percentiles().median()
    }
}

impl<A> ops::Deref for Sample<A> {
    type Target = [A];

    fn deref(&self) -> &[A] {
        &self.0
    }
}
