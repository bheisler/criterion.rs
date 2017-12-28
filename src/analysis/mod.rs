use std::fmt;
use std::iter::IntoIterator;
use std::path::Path;
use std::process::Command;

use stats::Distribution;
use stats::bivariate::Data;
use stats::bivariate::regression::Slope;
use stats::univariate::Sample;
use stats::univariate::outliers::tukey::{LabeledSample, self};

use estimate::{Distributions, Estimates, Statistic};
use program::Program;
use routine::{Function, Routine};
use {Bencher, ConfidenceInterval, Criterion, Estimate};
use {format, fs, plot};
use ::Fun;

macro_rules! elapsed {
    ($msg:expr, $block:expr) => ({
        let start = ::std::time::Instant::now();
        let out = $block;
        let elapsed = &start.elapsed();

        info!("{} took {}", $msg, format::time(::DurationExt::to_nanos(elapsed) as f64));

        out
    })
}

mod compare;

pub fn summarize(id: &str, criterion: &Criterion) {
    if criterion.plotting.is_enabled() {
        print!("Summarizing results of {}... ", id);
        plot::summarize(id);
        println!("DONE\n");
    } else {
        println!("Plotting disabled, skipping summarization");
    }
}

pub fn function<F>(id: &str, f: F, criterion: &Criterion) where F: FnMut(&mut Bencher) {
    common(id, &mut Function(f), criterion);

    println!();
}

pub fn functions<I>(id: &str,
    funs: Vec<Fun<I>>,
    input: &I,
    criterion: &Criterion)
    where I: fmt::Display
{
    for fun in funs {
        let id = format!("{}/{}", id, fun.n);
        let mut f = fun.f;
        common(&id, &mut Function(|b| f(b, input)), criterion);
    }
    summarize(id, criterion);
}

pub fn function_over_inputs<I, F>(
    id: &str,
    mut f: F,
    inputs: I,
    criterion: &Criterion,
) where
    F: FnMut(&mut Bencher, &I::Item),
    I: IntoIterator,
    I::Item: fmt::Display,
{
    for input in inputs {
        let id = format!("{}/{}", id, input);

        common(&id, &mut Function(|b| f(b, &input)), criterion);
    }

    summarize(id, criterion);
}

pub fn program(id: &str, prog: &mut Command, criterion: &Criterion) {
    common(id, &mut Program::spawn(prog), criterion);

    println!();
}

pub fn program_over_inputs<I, F>(
    id: &str,
    mut prog: F,
    inputs: I,
    criterion: &Criterion,
) where
    F: FnMut() -> Command,
    I: IntoIterator,
    I::Item: fmt::Display,
{
    for input in inputs {
        let id = format!("{}/{}", id, input);

        program(&id, prog().arg(&format!("{}", input)), criterion);
    }

    summarize(id, criterion);
}

// Common analysis procedure
fn common<R>(id: &str, routine: &mut R, criterion: &Criterion) where
    R: Routine,
{
    criterion.report.benchmark_start(id);

    let (iters, times) = routine.sample(id, criterion);

    criterion.report.analysis(id);

    rename_new_dir_to_base(id);

    let avg_times = iters.iter().zip(times.iter()).map(|(&iters, &elapsed)| {
        elapsed / iters
    }).collect::<Vec<f64>>();
    let avg_times = Sample::new(&avg_times);

    log_if_err!(fs::mkdirp(&format!(".criterion/{}/new", id)));

    let data = Data::new(&iters, &times);
    let labeled_sample = outliers(id, avg_times);
    let (distribution, slope) = regression(id, data, criterion);
    let (mut distributions, mut estimates) = estimates(avg_times, criterion);

    estimates.insert(Statistic::Slope, slope);
    distributions.insert(Statistic::Slope, distribution);

    if criterion.plotting.is_enabled() {
        elapsed!(
            "Plotting the estimated sample PDF",
            plot::pdf(data, labeled_sample, id));
        elapsed!(
            "Plotting the distribution of the absolute statistics",
            plot::abs_distributions(
                &distributions,
                &estimates,
                id));
    }

    log_if_err!(fs::save(
        &(data.x().as_slice(), data.y().as_slice()),
        &format!(".criterion/{}/new/sample.json", id),
    ));
    log_if_err!(fs::save(&estimates, &format!(".criterion/{}/new/estimates.json", id)));

    let compare_data = if base_dir_exists(id) {
        let result = compare::common(id, data, avg_times, &estimates, criterion);
        match result {
            Ok((t_val, p_val, rel_est)) => {
                Some(::report::ComparisonData {
                    p_value: p_val,
                    t_value: t_val,
                    relative_estimates: rel_est, 
                    significance_threshold: criterion.significance_level,
                    noise_threshold: criterion.noise_threshold,
                })
            }
            Err(e) => {
                ::error::log_error(e);
                None
            }
        }
    } else { None };

    let measurement_data = ::report::MeasurementData {
        iter_counts: &Sample::new(&*iters),
        sample_times: &Sample::new(&*times),
        avg_times: labeled_sample,
        absolute_estimates: estimates.clone(),
        distributions: distributions,
        comparison: compare_data,
    };

    criterion.report.measurement_complete(id, &measurement_data);
}

