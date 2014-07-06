use std::rand::TaskRng;
use std::rand::distributions::{IndependentSample,Range};
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

pub struct Estimate {
    confidence_level: f64,
    lower_bound: f64,
    point: f64,
    standard_error: f64,
    upper_bound: f64,
}

impl Estimate {
    fn new(point: f64, bootstrap: &[f64], cl: f64) -> Estimate {
        let standard_error = bootstrap.std_dev();
        let lower_bound = bootstrap.percentile(50.0 * (1.0 - cl));
        let upper_bound = bootstrap.percentile(50.0 * (1.0 + cl));

        Estimate {
            confidence_level: cl,
            lower_bound: lower_bound,
            point: point,
            standard_error: standard_error,
            upper_bound: upper_bound,
        }
    }
}

impl AsPercent for Estimate {
    fn as_percent(&self) -> String {
        format!("{} ± {} [{} {}] {}% CI",
                self.point.as_signed_percent(),
                self.standard_error.as_percent(),
                self.lower_bound.as_signed_percent(),
                self.upper_bound.as_signed_percent(),
                self.confidence_level * 100.0)
    }
}

impl AsTime for Estimate {
    fn as_time(&self) -> String {
        format!("{} ± {} [{} {}] {}% CI",
                self.point.as_time(),
                self.standard_error.as_time(),
                self.lower_bound.as_time(),
                self.upper_bound.as_time(),
                self.confidence_level * 100.0)
    }
}

pub fn estimate(sample: &Sample, nresamples: uint, cl: f64) {
    assert!(cl > 0.0 && cl < 1.0,
            "confidence level must be between 0.0 and 1.0");

    println!("> estimating statistics");
    println!("  > bootstrapping sample with {} resamples", nresamples);

    let (points, boots) = bootstrap(sample.data(), nresamples, STATISTICS_FNS);

    for (name, (point, boot)) in zip!(
        STATISTICS_NAMES.iter(),
        points.move_iter(),
        boots.move_iter(),
    ) {
        let estimate = Estimate::new(point, boot.as_slice(), cl);

        println!("  > {:<6} {}", name, estimate.as_time());
    }
}

struct Resamples<'a> {
    range: Range<uint>,
    rng: TaskRng,
    sample: &'a [f64],
    stage: Vec<f64>,
}

impl<'a> Resamples<'a> {
    pub fn new(sample: &'a [f64]) -> Resamples<'a> {
        let size = sample.len();

        Resamples {
            range: Range::new(0, size),
            rng: rand::task_rng(),
            sample: sample,
            stage: Vec::from_elem(size, 0f64),
        }
    }

    pub fn next<'b>(&'b mut self) -> &'b [f64] {
        let size = self.sample.len();

        // resampling *with* replacement
        for i in range(0, size) {
            let j = self.range.ind_sample(&mut self.rng);

            self.stage.as_mut_slice()[i] = self.sample[j];
        }

        self.stage.as_slice()
    }
}

#[cfg(bench)]
mod bench {
    use std::rand;
    use {Bencher,Criterion};

    static NSAMPLES: uint = 100;
    static NRESAMPLES: uint = 1_000;

    #[test]
    fn mean() {
        let mut c = Criterion::new();

        c.bench("bootstrap_mean", |b: &mut Bencher| {
            let xs: Vec<f64> = range(0, NSAMPLES).map(|_| {
                rand::random()
            }).collect();
            let xs = xs.as_slice();

            b.iter(|| {
                super::bootstrap(xs, NRESAMPLES, &[super::mean])
            });
        });
    }

    #[test]
    fn statistics() {
        let mut c = Criterion::new();

        c.bench("bootstrap_statistics", |b: &mut Bencher| {
            let xs: Vec<f64> = range(0, NSAMPLES).map(|_| {
                rand::random()
            }).collect();
            let xs = xs.as_slice();

            b.iter(|| {
                super::bootstrap(xs, NRESAMPLES, super::STATISTICS_FNS)
            });
        });
    }

    #[test]
    fn diff_mean() {
        let mut c = Criterion::new();

        c.bench("bootstrap_diff_mean", |b: &mut Bencher| {
            let xs: Vec<f64> = range(0, NSAMPLES).map(|_| {
                rand::random()
            }).collect();
            let ys: Vec<f64> = range(0, NSAMPLES).map(|_| {
                rand::random()
            }).collect();
            let xs = xs.as_slice();
            let ys = ys.as_slice();

            b.iter(|| {
                super::diff(xs, ys, NRESAMPLES, &[super::mean])
            });
        });
    }
}
