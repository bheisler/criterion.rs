use stats::outliers::Outliers;
use stats::ttest::{TDistribution, TwoTailed};
use stats::{Sample, t};
use std::fmt::Show;
use std::io::Command;
use std::time::Duration;
use std::{mem, num};
use time;

use estimate::{Distributions, Estimate, Estimates, Mean, Median, MedianAbsDev, Statistic, StdDev};
use fs;
use plot;
use stream::Stream;
use target::{Bencher, Function, Program, Target};

/// The "criterion" for the benchmark, which is also the benchmark "manager"
// TODO (rust-lang/rust#15934) The `*_ns` fields should use the `Duration` type
#[experimental]
pub struct Criterion {
    confidence_level: f64,
    measurement_time: Duration,
    noise_threshold: f64,
    nresamples: uint,
    sample_size: uint,
    significance_level: f64,
    warm_up_time: Duration,
}

#[experimental]
impl Criterion {
    /// This is the default criterion:
    ///
    /// * Confidence level: 0.95
    /// * Measurement time: 1 s
    /// * Noise threshold: 0.01 (1%)
    /// * Bootstrap with 100 000 resamples
    /// * Sample size: 100 measurements
    /// * Significance level: 0.05
    /// * Warm-up time: 1 s
    #[experimental]
    pub fn default() -> Criterion {
        Criterion {
            confidence_level: 0.95,
            measurement_time: Duration::seconds(1),
            noise_threshold: 0.01,
            nresamples: 100_000,
            sample_size: 100,
            significance_level: 0.05,
            warm_up_time: Duration::seconds(1),
        }
    }

    /// Changes the confidence level
    ///
    /// The confidence level is used to calculate the confidence intervals of the estimates
    #[experimental]
    pub fn confidence_level(&mut self, cl: f64) -> &mut Criterion {
        assert!(cl > 0.0 && cl < 1.0);

        self.confidence_level = cl;
        self
    }

    /// Change the measurement time
    ///
    /// The program/function under test is iterated for `measurement_time` ms. And the average run
    /// time is reported as a measurement
    #[experimental]
    pub fn measurement_time(&mut self, dur: Duration) -> &mut Criterion {
        assert!(dur.num_nanoseconds().expect("duration overflow") > 0);

        self.measurement_time = dur;
        self
    }

    /// Changes the noise threshold
    ///
    /// When comparing benchmark results, only relative changes of the execution time above this
    /// threshold are considered significant
    #[experimental]
    pub fn noise_threshold(&mut self, nt: f64) -> &mut Criterion {
        assert!(nt >= 0.0);

        self.noise_threshold = nt;
        self
    }

    /// Changes the number of resamples
    ///
    /// Number of resamples to use for bootstraping via case resampling
    #[experimental]
    pub fn nresamples(&mut self, n: uint) -> &mut Criterion {
        self.nresamples = n;
        self
    }

    /// Changes the size of a sample
    ///
    /// A sample consists of severals measurements
    #[experimental]
    pub fn sample_size(&mut self, n: uint) -> &mut Criterion {
        self.sample_size = n;
        self
    }

    /// Changes the significance level
    ///
    /// Significance level to use for hypothesis testing
    #[experimental]
    pub fn significance_level(&mut self, sl: f64) -> &mut Criterion {
        assert!(sl > 0.0 && sl < 1.0);

        self.significance_level = sl;
        self
    }

    /// Changes the warm up time
    ///
    /// The program/function under test is executed during `warm_up_time` ms before the real
    /// measurement starts
    #[experimental]
    pub fn warm_up_time(&mut self, dur: Duration) -> &mut Criterion {
        assert!(dur.num_nanoseconds().expect("duration overflow") > 0);

        self.warm_up_time = dur;
        self
    }

