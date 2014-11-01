#![deny(warnings)]
#![feature(if_let, macro_rules, overloaded_calls, phase, slicing_syntax, tuple_indexing)]

extern crate parallel;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate serialize;
extern crate "test" as std_test;

pub use bootstrap::bootstrap;
pub use ci::ConfidenceInterval;
pub use stats::t;

pub mod kde;
pub mod outliers;
pub mod regression;
pub mod ttest;

mod bootstrap;
mod ci;
mod resamples;
mod stats;
#[cfg(test)]
mod test;

/// [T] extension trait that provides the `bootstrap` method
pub trait Bootstrap for Sized? {
    /// Returns the bootstrap distribution of the parameter estimated by the 1-sample statistic
    ///
    /// * Bootstrap method: Case resampling
    fn bootstrap<A: Send>(&self, statistic: fn(&Self) -> A, nresamples: uint) -> Distribution<A>;
}

/// The bootstrap distribution of a population parameter
#[experimental]
pub struct Distribution<A>(Vec<A>);

impl<A> Distribution<A> {
    /// Returns an slice to the data points of the distribution
    pub fn as_slice(&self) -> &[A] {
        self.0[]
    }

    /// Returns a vector that contains the data points of the distribution
    pub fn unwrap(self) -> Vec<A> {
        self.0
    }
}

// XXX How to make this generic or via a macro?
impl<A, B> Distribution<(A, B)> {
    pub fn split2(self) -> (Distribution<A>, Distribution<B>) {
        let n = self.0.len();
        let mut va = Vec::with_capacity(n);
        let mut vb = Vec::with_capacity(n);

        for (a, b) in self.unwrap().into_iter() {
            va.push(a);
            vb.push(b);
        }

        (Distribution(va), Distribution(vb))
    }
}

// XXX Why can't this have the same name as the previous method?
impl<A, B, C, D> Distribution<(A, B, C, D)> {
    pub fn split4(self) -> (Distribution<A>, Distribution<B>, Distribution<C>, Distribution<D>) {
        let n = self.0.len();
        let mut va = Vec::with_capacity(n);
        let mut vb = Vec::with_capacity(n);
        let mut vc = Vec::with_capacity(n);
        let mut vd = Vec::with_capacity(n);

        for (a, b, c, d) in self.unwrap().into_iter() {
            va.push(a);
            vb.push(b);
            vc.push(c);
            vd.push(d);
        }

        (Distribution(va), Distribution(vb), Distribution(vc), Distribution(vd))
    }
}

impl<A: FromPrimitive + FloatMath> Distribution<A> {
    /// Computes the confidence interval of the population parameter using percentiles
    // TODO Add more methods to find the confidence interval (e.g. with bias correction)
    pub fn confidence_interval(&self, confidence_level: A) -> ConfidenceInterval<A> {
        use std::num;
        use std_test::stats::Stats;

        assert!(confidence_level > num::zero() && confidence_level < num::one());

        let distribution = self.as_slice();

        let one = num::one::<A>();
        let fifty = num::cast::<f64, A>(50.).unwrap();

        ConfidenceInterval {
            confidence_level: confidence_level,
            lower_bound: distribution.percentile(fifty * (one - confidence_level)),
            upper_bound: distribution.percentile(fifty * (one + confidence_level)),
        }
    }

    /// Computes the standard error of the population parameter
    pub fn standard_error(&self) -> A {
        use std_test::stats::Stats;

        self.as_slice().std_dev()
    }
}

static EMPTY_MSG: &'static str = "sample is empty";

/// SIMD accelerated statistics
// XXX T should be an associated type (?)
pub trait Stats<T: FloatMath + FromPrimitive>: AsSlice<T> + Copy {
    /// Returns the biggest element in the sample
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty
    fn max(self) -> T {
        let mut elems = self.as_slice().iter();

        match elems.next() {
            Some(&elem) => elems.fold(elem, |a, &b| a.max(b)),
            None => panic!(EMPTY_MSG),
        }
    }

    /// Returns the arithmetic average of the sample
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty
    fn mean(self) -> T {
        let len = self.as_slice().len();

        assert!(len > 0);

        self.sum() / FromPrimitive::from_uint(len).unwrap()
    }

    /// Returns the median absolute deviation
    ///
    /// The `median` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty or if the sample contains NaN
    fn median_abs_dev(self, median: Option<T>) -> T;

