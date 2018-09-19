//! [Criterion]'s statistics library.
//!
//! [Criterion]: https://github.com/japaric/criterion.rs
//!
//! **WARNING** This library is criterion's implementation detail and there no plans to stabilize
//! it. In other words, the API may break at any time without notice.

#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        used_underscore_binding,
        just_underscores_and_digits,
        transmute_ptr_to_ptr
    )
)]

extern crate cast;
extern crate num_cpus;
extern crate num_traits;
extern crate rand;
extern crate thread_scoped;

#[cfg(test)]
#[macro_use]
extern crate approx;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
mod test;

pub mod bivariate;
pub mod tuple;
pub mod univariate;

mod float;

use std::mem;
use std::ops::Deref;

use float::Float;
use univariate::Sample;

/// The bootstrap distribution of some parameter
pub struct Distribution<A>(Box<[A]>);

impl<A> Distribution<A>
where
    A: Float,
{
    /// Create a distribution from the given values
    pub fn from(values: Box<[A]>) -> Distribution<A> {
        Distribution(values)
    }

    /// Computes the confidence interval of the population parameter using percentiles
    ///
    /// # Panics
    ///
    /// Panics if the `confidence_level` is not in the `(0, 1)` range.
    pub fn confidence_interval(&self, confidence_level: A) -> (A, A)
    where
        usize: cast::From<A, Output = Result<usize, cast::Error>>,
    {
        let _0 = A::cast(0);
        let _1 = A::cast(1);
        let _50 = A::cast(50);

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
    pub fn p_value(&self, t: A, tails: &Tails) -> A {
        use std::cmp;

        let n = self.0.len();
        let hits = self.0.iter().filter(|&&x| x < t).count();

        let tails = A::cast(match *tails {
            Tails::One => 1,
            Tails::Two => 2,
        });

        A::cast(cmp::min(hits, n - hits)) / A::cast(n) * tails
    }
}

impl<A> Deref for Distribution<A> {
    type Target = Sample<A>;

    fn deref(&self) -> &Sample<A> {
        let slice: &[_] = &self.0;

        unsafe { mem::transmute(slice) }
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
where
    A: Float,
{
    xs.iter()
        .zip(ys)
        .fold(A::cast(0), |acc, (&x, &y)| acc + x * y)
}

fn sum<A>(xs: &[A]) -> A
where
    A: Float,
{
    use std::ops::Add;

    xs.iter().cloned().fold(A::cast(0), Add::add)
}
