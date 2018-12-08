//! Mixed bootstrap

use std::cmp;

use float::Float;
use num_cpus;
use thread_scoped as thread;

use tuple::{Tuple, TupledDistributionsBuilder};
use univariate::resamples::Resamples;
use univariate::Sample;

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
    let n_a = a.len();
    let n_b = b.len();
    let mut c = Vec::with_capacity(n_a + n_b);
    c.extend_from_slice(a);
    c.extend_from_slice(b);

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
                            let resample = resamples.next();
                            let a: &Sample<A> = Sample::new(&resample[..n_a]);
                            let b: &Sample<A> = Sample::new(&resample[n_a..]);

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
                let resample = resamples.next();
                let a: &Sample<A> = Sample::new(&resample[..n_a]);
                let b: &Sample<A> = Sample::new(&resample[n_a..]);

                distributions.push(statistic(a, b))
            }

            distributions.complete()
        }
    }
}
