use std::collections::BTreeMap;

use stats::Tails;
use stats::bivariate::Data;
use stats::univariate::Sample;
use stats::univariate::{mixed, self};

use estimate::Statistic;
use estimate::{Distributions, Estimates};
use benchmark::BenchmarkConfig;
use {Criterion, Estimate, format, fs, plot};
use error::Result;

// Common comparison procedure
pub(crate) fn common(
    id: &str,
    data: Data<f64, f64>,
    avg_times: &Sample<f64>,
    estimates_: &Estimates,
    config: &BenchmarkConfig,
    criterion: &Criterion,
) -> Result<(f64, f64, Estimates), > {
    let sample_dir = format!(".criterion/{}/base/sample.json", id);
    let (iters, times): (Vec<f64>, Vec<f64>) = fs::load(&sample_dir)?;

    let base_data = Data::new(&iters, &times);

    let base_estimates: Estimates =
        fs::load(&format!(".criterion/{}/base/estimates.json", id))?;

    let base_avg_times: Vec<f64> = iters.iter().zip(times.iter()).map(|(iters, elapsed)| {
        elapsed / iters
    }).collect();
    let base_avg_times = Sample::new(&base_avg_times);

    fs::mkdirp(&format!(".criterion/{}/both", id))?;
    if criterion.plotting.is_enabled() {
        elapsed!(
            "Plotting both linear regressions",
            plot::both::regression(
                base_data,
                &base_estimates,
                data,
                estimates_,
                id));
        elapsed!(
            "Plotting both estimated PDFs",
            plot::both::pdfs(
                base_avg_times,
                avg_times,
                id));
    }

    fs::mkdirp(&format!(".criterion/{}/change", id))?;
    let (t_statistic, p_statistic) = t_test(id, avg_times, base_avg_times, config, criterion);

    let estimates = estimates(id, avg_times, base_avg_times, config, criterion);
    Ok((t_statistic, p_statistic, estimates))
}

// Performs a two sample t-test
fn t_test(
    id: &str,
    avg_times: &Sample<f64>,
    base_avg_times: &Sample<f64>,
    config: &BenchmarkConfig,
    criterion: &Criterion,
) -> (f64, f64) {
    let nresamples = config.nresamples;

    let t_statistic = avg_times.t(base_avg_times);
    let t_distribution = elapsed!(
        "Bootstrapping the T distribution",
        mixed::bootstrap(avg_times, base_avg_times, nresamples, |a, b| (a.t(b),))).0;
    let p_value = t_distribution.p_value(t_statistic, &Tails::Two);

    if criterion.plotting.is_enabled() {
        elapsed!(
            "Plotting the T test",
            plot::t_test(
                t_statistic,
                &t_distribution,
                id));
    }

    (t_statistic, p_value)
}

// Estimates the relative change in the statistics of the population
fn estimates(
    id: &str,
    avg_times: &Sample<f64>,
    base_avg_times: &Sample<f64>,
    config: &BenchmarkConfig,
    criterion: &Criterion,
) -> Estimates {
    fn stats(a: &Sample<f64>, b: &Sample<f64>) -> (f64, f64) {
        (a.mean() / b.mean() - 1., a.percentiles().median() / b.percentiles().median() - 1.)
    }

    let cl = config.confidence_level;
    let nresamples = config.nresamples;
    let threshold = config.noise_threshold;

    let (dist_mean, dist_median) = elapsed!(
        "Bootstrapping the relative statistics",
        univariate::bootstrap(avg_times, base_avg_times, nresamples, stats)
    );

    let mut distributions = Distributions::new();
    distributions.insert(Statistic::Mean, dist_mean);
    distributions.insert(Statistic::Median, dist_median);

    let (mean, median) = stats(avg_times, base_avg_times);
    let mut point_estimates = BTreeMap::new();
    point_estimates.insert(Statistic::Mean, mean);
    point_estimates.insert(Statistic::Median, median);

    let estimates = Estimate::new(&distributions, &point_estimates, cl);

    {
        log_if_err!(fs::save(&estimates, &format!(".criterion/{}/change/estimates.json", id)));
    }

    if criterion.plotting.is_enabled() {
        elapsed!(
            "Plotting the distribution of the relative statistics",
            plot::rel_distributions(
                &distributions,
                &estimates,
                id,
                threshold));
    }

    estimates
}
