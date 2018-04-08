//! Regression analysis

use float::Float;

use bivariate::Data;

/// A straight line that passes through the origin `y = m * x`
#[derive(Clone, Copy)]
pub struct Slope<A>(pub A)
where
    A: Float;

impl<A> Slope<A>
where
    A: Float,
{
    /// Fits the data to a straight line that passes through the origin using ordinary least
    /// squares
    ///
    /// - Time: `O(length)`
    pub fn fit(data: Data<A, A>) -> Slope<A> {
        let xs = data.0;
        let ys = data.1;

        let xy = ::dot(xs, ys);
        let x2 = ::dot(xs, xs);

        Slope(xy / x2)
    }

    /// Computes the goodness of fit (coefficient of determination) for this data set
    ///
    /// - Time: `O(length)`
    pub fn r_squared(&self, data: Data<A, A>) -> A {
        let _0 = A::cast(0);
        let _1 = A::cast(1);
        let m = self.0;
        let xs = data.0;
        let ys = data.1;

        let n = A::cast(xs.len());
        let y_bar = ::sum(ys) / n;

        let mut ss_res = _0;
        let mut ss_tot = _0;

        for (&x, &y) in data.iter() {
            ss_res = ss_res + (y - m * x).powi(2);
            ss_tot = ss_res + (y - y_bar).powi(2);
        }

        _1 - ss_res / ss_tot
    }
}

/// A straight line `y = m * x + b`
#[derive(Clone, Copy)]
pub struct StraightLine<A>
where
    A: Float,
{
    /// The y-intercept of the line
    pub intercept: A,
    /// The slope of the line
    pub slope: A,
}

impl<A> StraightLine<A>
where
    A: Float,
{
    /// Fits the data to a straight line using ordinary least squares
    ///
    /// - Time: `O(length)`
    #[cfg_attr(feature = "cargo-clippy", allow(similar_names))]
    pub fn fit(data: Data<A, A>) -> StraightLine<A> {
        let xs = data.0;
        let ys = data.1;

        let x2 = ::dot(xs, xs);
        let xy = ::dot(xs, ys);

        let n = A::cast(xs.len());
        let x2_bar = x2 / n;
        let x_bar = ::sum(xs) / n;
        let xy_bar = xy / n;
        let y_bar = ::sum(ys) / n;

        let slope = {
            let num = xy_bar - x_bar * y_bar;
            let den = x2_bar - x_bar * x_bar;

            num / den
        };

        let intercept = y_bar - slope * x_bar;

        StraightLine { intercept, slope }
    }

    /// Computes the goodness of fit (coefficient of determination) for this data set
    ///
    /// - Time: `O(length)`
    pub fn r_squared(&self, data: Data<A, A>) -> A {
        let _0 = A::cast(0);
        let _1 = A::cast(1);
        let m = self.slope;
        let b = self.intercept;
        let xs = data.0;
        let ys = data.1;

        let n = A::cast(xs.len());
        let y_bar = ::sum(ys) / n;

        let mut ss_res = _0;
        let mut ss_tot = _0;
        for (&x, &y) in data.iter() {
            ss_res = ss_res + (y - m * x - b).powi(2);
            ss_tot = ss_tot + (y - y_bar).powi(2);
        }

        _1 - ss_res / ss_tot
    }
}

#[cfg(test)]
macro_rules! test {
    ($ty:ident) => {
        mod $ty {
            use quickcheck::TestResult;

            use bivariate::Data;
            use bivariate::regression::StraightLine;

            quickcheck!{
                fn r_squared(size: usize, start: usize, offset: usize) -> TestResult {
                    if let Some(x) = ::test::vec::<$ty>(size, start) {
                        let y = ::test::vec::<$ty>(size + offset, start + offset).unwrap();
                        let data = Data::new(&x[start..], &y[start+offset..]);

                        let sl = StraightLine::fit(data);

                        let r_squared = sl.r_squared(data);

                        TestResult::from_bool(
                            (r_squared > 0. || relative_eq!(r_squared, 0.)) &&
                                (r_squared < 1. || relative_eq!(r_squared, 1.))
                        )
                    } else {
                        TestResult::discard()
                    }
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    test!(f32);
    test!(f64);
}
