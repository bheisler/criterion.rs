//! Regression analysis

use std::iter::AdditiveIterator;
use std::num::{Float, mod};

use Sum;

/// A straight line that passes through the origin `y = m * x`
#[deriving(Clone)]
#[experimental]
pub struct Slope<A: Float>(pub A);

impl<A> Slope<A> where A: Float {
    pub fn fit(sample: &[(A, A)]) -> Slope<A> {
        let n = num::cast(sample.len()).unwrap();
        let xy_bar = sample.iter().map(|&(x, y)| x * y).sum() / n;
        let x2_bar = sample.iter().map(|&(x, _)| x * x).sum() / n;

        let slope = xy_bar / x2_bar;

        Slope(slope)
    }
}

impl<A> Slope<A> where A: Float {
    pub fn r_squared(&self, sample: &[(A, A)]) -> A {
        let alpha = self.0;

        let n = num::cast(sample.len()).unwrap();
        let y_bar = sample.iter().map(|&(_, y)| y).sum() / n;

        let ss_res = sample.iter().map(|&(x, y)| {
            let diff = y - alpha * x;

            diff * diff
        }).sum();

        let ss_tot = sample.iter().map(|&(_, y)| {
            let diff = y - y_bar;

            diff * diff
        }).sum();

        let _1: A = Float::one();

        _1 - ss_res / ss_tot
    }
}

/// An straight line `y = m * x + b`
#[deriving(Clone)]
#[experimental]
pub struct StraightLine<A: Float> {
    /// The y-intercept of the line
    pub intercept: A,
    /// The slope of the line
    pub slope: A,
}

impl<A> StraightLine<A> where A: Float {
    /// Fits the sample to a straight line using ordinary least squares
    pub fn fit(sample: &[(A, A)]) -> StraightLine<A> {
        assert!(sample.len() > 0);

        let n = num::cast(sample.len()).unwrap();
        let x2_bar = sample.iter().map(|&(x, _)| x * x).sum() / n;
        let x_bar = sample.iter().map(|&(x, _)| x).sum() / n;
        let xy_bar = sample.iter().map(|&(x, y)| x * y).sum() / n;
        let y_bar = sample.iter().map(|&(_, y)| y).sum() / n;

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

impl<A> StraightLine<A> where A: Float {
    /// Computes the goodness of fit (coefficient of determination) for this sample
    pub fn r_squared(&self, sample: &[(A, A)]) -> A {
        let alpha = self.slope;
        let beta = self.intercept;

        let n = num::cast(sample.len()).unwrap();
        let y_bar = sample.iter().map(|&(_, y)| y).sum() / n;

        let ss_res = sample.iter().map(|&(x, y)| {
            let diff = y - alpha * x - beta;

            diff * diff
        }).sum();

        let ss_tot = sample.iter().map(|&(_, y)| {
            let diff = y - y_bar;

            diff * diff
        }).sum();

        let _1: A = Float::one();

        _1 - ss_res / ss_tot
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use Stats;
    use regression::StraightLine;
    use test::{ApproxEq, mod};

    #[quickcheck]
    fn normalized(size: uint, scale: f64) -> TestResult {
        if scale == 0. {
            return TestResult::discard();
        }

        if let Some(mut data) = test::vec::<(f64, f64)>(size) {
            for x in data.iter_mut() {
                x.0 *= scale;
                x.1 *= scale;
            }

            let data = data[];

            let (mut x, mut y) = (vec!(), vec!());

            for &(a, b) in data.iter() {
                x.push(a);
                y.push(b);
            }

            let (x, y) = (x[], y[]);

            let (x_bar, y_bar) = (x.mean(), y.mean());
            let (sigma_x, sigma_y) = (x.std_dev(Some(x_bar)), y.std_dev(Some(y_bar)));

            let normalized_data: Vec<(f64, f64)> = data.iter().map(|&(x, y)| {
                ((x - x_bar) / sigma_x, (y - y_bar) / sigma_y)
            }).collect();
            let normalized_data = normalized_data[];

            let sl = StraightLine::fit(data);
            let nsl = StraightLine::fit(normalized_data);

            TestResult::from_bool(
                sl.r_squared(data).approx_eq(nsl.r_squared(normalized_data)) &&
                sl.slope.approx_eq(nsl.slope * sigma_y / sigma_x)
            )
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn r_squared(size: uint) -> TestResult {
        if let Some(data) = test::vec::<(f64, f64)>(size) {
            let data = data[];
            let sl = StraightLine::fit(data);

            let r_squared = sl.r_squared(data);

            TestResult::from_bool(
                (r_squared > 0. || r_squared.approx_eq(0.)) &&
                    (r_squared < 1. || r_squared.approx_eq(1.))
            )
        } else {
            TestResult::discard()
        }
    }
}
