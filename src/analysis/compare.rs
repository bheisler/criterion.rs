use stats::ttest::{TDistribution, TwoTailed};
use stats::{Stats, mod};
use time;

use Criterion;
use estimate::{Distributions, Estimate, Estimates, Mean, Median};
use format;
use fs;
use plot;
use report;

// Common comparison procedure
pub fn common(
    id: &str,
    pairs: &[(f64, f64)],
    times: &[f64],
    estimates_: &Estimates,
    criterion: &Criterion,
) {
    println!("{}: Comparing with previous sample", id);

    let base_pairs: Vec<(u64, u64)> =
        fs::load(&Path::new(format!(".criterion/{}/base/sample.json", id)));

    let base_estimates: Estimates =
        fs::load(&Path::new(format!(".criterion/{}/base/estimates.json", id)));

    let base_times: Vec<f64> = base_pairs.iter().map(|&(iters, elapsed)| {
        elapsed as f64 / iters as f64
    }).collect();
    let base_times = base_times[];

    fs::mkdirp(&Path::new(format!(".criterion/{}/both", id)));
    elapsed!(
        "Plotting both linear regressions",
        plot::both::regression(
            base_pairs[],
            &base_estimates,
            pairs,
            estimates_,
            id));
    elapsed!(
        "Plotting both estimated PDFs",
        plot::both::pdfs(
            base_times,
            times,
            id));

    fs::mkdirp(&Path::new(format!(".criterion/{}/change", id)));
    let different_mean = t_test(id, times, base_times, criterion);
    let regressed = estimates(id, times, base_times, criterion);

    if different_mean && regressed.into_iter().all(|x| x) {
        panic!("{} has regressed", id);
    }
}

// Performs a two sample t-test
fn t_test(id: &str, times: &[f64], base_times: &[f64], criterion: &Criterion) -> bool {
    let nresamples = criterion.nresamples;
    let sl = criterion.significance_level;

    println!("> Performing a two-sample t-test");
    println!("  > H0: Both samples have the same mean");

    let t_statistic = times.t(base_times);
    let t_distribution = elapsed!(
        "Bootstrapping the T distribution",
        TDistribution::new(times, base_times, nresamples));
    let p_value = t_distribution.p_value(t_statistic, TwoTailed);
    let different_mean = p_value < sl;

    println!("  > p = {}", p_value);
    println!("  > {} reject the null hypothesis",
             if different_mean { "Strong evidence to" } else { "Can't" });

    elapsed!(
        "Plotting the T test",
        plot::t_test(
            t_statistic,
            t_distribution.as_slice(),
            id));

    different_mean
}

// Estimates the relative change in the statistics of the population
fn estimates(id: &str, times: &[f64], base_times: &[f64], criterion: &Criterion) -> Vec<bool> {
    fn stats(a: &[f64], b: &[f64]) -> (f64, f64) {
        (a.mean() / b.mean() - 1., a.percentiles().median() / b.percentiles().median() - 1.)
    }

    let cl = criterion.confidence_level;
    let nresamples = criterion.nresamples;
    let threshold = criterion.noise_threshold;

    println!("> Estimating relative change of statistics");
    let distributions = {
        let (a, b) = elapsed!(
            "Bootstrapping the relative statistics",
            stats::bootstrap(times, base_times, stats, nresamples)
        ).split2();

        vec![a, b]
    };

    let points = {
        let (a, b) = stats(times, base_times);
        [a, b]
    };
    let distributions: Distributions =
        [Mean, Median].iter().map(|&x| x).zip(distributions.into_iter()).collect();
    let estimates = Estimate::new(&distributions, points[], cl);

    report::rel(&estimates);

    fs::save(&estimates, &Path::new(format!(".criterion/{}/change/estimates.json", id)));

    elapsed!(
        "Plotting the distribution of the relative statistics",
        plot::rel_distributions(
            &distributions,
            &estimates,
            id,
            threshold));

    let mut regressed = vec!();
    for (&statistic, estimate) in estimates.iter() {
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
