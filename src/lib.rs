#![allow(unused_features)]
#![deny(warnings)]
#![feature(collections)]
#![feature(core)]
#![feature(os)]
#![feature(plugin)]
#![feature(rand)]
#![feature(std_misc)]
#![feature(test)]
#![feature(unboxed_closures)]

extern crate parallel;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[plugin]
extern crate quickcheck_macros;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;
#[cfg(test)]
extern crate "test" as std_test;

use std::num::{Float, FromPrimitive};

pub use bootstrap::bootstrap;
pub use ci::ConfidenceInterval;

pub mod kde;
pub mod outliers;
pub mod regression;
pub mod ttest;

mod bootstrap;
mod ci;
mod resamples;
mod simd;
mod stats;
#[cfg(test)]
mod test;

/// [T] extension trait that provides the `bootstrap` method
pub trait Bootstrap {
    /// Returns the bootstrap distribution of the parameter estimated by the 1-sample statistic
    ///
    /// * Bootstrap method: Case resampling
    fn bootstrap<A, S>(&self, statistic: S, nresamples: usize) -> Distribution<A> where
        A: Send,
        S: Fn(&Self) -> A + Sync;
}

/// The bootstrap distribution of a population parameter
pub struct Distribution<A>(Vec<A>);

impl<A> Distribution<A> {
    /// Returns an slice to the data points of the distribution
    pub fn as_slice(&self) -> &[A] {
        &*self.0
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

impl<A> Distribution<A> where A: Simd {
    /// Computes the confidence interval of the population parameter using percentiles
    // TODO Add more methods to find the confidence interval (e.g. with bias correction)
    pub fn confidence_interval(&self, confidence_level: A) -> ConfidenceInterval<A> {
        use std::num;

        assert!(confidence_level > Float::zero() && confidence_level < Float::one());

        let percentiles = self.as_slice().percentiles();

        let _1: A = Float::one();
        let fifty = num::cast::<f64, A>(50.).unwrap();

        ConfidenceInterval {
            confidence_level: confidence_level,
            lower_bound: percentiles.at(fifty * (_1 - confidence_level)),
            upper_bound: percentiles.at(fifty * (_1 + confidence_level)),
        }
    }

    /// Computes the standard error of the population parameter
    pub fn standard_error(&self) -> A {
        self.as_slice().std_dev(None)
    }
}

/// SIMD accelerated statistics
// XXX T should be an associated type (?)
pub trait Stats<T> where T: Simd {
    /// Returns the biggest element in the sample
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty
    fn max(&self) -> T;

    /// Returns the arithmetic average of the sample
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty
    fn mean(&self) -> T;

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
    fn median_abs_dev(&self, median: Option<T>) -> T;

    /// Returns the median absolute deviation as a percentage of the median
    ///
    /// - Time: `O(length)`
    /// - Memory: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty or if the sample contains NaN
    fn median_abs_dev_pct(&self) -> T;

    /// Returns the smallest element in the sample
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample is empty
    fn min(&self) -> T;

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
    fn percentiles(&self) -> Percentiles<T>;

    /// Returns the standard deviation of the sample
    ///
    /// The `mean` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample contains less than 2 elements
    fn std_dev(&self, mean: Option<T>) -> T;

    /// Returns the standard deviation as a percentage of the mean
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample contains less than 2 elements
    fn std_dev_pct(&self) -> T;

    /// Returns the sum of all the elements of the sample
    ///
    /// - Time: `O(length)`
    fn sum(&self) -> T;

    /// Returns the t score between these two samples
    fn t(&self, other: &Self) -> T;

    /// Returns the variance of the sample
    ///
    /// The `mean` can be optionally passed along to speed up (2X) the computation
    ///
    /// - Time: `O(length)`
    ///
    /// # Panics
    ///
    /// Panics if the sample contains less than 2 elements
    fn var(&self, mean: Option<T>) -> T;

    #[cfg(test)]
    fn iqr(&self) -> T { self.percentiles().iqr() }

    #[cfg(test)]
    fn median(&self) -> T { self.percentiles().median() }

    #[cfg(test)]
    fn quartiles(&self) -> (T, T, T) { self.percentiles().quartiles() }
}

/// Types than can be SIMD accelerated
///
/// *Note* You shouldn't use these methods, instead use the methods under the `Stats` trait
pub trait Simd: Float + FromPrimitive {
    fn sum(sample: &[Self]) -> Self;
    fn var(sample: &[Self], mean: Option<Self>) -> Self;
}

/// A "view" into the percentiles of a sample
pub struct Percentiles<T>(Vec<T>);

impl<T> Percentiles<T> where T: Float + FromPrimitive {
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

trait Sum<T> {
    fn sum(self) -> T;
}

impl<T, I> Sum<T> for I where T: Float, I: Iterator<Item=T> {
    fn sum(self) -> T {
        self.fold(Float::zero(), |s, x| x + s)
    }
}