    /// Benchmark a function. See `Bench::iter()` for an example of how `fun` should look
    #[experimental]
    pub fn bench(&mut self, id: &str, fun: |&mut Bencher|:'static) -> &mut Criterion {
        // FIXME (rust-lang/rust#16453) Remove unsafe transmute
        bench(id, Function(Some(unsafe { mem::transmute(fun) })), self);

        println!("");

        self
    }

    /// Benchmark a family of functions
    ///
    /// `fun` will be benchmarked under each input
    ///
    /// For example, if you want to benchmark `Vec::from_elem` with different size, use these
    /// arguments:
    ///
    ///     let fun = |b, n| Vec::from_elem(n, 0u);
    ///     let inputs = [100, 10_000, 1_000_000];
    ///
    /// This is equivalent to calling `bench` on each of the following functions:
    ///
    ///     let fun1 = |b| Vec::from_elem(100, 0u);
    ///     let fun2 = |b| Vec::from_elem(10_000, 0u);
    ///     let fun3 = |b| Vec::from_elem(1_000_000, 0u);
    #[experimental]
    pub fn bench_family<I: Show>(
                        &mut self,
                        id: &str,
                        fun: |&mut Bencher, &I|:'static,
                        inputs: &[I])
                        -> &mut Criterion {
        for input in inputs.iter() {
            let id = format!("{}/{}", id, input);
            // FIXME (rust-lang/rust#16473) drop type annotation on `b`
            let fun = |b: &mut Bencher| fun(b, input);

            bench(id.as_slice(), Function(Some(fun)), self);
        }

        print!("Summarizing results of {}... ", id);
        plot::summarize(&Path::new(".criterion").join(id), id);
        println!("DONE\n");

        self
    }

    /// Benchmark an external program
    ///
    /// The program must conform to the following specification:
    ///
    ///     extern crate time;
    ///
    ///     fn main() {
    ///         // Optional: Get the program arguments
    ///         let args = std::os::args();
    ///
    ///         for line in std::io::stdio::stdin().lines() {
    ///             // Get number of iterations to do
    ///             let iters: u64 = from_str(line.unwrap().as_slice().trim()).unwrap();
    ///
    ///             // Setup
    ///
    ///             // (For best results, use a monotonic timer)
    ///             let start = time::precise_time_ns();
    ///             for _ in range(0, iters) {
    ///                 // Routine to benchmark goes here
    ///             }
    ///             let end = time::precise_time_ns();
    ///
    ///             // Teardown
    ///
    ///             // Report back the time (in nanoseconds) required to execute the routine
    ///             // `iters` times
    ///             println!("{}", end - start);
    ///         }
    ///     }
    ///
    /// For example, to benchmark a python script use the following command
    ///
    ///     let cmd = Command::new("python3").args(["-O", "clock.py"]);
    #[experimental]
    pub fn bench_prog(&mut self,
                      id: &str,
                      prog: &Command)
                      -> &mut Criterion {
        bench(id, Program(Stream::spawn(prog)), self);

        println!("");

        self
    }

    /// Benchmark an external program under various inputs
    ///
    /// For example, to benchmark a python script under various inputs, use this combination:
    ///
    ///     let cmd = Command::new("python3").args(["-O", "fib.py"]);
    ///     let inputs = [5u, 10, 15];
    ///
    /// This is equivalent to calling `bench_prog` on each of the following commands:
    ///
    ///     let cmd1 = Command::new("python3").args(["-O", "fib.py", "5"]);
    ///     let cmd2 = Command::new("python3").args(["-O", "fib.py", "10"]);
    ///     let cmd2 = Command::new("python3").args(["-O", "fib.py", "15"]);
    #[experimental]
    pub fn bench_prog_family<I: Show>(
                             &mut self,
                             id: &str,
                             prog: &Command,
                             inputs: &[I])
                             -> &mut Criterion {
        for input in inputs.iter() {
            let id = format!("{}/{}", id, input);
            self.bench_prog(id.as_slice(), prog.clone().arg(format!("{}", input)));
        }

        print!("Summarizing results of {}... ", id);
        plot::summarize(&Path::new(".criterion").join(id), id);
        println!("DONE\n");

        self
    }

