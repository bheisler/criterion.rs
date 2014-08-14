//! Regression analysis

use std::num::{Zero, mod};
use test::stats::Stats;

use mean;

/// Simple linear regression: `a * x + b`
#[deriving(Clone)]
#[experimental]
pub struct LinearRegression<A> {
    /// The intercept of the line used to fit the data
    pub intercept: A,
    /// The square root of the coefficient of determination
    pub r_value: A,
    /// The slope of the line used to fit the data
    pub slope: A,
}

/// Performs a simple linear regression
#[experimental]
pub fn linregress<A: FloatMath + FromPrimitive>(xy: &[(A, A)]) -> LinearRegression<A> {
    assert!(xy.len() > 1);

    let n = xy.len();
    let (mut x, mut y) = (Vec::with_capacity(n), Vec::with_capacity(n));

    // Split
    for &(ref a, ref b) in xy.iter() {
        x.push(a.clone());
        y.push(b.clone());
    }

    // Normalize
    fn max_abs<A: FloatMath + FromPrimitive>(x: &[A]) -> A {
        let max = x.max().abs();
        let min = x.min().abs();

        if max > min {
            max
        } else {
            min
        }
    }

    let k_x = max_abs(x.as_slice()).recip();
    let k_y = max_abs(y.as_slice()).recip();

    let x: Vec<A> = x.move_iter().map(|x| k_x * x).collect();
    let y: Vec<A> = y.move_iter().map(|y| k_y * y).collect();

    // Linear regression
    // TODO This dot product can be accelerated via SIMD or BLAS
    fn dot<A: Add<A, A> + Mul<A, A> + Zero>(x: &[A], y: &[A]) -> A {
        use std::iter::AdditiveIterator;

        x.iter().zip(y.iter()).map(|(x, y)| x.mul(y)).sum()
    }

    let x = x.as_slice();
    let y = y.as_slice();

    let n = num::cast(n).unwrap();
    let x2_bar = dot(x, x) / n;
    let x_bar = mean(x);
    let xy_bar = dot(x, y) / n;
    let y2_bar = dot(y, y) / n;
    let y_bar = mean(y);

    let slope = {
        let num = xy_bar - x_bar * y_bar;
        let den = x2_bar - x_bar.powi(2);

        num / den
    };

    let intercept = y_bar - slope * x_bar;

    let r_value = {
        let num = xy_bar - x_bar * y_bar;
        let den = (x2_bar - x_bar.powi(2)) * (y2_bar - y_bar.powi(2));

        num / den.sqrt()
    };

    LinearRegression {
        intercept: intercept / k_y,
        r_value: r_value,
        slope: slope * k_x / k_y,
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::rand::{Rng, mod};

    use regression::linregress;
    use tol::TOLERANCE;

    #[quickcheck]
    fn r_squared(sample_size: uint) -> TestResult {
        let data = if sample_size > 1 {
            let mut rng = rand::task_rng();

            Vec::from_fn(sample_size, |_| rng.gen::<(f64, f64)>())
        } else {
            return TestResult::discard();
        };

        let lr = linregress(data.as_slice());

        let r_squared = lr.r_value.powi(2);

        TestResult::from_bool(
            r_squared > -TOLERANCE && r_squared < 1. + TOLERANCE
        )
    }
}
