//! A statistics-driven micro-benchmarking library written in Rust.
//!
//! This crate is a microbenchmarking library which aims to provide strong
//! statistical confidence in detecting and estimating the size of performance
//! improvements and regressions, whle also being easy to use.
//! 
//! See
//! [the user guide](https://japaric.github.io/criterion.rs/book/index.html)
//! for examples as well as details on the measurement and analysis process,
//! and the output.
//! 
//! ## Features:
//! * Benchmark Rust code as well as external programs
//! * Collects detailed statistics, providing strong confidence that changes
//!   to performance are real, not measurement noise
//! * Produces detailed charts, providing thorough understanding of your code's
//!   performance behavior.

#![deny(missing_docs)]

#![cfg_attr(feature = "real_blackbox", feature(test))]

#[macro_use]
extern crate log;
extern crate failure;
extern crate itertools;
extern crate serde;
extern crate serde_json;
extern crate criterion_plot as simplot;
extern crate criterion_stats as stats;
extern crate simplelog;
#[cfg(feature = "real_blackbox")] extern crate test;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;

// Needs to be declared before other modules
// in order to be usable there.
#[macro_use]
mod macros_private;

mod analysis;
mod error;
mod estimate;
mod format;
mod fs;
mod kde;
mod plot;
mod program;
mod report;
mod routine;
mod macros;

use std::default::Default;
use std::iter::IntoIterator;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{fmt, mem};

use std::fs::File;
use std::io::Read;
use std::path::Path;

use estimate::{Distributions, Estimates};

fn debug_enabled() -> bool {
    std::env::vars().any(|(key, _)| key == "CRITERION_DEBUG")
}

/// Initialize the logging for a Criterion benchmark. This should be called
/// first before executing Criterion benchmarks, unless the user provides their
/// own logging infrastructure.
pub fn init_logging() {
    use simplelog::*;
    let filter = if debug_enabled() { LogLevelFilter::max() } else { LogLevelFilter::Warn };

    SimpleLogger::init(
        filter,
        Config::default(),
    ).unwrap();
}

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
/// 
/// This variant is backed by the (unstable) test::black_box function.
#[cfg(feature = "real_blackbox")]
pub fn black_box<T>(dummy: T) -> T {
    test::black_box(dummy)
}

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
/// 
/// This variant is stable-compatible, but it may cause some performance overhead
/// or fail to prevent code from being eliminated.
#[cfg(not(feature = "real_blackbox"))]
pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

/// Representing a function to benchmark together with a name of that function.
/// Used together with `bench_functions` to represent one out of multiple functions
/// under benchmark.
pub struct Fun<I: fmt::Display> {
    n: String,
    f: Box<FnMut(&mut Bencher, &I)>,
}

impl<I> Fun<I> where I: fmt::Display {
    /// Create a new `Fun` given a name and a closure
    pub fn new<F>(name: &str, f: F) -> Fun<I>
        where F: FnMut(&mut Bencher, &I) + 'static
    {
        Fun {
            n: name.to_owned(),
            f: Box::new(f),
        }
    }
}

/// Helper struct to time routines
///
/// This struct provides different timing loops as methods. Each timing loop provides a different
/// way to time a routine and each has advantages and disadvantages.
/// 
/// * If your routine returns a value with an expensive `drop` method, use 
///   `iter_with_large_drop`.
/// * If your routine requires some per-iteration setup that shouldn't be timed,
///   use `iter_with_setup` or (if the setup is expensive) use `iter_with_large_setup`
///   to construct a pool of input data ahead of time
/// * Otherwise, use `iter`.
#[derive(Clone, Copy)]
pub struct Bencher {
    iters: u64,
    elapsed: Duration,
}

