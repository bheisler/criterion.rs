use std::cmp;
use std::fmt::Show;
use std::io::Command;
use std::mem;
use std::num;

use fs;
use outliers::Outliers;
use plot;
use statistics::{Estimate, Estimates, Mean, Median, MedianAbsDev, Sample, StdDev};
use stream::Stream;
use target::{Bencher, Function, Program, Target};
use time::prefix::{Mili, Nano};
use time::traits::{Milisecond, Nanosecond, Second};
use time::types::Ns;
use time;

/// The "criterion" for the benchmark, which is also the benchmark "manager"
#[experimental]
pub struct Criterion {
    confidence_level: f64,
    measurement_time: Ns<u64>,
    noise_threshold: f64,
    nresamples: uint,
    sample_size: uint,
    significance_level: f64,
    warm_up_time: Ns<u64>,
}

#[experimental]
impl Criterion {
    /// This is the default criterion:
    ///
    /// * Confidence level: 0.95
    /// * Measurement time: 10 ms
    /// * Noise threshold: 0.01 (1%)
    /// * Bootstrap with 100 000 resamples
    /// * Sample size: 100 measurements
    /// * Significance level: 0.05
    /// * Warm-up time: 1 s
    #[experimental]
    pub fn default() -> Criterion {
        Criterion {
            confidence_level: 0.95,
            measurement_time: 10.ms().to::<Nano>(),
            noise_threshold: 0.01,
            nresamples: 100_000,
            sample_size: 100,
            significance_level: 0.05,
            warm_up_time: 1.s().to::<Nano>(),
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
    pub fn measurement_time(&mut self, ms: u64) -> &mut Criterion {
        self.measurement_time = ms.ms().to::<Nano>();
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
    pub fn warm_up_time(&mut self, ms: u64) -> &mut Criterion {
        self.warm_up_time = ms.ms().to::<Nano>();
        self
    }

    /// Benchmark a function. See `Bench::iter()` for an example of how `fun` should look
    #[experimental]
    pub fn bench(&mut self, id: &str, fun: |&mut Bencher|:'static) -> &mut Criterion {
        // FIXME Figure out what's the problem with lifetimes
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
            let fun = |b| fun(b, input);

            // FIXME Figure out what's the problem with lifetimes
            bench(id.as_slice(), Function(Some(unsafe { mem::transmute(fun) })), self);
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

fn bench(id: &str, mut target: Target, criterion: &Criterion) {
    println!("Benchmarking {}", id);

    rename_new_dir_to_base(id);
    build_directory_skeleton(id);

    let root = Path::new(".criterion").join(id);
    let base_dir = root.join("base");
    let change_dir = root.join("change");
    let new_dir = root.join("new");

    let start = time::now();
    let sample = take_sample(&mut target, criterion).unwrap();
    info!("Sampling took {}", format_time((time::now() - start).unwrap() as f64))
    sample.save(&new_dir.join("sample.json"));

    plot::sample(&sample, new_dir.join("points.svg"), id);
    plot::pdf(&sample, new_dir.join("pdf.svg"), id);

    let outliers = Outliers::classify(sample.as_slice());
    outliers.report();
    outliers.save(&new_dir.join("outliers/classification.json"));
    plot::outliers(&outliers, new_dir.join("outliers/boxplot.svg"), id);

    println!("> Estimating the statistics of the sample");
    let nresamples = criterion.nresamples;
    let cl = criterion.confidence_level;
    println!("  > Bootstrapping the sample with {} resamples", nresamples);
    let start = time::now();
    let (estimates, distributions) =
        sample.bootstrap([Mean, Median, StdDev, MedianAbsDev], nresamples, cl);
    info!("Bootstraping took {}", format_time((time::now() - start).unwrap() as f64))
    estimates.save(&new_dir.join("bootstrap/estimates.json"));

    report_time(&estimates);
    plot::time_distributions(&distributions,
                             &estimates,
                             &new_dir.join("bootstrap/distribution"),
                             id);

    if !base_dir.exists() {
        return;
    }

    println!("{}: Comparing with previous sample", id);
    let base_sample = Sample::<Vec<f64>>::load(&base_dir.join("sample.json"));

    let both_dir = root.join("both");
    plot::both::pdfs(&base_sample, &sample, both_dir.join("pdfs.svg"), id);
    plot::both::points(&base_sample, &sample, both_dir.join("points.svg"), id);

    println!("> H0: Both samples belong to the same population");
    println!("  > Bootstrapping with {} resamples", nresamples);
    let t_statistic = sample.t_test(&base_sample);
    let start = time::now();
    let t_distribution = sample.bootstrap_t_test(&base_sample, nresamples, cl);
    info!("Bootstraping took {}", format_time((time::now() - start).unwrap() as f64))
    let t = t_statistic.abs();
    let hits = t_distribution.as_slice().iter().filter(|&&x| x > t || x < -t).count();
    let p_value = hits as f64 / nresamples as f64;
    let sl = criterion.significance_level;
    let different_population = p_value < sl;

    println!("  > p = {}", p_value);
    println!("  > {} reject the null hypothesis",
             if different_population { "Strong evidence to" } else { "Can't" })
    plot::t_test(t_statistic, &t_distribution, change_dir.join("bootstrap/t_test.svg"), id);

    let nresamples_sqrt = (nresamples as f64).sqrt().ceil() as uint;
    let nresamples = nresamples_sqrt * nresamples_sqrt;

    println!("> Estimating relative change of statistics");
    println!("  > Bootstrapping with {} resamples", nresamples);
    let start = time::now();
    let (estimates, distributions) =
        sample.bootstrap_compare(&base_sample, [Mean, Median], nresamples_sqrt, cl);
    info!("Bootstraping took {}", format_time((time::now() - start).unwrap() as f64))
    estimates.save(&change_dir.join("bootstrap/estimates.json"));

    report_change(&estimates);
    plot::ratio_distributions(&distributions,
                              &estimates,
                              &change_dir.join("bootstrap/distribution"),
                              id);

    let threshold = criterion.noise_threshold;
    let mut regressed = vec!();
    for &statistic in [Mean, Median].iter() {
        let estimate = estimates.get(statistic);
        let result = compare_to_threshold(estimate, threshold);

        let p = estimate.point_estimate();
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

fn extrapolate_iters(iters: u64, took: Ns<u64>, want: Ns<u64>) -> (Ns<f64>, u64) {
    let e_iters = cmp::max(want * iters / took, 1);
    let e_time = (took * e_iters).cast::<f64>() / iters as f64;

    (e_time, e_iters)
}

fn take_sample(t: &mut Target, criterion: &Criterion) -> Ns<Sample<Vec<f64>>> {
    let wu_time = criterion.warm_up_time;
    println!("> Warming up for {}", wu_time.to::<Mili>())
    let (took, iters) = t.warm_up(wu_time);

    let m_time = criterion.measurement_time;
    let (m_time, m_iters) = extrapolate_iters(iters, took, m_time);

    let sample_size = criterion.sample_size;
    println!("> Collecting {} measurements, {} iters each in estimated {}",
             sample_size,
             m_iters,
             format_time((m_time * sample_size as f64).unwrap()));

    let sample = t.bench(sample_size, m_iters).unwrap();

    sample.ns()
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
    for &statistic in [Mean, Median, StdDev, MedianAbsDev].iter() {
        let estimate = estimates.get(statistic);
        let p = format_time(estimate.point_estimate());
        let ci = estimate.confidence_interval();
        let lb = format_time(ci.lower_bound());
        let ub = format_time(ci.upper_bound());
        let se = format_time(estimate.standard_error());
        let cl = ci.confidence_level();

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
    for &statistic in [Mean, Median].iter() {
        let estimate = estimates.get(statistic);
        let p = format_change(estimate.point_estimate(), true);
        let ci = estimate.confidence_interval();
        let lb = format_change(ci.lower_bound(), true);
        let ub = format_change(ci.upper_bound(), true);
        let se = format_change(estimate.standard_error(), false);
        let cl = ci.confidence_level();

        println!("  > {:<7} {} ± {} [{} {}] {}% CI", statistic, p, se, lb, ub, cl * 100.0);
    }
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
    let ci = estimate.confidence_interval();
    let lb = ci.lower_bound();
    let ub = ci.upper_bound();

    if lb < -noise && ub < -noise {
        Improved
    } else if lb > noise && ub > noise {
        Regressed
    } else {
        NonSignificant
    }
}
