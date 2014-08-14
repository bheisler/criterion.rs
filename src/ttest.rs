//! t-test

use std::raw::Slice;
use std::{cmp, comm, mem, num, os, ptr};

use resamples::Resamples;

/// A bootstrapped t distribution
pub struct TDistribution<A>(Vec<A>);

impl<A: FloatMath + FromPrimitive + Send> TDistribution<A> {
    /// Computes a t distribution by bootstrapping the t-statistic between two samples
    ///
    /// * Bootstrap method: Case resampling
    pub fn new(a: &[A], b: &[A], nresamples: uint) -> TDistribution<A> {
        assert!(nresamples > 0);

        // FIXME `RUST_THREADS` should be favored over `num_cpus`
        let ncpus = os::num_cpus();
        let n = a.len();
        let joint_sample = [a.as_slice(), b.as_slice()].concat_vec();
        let joint_sample = joint_sample.as_slice();

        // TODO Under what conditions should multi thread by favored?
        if ncpus > 1 && nresamples > a.len() + b.len() {
            let chunk_size = nresamples / ncpus + 1;
            let (tx, rx) = comm::channel();

            let mut distribution = Vec::with_capacity(nresamples);
            unsafe { distribution.set_len(nresamples) }
            let distribution_ptr = distribution.as_mut_ptr();

            // FIXME (when available) Use a safe fork-join API
            let Slice { data: ptr, len: len } =
                unsafe { mem::transmute::<&[A], Slice<A>>(joint_sample) };

            for i in range(0, ncpus) {
                let tx = tx.clone();

                spawn(proc() {
                    // NB This task will finish before this slice becomes invalid
                    let slice: &[A] = unsafe { mem::transmute(Slice { data: ptr, len: len }) };

                    let mut resamples = Resamples::new(slice);

                    let start = cmp::min(i * chunk_size, nresamples) as int;
                    let end = cmp::min((i + 1) * chunk_size, nresamples) as int;

                    for j in range(start, end) {
                        let joint_resample = resamples.next();

                        let resample = joint_resample.slice_to(n);
                        let other_resample = joint_resample.slice_from(n);

                        unsafe {
                            ptr::write(
                                distribution_ptr.offset(j),
                                ::stats::t(resample, other_resample))
                        }
                    }

                    tx.send(());
                })
            }

            for i in range(0, ncpus) {
                rx.recv();
            }

            TDistribution(distribution)
        } else {
            let mut resamples = Resamples::new(joint_sample);

            TDistribution(range(0, nresamples).map(|_| {
                let joint_resample = resamples.next();

                let resample = joint_resample.slice_to(n);
                let other_resample = joint_resample.slice_from(n);

                ::stats::t(resample, other_resample)
            }).collect())
        }
    }
}

impl<A> TDistribution<A> {
    /// Returns an slice to the data points of the distribution
    pub fn as_slice(&self) -> &[A] {
        let &TDistribution(ref distribution) = self;

        distribution.as_slice()
    }


    /// Returns a vector that contains the data points of the distribution
    pub fn unwrap(self) -> Vec<A> {
        let TDistribution(distribution) = self;

        distribution
    }
}

impl<A: Float> TDistribution<A> {
    /// Computes the p-value of the t-statistic against the t-distribution
    pub fn p_value(&self, t_statistic: A, tails: Tails) -> A {
        let t = t_statistic.abs();

        let hits = self.as_slice().iter().filter(|&&x| x < -t || x > t).count();
        let n = self.as_slice().len();

        let p_value = num::cast::<_, A>(hits).unwrap() / num::cast::<_, A>(n).unwrap();

        match tails {
            // XXX This division by two assumes that the t-distribution is symmetric
            OneTailed => p_value / num::cast(2f64).unwrap(),
            TwoTailed => p_value,
        }
    }
}

/// Number of tails to consider for the t-test
pub enum Tails {
    OneTailed,
    TwoTailed,
}

#[cfg(test)]
mod bench {
    use test::Bencher;
    use std::rand::{Rng, mod};

    use super::TDistribution;

    static SAMPLE_SIZE: uint = 100;
    static NRESAMPLES: uint = 100_000;

    #[bench]
    fn new(b: &mut Bencher) {
        let mut rng = rand::task_rng();

        let a = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<f64>());
        let c = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<f64>());

        b.iter(|| {
            TDistribution::new(a.as_slice(), c.as_slice(), NRESAMPLES)
        })
    }

}