impl Bencher {
    /// Times a `routine` by executing it many times and timing the total elapsed time.
    ///
    /// Prefer this timing loop when `routine` returns a value that doesn't have a destructor.
    ///
    /// # Timing loop
    ///
    /// ```rust,no_run
    /// # use std::time::Instant;
    /// # fn routine() {}
    /// # let iters = 4_000_000;
    /// let start = Instant::now();
    /// for _ in 0..iters {
    ///     routine();
    /// }
    /// let elapsed = start.elapsed();
    /// ```
    ///
    /// # Timing model
    ///
    /// Note that the `Bencher` also times the time required to destroy the output of `routine()`.
    /// Therefore prefer this timing loop when the runtime of `mem::drop(O)` is negligible compared
    /// to the runtime of the `routine`.
    ///
    /// ```text
    /// elapsed = Instant::now + iters * (routine + mem::drop(O) + Range::next)
    /// ```
    ///
    pub fn iter<O, R>(&mut self, mut routine: R) where
        R: FnMut() -> O,
    {
        let start = Instant::now();
        for _ in 0..self.iters {
            black_box(routine());
        }
        self.elapsed = start.elapsed();
    }

    /// Times a `routine` that requires some `setup` on each iteration.
    ///
    /// For example, use this loop to benchmark sorting algorithms because they require unsorted
    /// data on each iteration.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// extern crate criterion;
    ///
    /// use criterion::Bencher;
    ///
    /// fn create_scrambled_data() -> Vec<u64> {
    ///     # vec![]
    ///     // ...
    /// }
    ///
    /// // The sorting algorithm to test
    /// fn sort(data: &mut [u64]) {
    ///     // ...
    /// }
    ///
    /// fn benchmark(b: &mut Bencher) {
    ///     let data = create_scrambled_data();
    ///
    ///     b.iter_with_setup(move || data.to_vec(), |mut data| sort(&mut data))
    /// }
    ///
    /// # fn main() {}
    /// ```
    ///
    /// # Timing loop
    ///
    /// ```rust,no_run
    /// # use std::time::{Instant, Duration};
    /// # use std::mem;
    /// # fn setup() {}
    /// # fn routine(input: ()) {}
    /// # let iters = 4_000_000;
    /// let mut elapsed = Duration::new(0, 0);
    /// for _ in 0..iters {
    ///     let input = setup();
    ///
    ///     let start = Instant::now();
    ///     let output = routine(input);
    ///     let elapsed_in_iter = start.elapsed();
    ///
    ///     mem::drop(output);
    ///
    ///     elapsed = elapsed + elapsed_in_iter;
    /// }
    /// ```
    ///
    /// # Timing model
    ///
    /// ``` text
    /// elapsed = iters * (Instant::now + routine)
    /// ```
    pub fn iter_with_setup<I, O, S, R>(&mut self, mut setup: S, mut routine: R)
        where S: FnMut() -> I,
              R: FnMut(I) -> O
    {
        self.elapsed = Duration::from_secs(0);
        for _ in 0..self.iters {
            let input = setup();

            let start = Instant::now();
            let output = black_box(routine(black_box(input)));
            let elapsed = start.elapsed();

            mem::drop(output);

            self.elapsed += elapsed;
        }
    }

    /// Times a `routine` by collecting its output on each iteration. This avoids timing the
    /// destructor of the value returned by `routine`.
    ///
    /// WARNING: This requires `iters * mem::size_of::<O>()` of memory, and `iters` is not under the
    /// control of the caller.
    ///
    /// # Timing loop
    ///
    /// ```rust,no_run
    /// # use std::mem;
    /// # use std::time::Instant;
    /// # let iters = 4_000_000;
    /// # fn routine() {}
    /// let mut outputs = Vec::with_capacity(iters);
    ///
    /// let start = Instant::now();
    /// for _ in 0..iters {
    ///     outputs.push(routine());
    /// }
    /// let elapsed = start.elapsed();
    ///
    /// mem::drop(outputs);
    /// ```
    ///
    /// # Timing model
    ///
    /// ``` text
    /// elapsed = Instant::now + iters * (routine + Vec::push + Range::next)
    /// ```
    ///
    pub fn iter_with_large_drop<O, R>(&mut self, mut routine: R)
        where R: FnMut() -> O
    {
        let mut outputs = Vec::with_capacity(self.iters as usize);

        let start = Instant::now();
        for _ in 0..self.iters {
            outputs.push(black_box(routine()));
        }
        self.elapsed = start.elapsed();

        mem::drop(outputs);
    }

