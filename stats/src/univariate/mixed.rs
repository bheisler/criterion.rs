//! Mixed bootstrap

use std::{cmp, mem};

use float::Float;
use num_cpus;
use thread_scoped as thread;

use tuple::{Tuple, TupledDistributionsBuilder};
use univariate::Sample;
use univariate::resamples::Resamples;

/// Performs a *mixed* two-sample bootstrap
pub fn bootstrap<A, T, S>(
    a: &Sample<A>,
    b: &Sample<A>,
    nresamples: usize,
    statistic: S,
) -> T::Distributions
where
    A: Float,
    S: Fn(&Sample<A>, &Sample<A>) -> T + Sync,
    T: Tuple,
    T::Distributions: Send,
    T::Builder: Send,
{
    let ncpus = num_cpus::get();
    let n_a = a.as_slice().len();
    let n_b = b.as_slice().len();
    let mut c = Vec::with_capacity(n_a + n_b);
    c.extend_from_slice(a.as_slice());
    c.extend_from_slice(b.as_slice());

    unsafe {
        let c = Sample::new(&c);

        // TODO need some sensible threshold to trigger the multi-threaded path
        if ncpus > 1 && nresamples > n_a {
            let granularity = nresamples / ncpus + 1;
            let statistic = &statistic;

            let chunks = (0..ncpus)
                .map(|i| {
                    let mut sub_distributions: T::Builder =
                        TupledDistributionsBuilder::new(granularity);
                    let offset = i * granularity;

                    thread::scoped(move || {
                        let end = cmp::min(offset + granularity, nresamples);
                        let mut resamples = Resamples::new(c);

                        for _ in offset..end {
                            let resample = resamples.next().as_slice();
                            let a: &Sample<A> = mem::transmute(&resample[..n_a]);
                            let b: &Sample<A> = mem::transmute(&resample[n_a..]);

                            sub_distributions.push(statistic(a, b))
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
            let mut resamples = Resamples::new(c);
            let mut distributions: T::Builder = TupledDistributionsBuilder::new(nresamples);

            for _ in 0..nresamples {
                let resample = resamples.next().as_slice();
                let a: &Sample<A> = mem::transmute(&resample[..n_a]);
                let b: &Sample<A> = mem::transmute(&resample[n_a..]);

                distributions.push(statistic(a, b))
            }

            distributions.complete()
        }
    }
}
