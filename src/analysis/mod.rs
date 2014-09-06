use stats::ConfidenceInterval;
use stats::outliers::Outliers;
use stats::regression::Slope ;
use stats::{Sample, mod};
use std::fmt::Show;
use std::io::Command;
use time;

use estimate::{
    Distributions,
    Estimate,
    Estimates,
    Mean,
    Median,
    MedianAbsDev,
    Statistic,
    StdDev,
    mod,
};
use format;
use fs;
use plot;
use program::Program;
use report;
use routine::{Function, Routine};
use {Bencher, Criterion};

macro_rules! elapsed {
    ($msg:expr, $block:expr) => ({
        let start = time::precise_time_ns();
        let out = $block;
        let stop = time::precise_time_ns();

        info!("{} took {}", $msg, format::time((stop - start) as f64));

        out
    })
}

mod compare;

pub fn summarize(id: &str) {
    print!("Summarizing results of {}... ", id);
    plot::summarize(id);
    println!("DONE\n");
}

pub fn function(id: &str, f: |&mut Bencher|:'static, criterion: &Criterion) {
    common(id, &mut Function(f), criterion);

    println!("");
}

pub fn function_with_inputs<I: Show>(
    id: &str,
    f: |&mut Bencher, &I|:'static,
    inputs: &[I],
    criterion: &Criterion,
) {
    for input in inputs.iter() {
        let id = format!("{}/{}", id, input);

        common(id.as_slice(), &mut Function(|b| f(b, input)), criterion);
    }

    summarize(id);
}

pub fn program(id: &str, prog: &Command, criterion: &Criterion) {
    common(id, &mut Program::spawn(prog), criterion);

    println!("");
}

pub fn program_with_inputs<I: Show>(
    id: &str,
    prog: &Command,
    inputs: &[I],
    criterion: &Criterion,
) {
    for input in inputs.iter() {
        let id = format!("{}/{}", id, input);

        program(id.as_slice(), prog.clone().arg(format!("{}", input)), criterion);
    }

    summarize(id);
}

// Common analysis procedure
fn common(id: &str, routine: &mut Routine, criterion: &Criterion) {
    println!("Benchmarking {}", id);

    let pairs = routine.sample(criterion);

    rename_new_dir_to_base(id);

    let pairs_f64 = pairs.iter().map(|&(iters, elapsed)| {
        (iters as f64, elapsed as f64)
    }).collect::<Vec<(f64, f64)>>();

    let times = pairs.iter().map(|&(iters, elapsed)| {
        elapsed as f64 / iters as f64
    }).collect::<Vec<f64>>();
    let times = times.as_slice();

    fs::mkdirp(&Path::new(format!(".criterion/{}/new", id)));
    elapsed!(
        "Plotting sample points",
        plot::sample(times, id));
    elapsed!(
        "Plotting the estimated sample PDF",
        plot::pdf(times, id));

    let outliers = outliers(id, times);
    let slope = regression(id, pairs_f64.as_slice(), criterion);
    let mut estimates = estimates(id, times, criterion);

    estimates.insert(estimate::Slope, slope);

    fs::save(&pairs, &Path::new(format!(".criterion/{}/new/sample.json", id)));
    fs::save(&outliers, &Path::new(format!(".criterion/{}/new/outliers.json", id)));
    fs::save(&estimates, &Path::new(format!(".criterion/{}/new/estimates.json", id)));

    if base_dir_exists(id) {
        compare::common(id, pairs_f64.as_slice(), times, criterion);
    }
}

fn base_dir_exists(id: &str) -> bool {
    Path::new(format!(".criterion/{}/base", id)).exists()
}
// Performs a simple linear regression on the sample
fn regression(id: &str, pairs: &[(f64, f64)], criterion: &Criterion) -> Estimate {
    fn slr(sample: &[(f64, f64)]) -> Slope<f64> {
        Slope::fit(sample)
    }

    let cl = criterion.confidence_level;

    println!("> Performing linear regression");

    let sample = Sample::new(pairs);
    let mut distribution = elapsed!(
        "Bootstrapped linear regression",
        sample.bootstrap(slr, criterion.nresamples).unwrap());

    // Non-interpolating percentiles
    distribution.sort_by(|&x, &y| x.slope().partial_cmp(&y.slope()).unwrap());
    let n = distribution.len() as f64;
    let lb = distribution[(n * (1. - cl) / 2.).round() as uint];
    let ub = distribution[(n * (1. + cl) / 2.).round() as uint];
    let point = Slope::fit(pairs);

    report::regression(pairs, (&lb, &ub));

    elapsed!(
        "Plotting linear regression",
        plot::regression(
            pairs,
            (&lb, &ub),
            id));

    let distribution: Vec<f64> = distribution.move_iter().map(|x| x.slope()).collect();
    let lb = lb.slope();
    let point = point.slope();
    let ub = ub.slope();

    elapsed!(
        "Plotting the distribution of the slope",
        plot::slope(
            distribution.as_slice(),
            (lb, point, ub),
            id));

    Estimate {
        confidence_interval: ConfidenceInterval {
            confidence_level: cl,
            lower_bound: lb,
            upper_bound: ub,
        },
        point_estimate: point,
        standard_error: stats::std_dev(distribution.as_slice()),
    }
}

// Classifies the outliers in the sample
fn outliers(id: &str, times: &[f64]) -> Outliers<f64> {
    let outliers = Outliers::classify(times);

    report::outliers(&outliers);

    fs::save(&outliers, &Path::new(format!(".criterion/{}/new/outliers.json", id)));
    elapsed!(
        "Plotting the outliers",
        plot::outliers(&outliers, times, id));

    outliers
}

// Estimates the statistics of the population from the sample
fn estimates(id: &str, times: &[f64], criterion: &Criterion) -> Estimates {
    static ABS_STATS: &'static [Statistic] = &[Mean, Median, MedianAbsDev, StdDev];

    let abs_stats_fns: Vec<fn(&[f64]) -> f64> =
        ABS_STATS.iter().map(|st| st.abs_fn()).collect();

    let cl = criterion.confidence_level;
    let nresamples = criterion.nresamples;

    let points: Vec<f64> = abs_stats_fns.iter().map(|&f| f(times)).collect();

    println!("> Estimating the statistics of the sample");
    let sample = Sample::new(times.as_slice());
    let distributions = elapsed!(
        "Bootstrapping the absolute statistics",
        sample.bootstrap_many(abs_stats_fns.as_slice(), nresamples));
    let distributions: Distributions =
        ABS_STATS.iter().map(|&x| x).zip(distributions.move_iter()).collect();
    let estimates = Estimate::new(&distributions, points.as_slice(), cl);

    report::abs(&estimates);

    elapsed!(
        "Plotting the distribution of the absolute statistics",
        plot::abs_distributions(
            &distributions,
            &estimates,
            id));

    estimates
}

fn rename_new_dir_to_base(id: &str) {
    let root_dir = Path::new(".criterion").join(id);
    let base_dir = root_dir.join("base");
    let new_dir = root_dir.join("new");

    if base_dir.exists() { fs::rmrf(&base_dir) }
    if new_dir.exists() { fs::mv(&new_dir, &base_dir) };
}