    /// Times a `routine` that needs to consume its input by first creating a pool of inputs.
    ///
    /// This function is handy for benchmarking destructors.
    ///
    /// WARNING This requires `iters * mem::size_of::<I>()` of memory, and `iters` is not under the
    /// control of the caller.
    ///
    /// # Timing loop
    ///
    /// ```rust,no_run
    /// # use std::time::Instant;
    /// # fn setup() {}
    /// # fn routine(input: ()) {}
    /// # let iters = 4_000_000;
    /// let inputs: Vec<()> = (0..iters).map(|_| setup()).collect();
    /// let start = Instant::now();
    ///
    /// for input in inputs {
    ///     routine(input);
    /// }
    ///
    /// let elapsed = start.elapsed();
    /// ```
    ///
    /// # Timing model
    ///
    /// ``` text
    /// elapsed = Instant::now + iters * (routine + vec::IntoIter::next)
    /// ```
    pub fn iter_with_large_setup<I, S, R>(&mut self, mut setup: S, mut routine: R)
        where S: FnMut() -> I,
              R: FnMut(I)
    {
        let inputs = (0..self.iters).map(|_| setup()).collect::<Vec<_>>();

        let start = Instant::now();
        for input in inputs {
            routine(black_box(input));
        }
        self.elapsed = start.elapsed();
    }
}

/// The benchmark manager
///
/// `Criterion` lets you configure and execute benchmarks
///
/// Each benchmark consists of four phases:
///
/// - **Warm-up**: The routine is repeatedly executed, to let the CPU/OS/JIT/interpreter adapt to
/// the new load
/// - **Measurement**: The routine is repeatedly executed, and timing information is collected into
/// a sample
/// - **Analysis**: The sample is analyzed and distiled into meaningful statistics that get
/// reported to stdout, stored in files, and plotted
/// - **Comparison**: The current sample is compared with the sample obtained in the previous
/// benchmark.
pub struct Criterion {
    confidence_level: f64,
    measurement_time: Duration,
    noise_threshold: f64,
    nresamples: usize,
    plotting: Plotting,
    sample_size: usize,
    significance_level: f64,
    warm_up_time: Duration,
    filter: Option<String>,
}

impl Default for Criterion {

    /// Creates a benchmark manager with the following default settings:
    ///
    /// - Sample size: 100 measurements
    /// - Warm-up time: 3 s
    /// - Measurement time: 5 s
    /// - Bootstrap size: 100 000 resamples
    /// - Noise threshold: 0.01 (1%)
    /// - Confidence level: 0.95
    /// - Significance level: 0.05
    /// - Plotting: enabled (if gnuplot is available)
    /// - No filter
    fn default() -> Criterion {
        let plotting = if simplot::version().is_ok() {
            Plotting::Enabled
        } else {
            println!("Gnuplot not found, disabling plotting");

            Plotting::NotAvailable
        };

        Criterion {
            confidence_level: 0.95,
            measurement_time: Duration::new(5, 0),
            noise_threshold: 0.01,
            nresamples: 100_000,
            sample_size: 100,
            plotting: plotting,
            significance_level: 0.05,
            warm_up_time: Duration::new(3, 0),
            filter: None,
        }
    }
}

impl Criterion {

    /// Changes the size of the sample
    ///
    /// A bigger sample should yield more accurate results, if paired with a "sufficiently" large
    /// measurement time, on the other hand, it also increases the analysis time
    ///
    /// # Panics
    ///
    /// Panics if set to zero
    pub fn sample_size(&mut self, n: usize) -> &mut Criterion {
        assert!(n > 0);

        self.sample_size = n;
        self
    }

