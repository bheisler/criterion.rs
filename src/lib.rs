//! A statistics-driven micro-benchmarking library written in Rust.
//!
//! This crate is a microbenchmarking library which aims to provide strong
//! statistical confidence in detecting and estimating the size of performance
//! improvements and regressions, while also being easy to use.
//!
//! See
//! [the user guide](https://bheisler.github.io/criterion.rs/book/index.html)
//! for examples as well as details on the measurement and analysis process,
//! and the output.
//!
//! ## Features:
//! * Collects detailed statistics, providing strong confidence that changes
//!   to performance are real, not measurement noise
//! * Produces detailed charts, providing thorough understanding of your code's
//!   performance behavior.

#![deny(missing_docs)]
#![deny(bare_trait_objects)]
#![deny(warnings)]
#![cfg_attr(feature = "real_blackbox", feature(test))]
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
extern crate criterion_plot;
extern crate csv;
extern crate itertools;
extern crate num_traits;
extern crate rand_core;
extern crate rand_os;
extern crate rand_xoshiro;
extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
extern crate walkdir;

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
#[macro_use]
mod benchmark_group;
mod csv_report;
mod error;
mod estimate;
mod format;
mod fs;
mod html;
mod kde;
mod macros;
pub mod measurement;
mod plot;
pub mod profiler;
mod report;
mod routine;
mod stats;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::default::Default;
use std::fmt;
use std::iter::IntoIterator;
use std::marker::PhantomData;
use std::time::Duration;

use crate::benchmark::BenchmarkConfig;
use crate::benchmark::NamedRoutine;
use crate::csv_report::FileCsvReport;
use crate::estimate::{Distributions, Estimates, Statistic};
use crate::html::Html;
use crate::measurement::{Measurement, WallTime};
use crate::plotting::Plotting;
use crate::profiler::{ExternalProfiler, Profiler};
use crate::report::{CliReport, Report, ReportContext, Reports};
use crate::routine::Function;

pub use crate::benchmark::{Benchmark, BenchmarkDefinition, ParameterizedBenchmark};
pub use crate::benchmark_group::{BenchmarkGroup, BenchmarkId};

lazy_static! {
    static ref DEBUG_ENABLED: bool = { std::env::vars().any(|(key, _)| key == "CRITERION_DEBUG") };
}

fn debug_enabled() -> bool {
    *DEBUG_ENABLED
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
#[doc(hidden)]
pub struct Fun<I: fmt::Debug, M: Measurement + 'static = WallTime> {
    f: NamedRoutine<I, M>,
    _phantom: PhantomData<M>,
}

