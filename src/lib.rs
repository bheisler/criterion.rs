//! A statistics-driven micro-benchmarking library written in Rust.
//!
//! This crate is a microbenchmarking library which aims to provide strong
//! statistical confidence in detecting and estimating the size of performance
//! improvements and regressions, whle also being easy to use.
//!
//! See
//! [the user guide](https://bheisler.github.io/criterion.rs/book/index.html)
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
#![cfg_attr(not(feature = "html_reports"), allow(dead_code))]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        clippy::used_underscore_binding,
        clippy::just_underscores_and_digits,
        clippy::transmute_ptr_to_ptr
    )
)]

#[cfg(test)]
#[macro_use]
extern crate approx;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
extern crate rand;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

extern crate atty;
extern crate cast;
extern crate csv;
extern crate itertools;
extern crate num_traits;
extern crate rand_core;
extern crate rand_os;
extern crate rand_xoshiro;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate walkdir;

#[cfg(feature = "html_reports")]
extern crate criterion_plot;

#[cfg(feature = "html_reports")]
extern crate tinytemplate;

#[cfg(feature = "real_blackbox")]
extern crate test;

#[macro_use]
extern crate serde_derive;

// Needs to be declared before other modules
// in order to be usable there.
#[macro_use]
mod macros_private;
#[macro_use]
mod analysis;
mod benchmark;
mod csv_report;
mod error;
mod estimate;
mod format;
mod fs;
mod macros;
mod program;
mod report;
mod routine;
mod stats;

#[cfg(feature = "html_reports")]
mod kde;

#[cfg(feature = "html_reports")]
mod plot;

#[cfg(feature = "html_reports")]
mod html;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt;
use std::iter::IntoIterator;
use std::process::Command;
use std::time::{Duration, Instant};

use benchmark::BenchmarkConfig;
use benchmark::NamedRoutine;
use csv_report::FileCsvReport;
use estimate::{Distributions, Estimates, Statistic};
use plotting::Plotting;
use report::{CliReport, Report, ReportContext, Reports};
use routine::Function;

#[cfg(feature = "html_reports")]
use html::Html;

pub use benchmark::{Benchmark, BenchmarkDefinition, ParameterizedBenchmark};

lazy_static! {
    static ref DEBUG_ENABLED: bool = { std::env::vars().any(|(key, _)| key == "CRITERION_DEBUG") };
}

fn debug_enabled() -> bool {
    *DEBUG_ENABLED
}

// Fake function which shows a deprecation warning when compiled without the html_reports feature.
#[cfg(not(feature = "html_reports"))]
#[cfg_attr(not(feature = "html_reports"), doc(hidden))]
pub fn deprecation_warning() {
    #[deprecated(
        since = "0.2.6",
        note = "The html_reports cargo feature is deprecated. As of 0.3.0, HTML reports will no longer be optional."
    )]
    fn deprecation_warning_inner() {}

    deprecation_warning_inner()
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
pub struct Fun<I: fmt::Debug> {
    f: NamedRoutine<I>,
}

impl<I> Fun<I>
where
    I: fmt::Debug + 'static,
{
    /// Create a new `Fun` given a name and a closure
    pub fn new<F>(name: &str, f: F) -> Fun<I>
    where
        F: FnMut(&mut Bencher, &I) + 'static,
    {
        let routine = NamedRoutine {
            id: name.to_owned(),
            f: Box::new(RefCell::new(Function::new(f))),
        };

        Fun { f: routine }
    }
}

/// Argument to [`Bencher::iter_batched`](struct.Bencher.html#method.iter_batched) and
/// [`Bencher::iter_batched_ref`](struct.Bencher.html#method.iter_batched_ref) which controls the
/// batch size.
///
/// Generally speaking, almost all benchmarks should use `SmallInput`. If the input or the result
/// of the benchmark routine is large enough that `SmallInput` causes out-of-memory errors,
/// `LargeInput` can be used to reduce memory usage at the cost of increasing the measurement
/// overhead. If the input or the result is extremely large (or if it holds some
/// limited external resource like a file handle), `PerIteration` will set the number of iterations
/// per batch to exactly one. `PerIteration` can increase the measurement overhead substantially
/// and should be avoided wherever possible.
///
/// Each value lists an estimate of the measurement overhead. This is intended as a rough guide
/// to assist in choosing an option, it should not be relied upon. In particular, it is not valid
/// to subtract the listed overhead from the measurement and assume that the result represents the
/// true runtime of a function. The actual measurement overhead for your specific benchmark depends
/// on the details of the function you're benchmarking and the hardware and operating
/// system running the benchmark.
///
/// With that said, if the runtime of your function is small relative to the measurement overhead
/// it will be difficult to take accurate measurements. In this situation, the best option is to use
/// [`Bencher::iter`](struct.Bencher.html#method.iter_batched_ref) which has next-to-zero measurement
/// overhead.
#[derive(Debug, Eq, PartialEq, Copy, Hash, Clone)]
pub enum BatchSize {
    /// `SmallInput` indicates that the input to the benchmark routine (the value returned from
    /// the setup routine) is small enough that millions of values can be safely held in memory.
    /// Always prefer `SmallInput` unless the benchmark is using too much memory.
    ///
    /// In testing, the maximum measurement overhead from benchmarking with `SmallInput` is on the
    /// order of 500 picoseconds. This is presented as a rough guide; your results may vary.
    SmallInput,

