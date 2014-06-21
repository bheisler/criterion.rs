use criterion::CriterionConfig;
use sample::Sample;
use std::rand::TaskRng;
use std::rand::distributions::{IndependentSample,Range};
use std::rand;
use test::stats::Stats;
use units::AsTime;

// FIXME UCFS may make these unnecessary, e.g. `<&[f64]>::mean`?
fn mean(xs: &[f64]) -> f64 { xs.mean() }
fn median(xs: &[f64]) -> f64 { xs.median() }
fn median_abs_dev(xs: &[f64]) -> f64 { xs.median_abs_dev() }
fn std_dev(xs: &[f64]) -> f64 { xs.std_dev() }

static STATISTICS: &'static [fn(&[f64]) -> f64] =
    &[mean, median, median_abs_dev, std_dev];

fn bootstrap(sample: &[f64],
             nresamples: uint,
             statistics: &[fn(&[f64]) -> f64])
             -> Vec<Vec<f64>> {
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

    boots
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

impl Estimate {
    fn report(&self) -> String {
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

    let boots = bootstrap(sample.data(), nresamples, STATISTICS);
    let boots = boots.as_slice();

    let mad = Estimate::new(sample.median_abs_dev(), boots[2].as_slice(), cl);
    let mean = Estimate::new(sample.mean(), boots[0].as_slice(), cl);
    let median = Estimate::new(sample.median(), boots[1].as_slice(), cl);
    let std_dev = Estimate::new(sample.std_dev(), boots[3].as_slice(), cl);

    println!("  > mean:   {}", mean.report());
    println!("  > SD:     {}", std_dev.report());
    println!("  > median: {}", median.report());
    println!("  > MAD:    {}", mad.report());
}

// Null hypothesis: new.mean() == old.mean() || new.median() == old.median()
// Alternative hypothesis: samples don't belong to the same population
pub fn same_population(x: &[f64],
                       y: &[f64],
                       config: &CriterionConfig)
    -> bool
{
    println!("  > H0: both samples belong to the same population");

    let (n_x, n_y) = (x.len(), y.len());
    let nresamples = config.nresamples;
    let sl = config.significance_level;

    let t_mean = (x.mean() - y.mean()).abs();
    let t_median = (x.median() - y.median()).abs();

    let mut z = Vec::with_capacity(n_x + n_y);
    z.push_all(x);
    z.push_all(y);

    println!("    > bootstrapping with {} resamples", nresamples);
    let (mut n_mean, mut n_median) = (0, 0);
    let mut resamples = Resamples::new(z.as_slice());
    for _ in range(0, nresamples) {
        let resample = resamples.next();
        let x = resample.slice_to(n_x);
        let y = resample.slice_from(n_x);

        if (x.mean() - y.mean()).abs() > t_mean {
            n_mean += 1;
        }

        if (x.median() - y.median()).abs() > t_median {
            n_median += 1;
        }
    }

    let p_mean = n_mean as f64 / nresamples as f64;
    let p_median = n_median as f64 / nresamples as f64;

    match (p_mean < sl, p_median < sl) {
        (true, true) => {
            println!("    > both mean and median contradict H0 ({}, {} < {})",
                     p_mean, p_median, sl);
            false
        },
        (true, false) => {
            println!("    > mean contradicts H0 ({} < {})", p_mean, sl);
            true
        },
        (false, true) => {
            println!("    > median contradicts H0 ({} < {})", p_median, sl);
            true
        },
        (false, false) => {
            println!("    > no evidence to contradict H0");
            true
        },
    }
}

// Null hypothesis: new.mean() <= old.mean() + 3 * standard_error
// Alternative hypothesis: new.mean() has regressed X%
// Bootstrap hypothesis testing using Welch T statistic
// http://en.wikipedia.org/wiki/Welch_t_test
pub fn mean_regressed(x: &[f64], y: &[f64], config: &CriterionConfig) -> bool {
    let (mu_x, mu_y) = (x.mean(), y.mean());
    let (n_x, n_y) = (x.len() as f64, y.len() as f64);
    let diff = mu_y / mu_x - 1.0;
    let nresamples = config.nresamples;
    let sl = config.significance_level;

    if diff < 0.0 {
        println!("  > H0: new mean >= old mean");
        println!("  > Ha: mean improved by {:.2}%", -diff * 100.0);
    } else {
        println!("  > H0: new mean <= old mean");
        println!("  > Ha: mean regressed by {:.2}%", diff * 100.0);
    }

    println!("    > bootstrapping with {} resamples", nresamples);
    let mut n_t = 0;
    let mut x_resamples = Resamples::new(x);
    let mut y_resamples = Resamples::new(y);
    for _ in range(0, nresamples) {
        let x = x_resamples.next();
        let y = y_resamples.next();

        let (mu_x, mu_y) = (x.mean(), y.mean());
        let (sigma_x, sigma_y) = (x.std_dev(), y.std_dev());

        let num = mu_x - mu_y;
        let den = (sigma_x * sigma_x / n_x + sigma_y * sigma_y / n_y).sqrt();
        let t = num / den;

        if (diff < 0.0 && t <= 3.0) || (diff > 0.0 && t >= -3.0 ) {
            n_t += 1;
        }
    }

    let p = n_t as f64 / nresamples as f64;

    if p < sl {
        println!("    > strong evidence to contradict H0 ({} < {})", p, sl);
        diff > 0.0
    } else {
        println!("    > no evidence to contradict H0 ({} > {})", p, sl);
        false
    }
}

// Null hypothesis: new.median() <= old.median() + 3 * standard_error
// Alternative hypothesis: new.median() has regressed X%
// Bootstrap hypothesis testing using Welch T statistic
pub fn median_regressed(x: &[f64],
                        y: &[f64],
                        config: &CriterionConfig)
    -> bool
{
    let (mu_x, mu_y) = (x.median(), y.median());
    let (n_x, n_y) = (x.len() as f64, y.len() as f64);
    let diff = mu_y / mu_x - 1.0;
    let nresamples = config.nresamples;
    let sl = config.significance_level;

    if diff < 0.0 {
        println!("  > H0: new median >= old median");
        println!("  > Ha: median improved by {:.2}%", -diff * 100.0);
    } else {
        println!("  > H0: new median <= old median");
        println!("  > Ha: median regressed by {:.2}%", diff * 100.0);
    }

    println!("    > bootstrapping with {} resamples", nresamples);
    let mut n_t = 0;
    let mut x_resamples = Resamples::new(x);
    let mut y_resamples = Resamples::new(y);
    for _ in range(0, nresamples) {
        let x = x_resamples.next();
        let y = y_resamples.next();

        let (mu_x, mu_y) = (x.median(), y.median());
        let (sigma_x, sigma_y) = (x.median_abs_dev(), y.median_abs_dev());

        let num = mu_x - mu_y;
        let den = (sigma_x * sigma_x / n_x + sigma_y * sigma_y / n_y).sqrt();
        let t = num / den;

        if (diff < 0.0 && t <= 3.0) || (diff > 0.0 && t >= -3.0 ) {
            n_t += 1;
        }
    }

    let p = n_t as f64 / nresamples as f64;

    if p < sl {
        println!("    > strong evidence to contradict H0 ({} < {})", p, sl);
        diff > 0.0
    } else {
        println!("    > no evidence to contradict H0 ({} > {})", p, sl);
        false
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
            stage: Vec::from_elem(size, 0.0),
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
    static NRESAMPLES: uint = 100_000;

    #[test]
    fn bootstrap_mean() {
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
    fn bootstrap_statistics() {
        let mut c = Criterion::new();

        c.bench("bootstrap_statistics", |b: &mut Bencher| {
            let xs: Vec<f64> = range(0, NSAMPLES).map(|_| {
                rand::random()
            }).collect();
            let xs = xs.as_slice();

            b.iter(|| {
                super::bootstrap(xs, NRESAMPLES, super::STATISTICS)
            });
        });
    }

    #[test]
    fn bootstrap_mean_diff() {
        let mut c = Criterion::new();

        c.bench("bootstrap_mean_diff", |b: &mut Bencher| {
            let xs: Vec<f64> = range(0, NSAMPLES).map(|_| {
                rand::random()
            }).collect();
            let ys: Vec<f64> = range(0, NSAMPLES).map(|_| {
                rand::random()
            }).collect();
            let xs = xs.as_slice();
            let ys = ys.as_slice();

            b.iter(|| {
                let xs = super::bootstrap(xs, NRESAMPLES, &[super::mean]);
                let ys = super::bootstrap(ys, NRESAMPLES, &[super::mean]);

                xs.get(0).iter().zip(ys.get(0).iter()).map(|(x, &y)| {
                    x / y - 1.0
                }).collect::<Vec<f64>>()
            });
        });
    }
}