impl<I, M: Measurement> Fun<I, M>
where
    I: fmt::Debug + 'static,
{
    /// Create a new `Fun` given a name and a closure
    pub fn new<F>(name: &str, f: F) -> Fun<I, M>
    where
        F: FnMut(&mut Bencher<M>, &I) + 'static,
    {
        let routine = NamedRoutine {
            id: name.to_owned(),
            f: Box::new(RefCell::new(Function::new(f))),
        };

        Fun {
            f: routine,
            _phantom: PhantomData,
        }
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
/// [`Bencher::iter`](struct.Bencher.html#method.iter) which has next-to-zero measurement overhead.
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

/// Timer struct used to iterate a benchmarked function and measure the runtime.
///
/// This struct provides different timing loops as methods. Each timing loop provides a different
/// way to time a routine and each has advantages and disadvantages.
///
/// * If you want to do the iteration and measurement yourself (eg. passing the iteration count
///   to a separate process), use `iter_custom`.
/// * If your routine requires no per-iteration setup and returns a value with an expensive `drop`
///   method, use `iter_with_large_drop`.
/// * If your routine requires some per-iteration setup that shouldn't be timed, use `iter_batched`
///   or `iter_batched_ref`. See [`BatchSize`](enum.BatchSize.html) for a discussion of batch sizes.
///   If the setup value implements `Drop` and you don't want to include the `drop` time in the
///   measurement, use `iter_batched_ref`, otherwise use `iter_batched`. These methods are also
///   suitable for benchmarking routines which return a value with an expensive `drop` method,
///   but are more complex than `iter_with_large_drop`.
/// * Otherwise, use `iter`.
pub struct Bencher<'a, M: Measurement = WallTime> {
    iterated: bool,
    iters: u64,
    value: M::Value,
    measurement: &'a M,
}
impl<'a, M: Measurement> Bencher<'a, M> {
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
        let start = self.measurement.start();
        for _ in 0..self.iters {
            black_box(routine());
        }
        self.value = self.measurement.end(start);
    }

    /// Times a `routine` by executing it many times and relying on `routine` to measure it's own execution time.
    ///
    /// Prefer this timing loop in cases where `routine` has to do it's own measurements to
    /// get accurate timing information (for example in multi-threaded scenarios where you spawn
    /// and coordinate with multiple threads).
    ///
    /// # Timing model
    /// Custom, the timing model is whatever is returned as the Duration from `routine`.
    ///
    /// # Example
    /// ```rust
    /// #[macro_use] extern crate criterion;
    /// use criterion::*;
    /// use criterion::black_box;
    /// use std::time::Instant;
    ///
    /// fn foo() {
    ///     // ...
    /// }
    ///
    /// fn bench(c: &mut Criterion) {
    ///     c.bench_function("iter", move |b| {
    ///         b.iter_custom(|iters| {
    ///             let start = Instant::now();
    ///             for _i in 0..iters {
    ///                 black_box(foo());
    ///             }
    ///             start.elapsed()
    ///         })
    ///     });
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    ///
    #[inline(never)]
    pub fn iter_custom<R>(&mut self, mut routine: R)
    where
        R: FnMut(u64) -> M::Value,
    {
        self.iterated = true;
        self.value = routine(self.iters);
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
        self.value = self.measurement.zero();

        if batch_size == 1 {
            for _ in 0..self.iters {
                let input = black_box(setup());

                let start = self.measurement.start();
                let output = routine(input);
                let end = self.measurement.end(start);
                self.value = self.measurement.add(&self.value, &end);

                drop(black_box(output));
            }
        } else {
            let mut iteration_counter = 0;

            while iteration_counter < self.iters {
                let batch_size = ::std::cmp::min(batch_size, self.iters - iteration_counter);

                let inputs = black_box((0..batch_size).map(|_| setup()).collect::<Vec<_>>());
                let mut outputs = Vec::with_capacity(batch_size as usize);

                let start = self.measurement.start();
                outputs.extend(inputs.into_iter().map(&mut routine));
                let end = self.measurement.end(start);
                self.value = self.measurement.add(&self.value, &end);

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
        self.value = self.measurement.zero();

        if batch_size == 1 {
            for _ in 0..self.iters {
                let mut input = black_box(setup());

                let start = self.measurement.start();
                let output = routine(&mut input);
                let end = self.measurement.end(start);
                self.value = self.measurement.add(&self.value, &end);

                drop(black_box(output));
                drop(black_box(input));
            }
        } else {
            let mut iteration_counter = 0;

            while iteration_counter < self.iters {
                let batch_size = ::std::cmp::min(batch_size, self.iters - iteration_counter);

                let mut inputs = black_box((0..batch_size).map(|_| setup()).collect::<Vec<_>>());
                let mut outputs = Vec::with_capacity(batch_size as usize);

                let start = self.measurement.start();
                outputs.extend(inputs.iter_mut().map(&mut routine));
                let end = self.measurement.end(start);
                self.value = self.measurement.add(&self.value, &end);

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
pub struct Criterion<M: Measurement = WallTime> {
    config: BenchmarkConfig,
    plotting: Plotting,
    filter: Option<String>,
    report: Box<dyn Report>,
    output_directory: String,
    baseline_directory: String,
    baseline: Baseline,
    profile_time: Option<Duration>,
    test_mode: bool,
    list_mode: bool,
    all_directories: HashSet<String>,
    all_titles: HashSet<String>,
    measurement: M,
    profiler: Box<RefCell<dyn Profiler>>,
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

        let mut reports: Vec<Box<dyn Report>> = vec![];
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
            all_directories: HashSet::new(),
            all_titles: HashSet::new(),
            measurement: WallTime,
            profiler: Box::new(RefCell::new(ExternalProfiler)),
        }
    }
}

impl<M: Measurement> Criterion<M> {
    /// Changes the measurement for the benchmarks run with this runner. See the
    /// Measurement trait for more details
    pub fn with_measurement<M2: Measurement>(self, m: M2) -> Criterion<M2> {
        Criterion {
            config: self.config,
            plotting: self.plotting,
            filter: self.filter,
            report: self.report,
            baseline_directory: self.baseline_directory,
            baseline: self.baseline,
            profile_time: self.profile_time,
            test_mode: self.test_mode,
            list_mode: self.list_mode,
            output_directory: self.output_directory,
            all_directories: self.all_directories,
            all_titles: self.all_titles,
            measurement: m,
            profiler: self.profiler,
        }
    }

    /// Changes the internal profiler for benchmarks run with this runner. See
    /// the Profiler trait for more details.
    pub fn with_profiler<P: Profiler + 'static>(self, p: P) -> Criterion<M> {
        Criterion {
            profiler: Box::new(RefCell::new(p)),
            ..self
        }
    }

    /// Changes the default size of the sample for benchmarks run with this runner.
    ///
    /// A bigger sample should yield more accurate results if paired with a sufficiently large
    /// measurement time.
    ///
    /// Sample size must be at least 10.
    ///
    /// # Panics
    ///
    /// Panics if n < 10
    pub fn sample_size(mut self, n: usize) -> Criterion<M> {
        assert!(n >= 10);

        self.config.sample_size = n;
        self
    }

    /// Changes the default warm up time for benchmarks run with this runner.
    ///
    /// # Panics
    ///
    /// Panics if the input duration is zero
    pub fn warm_up_time(mut self, dur: Duration) -> Criterion<M> {
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
    pub fn measurement_time(mut self, dur: Duration) -> Criterion<M> {
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
    pub fn nresamples(mut self, n: usize) -> Criterion<M> {
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
    pub fn noise_threshold(mut self, threshold: f64) -> Criterion<M> {
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
    pub fn confidence_level(mut self, cl: f64) -> Criterion<M> {
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
    pub fn significance_level(mut self, sl: f64) -> Criterion<M> {
        assert!(sl > 0.0 && sl < 1.0);

        self.config.significance_level = sl;
        self
    }

    /// Enables plotting
    pub fn with_plots(mut self) -> Criterion<M> {
        use criterion_plot::VersionError;
        self.plotting = match criterion_plot::version() {
            Ok(_) => {
                let mut reports: Vec<Box<dyn Report>> = vec![];
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

    /// Disables plotting
    pub fn without_plots(mut self) -> Criterion<M> {
        self.plotting = Plotting::Disabled;
        self
    }

    /// Return true if generation of the plots is possible.
    pub fn can_plot(&self) -> bool {
        match self.plotting {
            Plotting::NotAvailable => false,
            Plotting::Enabled => true,
            _ => criterion_plot::version().is_ok(),
        }
    }

    /// Names an explicit baseline and enables overwriting the previous results.
    pub fn save_baseline(mut self, baseline: String) -> Criterion<M> {
        self.baseline_directory = baseline;
        self.baseline = Baseline::Save;
        self
    }

    /// Names an explicit baseline and disables overwriting the previous results.
    pub fn retain_baseline(mut self, baseline: String) -> Criterion<M> {
        self.baseline_directory = baseline;
        self.baseline = Baseline::Compare;
        self
    }

    /// Filters the benchmarks. Only benchmarks with names that contain the
    /// given string will be executed.
    pub fn with_filter<S: Into<String>>(mut self, filter: S) -> Criterion<M> {
        self.filter = Some(filter.into());

        self
    }

    /// Set the output directory (currently for testing only)
    #[doc(hidden)]
    pub fn output_directory(mut self, path: &std::path::Path) -> Criterion<M> {
        self.output_directory = path.to_string_lossy().into_owned();

        self
    }

    /// Set the profile time (currently for testing only)
    #[doc(hidden)]
    pub fn profile_time(mut self, profile_time: Option<Duration>) -> Criterion<M> {
        self.profile_time = profile_time;

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
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cognitive_complexity))]
    pub fn configure_from_args(mut self) -> Criterion<M> {
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
            .arg(Arg::with_name("profile-time")
                .long("profile-time")
                .takes_value(true)
                .help("Iterate each benchmark for approximately the given number of seconds, doing no analysis and without storing the results. Useful for running the benchmarks in a profiler."))
            .arg(Arg::with_name("sample-size")
                .long("sample-size")
                .takes_value(true)
                .help("Changes the default size of the sample for this run."))
            .arg(Arg::with_name("warm-up-time")
                .long("warm-up-time")
                .takes_value(true)
                .help("Changes the default warm up time for this run."))
            .arg(Arg::with_name("measurement-time")
                .long("measurement-time")
                .takes_value(true)
                .help("Changes the default measurement time for this run."))
            .arg(Arg::with_name("nresamples")
                .long("nresamples")
                .takes_value(true)
                .help("Changes the default number of resamples for this run."))
            .arg(Arg::with_name("noise-threshold")
                .long("noise-threshold")
                .takes_value(true)
                .help("Changes the default noise threshold for this run."))
            .arg(Arg::with_name("confidence-level")
                .long("confidence-level")
                .takes_value(true)
                .help("Changes the default confidence level for this run."))
            .arg(Arg::with_name("significance-level")
                .long("significance-level")
                .takes_value(true)
                .help("Changes the default significance level for this run."))
            .arg(Arg::with_name("test")
                .hidden(true)
                .long("test")
                .help("Run the benchmarks once, to verify that they execute successfully, but do not measure or report the results."))
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

To test that the benchmarks work, run `cargo test --benches`
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

        let mut reports: Vec<Box<dyn Report>> = vec![];
        reports.push(Box::new(CliReport::new(
            enable_text_overwrite,
            enable_text_coloring,
            verbose,
        )));
        reports.push(Box::new(FileCsvReport));

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

        let bench = matches.is_present("bench");
        let test = matches.is_present("test");
        self.test_mode = match (bench, test) {
            (true, true) => true,   // cargo bench -- --test should run tests
            (true, false) => false, // cargo bench should run benchmarks
            (false, _) => true,     // cargo test --benches should run tests
        };

        if matches.is_present("sample-size") {
            let num_size = value_t!(matches.value_of("sample-size"), usize).unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1)
            });

            assert!(num_size >= 10);
            self.config.sample_size = num_size;
        }
        if matches.is_present("warm-up-time") {
            let num_seconds = value_t!(matches.value_of("warm-up-time"), u64).unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1)
            });

            let dur = std::time::Duration::new(num_seconds, 0);
            assert!(dur.to_nanos() > 0);

            self.config.warm_up_time = dur;
        }
        if matches.is_present("measurement-time") {
            let num_seconds =
                value_t!(matches.value_of("measurement-time"), u64).unwrap_or_else(|e| {
                    println!("{}", e);
                    std::process::exit(1)
                });

            let dur = std::time::Duration::new(num_seconds, 0);
            assert!(dur.to_nanos() > 0);

            self.config.measurement_time = dur;
        }
        if matches.is_present("nresamples") {
            let num_resamples =
                value_t!(matches.value_of("nresamples"), usize).unwrap_or_else(|e| {
                    println!("{}", e);
                    std::process::exit(1)
                });

            assert!(num_resamples > 0);

            self.config.nresamples = num_resamples;
        }
        if matches.is_present("noise-threshold") {
            let num_noise_threshold = value_t!(matches.value_of("noise-threshold"), f64)
                .unwrap_or_else(|e| {
                    println!("{}", e);
                    std::process::exit(1)
                });

            assert!(num_noise_threshold > 0.0);

            self.config.noise_threshold = num_noise_threshold;
        }
        if matches.is_present("confidence-level") {
            let num_confidence_level = value_t!(matches.value_of("confidence-level"), f64)
                .unwrap_or_else(|e| {
                    println!("{}", e);
                    std::process::exit(1)
                });

            assert!(num_confidence_level > 0.0 && num_confidence_level < 1.0);

            self.config.confidence_level = num_confidence_level;
        }
        if matches.is_present("significance-level") {
            let num_significance_level = value_t!(matches.value_of("significance-level"), f64)
                .unwrap_or_else(|e| {
                    println!("{}", e);
                    std::process::exit(1)
                });

            assert!(num_significance_level > 0.0 && num_significance_level < 1.0);

            self.config.significance_level = num_significance_level;
        }

        if matches.is_present("list") {
            self.test_mode = true;
            self.list_mode = true;
        }

        if self.profile_time.is_none() {
            reports.push(Box::new(Html::new()));
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

    /// Return a benchmark group. All benchmarks performed using a benchmark group will be
    /// grouped together in the final report.
    ///
    /// # Examples:
    ///
    /// ```rust
    /// #[macro_use] extern crate criterion;
    /// use self::criterion::*;
    ///
    /// fn bench_simple(c: &mut Criterion) {
    ///     let mut group = c.benchmark_group("My Group");
    ///
    ///     // Now we can perform benchmarks with this group
    ///     group.bench_function("Bench 1", |b| b.iter(|| 1 ));
    ///     group.bench_function("Bench 2", |b| b.iter(|| 2 ));
    ///    
    ///     group.finish();
    /// }
    /// criterion_group!(benches, bench_simple);
    /// criterion_main!(benches);
    /// ```
    pub fn benchmark_group<S: Into<String>>(&mut self, group_name: S) -> BenchmarkGroup<M> {
        BenchmarkGroup::new(self, group_name.into())
    }
}
impl<M> Criterion<M>
where
    M: Measurement + 'static,
{
    /// Benchmarks a function. For comparing multiple functions, see `benchmark_group`.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate criterion;
    /// use self::criterion::*;
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
    pub fn bench_function<F>(&mut self, id: &str, f: F) -> &mut Criterion<M>
    where
        F: FnMut(&mut Bencher<M>),
    {
        self.benchmark_group(id)
            .bench_function(BenchmarkId::no_function(), f);
        self
    }

    /// Benchmarks a function with an input. For comparing multiple functions or multiple inputs,
    /// see `benchmark_group`.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate criterion;
    /// use self::criterion::*;
    ///
    /// fn bench(c: &mut Criterion) {
    ///     // Setup (construct data, allocate memory, etc)
    ///     let input = 5u64;
    ///     c.bench_with_input(
    ///         BenchmarkId::new("function_name", input), &input,
    ///         |b, i| b.iter(|| {
    ///             // Code to benchmark using input `i` goes here
    ///         }),
    ///     );
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    pub fn bench_with_input<F, I>(&mut self, id: BenchmarkId, input: &I, f: F) -> &mut Criterion<M>
    where
        F: FnMut(&mut Bencher<M>, &I),
    {
        // Guaranteed safe because external callers can't create benchmark IDs without a function
        // name or parameter
        let group_name = id.function_name.unwrap();
        let parameter = id.parameter.unwrap();
        self.benchmark_group(group_name).bench_with_input(
            BenchmarkId::no_function_with_input(parameter),
            input,
            f,
        );
        self
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
    #[doc(hidden)] // Soft-deprecated, use benchmark groups instead
    pub fn bench_function_over_inputs<I, F>(
        &mut self,
        id: &str,
        f: F,
        inputs: I,
    ) -> &mut Criterion<M>
    where
        I: IntoIterator,
        I::Item: fmt::Debug + 'static,
        F: FnMut(&mut Bencher<M>, &I::Item) + 'static,
    {
        self.bench(id, ParameterizedBenchmark::new(id, f, inputs))
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
    #[doc(hidden)] // Soft-deprecated, use benchmark groups instead
    pub fn bench_functions<I>(
        &mut self,
        id: &str,
        funs: Vec<Fun<I, M>>,
        input: I,
    ) -> &mut Criterion<M>
    where
        I: fmt::Debug + 'static,
    {
        let benchmark = ParameterizedBenchmark::with_functions(
            funs.into_iter().map(|fun| fun.f).collect(),
            vec![input],
        );

        self.bench(id, benchmark)
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
    #[doc(hidden)] // Soft-deprecated, use benchmark groups instead
    pub fn bench<B: BenchmarkDefinition<M>>(
        &mut self,
        group_id: &str,
        benchmark: B,
    ) -> &mut Criterion<M> {
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Throughput {
    /// Measure throughput in terms of bytes/second. The value should be the number of bytes
    /// processed by one iteration of the benchmarked code. Typically, this would be the length of
    /// an input string or `&[u8]`.
    Bytes(u64),

    /// Measure throughput in terms of elements/second. The value should be the number of elements
    /// processed by one iteration of the benchmarked code. Typically, this would be the size of a
    /// collection, but could also be the number of lines of input text or the number of values to
    /// parse.
    Elements(u64),
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
/// // Using Criterion::default() for simplicity; normally you'd use the macros.
/// let mut criterion = Criterion::default();
/// let mut benchmark_group = criterion.benchmark_group("Group name");
/// benchmark_group.plot_config(plot_config);
/// // Use benchmark group
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

/// Custom-test-framework runner. Should not be called directly.
#[doc(hidden)]
pub fn runner(benches: &[&dyn Fn()]) {
    for bench in benches {
        bench();
    }
    Criterion::default().configure_from_args().final_summary();
}
