use stats::Tails;
use stats::bivariate::Data;
use stats::univariate::Sample;
use stats::univariate::{mixed, self};

use {Criterion, Estimate};
use estimate::Statistic::{Mean, Median};
use estimate::{Distributions, Estimates};
use self::ComparisonResult::*;
use {format, fs, plot, report};

// Common comparison procedure
pub fn common(
    id: &str,
    data: Data<f64, f64>,
    avg_times: &Sample<f64>,
    estimates_: &Estimates,
    criterion: &Criterion,
) {
    println!("{}: Comparing with previous sample", id);

    let (iters, times): (Vec<f64>, Vec<f64>) =
        fs::load(&format!(".criterion/{}/base/sample.json", id));

    let base_data = Data::new(&iters, &times);

    let base_estimates: Estimates =
        fs::load(&format!(".criterion/{}/base/estimates.json", id));

    let base_avg_times: Vec<f64> = iters.iter().zip(times.iter()).map(|(iters, elapsed)| {
        elapsed / iters
    }).collect();
    let base_avg_times = Sample::new(&base_avg_times);

    fs::mkdirp(&format!(".criterion/{}/both", id));
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

    fs::mkdirp(&format!(".criterion/{}/change", id));
    let different_mean = t_test(id, avg_times, base_avg_times, criterion);

    if different_mean {
        let regressed = estimates(id, avg_times, base_avg_times, criterion);
        if regressed.into_iter().all(|x| x) {
            println!("{} has regressed", id);
        }
    }
}

// Performs a two sample t-test
fn t_test(
    id: &str,
    avg_times: &Sample<f64>,
    base_avg_times: &Sample<f64>,
    criterion: &Criterion,
) -> bool {
    let nresamples = criterion.nresamples;
    let sl = criterion.significance_level;

    println!("> Performing a two-sample t-test");
    println!("  > H0: Both samples have the same mean");

    let t_statistic = avg_times.t(base_avg_times);
    let t_distribution = elapsed!(
        "Bootstrapping the T distribution",
        mixed::bootstrap(avg_times, base_avg_times, nresamples, |a, b| (a.t(b),))).0;
    let p_value = t_distribution.p_value(t_statistic, Tails::Two);
    let different_mean = p_value < sl;

    println!("  > p = {}", p_value);
    println!("  > {} reject the null hypothesis",
             if different_mean { "Strong evidence to" } else { "Can't" });

    if criterion.plotting.is_enabled() {
        elapsed!(
            "Plotting the T test",
            plot::t_test(
                t_statistic,
                &t_distribution,
                id));
    }

    different_mean
}

// Estimates the relative change in the statistics of the population
fn estimates(
    id: &str,
    avg_times: &Sample<f64>,
    base_avg_times: &Sample<f64>,
    criterion: &Criterion,
) -> Vec<bool> {
    fn stats(a: &Sample<f64>, b: &Sample<f64>) -> (f64, f64) {
        (a.mean() / b.mean() - 1., a.percentiles().median() / b.percentiles().median() - 1.)
    }

    let cl = criterion.confidence_level;
    let nresamples = criterion.nresamples;
    let threshold = criterion.noise_threshold;

    println!("> Estimating relative change of statistics");
    let distributions = {
        let (a, b) = elapsed!(
            "Bootstrapping the relative statistics",
            univariate::bootstrap(avg_times, base_avg_times, nresamples, stats)
        );

        vec![a, b]
    };

    let points = {
        let (a, b) = stats(avg_times, base_avg_times);
        [a, b]
    };
    let distributions: Distributions =
        [Mean, Median].iter().cloned().zip(distributions.into_iter()).collect();
    let estimates = Estimate::new(&distributions, &points, cl);

    report::rel(&estimates);

    fs::save(&estimates, &format!(".criterion/{}/change/estimates.json", id));

    if criterion.plotting.is_enabled() {
        elapsed!(
            "Plotting the distribution of the relative statistics",
            plot::rel_distributions(
                &distributions,
                &estimates,
                id,
                threshold));
    }

    let mut regressed = vec!();
    for (&statistic, estimate) in &estimates {
        let result = compare_to_threshold(estimate, threshold);

        let p = estimate.point_estimate;
        match result {
            Improved => {
                println!("  > {} has improved by {:.2}%", statistic, -100.0 * p);
                regressed.push(false);
            },
            Regressed => {
                println!("  > {} has regressed by {:.2}%", statistic, 100.0 * p);
                regressed.push(true);
            },
            NonSignificant => {
                regressed.push(false);
            },
        }
    }

    regressed
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
        Improved
    } else if lb > noise && ub > noise {
        Regressed
    } else {
        NonSignificant
    }
}
