use std::collections::BTreeMap;

use stats::Distribution;
use stats::univariate::Sample;
use stats::univariate::{self, mixed};

use benchmark::BenchmarkConfig;
use error::Result;
use estimate::Statistic;
use estimate::{Distributions, Estimates};
use report::BenchmarkId;
use {format, fs, Criterion, Estimate};

// Common comparison procedure
#[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
pub(crate) fn common(
    id: &BenchmarkId,
    avg_times: &Sample<f64>,
    config: &BenchmarkConfig,
    criterion: &Criterion,
) -> Result<(
    f64,
    Distribution<f64>,
    Estimates,
    Distributions,
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    Estimates,
)> {
    let sample_dir = format!(
        "{}/{}/{}/sample.json",
        criterion.output_directory,
        id.as_directory_name(),
        criterion.baseline_directory
    );
    let (iters, times): (Vec<f64>, Vec<f64>) = fs::load(&sample_dir)?;

    let estimates_file = &format!(
        "{}/{}/{}/estimates.json",
        criterion.output_directory,
        id.as_directory_name(),
        criterion.baseline_directory
    );
    let base_estimates: Estimates = fs::load(&estimates_file)
        .map_err(|e| e.context(format!("Failed to load {}!", &estimates_file)))?;

    let base_avg_times: Vec<f64> = iters
        .iter()
        .zip(times.iter())
        .map(|(iters, elapsed)| elapsed / iters)
        .collect();
    let base_avg_time_sample = Sample::new(&base_avg_times);

    fs::mkdirp(&format!(
        "{}/{}/change",
        criterion.output_directory,
        id.as_directory_name()
    ))?;
    let (t_statistic, t_distribution) = t_test(avg_times, base_avg_time_sample, config);

    let (estimates, relative_distributions) =
        estimates(id, avg_times, base_avg_time_sample, config, criterion);
    Ok((
        t_statistic,
        t_distribution,
        estimates,
        relative_distributions,
        iters,
        times,
        base_avg_times.clone(),
        base_estimates,
    ))
}

// Performs a two sample t-test
fn t_test(
    avg_times: &Sample<f64>,
    base_avg_times: &Sample<f64>,
    config: &BenchmarkConfig,
) -> (f64, Distribution<f64>) {
    let nresamples = config.nresamples;

    let t_statistic = avg_times.t(base_avg_times);
    let t_distribution = elapsed!(
        "Bootstrapping the T distribution",
        mixed::bootstrap(avg_times, base_avg_times, nresamples, |a, b| (a.t(b),))
    ).0;

    // HACK: Filter out non-finite numbers, which can happen sometimes when sample size is very small.
    // Downstream code doesn't like non-finite values here.
    let t_distribution = Distribution::from(
        t_distribution
            .as_slice()
            .iter()
            .filter(|a| a.is_finite())
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );

    (t_statistic, t_distribution)
}

// Estimates the relative change in the statistics of the population
fn estimates(
    id: &BenchmarkId,
    avg_times: &Sample<f64>,
    base_avg_times: &Sample<f64>,
    config: &BenchmarkConfig,
    criterion: &Criterion,
) -> (Estimates, Distributions) {
    fn stats(a: &Sample<f64>, b: &Sample<f64>) -> (f64, f64) {
        (
            a.mean() / b.mean() - 1.,
            a.percentiles().median() / b.percentiles().median() - 1.,
        )
    }

    let cl = config.confidence_level;
    let nresamples = config.nresamples;

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
        log_if_err!(fs::save(
            &estimates,
            &format!(
                "{}/{}/change/estimates.json",
                criterion.output_directory,
                id.as_directory_name()
            )
        ));
    }
    (estimates, distributions)
}
