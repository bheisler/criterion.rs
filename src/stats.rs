use std::num;
use test::stats::Stats;

#[experimental]
pub fn mean<A: FloatMath + FromPrimitive>(x: &[A]) -> A {
    x.mean()
}

#[experimental]
pub fn median<A: FloatMath + FromPrimitive>(x: &[A]) -> A {
    x.median()
}

#[experimental]
pub fn median_abs_dev<A: FloatMath + FromPrimitive>(x: &[A]) -> A {
    x.median_abs_dev()
}

#[experimental]
pub fn std_dev<A: FloatMath + FromPrimitive>(x: &[A]) -> A {
    x.std_dev()
}

/// Computes the Welch t-statistic between two samples
#[experimental]
pub fn t<A: FloatMath + FromPrimitive>(x: &[A], y: &[A]) -> A {
    let (x_bar, y_bar) = (mean(x), mean(y));
    let (s2_x, s2_y) = (var(x), var(y));
    let (n_x, n_y) = (num::cast::<_, A>(x.len()).unwrap(), num::cast::<_, A>(y.len()).unwrap());

    let num = x_bar - y_bar;
    let den = (s2_x / n_x + s2_y / n_y).sqrt();

    num / den
}

#[experimental]
pub fn var<A: FloatMath + FromPrimitive>(x: &[A]) -> A {
    x.var()
}