fn base_dir_exists(id: &str) -> bool {
    Path::new(&format!(".criterion/{}/base", id)).exists()
}

// Performs a simple linear regression on the sample
fn regression(
    id: &str,
    data: Data<f64, f64>,
    criterion: &Criterion,
) -> (Distribution<f64>, Estimate) {
    let cl = criterion.confidence_level;

    let distribution = elapsed!(
        "Bootstrapped linear regression",
        data.bootstrap(criterion.nresamples, |d| (Slope::fit(d).0,))).0;

    let point = Slope::fit(data);
    let (lb, ub) =  distribution.confidence_interval(criterion.confidence_level);
    let se = distribution.std_dev(None);

    let (lb_, ub_) = (Slope(lb), Slope(ub));

    if criterion.plotting.is_enabled() {
        elapsed!(
            "Plotting linear regression",
            plot::regression(
                data,
                &point,
                (lb_, ub_),
                id));
    }

    (distribution, Estimate {
        confidence_interval: ConfidenceInterval {
            confidence_level: cl,
            lower_bound: lb,
            upper_bound: ub,
        },
        point_estimate: point.0,
        standard_error: se,
    })
}

// Classifies the outliers in the sample
fn outliers<'a>(id: &str, avg_times: &'a Sample<f64>) -> LabeledSample<'a, f64> {
    let sample = tukey::classify(avg_times);
    log_if_err!(fs::save(&sample.fences(), &format!(".criterion/{}/new/tukey.json", id)));
    sample
}

// Estimates the statistics of the population from the sample
fn estimates(
    avg_times: &Sample<f64>,
    criterion: &Criterion,
) -> (Distributions, Estimates) {
    fn stats(sample: &Sample<f64>) -> (f64, f64, f64, f64) {
        let mean = sample.mean();
        let std_dev = sample.std_dev(Some(mean));
        let median = sample.percentiles().median();
        let mad = sample.median_abs_dev(Some(median));

        (mean, median, mad, std_dev)
    }

    let cl = criterion.confidence_level;
    let nresamples = criterion.nresamples;

    let points = {
        let (a, b, c, d) = stats(avg_times);

        [a, b, c, d]
    };

    let distributions = {
        let (a, b, c, d) = elapsed!(
        "Bootstrapping the absolute statistics",
        avg_times.bootstrap(nresamples, stats));

        vec![a, b, c, d]
    };
    let statistics = [
        Statistic::Mean,
        Statistic::Median,
        Statistic::MedianAbsDev,
        Statistic::StdDev,
    ];
    let distributions: Distributions = statistics.iter().cloned()
        .zip(distributions.into_iter()).collect();
    let estimates = Estimate::new(&distributions, &points, cl);

    (distributions, estimates)
}

fn rename_new_dir_to_base(id: &str) {
    let root_dir = Path::new(".criterion").join(id);
    let base_dir = root_dir.join("base");
    let new_dir = root_dir.join("new");

    if base_dir.exists() {
        try_else_return!(fs::rmrf(&base_dir));
    }
    if new_dir.exists() {
        try_else_return!(fs::mv(&new_dir, &base_dir));
    };
}