    /// Changes the warm up time
    ///
    /// # Panics
    ///
    /// Panics if the input duration is zero
    pub fn warm_up_time(&mut self, dur: Duration) -> &mut Criterion {
        assert!(dur.to_nanos() > 0);

        self.warm_up_time = dur;
        self
    }

    /// Changes the measurement time
    ///
    /// With a longer time, the measurement will become more resilient to transitory peak loads
    /// caused by external programs
    ///
    /// **Note**: If the measurement time is too "low", Criterion will automatically increase it
    ///
    /// # Panics
    ///
    /// Panics if the input duration in zero
    pub fn measurement_time(&mut self, dur: Duration) -> &mut Criterion {
        assert!(dur.to_nanos() > 0);

        self.measurement_time = dur;
        self
    }

    /// Changes the number of resamples
    ///
    /// Number of resamples to use for the
    /// [bootstrap](http://en.wikipedia.org/wiki/Bootstrapping_(statistics)#Case_resampling)
    ///
    /// A larger number of resamples reduces the random sampling errors, which are inherent to the
    /// bootstrap method, but also increases the analysis time
    ///
    /// # Panics
    ///
    /// Panics if the number of resamples is set to zero
    pub fn nresamples(&mut self, n: usize) -> &mut Criterion {
        assert!(n > 0);

        self.nresamples = n;
        self
    }

    /// Changes the noise threshold
    ///
    /// This threshold is used to decide if an increase of `X%` in the execution time is considered
    /// significant or should be flagged as noise
    ///
    /// *Note:* A value of `0.02` is equivalent to `2%`
    ///
    /// # Panics
    ///
    /// Panics is the threshold is set to a negative value
    pub fn noise_threshold(&mut self, threshold: f64) -> &mut Criterion {
        assert!(threshold >= 0.0);

        self.noise_threshold = threshold;
        self
    }

    /// Changes the confidence level
    ///
    /// The confidence level is used to calculate the
    /// [confidence intervals](https://en.wikipedia.org/wiki/Confidence_interval) of the estimated
    /// statistics
    ///
    /// # Panics
    ///
    /// Panics if the confidence level is set to a value outside the `(0, 1)` range
    pub fn confidence_level(&mut self, cl: f64) -> &mut Criterion {
        assert!(cl > 0.0 && cl < 1.0);

        self.confidence_level = cl;
        self
    }

    /// Changes the [significance level](https://en.wikipedia.org/wiki/Statistical_significance)
    ///
    /// The significance level is used for
    /// [hypothesis testing](https://en.wikipedia.org/wiki/Statistical_hypothesis_testing)
    ///
    /// # Panics
    ///
    /// Panics if the significance level is set to a value outside the `(0, 1)` range
    pub fn significance_level(&mut self, sl: f64) -> &mut Criterion {
        assert!(sl > 0.0 && sl < 1.0);

        self.significance_level = sl;
        self
    }

    /// Enables plotting
    pub fn with_plots(&mut self) -> &mut Criterion {
        match self.plotting {
            Plotting::NotAvailable => {},
            _ => self.plotting = Plotting::Enabled,
        }

        self
    }

    /// Disabled plotting
    pub fn without_plots(&mut self) -> &mut Criterion {
        match self.plotting {
            Plotting::NotAvailable => {},
            _ => self.plotting = Plotting::Disabled,
        }

        self
    }

    /// Checks if plotting is possible
    pub fn can_plot(&self) -> bool {
        match self.plotting {
            Plotting::NotAvailable => false,
            _ => true,
        }
    }

    /// Filters the benchmarks. Only benchmarks with names that contain the
    /// given string will be executed.
    pub fn with_filter<S: Into<String>>(&mut self, filter: S) -> &mut Criterion {
        self.filter = Some(filter.into());

        self
    }