    /// `LargeInput` indicates that the input to the benchmark routine or the value returned from
    /// that routine is large. This will reduce the memory usage but increase the measurement
    /// overhead.
    ///
    /// In testing, the maximum measurement overhead from benchmarking with `LargeInput` is on the
    /// order of 750 picoseconds. This is presented as a rough guide; your results may vary.
    LargeInput,

    /// `PerIteration` indicates that the input to the benchmark routine or the value returned from
    /// that routine is extremely large or holds some limited resource, such that holding many values
    /// in memory at once is infeasible. This provides the worst measurement overhead, but the
    /// lowest memory usage.
    ///
    /// In testing, the maximum measurement overhead from benchmarking with `PerIteration` is on the
    /// order of 350 nanoseconds or 350,000 picoseconds. This is presented as a rough guide; your
    /// results may vary.
    PerIteration,

    /// `NumBatches` will attempt to divide the iterations up into a given number of batches.
    /// A larger number of batches (and thus smaller batches) will reduce memory usage but increase
    /// measurement overhead. This allows the user to choose their own tradeoff between memory usage
    /// and measurement overhead, but care must be taken in tuning the number of batches. Most
    /// benchmarks should use `SmallInput` or `LargeInput` instead.
    NumBatches(u64),

    /// `NumIterations` fixes the batch size to a constant number, specified by the user. This
    /// allows the user to choose their own tradeoff between overhead and memory usage, but care must
    /// be taken in tuning the batch size. In general, the measurement overhead of NumIterations
    /// will be larger than that of `NumBatches`. Most benchmarks should use `SmallInput` or
    /// `LargeInput` instead.
    NumIterations(u64),

    #[doc(hidden)]
    __NonExhaustive,
}
impl BatchSize {
    /// Convert to a number of iterations per batch.
    ///
    /// We try to do a constant number of batches regardless of the number of iterations in this
    /// sample. If the measurement overhead is roughly constant regardless of the number of
    /// iterations the analysis of the results later will have an easier time separating the
    /// measurement overhead from the benchmark time.
    fn iters_per_batch(self, iters: u64) -> u64 {
        match self {
            BatchSize::SmallInput => (iters + 10 - 1) / 10,
            BatchSize::LargeInput => (iters + 1000 - 1) / 1000,
            BatchSize::PerIteration => 1,
            BatchSize::NumBatches(batches) => (iters + batches - 1) / batches,
            BatchSize::NumIterations(size) => size,
            BatchSize::__NonExhaustive => panic!("__NonExhaustive is not a valid BatchSize."),
        }
    }
}

/// Timer struct to iterate a benchmarked function and measure the runtime.
///
/// This struct provides different timing loops as methods. Each timing loop provides a different
/// way to time a routine and each has advantages and disadvantages.
///
/// * If your routine requires no per-iteration setup and returns a value with an expensive `drop`
///   method, use `iter_with_large_drop`.
/// * If your routine requires some per-iteration setup that shouldn't be timed, use `iter_batched`
///   or `iter_batched_ref`. See [`BatchSize`](enum.BatchSize.html) for a discussion of batch sizes.
///   If the setup value implements `Drop` and you don't want to include the `drop` time in the
///   measurement, use `iter_batched_ref`, otherwise use `iter_batched`. These methods are also
///   suitable for benchmarking routines which return a value with an expensive `drop` method,
///   but are more complex than `iter_with_large_drop`.
/// * Otherwise, use `iter`.
#[derive(Clone, Copy)]
pub struct Bencher {
    iterated: bool,
    iters: u64,
    elapsed: Duration,
}
impl Bencher {
    /// Times a `routine` by executing it many times and timing the total elapsed time.
    ///
    /// Prefer this timing loop when `routine` returns a value that doesn't have a destructor.
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
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate criterion;
    ///
    /// use criterion::*;
    ///
    /// // The function to benchmark
    /// fn foo() {
    ///     // ...
    /// }
    ///
    /// fn bench(c: &mut Criterion) {
    ///     c.bench_function("iter", move |b| {
    ///         b.iter(|| foo())
    ///     });
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    ///
    #[inline(never)]
    pub fn iter<O, R>(&mut self, mut routine: R)
    where
        R: FnMut() -> O,
    {
        self.iterated = true;
        let start = Instant::now();
        for _ in 0..self.iters {
            black_box(routine());
        }
        self.elapsed = start.elapsed();
    }

