use std::{cmp, comm, mem, os, ptr, raw};

use distribution::Distribution;
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

impl <'a, A: Clone + Send> Sample<'a, A> {
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
            let chunk_size = nresamples / ncpus + 1;
            let (tx, rx) = comm::channel();

            let mut distribution = Vec::with_capacity(nresamples);
            unsafe { distribution.set_len(nresamples) }
            let distribution_ptr = distribution.as_mut_ptr();

            // FIXME (when available) Use a safe fork-join API
            let raw::Slice { data: ptr, len: len } =
                unsafe { mem::transmute::<&[A], raw::Slice<A>>(self.as_slice()) };

            for i in range(0, ncpus) {
                let tx = tx.clone();

                spawn(proc() {
                    // NB This task will finish before this slice becomes invalid
                    let slice: &[A] =
                        unsafe { mem::transmute(raw::Slice { data: ptr, len: len }) };

                    let mut resamples = Resamples::new(slice);

                    let start = cmp::min(i * chunk_size, nresamples) as int;
                    let end = cmp::min((i + 1) * chunk_size, nresamples) as int;

                    for j in range(start, end) {
                        unsafe {
                            ptr::write(distribution_ptr.offset(j), statistic(resamples.next()))
                        }
                    }

                    tx.send(());
                })
            }

            for _ in range(0, ncpus) {
                rx.recv();
            }