    /// Configure this criterion struct based on the command-line arguments to
    /// this process.
    pub fn configure_from_args(&mut self) {
        if let Some(arg) = ::std::env::args().skip(1).find(|arg| *arg != "--bench") {
            self.with_filter(arg);
        }
    }

    fn filter_matches(&self, id: &str) -> bool {
        match &self.filter {
            &Some(ref string) => id.contains(string),
            &None => true,
        }
    }

    /// Benchmarks a function
    ///
    /// The function under test must follow the setup - bench - teardown pattern:
    ///
    /// ```rust,no_run
    /// use self::criterion::{Bencher, Criterion};
    ///
    /// fn routine(b: &mut Bencher) {
    ///     // Setup (construct data, allocate memory, etc)
    ///
    ///     b.iter(|| {
    ///         // Code to benchmark goes here
    ///     })
    ///
    ///     // Teardown (free resources)
    /// }
    ///
    /// Criterion::default().bench_function("routine", routine);
    /// ```
    pub fn bench_function<F>(&mut self, id: &str, f: F) -> &mut Criterion where
        F: FnMut(&mut Bencher),
    {
        if self.filter_matches(id) {
            analysis::function(id, f, self);
        }

        self
    }

    /// Benchmarks multiple functions
    ///
    /// All functions get the same input and are compared with the other implementations.
    /// Works similar to `bench`, but with multiple functions.
    ///
    /// ``` rust,no_run
    /// # use self::criterion::{Bencher, Criterion, Fun};
    /// # fn seq_fib(i: &u32) {}
    /// # fn par_fib(i: &u32) {}
    ///
    /// fn bench_seq_fib(b: &mut Bencher, i: &u32) {
    ///     b.iter(|| {
    ///         seq_fib(i);
    ///     });
    /// }
    ///
    /// fn bench_par_fib(b: &mut Bencher, i: &u32) {
    ///     b.iter(|| {
    ///         par_fib(i);
    ///     });
    /// }
    ///
    /// let sequential_fib = Fun::new("Sequential", bench_seq_fib);
    /// let parallel_fib = Fun::new("Parallel", bench_par_fib);
    /// let funs = vec![sequential_fib, parallel_fib];
    ///
    /// Criterion::default().bench_functions("Fibonacci", funs, &14);
    /// ```
    pub fn bench_functions<I>(&mut self,
        id: &str,
        funs: Vec<Fun<I>>,
        input: &I) -> &mut Criterion
        where I: fmt::Display
    {
        if self.filter_matches(id) {
            analysis::functions(id, funs, input, self);
        }
        self
    }

    /// Benchmarks a function under various inputs
    ///
    /// This is a convenience method to execute several related benchmarks. Each benchmark will
    /// receive the id: `${id}/${input}`.
    ///
    /// ```rust,no_run
    /// use self::criterion::{Bencher, Criterion};
    ///
    /// Criterion::default()
    ///     .bench_function_over_inputs("from_elem", |b: &mut Bencher, &&size: &&usize| {
    ///         b.iter(|| vec![0u8; size]);
    ///     }, &[1024, 2048, 4096]);
    /// ```
    pub fn bench_function_over_inputs<I, F>(
        &mut self,
        id: &str,
        f: F,
        inputs: I,
    ) -> &mut Criterion where
        I: IntoIterator,
        I::Item: fmt::Display,
        F: FnMut(&mut Bencher, &I::Item),
    {
        if self.filter_matches(id) {
            analysis::function_over_inputs(id, f, inputs, self);
        }

        self
    }

