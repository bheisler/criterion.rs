//! Mixed bootstrap

use std::ptr::Unique;
use std::{cmp, mem};

use floaty::Floaty;
use num_cpus;
use thread_scoped as thread;

use tuple::{Tuple, TupledDistributions};
use univariate::Sample;
use univariate::resamples::Resamples;

/// Performs a *mixed* two-sample bootstrap
pub fn bootstrap<A, T, S>(
    a: &Sample<A>,
    b: &Sample<A>,
    nresamples: usize,
    statistic: S,
) -> T::Distributions where
    A: Floaty,
    S: Fn(&Sample<A>, &Sample<A>) -> T + Sync,
    T: Tuple,
    T::Distributions: Send,
{
    let ncpus = num_cpus::get();
    let n_a = a.as_slice().len();
    let n_b = b.as_slice().len();
    let mut c = Vec::with_capacity(n_a + n_b);
    c.extend_from_slice(a.as_slice());
    c.extend_from_slice(b.as_slice());

    unsafe {
        //let c: &Sample<A> = mem::transmute(c.as_slice());
        let c = Sample::new(&c);

        // TODO need some sensible threshold to trigger the multi-threaded path
        if ncpus > 10 && nresamples > n_a {
            let granularity = nresamples / ncpus + 1;
            let statistic = &statistic;
            let mut distributions: T::Distributions =
                TupledDistributions::uninitialized(nresamples);

            let _ = (0..ncpus).map(|i| {
                // NB Can't implement `chunks_mut` for the tupled distributions without HKT,
                // for now I'll make do with aliasing and careful non-overlapping indexing
                let mut ptr = Unique::new_unchecked(&mut distributions);
                let offset = i * granularity;

                thread::scoped(move || {
                    let distributions: &mut T::Distributions = ptr.as_mut();
                    let end = cmp::min(offset + granularity, nresamples);
                    let mut resamples = Resamples::new(c);

                    for i in offset..end {
                        let resample = resamples.next().as_slice();
                        let a: &Sample<A> = mem::transmute(&resample[..n_a]);
                        let b: &Sample<A> = mem::transmute(&resample[n_a..]);

                        distributions.set_unchecked(i, statistic(a, b))
                    }
                })
            }).collect::<Vec<_>>();

            distributions
        } else {
            let mut resamples = Resamples::new(c);
            let mut distributions: T::Distributions =
                TupledDistributions::uninitialized(nresamples);

            for i in 0..nresamples {
                let resample = resamples.next().as_slice();
                let a: &Sample<A> = mem::transmute(&resample[..n_a]);
                let b: &Sample<A> = mem::transmute(&resample[n_a..]);

                distributions.set_unchecked(i, statistic(a, b))
            }

            distributions
        }
    }
}
