#![deny(warnings)]
#![feature(overloaded_calls, phase, slicing_syntax, tuple_indexing)]

extern crate parallel;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate serialize;
// FIXME Drop dependency on the `test` crate
extern crate test;

pub use ci::ConfidenceInterval;
pub use sample::Sample;
pub use stats::{mean, median, median_abs_dev, std_dev, t, var};

pub mod kde;
pub mod outliers;
pub mod regression;
pub mod ttest;

mod ci;
mod resamples;
mod sample;
mod stats;
#[cfg(test)]
mod tol;

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
        use test::stats::Stats;

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
        std_dev(self.as_slice())
    }
}