    /// Summarize the results stored under the `.criterion/${id}` folder
    ///
    /// Note that `bench_family` and `bench_prog_family` internally call the `summarize` method
    #[experimental]
    pub fn summarize(&mut self, id: &str) -> &mut Criterion {
        print!("Summarizing results of {}... ", id);
        plot::summarize(&Path::new(".criterion").join(id), id);
        println!("DONE\n");

        self
    }
}

// FIXME Sorry! Everything below this point is a mess :/

macro_rules! elapsed {
    ($msg:expr, $block:expr) => ({
        let start = time::precise_time_ns();
        let out = $block;
        let stop = time::precise_time_ns();

        info!("{} took {}", $msg, format_time((stop - start) as f64));

        out
    })
}

fn bench(id: &str, mut target: Target, criterion: &Criterion) {
    static ABS_STATS: &'static [Statistic] = &[Mean, Median, MedianAbsDev, StdDev];
    static REL_STATS: &'static [Statistic] = &[Mean, Median];

    let abs_stats_fns: Vec<fn(&[f64]) -> f64> =
        ABS_STATS.iter().map(|st| st.abs_fn()).collect();
    let rel_stats_fns: Vec<fn(&[f64], &[f64]) -> f64> =
        REL_STATS.iter().map(|st| st.rel_fn()).collect();

    println!("Benchmarking {}", id);

    rename_new_dir_to_base(id);
    build_directory_skeleton(id);

    let root = Path::new(".criterion").join(id);
    let base_dir = root.join("base");
    let change_dir = root.join("change");
    let new_dir = root.join("new");

    let sample_pairs = elapsed!("Sampling", take_sample(&mut target, criterion));
    let sample: Vec<f64> = sample_pairs.iter().map(|&(iters, elapsed)| {
        elapsed as f64 / iters as f64
    }).collect();
    let points: Vec<f64> = abs_stats_fns.iter().map(|&f| f(sample.as_slice())).collect();
    elapsed!("Storing sample", fs::save(&sample_pairs, &new_dir.join("sample.json")));
    let sample = Sample::new(sample.as_slice());

    elapsed!(
        "Plotting sample points",
        plot::sample(sample.as_slice(), new_dir.join("points.svg"), id));
    elapsed!(
        "Plotting the estimated sample PDF",
        plot::pdf(sample.as_slice(), new_dir.join("pdf.svg"), id));

    let (filtered, outliers) = Outliers::classify(sample.as_slice());
    report_outliers(&outliers, filtered.as_slice());
    elapsed!(
        "Storing the filtered sample",
        fs::save(&filtered, &new_dir.join("outliers/filtered.json")));
    elapsed!(
        "Storing the classification of outliers",
         fs::save(&outliers, &new_dir.join("outliers/classification.json")));
    elapsed!(
        "Plotting the outliers",
        plot::outliers(&outliers, filtered.as_slice(), new_dir.join("outliers/boxplot.svg"), id));

    println!("> Estimating the statistics of the sample");
    let nresamples = criterion.nresamples;
    let cl = criterion.confidence_level;
    println!("  > Bootstrapping the sample with {} resamples", nresamples);
    let distributions = elapsed!(
        "Bootstrapping the absolute statistics",
        sample.bootstrap_many(abs_stats_fns.as_slice(), nresamples));
    let distributions: Distributions =
        ABS_STATS.iter().map(|&x| x).zip(distributions.move_iter()).collect();
    let estimates = Estimate::new(&distributions, points.as_slice(), cl);
    fs::save(&estimates, &new_dir.join("bootstrap/estimates.json"));

    report_time(&estimates);
    elapsed!(
        "Plotting the distribution of the absolute statistics",
        plot::time_distributions(
            &distributions,
            &estimates,
            &new_dir.join("bootstrap/distribution"), id));

    if !base_dir.exists() {
        return;
    }

    println!("{}: Comparing with previous sample", id);
    let base_sample_pairs =
        elapsed!(
            "Loading previous sample",
            fs::load::<Vec<(u64, u64)>>(&base_dir.join("sample.json")));
    let base_sample: Vec<f64> = base_sample_pairs.iter().map(|&(iters, elapsed)| {
        elapsed as f64 / iters as f64
    }).collect();
    let base_sample = Sample::new(base_sample.as_slice());

    let both_dir = root.join("both");
    elapsed!(
        "Plotting both sample points",
        plot::both::pdfs(
            base_sample.as_slice(),
            sample.as_slice(),
            both_dir.join("pdfs.svg"),
            id));
    elapsed!(
        "Plotting both estimated PDFs",
        plot::both::points(
            base_sample.as_slice(),
            sample.as_slice(),
            both_dir.join("points.svg"),
            id));

    println!("> H0: Both samples belong to the same population");
    println!("  > Bootstrapping with {} resamples", nresamples);
    let t_statistic = t(sample.as_slice(), base_sample.as_slice());
    let t_distribution = elapsed!(
        "Bootstrapping the T distribution",
        TDistribution::new(sample.as_slice(), base_sample.as_slice(), nresamples));
    let p_value = t_distribution.p_value(t_statistic, TwoTailed);
    let sl = criterion.significance_level;
    let different_population = p_value < sl;

    println!("  > p = {}", p_value);
    println!("  > {} reject the null hypothesis",
             if different_population { "Strong evidence to" } else { "Can't" });
    elapsed!(
        "Plotting the T test",
        plot::t_test(
            t_statistic,
            t_distribution.as_slice(),
            change_dir.join("bootstrap/t_test.svg"),
            id));

    println!("> Estimating relative change of statistics");
    println!("  > Bootstrapping with {} resamples", nresamples);
    let distributions = elapsed!(
        "Bootstrapping the relative statistics",
        sample.bootstrap2_many(&base_sample, rel_stats_fns.as_slice(), nresamples)
    );
    let points: Vec<f64> = rel_stats_fns.iter().map(|&f| {
        f(sample.as_slice(), base_sample.as_slice())
    }).collect();
    let distributions: Distributions =
        REL_STATS.iter().map(|&x| x).zip(distributions.move_iter()).collect();
    let estimates = Estimate::new(&distributions, points.as_slice(), cl);
    elapsed!(
        "Storing the relative estimates",
        fs::save(&estimates, &change_dir.join("bootstrap/estimates.json")));

    report_change(&estimates);
    elapsed!(
        "Plotting the distribution of the relative statistics",
        plot::ratio_distributions(
            &distributions,
            &estimates,
            &change_dir.join("bootstrap/distribution"),
            id));

    let threshold = criterion.noise_threshold;
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
    if different_population && regressed.iter().all(|&x| x) {
        fail!("{} has regressed", id);
    }
}

