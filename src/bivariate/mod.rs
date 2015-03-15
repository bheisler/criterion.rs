//! Bivariate analysis

mod bootstrap;
mod resamples;

pub mod regression;

use std::num::Float;
use std::ptr::Unique;
use std::{cmp, mem, os, thread};

use bivariate::resamples::Resamples;
use tuple::{Tuple, TupledDistributions};
use univariate::Sample;

/// Bivariate `(X, Y)` data
///
/// Invariants:
///
/// - No `NaN`s in the data
/// - At least two data points in the set
#[derive(Copy)]
pub struct Data<'a, X, Y>(&'a [X], &'a [Y]) where X: 'a, Y: 'a;

impl<'a, X, Y> Data<'a, X, Y> where X: ::Float, Y: ::Float {
    /// Creates a new data set from two existing slices
    pub fn new(xs: &'a [X], ys: &'a [Y]) -> Data<'a, X, Y> {
        assert!(
            xs.len() == ys.len() &&
            xs.len() > 1 &&
            xs.iter().all(|x| !x.is_nan()) &&
            ys.iter().all(|y| !y.is_nan())
        );

        Data(xs, ys)
    }

    // TODO Remove the `T` parameter in favor of `S::Output`
    /// Returns the bootstrap distributions of the parameters estimated by the `statistic`
    ///
    /// - Multi-threaded
    /// - Time: `O(nresamples)`
    /// - Memory: `O(nresamples)`
    pub fn bootstrap<T, S>(&self, nresamples: usize, statistic: S) -> T::Distributions where
        S: Fn(Data<X, Y>) -> T,
        S: Sync,
        T: Tuple,
        T::Distributions: Send,
    {
        #![allow(deprecated)]

        let ncpus = os::num_cpus();

        unsafe {
            // TODO need some sensible threshold to trigger the multi-threaded path
            if ncpus > 1 && nresamples > self.0.len() {
                let granularity = nresamples / ncpus + 1;
                let statistic = &statistic;
                let mut distributions: T::Distributions =
                    TupledDistributions::uninitialized(nresamples);

                (0..ncpus).map(|i| {
                    // NB Can't implement `chunks_mut` for the tupled distributions without HKT,
                    // for now I'll make do with aliasing and careful non-overlapping indexing
                    let mut ptr = Unique::new(&mut distributions);
                    let mut resamples = Resamples::new(*self);
                    let offset = i * granularity;

                    thread::scoped(move || {
                        let distributions: &mut T::Distributions = ptr.get_mut();

                        for i in offset..cmp::min(offset + granularity, nresamples) {
                            distributions.set_unchecked(i, statistic(resamples.next()))
                        }
                    })
                }).collect::<Vec<_>>();

                distributions
            } else {
                let mut distributions: T::Distributions =
                    TupledDistributions::uninitialized(nresamples);
                let mut resamples = Resamples::new(*self);

                for i in 0..nresamples {
                    distributions.set_unchecked(i, statistic(resamples.next()));
                }

                distributions
            }
        }
    }

    /// Returns a view into the `X` data
    pub fn x(&self) -> &'a Sample<X> {
        unsafe {
            mem::transmute(self.0)
        }
    }

    /// Returns a view into the `Y` data
    pub fn y(&self) -> &'a Sample<Y> {
        unsafe {
            mem::transmute(self.1)
        }
    }
}
