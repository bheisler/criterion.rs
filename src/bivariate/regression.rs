//! Regression analysis

use std::num::Float;

use cast::CastTo;
use simd::traits::Vector;

use bivariate::Data;

/// A straight line that passes through the origin `y = m * x`
#[derive(Clone, Copy)]
pub struct Slope<A>(pub A) where A: ::Float;

impl<A: ::Float> Slope<A> {
    /// Fits the data to a straight line that passes through the origin using ordinary least
    /// squares
    ///
    /// - Acceleration: BLAS
    /// - Time: `O(length)`
    pub fn fit(data: Data<A, A>) -> Slope<A> {
        let xs = data.0;
        let ys = data.1;

        unsafe {
            let n = xs.len().to::<::blas::blasint>().unwrap();
            let dot = <A as ::blas::Dot>::dot();
            let xs = xs.as_ptr();
            let ys = ys.as_ptr();

            let xy = dot(&n, xs, &1, ys, &1);
            let x2 = dot(&n, xs, &1, xs, &1);

            Slope(xy / x2)
        }
    }

    /// Computes the goodness of fit (coefficient of determination) for this data set
    ///
    /// - Acceleration: SIMD (iff the `X` and `Y` slices are "aligned")
    /// - Time: `O(length)`
    pub fn r_squared(&self, data: Data<A, A>) -> A {
        let m = self.0;
        let xs = data.0;
        let ys = data.1;

        let n = xs.len().to::<A>();
        let y_bar = ::simd::sum(ys) / n;

        let (ss_res, ss_tot) = unsafe {
            let (xhead, xbody, xtail) = <A::Vector as Vector>::cast(xs);
            let (yhead, ybody, ytail) = <A::Vector as Vector>::cast(ys);

            if xhead.len() == yhead.len() {
                let mut ss_res = <A::Vector as Vector>::zeroed();
                let mut ss_tot = <A::Vector as Vector>::zeroed();

                let m_ = <A::Vector as Vector>::from_elem(m);
                let y_bar_ = <A::Vector as Vector>::from_elem(y_bar);

                for i in 0..xbody.len() {
                    let &x_ = xbody.get_unchecked(i);
                    let &y_ = ybody.get_unchecked(i);

                    let diff = y_ - m_ * x_;
                    ss_res = ss_res + diff * diff;

                    let diff = y_ - y_bar_;
                    ss_tot = ss_tot + diff * diff;
                }

                let ss_res = xhead.iter().zip(yhead.iter()).fold(ss_res.sum(), |acc, (&x, &y)| {
                    let diff = y - m * x;

                    acc + diff * diff
                });
                let ss_tot = yhead.iter().fold(ss_tot.sum(), |acc, &y| {
                    let diff = y - y_bar;

                    acc + diff * diff
                });

                let ss_res = xtail.iter().zip(ytail.iter()).fold(ss_res, |acc, (&x, &y)| {
                    let diff = y - m * x;

                    acc + diff * diff
                });
                let ss_tot = ytail.iter().fold(ss_tot, |acc, &y| {
                    let diff = y - y_bar;

                    acc + diff * diff
                });

                (ss_res, ss_tot)
            } else {
                let mut ss_res = Float::zero();

                for i in 0..xs.len() {
                    let &x = xs.get_unchecked(i);
                    let &y = ys.get_unchecked(i);

                    let diff = y - m * x;

                    ss_res = ss_res + diff * diff;
                }

                let ss_tot = ys.iter().fold(Float::zero(), |acc, &y| {
                    let diff = y - y_bar;

                    acc + diff * diff
                });

                (ss_res, ss_tot)
            }
        };

        let _1 = 1.to::<A>();

        _1 - ss_res / ss_tot
    }
}

/// A straight line `y = m * x + b`
#[derive(Clone, Copy)]
pub struct StraightLine<A> where A: ::Float {
    /// The y-intercept of the line
    pub intercept: A,
    /// The slope of the line
    pub slope: A,
}

