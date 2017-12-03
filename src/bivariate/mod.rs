//! Bivariate analysis

mod bootstrap;
mod resamples;

pub mod regression;

use std::ptr::Unique;
use std::{cmp, mem};

use floaty::Floaty;
use num_cpus;
use thread_scoped as thread;

use bivariate::resamples::Resamples;
use tuple::{Tuple, TupledDistributions};
use univariate::Sample;

/// Bivariate `(X, Y)` data
///
/// Invariants:
///
/// - No `NaN`s in the data
/// - At least two data points in the set
pub struct Data<'a, X, Y>(&'a [X], &'a [Y]) where X: 'a, Y: 'a;

impl<'a, X, Y> Copy for Data<'a, X, Y> {}

#[cfg_attr(clippy, allow(expl_impl_clone_on_copy))]
impl<'a, X, Y> Clone for Data<'a, X, Y> {
    fn clone(&self) -> Data<'a, X, Y> {
        *self
    }
}

impl<'a, X, Y> Data<'a, X, Y> {
    /// Returns the length of the data set
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks whether the data set is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over the data set
    pub fn iter(&self) -> Pairs<'a, X, Y> {
        Pairs {
            data: *self,
            state: 0,
        }
    }
}

impl<'a, X, Y> Data<'a, X, Y> where X: Floaty, Y: Floaty {
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
        let ncpus = num_cpus::get();

        unsafe {
            // TODO need some sensible threshold to trigger the multi-threaded path
            if ncpus > 1 && nresamples > self.0.len() {
                let granularity = nresamples / ncpus + 1;
                let statistic = &statistic;
                let mut distributions: T::Distributions =
                    TupledDistributions::uninitialized(nresamples);

                let _ = (0..ncpus).map(|i| {
                    // NB Can't implement `chunks_mut` for the tupled distributions without HKT,
                    // for now I'll make do with aliasing and careful non-overlapping indexing
                    let mut ptr = Unique::new_unchecked(&mut distributions);
                    let mut resamples = Resamples::new(*self);
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

/// Iterator over `Data`
pub struct Pairs<'a, X: 'a, Y: 'a> {
    data: Data<'a, X, Y>,
    state: usize,
}

impl<'a, X, Y> Iterator for Pairs<'a, X, Y> {
    type Item = (&'a X, &'a Y);

    fn next(&mut self) -> Option<(&'a X, &'a Y)> {
        if self.state < self.data.len() {
            let i = self.state;
            self.state += 1;

            unsafe {
                Some((
                    self.data.0.get_unchecked(i),
                    self.data.1.get_unchecked(i),
                ))
            }
        } else {
            None
        }
    }
}
