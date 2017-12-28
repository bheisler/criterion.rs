use stats::bivariate::Data;
use stats::bivariate::regression::Slope;
use stats::univariate::outliers::tukey::LabeledSample;

use format;
use stats::univariate::Sample;
use estimate::{Distributions, Estimates, Statistic};
use Estimate;

pub(crate) struct ComparisonData {
    pub p_value: f64,
    pub t_value: f64,
    pub relative_estimates: Estimates,
    pub significance_threshold: f64,
    pub noise_threshold: f64,
}

pub(crate) struct MeasurementData<'a> {
    pub iter_counts: &'a Sample<f64>,
    pub sample_times: &'a Sample<f64>,
    pub avg_times: LabeledSample<'a, f64>,
    pub absolute_estimates: Estimates,
    pub distributions: Distributions,
    pub comparison: Option<ComparisonData>,
}

pub(crate) trait Report {
    fn benchmark_start(&self, id: &str);
    fn warmup(&self, id: &str, warmup_ns: f64);
    fn analysis(&self, id: &str);
    fn measurement_start(&self, id: &str, sample_count: u64, estimate_ns: f64, iter_count: u64);
    fn measurement_complete(&self, id: &str, measurements: &MeasurementData);
}

pub(crate) struct CliReport;
impl Report for CliReport {
    fn benchmark_start(&self, id: &str) {
        println!("Benchmarking {}", id);
    }

    fn warmup(&self, _: &str, warmup_ns: f64) {
        println!("> Warming up for {}", format::time(warmup_ns));
    }

    fn analysis(&self, _: &str) {
        println!("> Analyzing");
    }

    fn measurement_start(&self, _: &str, sample_count: u64,
        estimate_ns: f64, iter_count: u64) {
        println!("> Collecting {} samples in estimated {} ({} iterations)",
            sample_count, format::time(estimate_ns), iter_count);
    }

    fn measurement_complete(&self, id: &str, meas: &MeasurementData) {
        outliers(&meas.avg_times);
        println!("> Performing linear regression");

        let data = Data::new(meas.iter_counts.as_slice(), meas.sample_times.as_slice());
        let slope_estimate = meas.absolute_estimates.get(&Statistic::Slope).unwrap();
        regression(data,
            (Slope(slope_estimate.confidence_interval.lower_bound),
             Slope(slope_estimate.confidence_interval.upper_bound)));

        println!("> Estimating the statistics of the sample");
        abs(&meas.absolute_estimates);

        if let &Some(ref comp) = &meas.comparison {
            println!("{}: Comparing with previous sample", id);
            println!("> Performing a two-sample t-test");
            println!("  > H0: Both samples have the same mean");
            println!("  > p = {}", comp.p_value);
            let different_mean = comp.p_value < comp.significance_threshold;
            println!("  > {} reject the null hypothesis",
                if different_mean { "Strong evidence to" } else { "Can't" });
            if different_mean {
                println!("> Estimating relative change of statistics");
                rel(&comp.relative_estimates);
                let mut regressed = true;
                for (&statistic, estimate) in comp.relative_estimates.iter() {
                    let result = compare_to_threshold(estimate, comp.noise_threshold);

                    let p = estimate.point_estimate;
                    match result {
                        ComparisonResult::Improved => {
                            println!("  > {} has improved by {:.2}%", statistic, -100.0 * p);
                            regressed = false;
                        },
                        ComparisonResult::Regressed => {
                            println!("  > {} has regressed by {:.2}%", statistic, 100.0 * p);
                        },
                        ComparisonResult::NonSignificant => {
                            regressed = true;
                        },
                    }
                }
                if regressed {
                    println!("{} has regressed", id);
                }
            }
        }
    }
}

pub fn abs(estimates: &Estimates) {
    for (&statistic, estimate) in estimates.iter() {
        let ci = estimate.confidence_interval;
        let lb = format::time(ci.lower_bound);
        let ub = format::time(ci.upper_bound);

        println!("  > {:>6} [{} {}]", statistic, lb, ub);
    }
}

pub fn rel(estimates: &Estimates) {
    for (&statistic, estimate) in estimates.iter() {
        let ci = estimate.confidence_interval;
        let lb = format::change(ci.lower_bound, true);
        let ub = format::change(ci.upper_bound, true);

        println!("  > {:>6} [{} {}]", statistic, lb, ub);
    }
}

pub fn outliers(sample: &LabeledSample<f64>) {
    let (los, lom, _, him, his) = sample.count();
    let noutliers = los + lom + him + his;
    let sample_size = sample.as_slice().len();

    if noutliers == 0 {
        return;
    }

    let percent = |n: usize| { 100. * n as f64 / sample_size as f64 };

    println!("> Found {} outliers among {} measurements ({:.2}%)",
             noutliers,
             sample_size,
             percent(noutliers));

    let print = |n, label| {
        if n != 0 {
            println!("  > {} ({:.2}%) {}", n, percent(n), label);
        }
    };

    print(los, "low severe");
    print(lom, "low mild");
    print(him, "high mild");
    print(his, "high severe");
}

pub fn regression(data: Data<f64, f64>, (lb, ub): (Slope<f64>, Slope<f64>)) {
    println!(
        "  > {:>6} [{} {}]",
        "slope",
        format::time(lb.0),
        format::time(ub.0),
        );

    println!(
         "  > {:>6}  {:0.7} {:0.7}",
         "R^2",
         lb.r_squared(data),
         ub.r_squared(data));
}

enum ComparisonResult {
    Improved,
    Regressed,
    NonSignificant,
}

fn compare_to_threshold(estimate: &Estimate, noise: f64) -> ComparisonResult {
    let ci = estimate.confidence_interval;
    let lb = ci.lower_bound;
    let ub = ci.upper_bound;

    if lb < -noise && ub < -noise {
        ComparisonResult::Improved
    } else if lb > noise && ub > noise {
        ComparisonResult::Regressed
    } else {
        ComparisonResult::NonSignificant
    }
}