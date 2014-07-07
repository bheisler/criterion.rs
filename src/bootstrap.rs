use std::collections::HashMap;
use std::rand;
use test::stats::Stats;

use criterion::Criterion;
use sample::Sample;
use units::{AsPercent,AsSignedPercent,AsTime};

// FIXME UCFS may make these unnecessary, e.g. `<&[f64]>::mean`?
fn mean(xs: &[f64]) -> f64 { xs.mean() }
fn median(xs: &[f64]) -> f64 { xs.median() }
fn median_abs_dev(xs: &[f64]) -> f64 { xs.median_abs_dev() }
fn std_dev(xs: &[f64]) -> f64 { xs.std_dev() }

static STATISTICS_FNS: &'static [fn(&[f64]) -> f64] =
    &[mean, median, median_abs_dev, std_dev];

static STATISTICS_NAMES: &'static [&'static str] =
    &["mean", "median", "MAD", "SD"];

fn bootstrap(sample: &[f64],
             nresamples: uint,
             statistics: &[fn(&[f64]) -> f64])
             -> (Vec<f64>, Vec<Vec<f64>>) {
    let mut resamples = Resamples::new(sample);
    let mut boots: Vec<Vec<f64>> = range(0, statistics.len()).map(|_| {
        Vec::with_capacity(nresamples)
    }).collect();

    for _ in range(0, nresamples) {
        let resample = resamples.next();

        for (statistic, boot) in statistics.iter().zip(boots.mut_iter()) {
            boot.push((*statistic)(resample));
        }
    }

    let points = statistics.iter().map(|&f| f(sample)).collect();

    (points, boots)
}

fn diff(base: &[f64],
        new: &[f64],
        nresamples: uint,
        statistics: &[fn(&[f64]) -> f64])
        -> (Vec<f64>, Vec<Vec<f64>>) {
    let (mut points, mut boots) = bootstrap(base, nresamples, statistics);
    let (new_points, new_boots) = bootstrap(new, nresamples, statistics);

    for (boot, new_boot) in zip!(boots.mut_iter(), new_boots.iter()) {
        for (b, &n) in zip!(boot.mut_iter(), new_boot.iter()) {
            *b = n / *b - 1.0
        }
    }

    for (point, &new_point) in zip!(points.mut_iter(), new_points.iter()) {
        *point = new_point / *point - 1.0;
    }

    (points, boots)
}

pub fn compare(base: &[f64], new: &[f64], criterion: &Criterion) {
    let cl = criterion.confidence_level;
    let nresamples = criterion.nresamples;

    println!("> comparing with previous sample");
    println!("  > bootstrapping sample with {} resamples", nresamples);

    let (points, boots) = diff(base, new, nresamples,
                               STATISTICS_FNS.slice_to(2));

    for (name, (point, boot)) in zip!(
        STATISTICS_NAMES.slice_to(2).iter(),
        points.move_iter(),
        boots.move_iter(),
    ) {
        let estimate = Estimate::new(point, boot.as_slice(), cl);

        println!("  > {:<6} {}", name, estimate.as_percent());
    }
}

pub fn estimate(sample: &Sample,
                nresamples: uint,
                cl: f64)
                -> HashMap<&'static str, Estimate> {
    assert!(cl > 0.0 && cl < 1.0,
            "confidence level must be between 0.0 and 1.0");

    println!("> estimating statistics");
    println!("  > bootstrapping sample with {} resamples", nresamples);

    let mut estimates = HashMap::new();
    let (points, boots) = bootstrap(sample.data(), nresamples, STATISTICS_FNS);

    for (&name, (point, boot)) in zip!(
        STATISTICS_NAMES.iter(),
        points.move_iter(),
        boots.move_iter(),
    ) {
        let estimate = Estimate::new(point, boot.as_slice(), cl);

        estimates.insert(name, estimate);

        println!("  > {:<6} {}", name, estimate.as_time());
    }

    estimates
}