    #[doc(hidden)]
    pub fn iter_with_setup<I, O, S, R>(&mut self, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> O,
    {
        self.iter_batched(setup, routine, BatchSize::PerIteration);
    }

    /// Times a `routine` by collecting its output on each iteration. This avoids timing the
    /// destructor of the value returned by `routine`.
    ///
    /// WARNING: This requires `O(iters * mem::size_of::<O>())` of memory, and `iters` is not under the
    /// control of the caller. If this causes out-of-memory errors, use `iter_batched` instead.
    ///
    /// # Timing model
    ///
    /// ``` text
    /// elapsed = Instant::now + iters * (routine) + Iterator::collect::<Vec<_>>
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate criterion;
    ///
    /// use criterion::*;
    ///
    /// fn create_vector() -> Vec<u64> {
    ///     # vec![]
    ///     // ...
    /// }
    ///
    /// fn bench(c: &mut Criterion) {
    ///     c.bench_function("with_drop", move |b| {
    ///         // This will avoid timing the Vec::drop.
    ///         b.iter_with_large_drop(|| create_vector())
    ///     });
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    ///
    #[doc(hidden)]
    pub fn iter_with_large_drop<O, R>(&mut self, mut routine: R)
    where
        R: FnMut() -> O,
    {
        self.iter_batched(|| (), |_| routine(), BatchSize::SmallInput);
    }

    #[doc(hidden)]
    pub fn iter_with_large_setup<I, O, S, R>(&mut self, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> O,
    {
        self.iter_batched(setup, routine, BatchSize::NumBatches(1));
    }

    /// Times a `routine` that requires some input by generating a batch of input, then timing the
    /// iteration of the benchmark over the input. See [`BatchSize`](struct.BatchSize.html) for
    /// details on choosing the batch size. Use this when the routine must consume its input.
    ///
    /// For example, use this loop to benchmark sorting algorithms, because they require unsorted
    /// data on each iteration.
    ///
    /// # Timing model
    ///
    /// ```text
    /// elapsed = (Instant::now * num_batches) + (iters * (routine + O::drop)) + Vec::extend
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate criterion;
    ///
    /// use criterion::*;
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
    /// fn bench(c: &mut Criterion) {
    ///     let data = create_scrambled_data();
    ///
    ///     c.bench_function("with_setup", move |b| {
    ///         // This will avoid timing the to_vec call.
    ///         b.iter_batched(|| data.clone(), |mut data| sort(&mut data), BatchSize::SmallInput)
    ///     });
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    ///
    #[inline(never)]
    pub fn iter_batched<I, O, S, R>(&mut self, mut setup: S, mut routine: R, size: BatchSize)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> O,
    {
        self.iterated = true;
        let batch_size = size.iters_per_batch(self.iters);
        assert!(batch_size != 0, "Batch size must not be zero.");
        self.elapsed = Duration::from_secs(0);

        if batch_size == 1 {
            for _ in 0..self.iters {
                let mut input = black_box(setup());

                let start = Instant::now();
                let output = routine(input);
                self.elapsed += start.elapsed();

                drop(black_box(output));
            }
        } else {
            let mut iteration_counter = 0;

            while iteration_counter < self.iters {
                let batch_size = ::std::cmp::min(batch_size, self.iters - iteration_counter);

                let inputs = black_box((0..batch_size).map(|_| setup()).collect::<Vec<_>>());
                let mut outputs = Vec::with_capacity(batch_size as usize);

                let start = Instant::now();
                outputs.extend(inputs.into_iter().map(&mut routine));
                self.elapsed += start.elapsed();

                black_box(outputs);

                iteration_counter += batch_size;
            }
        }
    }

    /// Times a `routine` that requires some input by generating a batch of input, then timing the
    /// iteration of the benchmark over the input. See [`BatchSize`](struct.BatchSize.html) for
    /// details on choosing the batch size. Use this when the routine should accept the input by
    /// mutable reference.
    ///
    /// For example, use this loop to benchmark sorting algorithms, because they require unsorted
    /// data on each iteration.
    ///
    /// # Timing model
    ///
    /// ```text
    /// elapsed = (Instant::now * num_batches) + (iters * routine) + Vec::extend
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate criterion;
    ///
    /// use criterion::*;
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
    /// fn bench(c: &mut Criterion) {
    ///     let data = create_scrambled_data();
    ///
    ///     c.bench_function("with_setup", move |b| {
    ///         // This will avoid timing the to_vec call.
    ///         b.iter_batched(|| data.clone(), |mut data| sort(&mut data), BatchSize::SmallInput)
    ///     });
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    ///
    #[inline(never)]
    pub fn iter_batched_ref<I, O, S, R>(&mut self, mut setup: S, mut routine: R, size: BatchSize)
    where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> O,
    {
        self.iterated = true;
        let batch_size = size.iters_per_batch(self.iters);
        assert!(batch_size != 0, "Batch size must not be zero.");
        self.elapsed = Duration::from_secs(0);

        if batch_size == 1 {
            for _ in 0..self.iters {
                let mut input = black_box(setup());

                let start = Instant::now();
                let output = routine(&mut input);
                self.elapsed += start.elapsed();

                drop(black_box(output));
                drop(black_box(input));
            }
        } else {
            let mut iteration_counter = 0;

            while iteration_counter < self.iters {
                let batch_size = ::std::cmp::min(batch_size, self.iters - iteration_counter);

                let mut inputs = black_box((0..batch_size).map(|_| setup()).collect::<Vec<_>>());
                let mut outputs = Vec::with_capacity(batch_size as usize);

                let start = Instant::now();
                outputs.extend(inputs.iter_mut().map(&mut routine));
                self.elapsed += start.elapsed();

                black_box(outputs);

                iteration_counter += batch_size;
            }
        }
    }

    // Benchmarks must actually call one of the iter methods. This causes benchmarks to fail loudly
    // if they don't.
    fn assert_iterated(&mut self) {
        if !self.iterated {
            panic!("Benchmark function must call Bencher::iter or related method.");
        }
        self.iterated = false;
    }
}

/// Baseline describes how the baseline_directory is handled.
pub enum Baseline {
    /// Compare ensures a previous saved version of the baseline
    /// exists and runs comparison against that.
    Compare,
    /// Save writes the benchmark results to the baseline directory,
    /// overwriting any results that were previously there.
    Save,
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
    config: BenchmarkConfig,
    plotting: Plotting,
    filter: Option<String>,
    report: Box<Report>,
    output_directory: String,
    baseline_directory: String,
    baseline: Baseline,
    profile_time: Option<Duration>,
    test_mode: bool,
    list_mode: bool,
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
        #[allow(unused_mut, unused_assignments)]
        let mut plotting = Plotting::Unset;

        let mut reports: Vec<Box<Report>> = vec![];
        reports.push(Box::new(CliReport::new(false, false, false)));
        reports.push(Box::new(FileCsvReport));

        let output_directory =
            match std::env::vars().find(|&(ref key, _)| key == "CARGO_TARGET_DIR") {
                Some((_, value)) => format!("{}/criterion", value),
                None => "target/criterion".to_owned(),
            };

        Criterion {
            config: BenchmarkConfig {
                confidence_level: 0.95,
                measurement_time: Duration::new(5, 0),
                noise_threshold: 0.01,
                nresamples: 100_000,
                sample_size: 100,
                significance_level: 0.05,
                warm_up_time: Duration::new(3, 0),
            },
            plotting,
            filter: None,
            report: Box::new(Reports::new(reports)),
            baseline_directory: "base".to_owned(),
            baseline: Baseline::Save,
            profile_time: None,
            test_mode: false,
            list_mode: false,
            output_directory,
        }
    }
}

impl Criterion {
    /// Changes the default size of the sample for benchmarks run with this runner.
    ///
    /// A bigger sample should yield more accurate results if paired with a sufficiently large
    /// measurement time.
    ///
    /// Sample size must be at least 2.
    ///
    /// # Panics
    ///
    /// Panics if set to zero or one
    pub fn sample_size(mut self, n: usize) -> Criterion {
        assert!(n >= 2);
        if n < 10 {
            println!("Warning: Sample sizes < 10 will be disallowed in Criterion.rs 0.3.0.");
        }

        self.config.sample_size = n;
        self
    }

