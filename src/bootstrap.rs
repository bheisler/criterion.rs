use bencher::BencherConfig;
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;
use rand::{TaskRng,task_rng};
use sample::Sample;
use test::stats::Stats;
use units::AsTime;

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

    let mut mads = Vec::with_capacity(nresamples);
    let mut means = Vec::with_capacity(nresamples);
    let mut medians = Vec::with_capacity(nresamples);
    let mut std_devs = Vec::with_capacity(nresamples);

    let mut resamples = Resamples::new(sample.data());
    for _ in range(0, nresamples) {
        let resample = resamples.next();

        mads.push(resample.median_abs_dev());
        means.push(resample.mean());
        medians.push(resample.median());
        std_devs.push(resample.std_dev());
    }

    let mad = Estimate::new(sample.median_abs_dev(), mads.as_slice(), cl);
    let mean = Estimate::new(sample.mean(), means.as_slice(), cl);
    let median = Estimate::new(sample.median(), medians.as_slice(), cl);
    let std_dev = Estimate::new(sample.std_dev(), std_devs.as_slice(), cl);

    println!("  > mean:   {}", mean.report());
    println!("  > SD:     {}", std_dev.report());
    println!("  > median: {}", median.report());
    println!("  > MAD:    {}", mad.report());
}

pub fn same_population(x: &[f64], y: &[f64], config: &BencherConfig) {
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
            println!("    > both mean and median contradict H0");
        },
        (true, false) => {
            println!("    > mean contradicts H0");
        },
        (false, true) => {
            println!("    > median contradicts H0");
        },
        (false, false) => {
            println!("    > no evidence to contradict H0");
        },
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
            range: Range::new(0, size - 1),
            rng: task_rng(),
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