    /// Benchmarks an external program
    ///
    /// The external program must conform to the following specification:
    ///
    /// ```rust,no_run
    /// # use std::io::{self, BufRead};
    /// # use std::time::Instant;
    /// # use std::time::Duration;
    /// # trait DurationExt { fn to_nanos(&self) -> u64 { 0 } }
    /// # impl DurationExt for Duration {}
    ///
    /// fn main() {
    ///     let stdin = io::stdin();
    ///     let ref mut stdin = stdin.lock();
    ///
    ///     // For each line in stdin
    ///     for line in stdin.lines() {
    ///         // Parse line as the number of iterations
    ///         let iters: u64 = line.unwrap().trim().parse().unwrap();
    ///
    ///         // Setup
    ///
    ///         // Benchmark
    ///         let start = Instant::now();
    ///         // Execute the routine "iters" times
    ///         for _ in 0..iters {
    ///             // Code to benchmark goes here
    ///         }
    ///         let elapsed = start.elapsed();
    ///
    ///         // Teardown
    ///
    ///         // Report elapsed time in nanoseconds to stdout
    ///         println!("{}", elapsed.to_nanos());
    ///     }
    /// }
    /// ```
    pub fn bench_program(&mut self, id: &str, mut program: Command) -> &mut Criterion {
        if self.filter_matches(id) {
            analysis::program(id, &mut program, self);
        }

        self
    }

    /// Benchmarks an external program under various inputs
    ///
    /// This is a convenience method to execute several related benchmarks. Each benchmark will
    /// receive the id: `${id}/${input}`.
    pub fn bench_program_over_inputs<I, F>(
        &mut self,
        id: &str,
        program: F,
        inputs: I,
    ) -> &mut Criterion where
        F: FnMut() -> Command,
        I: IntoIterator,
        I::Item: fmt::Display,
    {
        if self.filter_matches(id) {
            analysis::program_over_inputs(id, program, inputs, self);
        }

        self
    }

    /// Summarize the results stored under the `.criterion/${id}` folder
    ///
    /// *Note:* The `bench_with_inputs` and `bench_program_with_inputs` functions internally call
    /// the `summarize` method
    pub fn summarize(&mut self, id: &str) -> &mut Criterion {
        analysis::summarize(id, self);

        self
    }
}
enum Plotting {
    Disabled,
    Enabled,
    NotAvailable,
}

impl Plotting {
    fn is_enabled(&self) -> bool {
        match *self {
            Plotting::Enabled => true,
            _ => false,
        }
    }
}

trait DurationExt {
    fn to_nanos(&self) -> u64;
}

const NANOS_PER_SEC: u64 = 1_000_000_000;

impl DurationExt for Duration {
    fn to_nanos(&self) -> u64 {
        self.as_secs() * NANOS_PER_SEC + u64::from(self.subsec_nanos())
    }
}

// TODO make private again
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct ConfidenceInterval {
    confidence_level: f64,
    lower_bound: f64,
    upper_bound: f64,
}

// TODO make private again
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Estimate {
    /// The confidence interval for this estimate
    pub confidence_interval: ConfidenceInterval,
    ///
    pub point_estimate: f64,
    /// The standard error of this estimate
    pub standard_error: f64,
}

impl Estimate {
    fn new(distributions: &Distributions, points: &[f64], cl: f64) -> Estimates {
        distributions.iter().zip(points.iter()).map(|((&statistic, distribution), &point)| {
            let (lb, ub) = distribution.confidence_interval(cl);

            (statistic, Estimate {
                confidence_interval: ConfidenceInterval {
                    confidence_level: cl,
                    lower_bound: lb,
                    upper_bound: ub,
                },
                point_estimate: point,
                standard_error: distribution.std_dev(None),
            })
        }).collect()
    }

    fn load(path: &Path) -> Option<Estimates> {
        let mut string = String::new();

        match File::open(path) {
            Err(_) => None,
            Ok(mut f) => match f.read_to_string(&mut string) {
                Err(_) => None,
                Ok(_) => match serde_json::from_str(&string) {
                    Err(_) => None,
                    Ok(estimates) => Some(estimates),
                },
            }
        }
    }
}