    /// Changes the default warm up time for benchmarks run with this runner.
    ///
    /// # Panics
    ///
    /// Panics if the input duration is zero
    pub fn warm_up_time(mut self, dur: Duration) -> Criterion {
        assert!(dur.to_nanos() > 0);

        self.config.warm_up_time = dur;
        self
    }

    /// Changes the default measurement time for benchmarks run with this runner.
    ///
    /// With a longer time, the measurement will become more resilient to transitory peak loads
    /// caused by external programs
    ///
    /// **Note**: If the measurement time is too "low", Criterion will automatically increase it
    ///
    /// # Panics
    ///
    /// Panics if the input duration in zero
    pub fn measurement_time(mut self, dur: Duration) -> Criterion {
        assert!(dur.to_nanos() > 0);

        self.config.measurement_time = dur;
        self
    }

    /// Changes the default number of resamples for benchmarks run with this runner.
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
    pub fn nresamples(mut self, n: usize) -> Criterion {
        assert!(n > 0);

        self.config.nresamples = n;
        self
    }

    /// Changes the default noise threshold for benchmarks run with this runner.
    ///
    /// This threshold is used to decide if an increase of `X%` in the execution time is considered
    /// significant or should be flagged as noise
    ///
    /// *Note:* A value of `0.02` is equivalent to `2%`
    ///
    /// # Panics
    ///
    /// Panics is the threshold is set to a negative value
    pub fn noise_threshold(mut self, threshold: f64) -> Criterion {
        assert!(threshold >= 0.0);

        self.config.noise_threshold = threshold;
        self
    }

