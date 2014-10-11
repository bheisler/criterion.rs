use parallel;
//use std::{cmp, comm, mem, os, ptr, raw};
use std::{os, ptr};

use Distribution;
use resamples::Resamples;

/// A collection of observations drawn from the population
#[experimental]
pub struct Sample<'a, A: 'a>(&'a [A]);

impl <'a, A> Sample<'a, A> {
    /// Creates a new sample by wrapping an slice
    #[experimental]
    pub fn new(data: &[A]) -> Sample<A> {
        Sample(data)
    }

    /// Returns an slice that contains all the data points contained in the sample
    #[experimental]
    pub fn as_slice(&self) -> &[A] {
        let &Sample(data) = self;

        data
    }
}

impl <'a, A: Clone + Sync> Sample<'a, A> {
    /// Returns the bootstrap distribution of the parameter estimated by the statistic
    ///
    /// * Bootstrap method: Case resampling
    // TODO Add more bootstrap methods
    #[experimental]
    pub fn bootstrap<B: Clone + Send>(
        &self,
        statistic: fn(&[A]) -> B,
        nresamples: uint
    ) -> Distribution<B> {
        // FIXME `RUST_THREADS` should be favored over `num_cpus`
        let ncpus = os::num_cpus();

        // TODO Under what conditions should multi thread by favored?
        if ncpus > 1 && nresamples > self.len() {
            let granularity = nresamples / ncpus + 1;
            let mut distribution = Vec::with_capacity(nresamples);
            unsafe { distribution.set_len(nresamples) }

            parallel::divide(distribution[mut], granularity, |data, _| {
                let mut resamples = Resamples::new(self.as_slice());

                for ptr in data.iter_mut() {
                    unsafe { ptr::write(ptr, statistic(resamples.next())) }
                }
            });

            Distribution(distribution)
        } else {
            let mut resamples = Resamples::new(self.as_slice());

            Distribution(range(0, nresamples).map(|_| {
                statistic(resamples.next())
            }).collect())
        }
    }

    /// Returns the bootstrap distribution of the parameter estimated by the 2-sample statistic
    ///
    /// * Bootstrap method: Case resampling
    #[experimental]
    pub fn bootstrap2<B: Clone + Sync, C: Clone + Send>(
        &self,
        other: &Sample<B>,
        statistic: fn(&[A], &[B]) -> C,
        nresamples: uint
    ) -> Distribution<C> {
        assert!(nresamples > 0);

        // FIXME `RUST_THREADS` should be favored over `num_cpus`
        let ncpus = os::num_cpus();
        let nresamples_sqrt = (nresamples as f64).sqrt().ceil() as uint;
        let nresamples = nresamples_sqrt * nresamples_sqrt;

        // TODO Under what conditions should multi thread by favored?
        if ncpus > 1 && nresamples > self.len() + other.len() {
            let granularity = nresamples_sqrt / ncpus + 1;
            let mut distribution = Vec::with_capacity(nresamples);
            unsafe { distribution.set_len(nresamples) }

            parallel::divide(distribution[mut], granularity, |data, _| {
                let mut resamples = Resamples::new(self.as_slice());
                let mut other_resamples = Resamples::new(other.as_slice());

                for chunk in data.chunks_mut(granularity) {
                    let resample = resamples.next();

                    for ptr in chunk.iter_mut() {
                        let other_resample = other_resamples.next();

                        unsafe { ptr::write(ptr, statistic(resample, other_resample)) }
                    }
                }
            });

            Distribution(distribution)
        } else {
            let mut resamples = Resamples::new(self.as_slice());
            let mut other_resamples = Resamples::new(other.as_slice());
            let mut distribution = Vec::with_capacity(nresamples);

            for _ in range(0, nresamples_sqrt) {
                let resample = resamples.next();

                for _ in range(0, nresamples_sqrt) {
                    let other_resample = other_resamples.next();

                    distribution.push(statistic(resample, other_resample));
                }
            }

            Distribution(distribution)
        }
    }
}

impl<'a, A> Collection for Sample<'a, A> {
    fn len(&self) -> uint {
        self.as_slice().len()
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::rand::{Rng, mod};

    use Sample;
    use {mean, t};

    #[quickcheck]
    fn bootstrap(sample_size: uint, nresamples: uint) -> TestResult {
        let data = if sample_size > 0 {
            let mut rng = rand::task_rng();

            Vec::from_fn(sample_size, |_| rng.gen::<f64>())
        } else {
            return TestResult::discard();
        };

        let sample = Sample::new(data.as_slice());

        let distribution = if nresamples > 0 {
            sample.bootstrap(mean, nresamples).unwrap()
        } else {
            return TestResult::discard();
        };

        TestResult::from_bool(
            // Allocated memory in the most efficient way
            distribution.capacity() == distribution.len() &&
            // Computed the correct number of resamples
            distribution.len() == nresamples &&
            // No uninitialized values
            distribution.iter().all(|&x| x >= 0. && x <= 1.)
        )
    }

    #[quickcheck]
    fn bootstrap2((ssize, other_ssize): (uint, uint), nresamples: uint) -> TestResult {
        let (data, other_data) = if ssize > 0 && other_ssize > 0 {
            let mut rng = rand::task_rng();

            (
                Vec::from_fn(ssize, |_| rng.gen::<f64>()),
                Vec::from_fn(other_ssize, |_| rng.gen::<f64>()),
            )
        } else {
            return TestResult::discard();
        };

        let sample = Sample::new(data.as_slice());
        let other_sample = Sample::new(other_data.as_slice());

        let distribution = if nresamples > 0 {
            sample.bootstrap2(&other_sample, t, nresamples).unwrap()
        } else {
            return TestResult::discard();
        };

        let nresamples_sqrt = (nresamples as f64).sqrt().ceil() as uint;
        let nresamples = nresamples_sqrt * nresamples_sqrt;

        TestResult::from_bool(
            // Allocated memory in the most efficient way
            distribution.capacity() == distribution.len() &&
            // Computed the correct number of resamples
            distribution.len() == nresamples
        )
    }
}

#[cfg(test)]
mod bench {
    use std::rand::{Rng, mod};
    use test::Bencher;

    use Sample;
    use mean;
    use regression::{Slope, StraightLine};

    static NRESAMPLES: uint = 100_000;
    static SAMPLE_SIZE: uint = 100;

    #[bench]
    fn bootstrap_mean(b: &mut Bencher) {
        let mut rng = rand::task_rng();
        let data = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<f64>());

        let sample = Sample::new(data.as_slice());

        b.iter(|| {
            sample.bootstrap(mean, NRESAMPLES)
        });
    }

    #[bench]
    fn bootstrap_sl(b: &mut Bencher) {
        fn slr(sample: &[(f64, f64)]) -> StraightLine<f64> {
            StraightLine::fit(sample)
        }

        let mut rng = rand::task_rng();

        let data = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<(f64, f64)>());
        let sample = Sample::new(data.as_slice());

        b.iter(|| {
            sample.bootstrap(slr, NRESAMPLES)
        })
    }

    #[bench]
    fn bootstrap_slope(b: &mut Bencher) {
        fn slr(sample: &[(f64, f64)]) -> Slope<f64> {
            Slope::fit(sample)
        }

        let mut rng = rand::task_rng();

        let data = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<(f64, f64)>());
        let sample = Sample::new(data.as_slice());

        b.iter(|| {
            sample.bootstrap(slr, NRESAMPLES)
        })
    }
}