            Distribution::_new(distribution)
        } else {
            let mut resamples = Resamples::new(self.as_slice());

            Distribution::_new(range(0, nresamples).map(|_| {
                statistic(resamples.next())
            }).collect())
        }
    }

    /// Returns the bootstrap distribution of the parameter estimated by the 2-sample statistic
    ///
    /// * Bootstrap method: Case resampling
    #[experimental]
    pub fn bootstrap2<B: Clone + Send, C: Clone + Send>(
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
            let (tx, rx) = comm::channel();
            let chunk_size = nresamples_sqrt / ncpus + 1;

            let mut distribution = Vec::with_capacity(nresamples);
            unsafe { distribution.set_len(nresamples) }
            let d_ptr = distribution.as_mut_ptr();

            // FIXME (when available) Use a safe fork-join API
            let raw::Slice { data: ptr, len: len }: raw::Slice<A> =
                unsafe { mem::transmute(self.as_slice()) };
            let raw::Slice { data: o_ptr, len: o_len }: raw::Slice<B> =
                unsafe { mem::transmute(other.as_slice()) };

            for i in range(0, ncpus) {
                let tx = tx.clone();

                spawn(proc() {
                    // NB This task will finish before this slice becomes invalid
                    let sample: &[A] =
                        unsafe { mem::transmute(raw::Slice { data: ptr, len: len }) };

                    let other_sample: &[B] =
                        unsafe { mem::transmute(raw::Slice { data: o_ptr, len: o_len }) };

                    let mut resamples = Resamples::new(sample);
                    let mut other_resamples = Resamples::new(other_sample);

                    let start = cmp::min(i * chunk_size, nresamples_sqrt) as int;
                    let end = cmp::min((i + 1) * chunk_size, nresamples_sqrt) as int;

                    for j in range(start, end) {
                        let resample = resamples.next();

                        for k in range(0, nresamples_sqrt as int) {
                            let other_resample = other_resamples.next();

                            unsafe {
                                ptr::write(
                                    d_ptr.offset(j * nresamples_sqrt as int + k),
                                    statistic(resample, other_resample))
                            }
                        }
                    }

                    tx.send(())
                })
            }

            for _ in range(0, ncpus) {
                rx.recv();
            }

            Distribution::_new(distribution)
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

            Distribution::_new(distribution)
        }
    }

    /// Returns the bootstrap distributions of the parameters estimated by the statistics
    ///
    /// * Bootstrap method: Case resampling
    // TODO (Ideally) `statistics` should have type `[fn(&[A]) -> B, ..N]` and the return type
    // should have type `[Distribution<B>, ..N]`
    #[experimental]
    pub fn bootstrap_many<B: Clone + Send>(
        &self,
        statistics: &[fn(&[A]) -> B],
        nresamples: uint
    ) -> Vec<Distribution<B>> {
        assert!(statistics.len() > 0);
        assert!(nresamples > 0);

        // FIXME `RUST_THREADS` should be favored over `num_cpus`
        let ncpus = os::num_cpus();
        let nstatistics = statistics.len();

        // TODO Under what conditions should multi thread by favored?
        if ncpus > 1 && nresamples > self.len() {
            let chunk_size = nresamples / ncpus + 1;
            let (tx, rx) = comm::channel();

            let mut distributions: Vec<Vec<B>> = range(0, nstatistics).map(|_| {
                let mut v = Vec::with_capacity(nresamples);
                unsafe { v.set_len(nresamples) }
                v
            }).collect();
            let distribution_ptrs: Vec<*mut B> = distributions.iter_mut().map(|distribution| {
                distribution.as_mut_ptr()
            }).collect();

            // FIXME (when available) Use a safe fork-join API
            let raw::Slice { data: ptr, len: len }: raw::Slice<A> =
                unsafe { mem::transmute(self.as_slice()) };

            let raw::Slice { data: st_ptr, len: st_len }: raw::Slice<fn(&[A]) -> B> =
                unsafe { mem::transmute(statistics) };

            for i in range(0, ncpus) {
                let tx = tx.clone();
                let distribution_ptrs = distribution_ptrs.clone();

                spawn(proc() {
                    // NB This task will finish before these slices becomes invalid
                    let data: &[A] =
                        unsafe { mem::transmute(raw::Slice { data: ptr, len: len }) };

                    let statistics: &[fn(&[A]) -> B] =
                        unsafe { mem::transmute(raw::Slice { data: st_ptr, len: st_len }) };

                    let mut resamples = Resamples::new(data);

                    let start = cmp::min(i * chunk_size, nresamples) as int;
                    let end = cmp::min((i + 1) * chunk_size, nresamples) as int;

                    for j in range(start, end) {
                        let resample = resamples.next();

                        for (d, &statistic) in distribution_ptrs.iter().zip(statistics.iter()) {
                            unsafe { ptr::write(d.offset(j), statistic(resample)) }
                        }
                    }

                    tx.send(())
                })
            }

            for _ in range(0, ncpus) {
                rx.recv();
            }

            distributions.into_iter().map(|distribution| {
                Distribution::_new(distribution)
            }).collect()
        } else {
            let mut resamples = Resamples::new(self.as_slice());

            let mut distributions: Vec<Vec<B>> = range(0, nstatistics).map(|_| {
                Vec::with_capacity(nresamples)
            }).collect();

            for _ in range(0, nresamples) {
                for (d, &statistic) in distributions.iter_mut().zip(statistics.iter()) {
                    d.push(statistic(resamples.next()));
                }
            }

            distributions.into_iter().map(|distribution| {
                Distribution::_new(distribution)
            }).collect()
        }
    }

    /// Returns the bootstrap distributions of the parameters estimated by the 2-sample statistics
    ///
    /// * Bootstrap method: Case resampling
    // TODO (Ideally) `statistics` should have type `[fn(&[A], &[B]) -> C, ..N]` and the return
    // type should have type `[Distribution<C>, ..N]`
    #[experimental]
    pub fn bootstrap2_many<B: Clone + Send, C: Clone + Send>(
        &self,
        other: &Sample<B>,
        statistics: &[fn(&[A], &[B]) -> C],
        nresamples: uint
    ) -> Vec<Distribution<C>> {
        assert!(statistics.len() > 0);
        assert!(nresamples > 0);

        // FIXME `RUST_THREADS` should be favored over `num_cpus`
        let ncpus = os::num_cpus();
        let nresamples_sqrt = (nresamples as f64).sqrt().ceil() as uint;
        let nresamples = nresamples_sqrt * nresamples_sqrt;
        let nstatistics = statistics.len();

        // TODO Under what conditions should multi thread by favored?
        if ncpus > 1 && nresamples > self.len() {
            let (tx, rx) = comm::channel();
            let chunk_size = nresamples_sqrt / ncpus + 1;

            let mut distributions: Vec<Vec<C>> = range(0, nstatistics).map(|_| {
                let mut v = Vec::with_capacity(nresamples);
                unsafe { v.set_len(nresamples) }
                v
            }).collect();
            let d_ptrs: Vec<*mut C> = distributions.iter_mut().map(|distribution| {
                distribution.as_mut_ptr()
            }).collect();

            // FIXME (when available) Use a safe fork-join API
            let raw::Slice { data: ptr, len: len }: raw::Slice<A> =
                unsafe { mem::transmute(self.as_slice()) };

            let raw::Slice { data: o_ptr, len: o_len }: raw::Slice<B> =
                unsafe { mem::transmute(other.as_slice()) };

            let raw::Slice { data: st_ptr, len: st_len }: raw::Slice<fn(&[A]) -> B> =
                unsafe { mem::transmute(statistics) };

            for i in range(0, ncpus) {
                let tx = tx.clone();
                let d_ptrs = d_ptrs.clone();

                spawn(proc() {
                    // NB This task will finish before these slices becomes invalid
                    let data: &[A] =
                        unsafe { mem::transmute(raw::Slice { data: ptr, len: len }) };

                    let other_data: &[B] =
                        unsafe { mem::transmute(raw::Slice { data: o_ptr, len: o_len }) };

                    let statistics: &[fn(&[A], &[B]) -> C] =
                        unsafe { mem::transmute(raw::Slice { data: st_ptr, len: st_len }) };

                    let mut resamples = Resamples::new(data);
                    let mut other_resamples = Resamples::new(other_data);

                    let start = cmp::min(i * chunk_size, nresamples_sqrt) as int;
                    let end = cmp::min((i + 1) * chunk_size, nresamples_sqrt) as int;

                    for j in range(start, end) {
                        let resample = resamples.next();

                        for k in range(0, nresamples_sqrt as int) {
                            let other_resample = other_resamples.next();

                            for (d_ptr, &statistic) in d_ptrs.iter().zip(statistics.iter()) {
                                unsafe {
                                    ptr::write(
                                        d_ptr.offset(j * nresamples_sqrt as int + k),
                                        statistic(resample, other_resample))
                                }
                            }
                        }

                    }

                    tx.send(())
                })
            }

            for _ in range(0, ncpus) {
                rx.recv();
            }

            distributions.into_iter().map(|distribution| {
                Distribution::_new(distribution)
            }).collect()
        } else {
            let mut resamples = Resamples::new(self.as_slice());
            let mut other_resamples = Resamples::new(other.as_slice());

            let mut distributions: Vec<Vec<C>> = range(0, nstatistics).map(|_| {
                Vec::with_capacity(nresamples)
            }).collect();

            for _ in range(0, nresamples_sqrt) {
                let resample = resamples.next();

                for _ in range(0, nresamples_sqrt) {
                    let other_resample = other_resamples.next();

                    for (d, &statistic) in distributions.iter_mut().zip(statistics.iter()) {
                        d.push(statistic(resample, other_resample));
                    }
                }
            }

            distributions.into_iter().map(|distribution| {
                Distribution::_new(distribution)
            }).collect()
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
    use {mean, median, median_abs_dev, std_dev, t, var};

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

    #[quickcheck]
    fn bootstrap_many(
        sample_size: uint,
        nresamples: uint,
        (start, end): (uint, uint)
    ) -> TestResult {
        // FIXME (rust-lang/rust#13970) This should be a static array
        let estimators = [mean, median, median_abs_dev, std_dev, var];

        let (start, end, nestimators) = {
            let nestimators = estimators.len();
            let (start, end) = (start % nestimators, end % nestimators);

            if end > start {
                (start, end, end - start)
            } else if start < end {
                (end, start, start - end)
            } else {
                return TestResult::discard();
            }
        };

        let data = if sample_size > 1 {
            let mut rng = rand::task_rng();

            Vec::from_fn(sample_size, |_| rng.gen::<f64>())
        } else {
            return TestResult::discard();
        };

        let sample = Sample::new(data.as_slice());

        let distributions = if nresamples > 0 {
            sample.bootstrap_many(estimators.slice(start, end), nresamples)
        } else {
            return TestResult::discard();
        };

        TestResult::from_bool(
            // Computed the correct number of distributions
            distributions.len() == nestimators &&
            distributions.into_iter().all(|distribution| {
                let distribution = distribution.unwrap();

                // Allocated memory in the most efficient way
                distribution.capacity() == distribution.len() &&
                // Computed the correct number of resamples
                distribution.len() == nresamples &&
                // No uninitialized values
                distribution.iter().all(|&x| x >= 0. && x <= 1.)
            })
        )
    }

    #[quickcheck]
    fn bootstrap2_many(
        (ssize, other_ssize): (uint, uint),
        nresamples: uint,
        (start, end): (uint, uint)
    ) -> TestResult {
        fn rel_diff_mean(x: &[f64], y: &[f64]) -> f64 {
            let x = mean(x);
            let y = mean(y);

            (x - y) / (x + y)
        }

        fn rel_diff_median(x: &[f64], y: &[f64]) -> f64 {
            let x = median(x);
            let y = median(y);

            (x - y) / (x + y)
        }

        // FIXME (rust-lang/rust#13970) This should be a static array
        let estimators = [rel_diff_mean, rel_diff_median, t];

        let (start, end, nestimators) = {
            let nestimators = estimators.len();
            let (start, end) = (start % nestimators, end % nestimators);

            if end > start {
                (start, end, end - start)
            } else if start < end {
                (end, start, start - end)
            } else {
                return TestResult::discard();
            }
        };

        let (data, other_data) = if ssize > 1 && other_ssize > 1 {
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

        let distributions = if nresamples > 0 {
            sample.bootstrap2_many(&other_sample, estimators.slice(start, end), nresamples)
        } else {
            return TestResult::discard();
        };

        let nresamples_sqrt = (nresamples as f64).sqrt().ceil() as uint;
        let nresamples = nresamples_sqrt * nresamples_sqrt;

        TestResult::from_bool(
            // Computed the correct number of distributions
            distributions.len() == nestimators &&
            distributions.into_iter().all(|distribution| {
                let distribution = distribution.unwrap();

                // Allocated memory in the most efficient way
                distribution.capacity() == distribution.len() &&
                // Computed the correct number of resamples
                distribution.len() == nresamples &&
                // no uninitialized values
                distribution.iter().all(|&x| x < 1. && x > -1.)
            })
        )
    }
}

#[cfg(test)]
mod bench {
    use std::rand::{Rng, mod};
    use test::Bencher;

    use Sample;
    use {mean, median, median_abs_dev, std_dev, t};
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

    #[bench]
    fn bootstrap_many(b: &mut Bencher) {
        let mut rng = rand::task_rng();

        let data = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<f64>());
        let sample = Sample::new(data.as_slice());

        b.iter(|| {
            sample.bootstrap_many([mean, median, std_dev, median_abs_dev], NRESAMPLES)
        });
    }

    #[bench]
    fn bootstrap2_many(b: &mut Bencher) {
        fn rel_diff_mean(x: &[f64], y: &[f64]) -> f64 {
            let x = mean(x);
            let y = mean(y);

            (x - y) / (x + y)
        }

        fn rel_diff_median(x: &[f64], y: &[f64]) -> f64 {
            let x = median(x);
            let y = median(y);

            (x - y) / (x + y)
        }

        let mut rng = rand::task_rng();

        let data = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<f64>());
        let other_data = Vec::from_fn(SAMPLE_SIZE, |_| rng.gen::<f64>());

        let sample = Sample::new(data.as_slice());
        let other_sample = Sample::new(other_data.as_slice());

        b.iter(|| {
            sample.bootstrap2_many(&other_sample, [t, rel_diff_mean, rel_diff_median], NRESAMPLES)
        });
    }
}