    /// Changes the default confidence level for benchmarks run with this runner
    ///
    /// The confidence level is used to calculate the
    /// [confidence intervals](https://en.wikipedia.org/wiki/Confidence_interval) of the estimated
    /// statistics
    ///
    /// # Panics
    ///
    /// Panics if the confidence level is set to a value outside the `(0, 1)` range
    pub fn confidence_level(mut self, cl: f64) -> Criterion {
        assert!(cl > 0.0 && cl < 1.0);

        self.config.confidence_level = cl;
        self
    }

    /// Changes the default [significance level](https://en.wikipedia.org/wiki/Statistical_significance)
    /// for benchmarks run with this runner
    ///
    /// The significance level is used for
    /// [hypothesis testing](https://en.wikipedia.org/wiki/Statistical_hypothesis_testing)
    ///
    /// # Panics
    ///
    /// Panics if the significance level is set to a value outside the `(0, 1)` range
    pub fn significance_level(mut self, sl: f64) -> Criterion {
        assert!(sl > 0.0 && sl < 1.0);

        self.config.significance_level = sl;
        self
    }

    /// Enables plotting
    #[cfg(feature = "html_reports")]
    pub fn with_plots(mut self) -> Criterion {
        use criterion_plot::VersionError;
        self.plotting = match criterion_plot::version() {
            Ok(_) => {
                let mut reports: Vec<Box<Report>> = vec![];
                reports.push(Box::new(CliReport::new(false, false, false)));
                reports.push(Box::new(FileCsvReport));
                reports.push(Box::new(Html::new()));
                self.report = Box::new(Reports::new(reports));
                Plotting::Enabled
            }
            Err(e) => {
                match e {
                    VersionError::Exec(_) => println!("Gnuplot not found, disabling plotting"),
                    e => println!("Gnuplot not found or not usable, disabling plotting\n{}", e),
                }
                Plotting::NotAvailable
            }
        };

        self
    }

    /// Enables plotting
    #[cfg(not(feature = "html_reports"))]
    pub fn with_plots(self) -> Criterion {
        self
    }

    /// Disables plotting
    pub fn without_plots(mut self) -> Criterion {
        self.plotting = Plotting::Disabled;
        self
    }

    /// Return true if generation of the plots is possible.
    #[cfg(feature = "html_reports")]
    pub fn can_plot(&self) -> bool {
        match self.plotting {
            Plotting::NotAvailable => false,
            Plotting::Enabled => true,
            _ => criterion_plot::version().is_ok(),
        }
    }

    /// Return true if generation of the plots is possible.
    #[cfg(not(feature = "html_reports"))]
    pub fn can_plot(&self) -> bool {
        false
    }

    /// Names an explicit baseline and enables overwriting the previous results.
    pub fn save_baseline(mut self, baseline: String) -> Criterion {
        self.baseline_directory = baseline;
        self.baseline = Baseline::Save;
        self
    }

    /// Names an explicit baseline and disables overwriting the previous results.
    pub fn retain_baseline(mut self, baseline: String) -> Criterion {
        self.baseline_directory = baseline;
        self.baseline = Baseline::Compare;
        self
    }

    /// Filters the benchmarks. Only benchmarks with names that contain the
    /// given string will be executed.
    pub fn with_filter<S: Into<String>>(mut self, filter: S) -> Criterion {
        self.filter = Some(filter.into());

        self
    }

    /// Set the output directory (currently for testing only)
    #[doc(hidden)]
    pub fn output_directory(mut self, path: &std::path::Path) -> Criterion {
        self.output_directory = path.to_string_lossy().into_owned();

        self
    }

