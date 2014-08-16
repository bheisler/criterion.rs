//! Regression analysis

use std::iter::AdditiveIterator;
use std::num::{One, Zero, mod};
use test::stats::Stats;

/// An straight line `y = m * x + b`
#[deriving(Clone)]
#[experimental]
pub struct StraightLine<A> {
    /// The y-intercept of the line
    pub intercept: A,
    /// The slope of the line
    pub slope: A,
}

impl<A> StraightLine<A>
where A: Clone + Div<A, A> + Mul<A, A> + NumCast + Sub<A, A> + Zero
{
    /// Fits the sample to a straight line using ordinary least squares
    pub fn fit(sample: &[(A, A)]) -> StraightLine<A> {
        assert!(sample.len() > 0);

        let n = num::cast(sample.len()).unwrap();
        let x2_bar = sample.iter().map(|&(ref x, _)| x.mul(x)).sum() / n;
        let x_bar = sample.iter().map(|&(ref x, _)| x.clone()).sum() / n;
        let xy_bar = sample.iter().map(|&(ref x, ref y)| x.mul(y)).sum() / n;
        let y_bar = sample.iter().map(|&(_, ref y)| y.clone()).sum() / n;

        let slope = {
            let num = xy_bar - x_bar * y_bar;
            let den = x2_bar - x_bar * x_bar;

            num / den
        };

        let intercept = y_bar - slope * x_bar;

        StraightLine {
            intercept: intercept,
            slope: slope,
        }
    }
}

impl<A> StraightLine<A>
where A: Clone + Div<A, A> + Mul<A, A> + NumCast + One + Sub<A, A> + Zero
{
    /// Computes the goodness of fit (coefficient of determination) for this sample
    pub fn r_squared(&self, sample: &[(A, A)]) -> A {
        let &StraightLine { slope: ref alpha, intercept: ref beta } = self;
        let n = num::cast(sample.len()).unwrap();
        let y_bar = sample.iter().map(|&(_, ref y)| y.clone()).sum() / n;

        let ss_res = sample.iter().map(|&(ref x, ref y)| {
            let diff = y.sub(&alpha.mul(x)).sub(beta);

            diff * diff
        }).sum();

        let ss_tot = sample.iter().map(|&(_, ref y)| {
            let diff = y.sub(&y_bar);

            diff * diff
        }).sum();

        num::one::<A>() - ss_res / ss_tot
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::rand::{Rng, mod};

    use regression::StraightLine;
    use tol::TOLERANCE;

    #[quickcheck]
    fn r_squared(sample_size: uint) -> TestResult {
        let data = if sample_size > 1 {
            let mut rng = rand::task_rng();

            Vec::from_fn(sample_size, |_| rng.gen::<(f64, f64)>())
        } else {
            return TestResult::discard();
        };

        let data = data.as_slice();
        let sl = StraightLine::fit(data);

        let r_squared = sl.r_squared(data);

        TestResult::from_bool(
            r_squared > -TOLERANCE && r_squared < 1. + TOLERANCE
        )
    }
}