    /// Returns the median absolute deviation as a percentage of the median
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty or if the sample contains NaN
    fn median_abs_dev_pct(self) -> T {
        let hundred = FromPrimitive::from_uint(100).unwrap();
        let median = self.percentiles().median();
        let mad = self.median_abs_dev(Some(median));

        (mad / median) * hundred
    }

    /// Returns the smallest element in the sample
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty
    fn min(self) -> T {
        let mut elems = self.as_slice().iter();

        match elems.next() {
            Some(&elem) => elems.fold(elem, |a, &b| a.min(b)),
            None => panic!(EMPTY_MSG),
        }
    }

    /// Returns a "view" into the percentiles of the sample
    ///
    /// This "view" makes the consecutive computation of percentiles much faster
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty or if the sample contains NaN
    fn percentiles(self) -> Percentiles<T> {
        // NB This function assumes that there are no NaNs in the sample
        fn cmp<T: PartialOrd>(a: &T, b: &T) -> Ordering {
            if a < b {
                Less
            } else if a == b {
                Equal
            } else {
                Greater
            }
        }

        let slice = self.as_slice();

        assert!(slice.len() > 0 && !slice.iter().any(|x| x.is_nan()));

        let mut v = slice.to_vec();
        v[mut].sort_by(|a, b| cmp(a, b));
        Percentiles(v)
    }

    /// Returns the standard deviation of the sample
    ///
    /// The `mean` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample contains less than 2 elements
    fn std_dev(self, mean: Option<T>) -> T {
        self.var(mean).sqrt()
    }

    /// Returns the standard deviation as a percentage of the mean
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample contains less than 2 elements
    fn std_dev_pct(self) -> T {
        let hundred = FromPrimitive::from_uint(100).unwrap();
        let mean = self.mean();
        let std_dev = self.std_dev(Some(mean));

        (std_dev / mean) * hundred
    }

    /// Returns the sum of all the elements of the sample
    ///
    /// - Time: `O(length)`
    fn sum(self) -> T;

    /// Returns the t score between these two samples
    fn t(self, other: Self) -> T {
        use std::num;

        let (x_bar, y_bar) = (self.mean(), other.mean());
        let (s2_x, s2_y) = (self.var(Some(x_bar)), other.var(Some(y_bar)));
        let n_x = num::cast::<_, T>(self.as_slice().len()).unwrap();
        let n_y = num::cast::<_, T>(other.as_slice().len()).unwrap();
        let num = x_bar - y_bar;
        let den = (s2_x / n_x + s2_y / n_y).sqrt();

        num / den
    }

    /// Returns the variance of the sample
    ///
    /// The `mean` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample contains less than 2 elements
    fn var(self, mean: Option<T>) -> T;

    #[cfg(test)]
    fn iqr(self) -> T { self.percentiles().iqr() }

    #[cfg(test)]
    fn median(self) -> T { self.percentiles().median() }

    #[cfg(test)]
    fn quartiles(self) -> (T, T, T) { self.percentiles().quartiles() }
}

/// A "view" into the percentiles of a sample
pub struct Percentiles<T: FloatMath + FromPrimitive>(Vec<T>);

impl<T: FloatMath + FromPrimitive> Percentiles<T> {
    /// Returns the percentile at `p`%
    pub fn at(&self, p: T) -> T {
        let zero = FromPrimitive::from_uint(0).unwrap();
        let hundred = FromPrimitive::from_uint(100).unwrap();

        assert!(p >= zero && p <= hundred);

        let len = self.0.len() - 1;

        if len == 0 {
            self.0[0]
        } else if p == hundred {
            self.0[len]
        } else {
            let rank = (p / hundred) * FromPrimitive::from_uint(len).unwrap();
            let integer = rank.floor();
            let fraction = rank - integer;
            let n = integer.to_uint().unwrap();
            let floor = self.0[n];
            let ceiling = self.0[n + 1];

            floor + (ceiling - floor) * fraction
        }
    }

    /// Returns the 50th percentile
    pub fn median(&self) -> T {
        self.at(FromPrimitive::from_uint(50).unwrap())
    }

    /// Returns the 25th, 50th and 75th percentiles
    pub fn quartiles(&self) -> (T, T, T) {
        (
            self.at(FromPrimitive::from_uint(25).unwrap()),
            self.at(FromPrimitive::from_uint(50).unwrap()),
            self.at(FromPrimitive::from_uint(75).unwrap()),
        )
    }

    /// Returns the interquartile range
    pub fn iqr(&self) -> T {
        let q1 = self.at(FromPrimitive::from_uint(25).unwrap());
        let q3 = self.at(FromPrimitive::from_uint(75).unwrap());

        q3 - q1
    }
}