    /// Generate the final summary at the end of a run.
    #[doc(hidden)]
    pub fn final_summary(&self) {
        if self.profile_time.is_some() || self.test_mode {
            return;
        }

        let report_context = ReportContext {
            output_directory: self.output_directory.clone(),
            plotting: self.plotting,
            plot_config: PlotConfiguration::default(),
            test_mode: self.test_mode,
        };

        self.report.final_summary(&report_context);
    }

    /// Configure this criterion struct based on the command-line arguments to
    /// this process.
    pub fn configure_from_args(mut self) -> Criterion {
        use clap::{App, Arg};
        let matches = App::new("Criterion Benchmark")
            .arg(Arg::with_name("FILTER")
                .help("Skip benchmarks whose names do not contain FILTER.")
                .index(1))
            .arg(Arg::with_name("color")
                .short("c")
                .long("color")
                .alias("colour")
                .takes_value(true)
                .possible_values(&["auto", "always", "never"])
                .default_value("auto")
                .help("Configure coloring of output. always = always colorize output, never = never colorize output, auto = colorize output if output is a tty and compiled for unix."))
            .arg(Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Print additional statistical information."))
            .arg(Arg::with_name("noplot")
                .short("n")
                .long("noplot")
                .help("Disable plot and HTML generation."))
            .arg(Arg::with_name("save-baseline")
                .short("s")
                .long("save-baseline")
                .default_value("base")
                .help("Save results under a named baseline."))
            .arg(Arg::with_name("baseline")
                .short("b")
                .long("baseline")
                .takes_value(true)
                .conflicts_with("save-baseline")
                .help("Compare to a named baseline."))
            .arg(Arg::with_name("list")
                .long("list")
                .help("List all benchmarks"))
            .arg(Arg::with_name("measure-only")
                .long("measure-only")
                .hidden(true)
                .help("Only perform measurements; do no analysis or storage of results. This is useful eg. when profiling the benchmarks, to reduce clutter in the profiling data."))
            .arg(Arg::with_name("profile-time")
                .long("profile-time")
                .takes_value(true)
                .help("Iterate each benchmark for approximately the given number of seconds, doing no analysis and without storing the results. Useful for running the benchmarks in a profiler."))
            .arg(Arg::with_name("test")
                .long("test")
                .help("Run the benchmarks once, to verify that they execute successfully, but do not measure or report the results."))
            //Ignored but always passed to benchmark executables
            .arg(Arg::with_name("bench")
                .hidden(true)
                .long("bench"))
            .arg(Arg::with_name("version")
                .hidden(true)
                .short("V")
                .long("version"))
            .after_help("
This executable is a Criterion.rs benchmark.
See https://github.com/bheisler/criterion.rs for more details.

To enable debug output, define the environment variable CRITERION_DEBUG.
Criterion.rs will output more debug information and will save the gnuplot
scripts alongside the generated plots.
")
            .get_matches();

        if let Some(filter) = matches.value_of("FILTER") {
            self = self.with_filter(filter);
        }

        let verbose = matches.is_present("verbose");
        let stdout_isatty = atty::is(atty::Stream::Stdout);
        let mut enable_text_overwrite = stdout_isatty && !verbose && !debug_enabled();
        let enable_text_coloring;
        match matches.value_of("color") {
            Some("always") => {
                enable_text_coloring = true;
            }
            Some("never") => {
                enable_text_coloring = false;
                enable_text_overwrite = false;
            }
            _ => enable_text_coloring = stdout_isatty,
        }

        if matches.is_present("noplot") || matches.is_present("test") {
            self = self.without_plots();
        } else {
            self = self.with_plots();
        }

        if let Some(dir) = matches.value_of("save-baseline") {
            self.baseline = Baseline::Save;
            self.baseline_directory = dir.to_owned()
        }
        if let Some(dir) = matches.value_of("baseline") {
            self.baseline = Baseline::Compare;
            self.baseline_directory = dir.to_owned();
        }

        let mut reports: Vec<Box<Report>> = vec![];
        reports.push(Box::new(CliReport::new(
            enable_text_overwrite,
            enable_text_coloring,
            verbose,
        )));
        reports.push(Box::new(FileCsvReport));

        // TODO: Remove this in 0.3.0
        if matches.is_present("measure-only") {
            println!("Warning: The '--measure-only' argument is deprecated and will be removed in Criterion.rs 0.3.0. Use '--profile-time' instead.");
            self.profile_time = Some(Duration::from_secs(5));
        }
        if matches.is_present("profile-time") {
            let num_seconds = value_t!(matches.value_of("profile-time"), u64).unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1)
            });

            if num_seconds < 1 {
                println!("Profile time must be at least one second.");
                std::process::exit(1);
            }

            self.profile_time = Some(Duration::from_secs(num_seconds));
        }
        self.test_mode = matches.is_present("test");
        if matches.is_present("list") {
            self.test_mode = true;
            self.list_mode = true;
        }

