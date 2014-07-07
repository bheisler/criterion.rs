use std::collections::HashMap;
use test::stats::Stats;

use statistics::bootstrap;
use statistics::estimate::Estimate;
use units::{AsPercent,AsTime};

type Estimates = HashMap<&'static str, Estimate>;

macro_rules! zip {
    ($x:expr) => {
        $x
    };
    ($x:expr, $($y:expr),+) => {
        $x.zip(zip!($($y),+))
    };
    ($($x:expr),+,) => {
        zip!($($x),+)
    };
}

pub fn mean(xs: &[f64]) -> f64 { xs.mean() }
pub fn median(xs: &[f64]) -> f64 { xs.median() }
pub fn median_abs_dev(xs: &[f64]) -> f64 { xs.median_abs_dev() }
pub fn std_dev(xs: &[f64]) -> f64 { xs.std_dev() }

// FIXME UFCS -> Replace `mean` these with `<&[f64]>::mean` ?
static ESTIMATOR_FNS: &'static [fn (&[f64]) -> f64] = &[
    mean,
    median,
    std_dev,
    median_abs_dev,
];

static ESTIMATOR_NAMES: &'static [&'static str] = &[
    "mean",
    "median",
    "MAD",
    "SD",
];

pub fn estimate_statistics(sample: &[f64],
                           nresamples: uint,
                           cl: f64)
                           -> Estimates {
    let mut estimates = HashMap::new();
    let distributions = bootstrap::estimate(sample, nresamples, ESTIMATOR_FNS);

    println!("> estimating statistics");
    println!("  > bootstrapping sample with {} resamples", nresamples);

    for (estimator, (&name, distribution)) in zip!(
        ESTIMATOR_FNS.iter(),
        ESTIMATOR_NAMES.iter(),
        distributions.iter(),
    ) {
        let point = (*estimator)(sample);
        let estimate =
            Estimate::new(point, distribution.as_slice(), cl);

        estimates.insert(name, estimate);

        println!("  > {:<6} {}", name, estimate.as_time());
    }

    estimates
}

fn diff_mean(this: &[f64], that: &[f64]) -> f64 {
    that.mean() / this.mean() - 1.0
}

fn diff_median(this: &[f64], that: &[f64]) -> f64 {
    that.median() / this.median() - 1.0
}

static COMPARATOR_FNS: &'static [fn (&[f64], &[f64]) -> f64] = &[
    diff_mean,
    diff_median,
];

static COMPARATOR_NAMES: &'static [&'static str] = &[
    "mean",
    "median",
];

pub fn compare_samples(this: &[f64],
                       that: &[f64],
                       nresamples: uint,
                       cl: f64)
                       -> Estimates {
    let mut estimates = HashMap::new();
    let nresamples = (nresamples as f64).sqrt().ceil() as uint;
    let distributions =
        bootstrap::compare(this, that, nresamples, COMPARATOR_FNS);

    println!("> comparing with previous sample");
    println!("  > bootstrapping sample with {} resamples",
             nresamples * nresamples);

    for (comparator, (&name, distribution)) in zip!(
        COMPARATOR_FNS.iter(),
        COMPARATOR_NAMES.iter(),
        distributions.iter(),
    ) {
        let point = (*comparator)(this, that);
        let estimate =
            Estimate::new(point, distribution.as_slice(), cl);

        estimates.insert(name, estimate);

        println!("  > {:<6} {}", name, estimate.as_percent());
    }

    estimates
}
