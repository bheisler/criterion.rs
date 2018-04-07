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

mod routine;
mod macros;

use std::default::Default;
use std::time::{Duration, Instant};
use routine::Routine;

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
    pub fn iter<O, R>(&mut self, mut routine: R)
    where
        R: FnMut() -> O,
    {
        let start = Instant::now();
        for _ in 0..self.iters {
            black_box(routine());
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
    measurement_time: Duration,
    sample_size: usize,
    warm_up_time: Duration,
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
        Criterion {
            measurement_time: Duration::new(5, 0),
            sample_size: 100,
            warm_up_time: Duration::new(3, 0),
        }
    }
}

impl Criterion {
    /// Changes the default warm up time for benchmarks run with this runner.
    ///
    /// # Panics
    ///
    /// Panics if the input duration is zero
    pub fn warm_up_time(mut self, dur: Duration) -> Criterion {
        self.warm_up_time = dur;
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
        self.measurement_time = dur;
        self
    }
    
    /// Configure this criterion struct based on the command-line arguments to
    /// this process.
    pub fn configure_from_args(self) -> Criterion {
        self
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
    pub fn bench_function<F>(&mut self, id: &str, f: F) -> &mut Criterion
    where
        F: FnMut(&mut Bencher) + 'static,
    {

        let mut routine = routine::Function::new(f);
        sample(&mut routine, &id, &self);
        self
    }
}

fn sample<F>(
    routine: &mut routine::Function<F>,
    id: &str,
    criterion: &Criterion,
) where F: FnMut(&mut Bencher) + 'static {
    let wu = criterion.warm_up_time;
    let m_ns = criterion.measurement_time.to_nanos();

    let (wu_elapsed, wu_iters) = routine.warm_up(wu);

    // Initial guess for the mean execution time
    let met = wu_elapsed as f64 / wu_iters as f64;

    let n = criterion.sample_size as u64;
    // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns
    let total_runs = n * (n + 1) / 2;
    let d = (m_ns as f64 / met / total_runs as f64).ceil() as u64;

    let m_iters = (1..(n + 1) as u64).map(|a| a * d).collect::<Vec<u64>>();

    let iter_count : u64 = m_iters.iter().sum();
    println!("{}: Calculated iterations: {}",
        id,
        iter_count
    );
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