        #[cfg(feature = "html_reports")]
        {
            if self.profile_time.is_none() {
                reports.push(Box::new(Html::new()));
            }
        }

        self.report = Box::new(Reports::new(reports));

        self
    }

    fn filter_matches(&self, id: &str) -> bool {
        match self.filter {
            Some(ref string) => id.contains(string),
            None => true,
        }
    }

    /// Benchmarks a function
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[macro_use] extern crate criterion;
    /// # use self::criterion::*;
    ///
    /// fn bench(c: &mut Criterion) {
    ///     // Setup (construct data, allocate memory, etc)
    ///     c.bench_function(
    ///         "function_name",
    ///         |b| b.iter(|| {
    ///             // Code to benchmark goes here
    ///         }),
    ///     );
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    pub fn bench_function<F>(&mut self, id: &str, f: F) -> &mut Criterion
    where
        F: FnMut(&mut Bencher) + 'static,
    {
        self.bench(id, Benchmark::new(id, f))
    }

    /// Benchmarks multiple functions
    ///
    /// All functions get the same input and are compared with the other implementations.
    /// Works similar to `bench_function`, but with multiple functions.
    ///
    /// # Example
    ///
    /// ``` rust
    /// # #[macro_use] extern crate criterion;
    /// # use self::criterion::*;
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
    /// fn bench(c: &mut Criterion) {
    ///     let sequential_fib = Fun::new("Sequential", bench_seq_fib);
    ///     let parallel_fib = Fun::new("Parallel", bench_par_fib);
    ///     let funs = vec![sequential_fib, parallel_fib];
    ///
    ///     c.bench_functions("Fibonacci", funs, 14);
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    pub fn bench_functions<I>(&mut self, id: &str, funs: Vec<Fun<I>>, input: I) -> &mut Criterion
    where
        I: fmt::Debug + 'static,
    {
        let benchmark = ParameterizedBenchmark::with_functions(
            funs.into_iter().map(|fun| fun.f).collect(),
            vec![input],
        );

        self.bench(id, benchmark)
    }

    /// Benchmarks a function under various inputs
    ///
    /// This is a convenience method to execute several related benchmarks. Each benchmark will
    /// receive the id: `${id}/${input}`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[macro_use] extern crate criterion;
    /// # use self::criterion::*;
    ///
    /// fn bench(c: &mut Criterion) {
    ///     c.bench_function_over_inputs("from_elem",
    ///         |b: &mut Bencher, size: &usize| {
    ///             b.iter(|| vec![0u8; *size]);
    ///         },
    ///         vec![1024, 2048, 4096]
    ///     );
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    pub fn bench_function_over_inputs<I, F>(&mut self, id: &str, f: F, inputs: I) -> &mut Criterion
    where
        I: IntoIterator,
        I::Item: fmt::Debug + 'static,
        F: FnMut(&mut Bencher, &I::Item) + 'static,
    {
        self.bench(id, ParameterizedBenchmark::new(id, f, inputs))
    }

    /// Benchmarks an external program
    ///
    /// The external program must:
    ///
    /// * Read the number of iterations from stdin
    /// * Execute the routine to benchmark that many times
    /// * Print the elapsed time (in nanoseconds) to stdout
    ///
    /// ```rust,no_run
    /// # use std::io::{self, BufRead};
    /// # use std::time::Instant;
    /// # use std::time::Duration;
    /// # trait DurationExt { fn to_nanos(&self) -> u64 { 0 } }
    /// # impl DurationExt for Duration {}
    /// // Example of an external program that implements this protocol
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
    #[deprecated(
        since = "0.2.6",
        note = "External program benchmarks were rarely used and are awkward to maintain, so they are scheduled for deletion in 0.3.0"
    )]
    #[allow(deprecated)]
    pub fn bench_program(&mut self, id: &str, program: Command) -> &mut Criterion {
        self.bench(id, Benchmark::new_external(id, program))
    }

    /// Benchmarks an external program under various inputs
    ///
    /// This is a convenience method to execute several related benchmarks. Each benchmark will
    /// receive the id: `${id}/${input}`.
    #[deprecated(
        since = "0.2.6",
        note = "External program benchmarks were rarely used and are awkward to maintain, so they are scheduled for deletion in 0.3.0"
    )]
    #[allow(deprecated)]
    pub fn bench_program_over_inputs<I, F>(
        &mut self,
        id: &str,
        mut program: F,
        inputs: I,
    ) -> &mut Criterion
    where
        F: FnMut() -> Command + 'static,
        I: IntoIterator,
        I::Item: fmt::Debug + 'static,
    {
        self.bench(
            id,
            ParameterizedBenchmark::new_external(
                id,
                move |i| {
                    let mut command = program();
                    command.arg(format!("{:?}", i));
                    command
                },
                inputs,
            ),
        )
    }

    /// Executes the given benchmark. Use this variant to execute benchmarks
    /// with complex configuration. This can be used to compare multiple
    /// functions, execute benchmarks with custom configuration settings and
    /// more. See the Benchmark and ParameterizedBenchmark structs for more
    /// information.
    ///
    /// ```rust
    /// # #[macro_use] extern crate criterion;
    /// # use criterion::*;
    /// # fn routine_1() {}
    /// # fn routine_2() {}
    ///
    /// fn bench(c: &mut Criterion) {
    ///     // Setup (construct data, allocate memory, etc)
    ///     c.bench(
    ///         "routines",
    ///         Benchmark::new("routine_1", |b| b.iter(|| routine_1()))
    ///             .with_function("routine_2", |b| b.iter(|| routine_2()))
    ///             .sample_size(50)
    ///     );
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    pub fn bench<B: BenchmarkDefinition>(
        &mut self,
        group_id: &str,
        benchmark: B,
    ) -> &mut Criterion {
        benchmark.run(group_id, self);
        self
    }
}

