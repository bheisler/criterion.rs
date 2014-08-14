use std::num;
use test::stats::Stats;

use ci::ConfidenceInterval;
use stats::std_dev;

/// The bootstrap distribution of a population parameter
#[experimental]
pub struct Distribution<A>(Vec<A>);

impl<A> Distribution<A> {
    #[doc(hidden)]
    pub fn _new(data: Vec<A>) -> Distribution<A> {
        Distribution(data)
    }

    /// Returns an slice to the data points of the distribution
    pub fn as_slice(&self) -> &[A] {
        let &Distribution(ref distribution) = self;

        distribution.as_slice()
    }

    /// Returns a vector that contains the data points of the distribution
    pub fn unwrap(self) -> Vec<A> {
        let Distribution(distribution) = self;

        distribution
    }
}

impl<A: FromPrimitive + FloatMath> Distribution<A> {
    /// Computes the confidence interval of the population parameter using percentiles
    // TODO Add more methods to find the confidence interval (e.g. with bias correction)
    pub fn confidence_interval(&self, confidence_level: A) -> ConfidenceInterval<A> {
        assert!(confidence_level > num::zero() && confidence_level < num::one());

        let &Distribution(ref distribution) = self;
        let distribution = distribution.as_slice();

        let one = num::one::<A>();
        let fifty = num::cast::<f64, A>(50.).unwrap();

        ConfidenceInterval {
            confidence_level: confidence_level,
            lower_bound: distribution.percentile(fifty * (one - confidence_level)),
            upper_bound: distribution.percentile(fifty * (one + confidence_level)),
        }
    }

    /// Computes the standard error of the population parameter
    pub fn standard_error(&self) -> A {
        std_dev(self.as_slice())
    }
}
