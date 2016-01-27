//! Statistics

#![cfg_attr(test, allow(trivial_casts))]  // quickcheck
#![cfg_attr(test, feature(test))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#![deny(missing_docs)]
//#![deny(warnings)]
#![feature(custom_attribute)]
#![feature(fn_traits)]
#![feature(plugin)]
#![feature(unboxed_closures)]
#![feature(unique)]

extern crate cast;
extern crate float;
extern crate num_cpus;
extern crate rand;
extern crate thread_scoped;

#[cfg(test)] extern crate test as stdtest;
#[cfg(test)] extern crate approx;
#[cfg(test)] extern crate quickcheck;
#[cfg(test)] extern crate space;

#[cfg(test)]
macro_rules! approx_eq {
    ($lhs:expr, $rhs:expr) => ({
        let ref lhs = $lhs;
        let ref rhs = $rhs;

        ::approx::eq(lhs, rhs, ::approx::Abs::tol(1e-5)) ||
        ::approx::eq(lhs, rhs, ::approx::Rel::tol(1e-5))
    })
}

#[cfg(test)] mod bench;
#[cfg(test)] mod test;

pub mod bivariate;
pub mod tuple;
pub mod univariate;

use std::mem;
use std::ops::Deref;

use cast::From;

use univariate::Sample;

/// A floating point number
pub trait Float: float::Float + Send + Sync {}

impl Float for f32 {}
impl Float for f64 {}

/// The bootstrap distribution of some parameter
pub struct Distribution<A>(Box<[A]>);

impl<A> Distribution<A> where A: Float {
    /// Computes the confidence interval of the population parameter using percentiles
    ///
    /// # Panics
    ///
    /// Panics if the `confidence_level` is not in the `(0, 1)` range.
    pub fn confidence_interval(&self, confidence_level: A) -> (A, A) where
        usize: From<A, Output=Option<usize>>,
    {
        let _0 = A::from(0);
        let _1 = A::from(1);
        let _50 = A::from(50);

        assert!(confidence_level > _0 && confidence_level < _1);

        let percentiles = self.percentiles();

        // FIXME(privacy) this should use the `at_unchecked()` method
        (
            percentiles.at(_50 * (_1 - confidence_level)),
            percentiles.at(_50 * (_1 + confidence_level)),
        )
    }

    /// Computes the "likelihood" of seeing the value `t` or "more extreme" values in the
    /// distribution.
    pub fn p_value(&self, t: A, tails: Tails) -> A {
        use std::cmp;

        let n = self.0.len();
        let hits = self.0.iter().filter(|&&x| x < t).count();

        let tails = A::from(match tails {
            Tails::One => 1,
            Tails::Two => 2,
        });

        A::from(cmp::min(hits, n - hits)) / A::from(n) * tails
    }
}

impl<A> Deref for Distribution<A> {
    type Target = Sample<A>;

    fn deref(&self) -> &Sample<A> {
        let slice: &[_] = &self.0;

        unsafe {
            mem::transmute(slice)
        }
    }
}

/// Number of tails for significance testing
pub enum Tails {
    /// One tailed test
    One,
    /// Two tailed test
    Two,
}

fn dot<A>(xs: &[A], ys: &[A]) -> A
    where A: Float
{
    xs.iter().zip(ys).fold(A::from(0), |acc, (&x, &y)| acc + x * y)
}

fn sum<A>(xs: &[A]) -> A
    where A: Float
{
    use std::ops::Add;

    xs.iter().cloned().fold(A::from(0), Add::add)
}