mod plotting {
    #[derive(Debug, Clone, Copy)]
    pub enum Plotting {
        Unset,
        Disabled,
        Enabled,
        NotAvailable,
    }

    impl Plotting {
        pub fn is_enabled(self) -> bool {
            match self {
                Plotting::Enabled => true,
                _ => false,
            }
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

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Debug)]
struct ConfidenceInterval {
    confidence_level: f64,
    lower_bound: f64,
    upper_bound: f64,
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Debug)]
struct Estimate {
    /// The confidence interval for this estimate
    confidence_interval: ConfidenceInterval,
    ///
    point_estimate: f64,
    /// The standard error of this estimate
    standard_error: f64,
}

fn build_estimates(
    distributions: &Distributions,
    points: &BTreeMap<Statistic, f64>,
    cl: f64,
) -> Estimates {
    distributions
        .iter()
        .map(|(&statistic, distribution)| {
            let point_estimate = points[&statistic];
            let (lb, ub) = distribution.confidence_interval(cl);

            (
                statistic,
                Estimate {
                    confidence_interval: ConfidenceInterval {
                        confidence_level: cl,
                        lower_bound: lb,
                        upper_bound: ub,
                    },
                    point_estimate,
                    standard_error: distribution.std_dev(None),
                },
            )
        })
        .collect()
}

/// Enum representing different ways of measuring the throughput of benchmarked code.
/// If the throughput setting is configured for a benchmark then the estimated throughput will
/// be reported as well as the time per iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Throughput {
    /// Measure throughput in terms of bytes/second. The value should be the number of bytes
    /// processed by one iteration of the benchmarked code. Typically, this would be the length of
    /// an input string or `&[u8]`.
    Bytes(u32),

    /// Measure throughput in terms of elements/second. The value should be the number of elements
    /// processed by one iteration of the benchmarked code. Typically, this would be the size of a
    /// collection, but could also be the number of lines of input text or the number of values to
    /// parse.
    Elements(u32),
}

/// Axis scaling type
#[derive(Debug, Clone, Copy)]
pub enum AxisScale {
    /// Axes scale linearly
    Linear,

    /// Axes scale logarithmically
    Logarithmic,
}

/// Contains the configuration options for the plots generated by a particular benchmark
/// or benchmark group.
///
/// ```rust
/// use self::criterion::{Bencher, Criterion, Benchmark, PlotConfiguration, AxisScale};
///
/// let plot_config = PlotConfiguration::default()
///     .summary_scale(AxisScale::Logarithmic);
///
/// Benchmark::new("test", |b| b.iter(|| 10))
///     .plot_config(plot_config);
/// ```
#[derive(Debug, Clone)]
pub struct PlotConfiguration {
    summary_scale: AxisScale,
}

impl Default for PlotConfiguration {
    fn default() -> PlotConfiguration {
        PlotConfiguration {
            summary_scale: AxisScale::Linear,
        }
    }
}

impl PlotConfiguration {
    /// Set the axis scale (linear or logarithmic) for the summary plots. Typically, you would
    /// set this to logarithmic if benchmarking over a range of inputs which scale exponentially.
    /// Defaults to linear.
    pub fn summary_scale(mut self, new_scale: AxisScale) -> PlotConfiguration {
        self.summary_scale = new_scale;
        self
    }
}
