use std::iter::AdditiveIterator;
use test::stats::Stats;

use self::linspace::LinSpace;

mod linspace;

// XXX Should this be configurable?
static KDE_POINTS: uint = 200;

// Standard Normal Distribution
fn snpdf(x: f64) -> f64 {
    (- x * x / 2.0).exp() / (2f64 * Float::pi()).sqrt()
}

fn linspace(start: f64, end: f64, n: uint) -> LinSpace {
    LinSpace::new(start, end, n)
}

// Kernel Density Estimate
// FIXME unboxed closures -> return iterators instead of vectors?
pub fn kde(xs: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = xs.len() as f64;
    let sigma = xs.std_dev();

    // Bandwith: estimated using Silverman's rule of thumb
    let h = sigma * (4.0 / 3.0 / n).powf(0.2);

    // Kernel: the standard normal distribution
    let k = snpdf;

    let kde = |x: f64| {
        xs.iter().map(|&x_i| k((x - x_i) / h)).sum() / n / h
    };

    // XXX Is 3 bandwidths a good spread to cover the tail of the "normals"?
    let nsigmas = 3.0;
    let start = xs.min() - nsigmas * h;
    let stop = xs.max() + nsigmas * h;

    let mut x = linspace(start, stop, KDE_POINTS);
    let mut y = x.map(kde);

    (x.collect(), y.collect())
}