fn take_sample(t: &mut Target, criterion: &Criterion) -> Vec<(u64, u64)> {
    let wu_ns = criterion.warm_up_time.num_nanoseconds().expect("Duration overflow") as u64;
    let m_ns = criterion.measurement_time.num_nanoseconds().expect("Duration overflow") as u64;
    let n = criterion.sample_size as u64;

    println!("> Warming up for {}", format_time(wu_ns as f64))
    let (wu_ns, wu_iters) = t.warm_up(wu_ns);

    let d = {
        let num = 2 * m_ns * wu_iters;
        let den = n * (n + 1) * wu_ns;

        num / den + 1
    };

    let m_ns = {
        let num = n * (n + 1) * d * wu_ns;
        let den = 2 * wu_iters;

        num / den
    } as f64;

    println!("> Collecting {} measurements in estimated {}", n, format_time(m_ns));

    t.bench(n as uint, d)
}

fn rename_new_dir_to_base(id: &str) {
    let root_dir = Path::new(".criterion").join(id);
    let base_dir = root_dir.join("base");
    let new_dir = root_dir.join("new");

    if base_dir.exists() { fs::rmrf(&base_dir) }
    if new_dir.exists() { fs::mv(&new_dir, &base_dir) };
}

fn build_directory_skeleton(id: &str) {
    let root = Path::new(".criterion").join(id);
    fs::mkdirp(&root.join("both"));
    fs::mkdirp(&root.join("change/bootstrap/distribution"));
    fs::mkdirp(&root.join("new/bootstrap/distribution"));
    fs::mkdirp(&root.join("new/outliers"));
}