impl<A: ::Float> StraightLine<A> {
    /// Fits the data to a straight line using ordinary least squares
    ///
    /// - Acceleration: BLAS + SIMD
    /// - Time: `O(length)`
    pub fn fit(data: Data<A, A>) -> StraightLine<A> {
        let xs = data.0;
        let ys = data.1;

        let (x2, xy) = unsafe {
            let dot = <A as ::blas::Dot>::dot();
            let n = xs.len().to::<::blas::blasint>().unwrap();;

            let x2 = dot(&n, xs.as_ptr(), &1, xs.as_ptr(), &1);
            let xy = dot(&n, xs.as_ptr(), &1, ys.as_ptr(), &1);

            (x2, xy)
        };

        let n = xs.len().to::<A>();
        let x2_bar = x2 / n;
        let x_bar = ::simd::sum(xs) / n;
        let xy_bar = xy / n;
        let y_bar = ::simd::sum(ys) / n;

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

    /// Computes the goodness of fit (coefficient of determination) for this data set
    ///
    /// - Acceleration: SIMD (iff the `X` and `Y` slices are "aligned")
    /// - Time: `O(length)`
    pub fn r_squared(&self, data: Data<A, A>) -> A {
        let m = self.slope;
        let b = self.intercept;
        let xs = data.0;
        let ys = data.1;

        let n = xs.len().to::<A>();
        let y_bar = ::simd::sum(ys) / n;

        let (ss_res, ss_tot) = unsafe {
            let (xhead, xbody, xtail) = <A::Vector as Vector>::cast(xs);
            let (yhead, ybody, ytail) = <A::Vector as Vector>::cast(ys);

            if xhead.len() == yhead.len() {
                let mut ss_res = <A::Vector as Vector>::zeroed();
                let mut ss_tot = <A::Vector as Vector>::zeroed();

                let b_ = <A::Vector as Vector>::from_elem(b);
                let m_ = <A::Vector as Vector>::from_elem(m);
                let y_bar_ = <A::Vector as Vector>::from_elem(y_bar);

                for i in 0..xbody.len() {
                    let &x_ = xbody.get_unchecked(i);
                    let &y_ = ybody.get_unchecked(i);

                    let diff = y_ - m_ * x_ - b_;
                    ss_res = ss_res + diff * diff;

                    let diff = y_ - y_bar_;
                    ss_tot = ss_tot + diff * diff;
                }

                let ss_res =
                    xhead.iter().chain(xtail.iter()).zip(yhead.iter().chain(ytail.iter())).
                        fold(ss_res.sum(), |acc, (&x, &y)| {
                            let diff = y - m * x - b;

                            acc + diff * diff
                        });
                let ss_tot = yhead.iter().chain(ytail.iter()).fold(ss_tot.sum(), |acc, &y| {
                    let diff = y - y_bar;

                    acc + diff * diff
                });

                (ss_res, ss_tot)
            } else {
                let mut ss_res = Float::zero();

                for i in 0..xs.len() {
                    let &x = xs.get_unchecked(i);
                    let &y = ys.get_unchecked(i);

                    let diff = y - m * x - b;

                    ss_res = ss_res + diff * diff;
                }

                let ss_tot = ys.iter().fold(Float::zero(), |acc, &y| {
                    let diff = y - y_bar;

                    acc + diff * diff
                });


                (ss_res, ss_tot)
            }
        };

        let _1 = 1.to::<A>();

        _1 - ss_res / ss_tot
    }
}

macro_rules! test {
    ($ty:ident) => {
        mod $ty {
            use quickcheck::TestResult;

            use bivariate::Data;
            use bivariate::regression::StraightLine;

            #[quickcheck]
            fn r_squared(size: usize, start: usize, offset: usize) -> TestResult {
                if let Some(x) = ::test::vec::<$ty>(size, start) {
                    let y = ::test::vec::<$ty>(size + offset, start + offset).unwrap();
                    let data = Data::new(&x[start..], &y[start+offset..]);

                    let sl = StraightLine::fit(data);

                    let r_squared = sl.r_squared(data);

                    TestResult::from_bool(
                        (r_squared > 0. || approx_eq!(r_squared, 0.)) &&
                            (r_squared < 1. || approx_eq!(r_squared, 1.))
                    )
                } else {
                    TestResult::discard()
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    test!(f32);
    test!(f64);
}

macro_rules! bench {
    ($ty:ident) => {
        mod $ty {
            use stdtest::Bencher;

            use bivariate::regression::{Slope, StraightLine};
            use bivariate::Data;

            #[bench]
            fn slope(b: &mut Bencher) {
                let x = ::bench::vec::<$ty>();
                let y = ::bench::vec();
                let data = Data::new(&x, &y);

                b.iter(|| {
                    Slope::fit(data)
                })
            }

            #[bench]
            fn straight_line(b: &mut Bencher) {
                let x = ::bench::vec::<$ty>();
                let y = ::bench::vec();
                let data = Data::new(&x, &y);

                b.iter(|| {
                    StraightLine::fit(data)
                })
            }
        }
    }
}

#[cfg(test)]
mod bench {
    bench!(f32);
    bench!(f64);
}
