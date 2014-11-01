//! Kernel density estimation

use parallel;
use std::iter::AdditiveIterator;
use std::ops::Fn;
use std::{os, ptr};

use Stats;

/// Univariate Kernel Density Estimator
#[experimental]
pub struct Kde<'a> {
    bandwidth: f64,
    kernel: fn(f64) -> f64,
    sample: &'a [f64],
}

impl<'a> Kde<'a> {
    /// Creates a new univariate kernel density estimator
    ///
    /// * Bandwidth: Estimated using Silverman's rule of thumb
    /// * Kernel: Gaussian
    // TODO bandwidth estimator should be configurable
    // TODO kernel should be configurable
    #[experimental]
    pub fn new(sample: &[f64]) -> Kde {
        Kde {
            bandwidth: silverman(sample),
            kernel: gaussian,
            sample: sample,
        }
    }

    /// Returns the bandwidth used by the estimator
    #[experimental]
    pub fn bandwidth(&self) -> f64 {
        self.bandwidth
    }

    /// Returns the sample used by the estimator
    #[experimental]
    pub fn sample(&self) -> &[f64] {
        self.sample
    }

    /// Sweeps the `[a, b]` range collecting `n` points of the estimated PDF
    #[experimental]
    pub fn sweep(&self, (a, b): (f64, f64), n: uint) -> Vec<(f64, f64)> {
        assert!(a < b);
        assert!(n > 1);

        let dx = (b - a) / (n - 1) as f64;
        let ncpus = os::num_cpus();

        // TODO Under what conditions should multi thread by favored?
        if ncpus > 1 {
            let granularity = n / ncpus + 1;
            let mut pdf = Vec::with_capacity(n);
            unsafe { pdf.set_len(n) }

            parallel::divide(pdf[mut], granularity, |data, offset| {
                let mut x = a + offset as f64 * dx;

                for ptr in data.iter_mut() {
                    unsafe { ptr::write(ptr, (x, self(x))) }
                    x += dx;
                }
            });

            pdf
        } else {
            let mut pdf = Vec::with_capacity(n);

            let mut x = a;
            for _ in range(0, n) {
                pdf.push((x, self(x)));

                x += dx;
            }

            pdf
        }
    }
}

impl<'a> Fn<(f64,), f64> for Kde<'a> {
    /// Estimates the probability *density* that the random variable takes the value `x`
    // XXX Can this be SIMD accelerated?
    #[experimental]
    extern "rust-call" fn call(&self, (x,): (f64,)) -> f64 {
        let frac_1_h = self.bandwidth.recip();
        let n = self.sample.len() as f64;
        let k = self.kernel;

        self.sample.iter().map(|&x_i| {
            k((x - x_i) * frac_1_h)
        }).sum() * frac_1_h / n
    }
}

/// Estimates the bandwidth using Silverman's rule of thumb
#[experimental]
fn silverman(x: &[f64]) -> f64 {
    static FACTOR: f64 = 4. / 3.;
    static EXPONENT: f64 = 1. / 5.;

    let n = x.len() as f64;
    let sigma = x.std_dev(None);

    sigma * (FACTOR / n).powf(EXPONENT)
}

/// The gaussian kernel
///
/// Equivalent to the Probability Density Function of a normally distributed random variable with
/// mean 0 and variance 1
#[experimental]
fn gaussian(x: f64) -> f64 {
    x.powi(2).exp().mul(&::std::f64::consts::PI_2).sqrt().recip()
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use Stats;
    use kde::Kde;
    use test::{ApproxEq, mod};

    mod gaussian {
        use quickcheck::TestResult;

        use super::super::gaussian;
        use test::ApproxEq;

        #[quickcheck]
        fn symmetric(x: f64) -> bool {
            gaussian(-x).approx_eq(gaussian(x))
        }

        // Any [a b] integral should be in the range [0 1]
        #[quickcheck]
        fn integral(a: f64, b: f64) -> TestResult {
            static DX: f64 = 1e-3;

            if a > b {
                TestResult::discard()
            } else {
                let mut acc = 0.;
                let mut x = a;
                let mut y = gaussian(a);

                while x < b {
                    acc += DX * y / 2.;

                    x += DX;
                    y = gaussian(x);

                    acc += DX * y / 2.;
                }

                TestResult::from_bool(
                    (acc > 0. || acc.approx_eq(0.)) && (acc < 1. || acc.approx_eq(1.)))
            }
        }
    }

    // The [-inf inf] integral of the estimated PDF should be one
    #[quickcheck]
    fn integral(size: uint) -> TestResult {
        static DX: f64 = 1e-3;

        if let Some(data) = test::vec::<f64>(size) {
            let data = data[];

            let kde = Kde::new(data);
            let h = kde.bandwidth();
            // NB Obviously a [-inf inf] integral is not feasible, but this range works quite well
            let (a, b) = (data.min() - 5. * h, data.max() + 5. * h);

            let mut acc = 0.;
            let mut x = a;
            let mut y = kde(a);

            while x < b {
                acc += DX * y / 2.;

                x += DX;
                y = kde(x);

                acc += DX * y / 2.;
            }

            TestResult::from_bool(acc.approx_eq(1.))
        } else {
            TestResult::discard()
        }

    }
}

#[cfg(test)]
mod bench {
    use std_test::Bencher;

    use kde::Kde;
    use test;

    static KDE_POINTS: uint = 500;
    static SAMPLE_SIZE: uint = 100_000;

    #[bench]
    fn sweep(b: &mut Bencher) {
        let data = test::vec(SAMPLE_SIZE).unwrap();
        let kde = Kde::new(data[]);

        b.iter(|| {
            kde.sweep((0., 1.), KDE_POINTS)
        })
    }
}