fn format_short(n: f64) -> String {
    if n < 10.0 { format!("{:.4}", n) }
    else if n < 100.0 { format!("{:.3}", n) }
    else if n < 1000.0 { format!("{:.2}", n) }
    else { format!("{}", n) }
}

fn format_signed_short(n: f64) -> String {
    let n_abs = n.abs();

    if n_abs < 10.0 { format!("{:+.4}", n) }
    else if n_abs < 100.0 { format!("{:+.3}", n) }
    else if n_abs < 1000.0 { format!("{:+.2}", n) }
    else { format!("{:+}", n) }
}

fn report_time(estimates: &Estimates) {
    for (&statistic, estimate) in estimates.iter() {
        let p = format_time(estimate.point_estimate);
        let ci = estimate.confidence_interval;
        let lb = format_time(ci.lower_bound);
        let ub = format_time(ci.upper_bound);
        let se = format_time(estimate.standard_error);
        let cl = ci.confidence_level;

        println!("  > {:<7} {} ± {} [{} {}] {}% CI", statistic, p, se, lb, ub, cl * 100.0);
    }
}

fn format_time(ns: f64) -> String {
    if ns < 1.0 {
        format!("{:>6} ps", format_short(ns * 1e3))
    } else if ns < num::pow(10.0, 3) {
        format!("{:>6} ns", format_short(ns))
    } else if ns < num::pow(10.0, 6) {
        format!("{:>6} us", format_short(ns / 1e3))
    } else if ns < num::pow(10.0, 9) {
        format!("{:>6} ms", format_short(ns / 1e6))
    } else {
        format!("{:>6} s", format_short(ns / 1e9))
    }
}

fn report_change(estimates: &Estimates) {
    for (&statistic, estimate) in estimates.iter() {
        let p = format_change(estimate.point_estimate, true);
        let ci = estimate.confidence_interval;
        let lb = format_change(ci.lower_bound, true);
        let ub = format_change(ci.upper_bound, true);
        let se = format_change(estimate.standard_error, false);
        let cl = ci.confidence_level;

        println!("  > {:<7} {} ± {} [{} {}] {}% CI", statistic, p, se, lb, ub, cl * 100.0);
    }
}

fn report_outliers(outliers: &Outliers<f64>, normal: &[f64]) {
    let total = outliers.len();

    if total == 0 {
        return
    }

    let sample_size = total + normal.len();

    let percent = |n: uint| { 100. * n as f64 / sample_size as f64 };

    println!("> Found {} outliers among {} measurements ({:.2}%)",
             total,
             sample_size,
             percent(total));

    let print = |n: uint, class| {
        if n != 0 {
            println!("  > {} ({:.2}%) {}", n, percent(n), class);
        }
    };

    print(outliers.low_severe.len(), "low severe");
    print(outliers.low_mild.len(), "low mild");
    print(outliers.high_mild.len(), "high mild");
    print(outliers.high_severe.len(), "high severe");
}

fn format_change(pct: f64, signed: bool) -> String {
    if signed {
        format!("{:>+6}%", format_signed_short(pct * 1e2))
    } else {
        format!("{:>6}%", format_short(pct * 1e2))
    }
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
