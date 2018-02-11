use itertools_num;
use stats::univariate::Sample;
use stats::univariate::kde::kernel::Gaussian;
use stats::univariate::kde::{Bandwidth, Kde};

pub fn sweep(
    sample: &Sample<f64>,
    npoints: usize,
    range: Option<(f64, f64)>,
) -> (Box<[f64]>, Box<[f64]>) {
    let (xs, ys, _) = sweep_and_estimate(sample, npoints, range, sample.as_slice()[0]);
    (xs, ys)
}

pub fn sweep_and_estimate(
    sample: &Sample<f64>,
    npoints: usize,
    range: Option<(f64, f64)>,
    point_to_estimate: f64,
) -> (Box<[f64]>, Box<[f64]>, f64) {
    let x_min = sample.min();
    let x_max = sample.max();

    let kde = Kde::new(sample, Gaussian, Bandwidth::Silverman);
    let h = kde.bandwidth();

    let (start, end) = match range {
        Some((start, end)) => (start, end),
        None => (x_min - 3. * h, x_max + 3. * h),
    };

    let xs: Vec<_> = itertools_num::linspace(start, end, npoints).collect();
    let ys = kde.map(&xs);
    let point_estimate = kde.estimate(point_to_estimate);

    (xs.into_boxed_slice(), ys, point_estimate)
}
