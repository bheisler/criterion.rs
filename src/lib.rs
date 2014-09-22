#![feature(macro_rules, phase, tuple_indexing)]

//! A statistics-driven micro-benchmarking library written in Rust.
//!
//! # Features
//!
//! - Can benchmark native (Rust) programs and also foreign (C, Python, Go, etc) programs
//! - Easily benchmark a program under several inputs
//! - Easy migration from `std::test::Bencher` to `criterion::Bencher`
//! - Plots!

#[phase(plugin, link)]
extern crate log;
extern crate serialize;
extern crate simplot;
extern crate stats;
extern crate test;
extern crate time;

use std::fmt::Show;
use std::io::Command;
use std::time::Duration;

mod analysis;
mod estimate;
mod format;
mod fs;
mod kde;
mod plot;
mod program;
mod report;
mod routine;

/// Helper struct to build functions that follow the setup - bench - teardown pattern
#[experimental]
pub struct Bencher {
    iters: u64,
    ns_end: u64,
    ns_start: u64,
}

#[experimental]
impl Bencher {
    /// Callback to benchmark a routine
    #[experimental]
    pub fn iter<T>(&mut self, routine: || -> T) {
        self.ns_start = time::precise_time_ns();
        for _ in range(0, self.iters) {
            test::black_box(routine());
        }
        self.ns_end = time::precise_time_ns();
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
/// benchmark. If a significant regression in performance is spotted, `Criterion` will trigger a
/// task failure
#[experimental]
pub struct Criterion {
    confidence_level: f64,
    measurement_time_ns: u64,
    noise_threshold: f64,
    nresamples: uint,
    plotting: Plotting,
    sample_size: uint,
    significance_level: f64,
    warm_up_time_ns: u64,
}

enum Plotting {
    Disabled,
    Enabled,
    NotAvailable,
}

impl Plotting {
    fn is_enabled(&self) -> bool {
        match *self {
            Enabled => true,
            _ => false,
        }
    }
}

#[experimental]
impl Criterion {
    /// Creates a benchmark manager with the following default settings:
    ///
    /// - Sample size: 100 measurements
    /// - Warm-up time: 1 s
    /// - Measurement time: 1 s
    /// - Bootstrap size: 100 000 resamples
    /// - Noise threshold: 0.01 (1%)
    /// - Confidence level: 0.95
    /// - Significance level: 0.05
    /// - Plotting: enabled (if gnuplot is available)
    // TODO (UFCS) remove this method and implement the `Default` trait
    #[experimental]
    pub fn default() -> Criterion {
        let plotting = if simplot::version().is_ok() {
            Enabled
        } else {
            println!("Gnuplot not found, disabling plotting");

            NotAvailable
        };

        Criterion {
            confidence_level: 0.95,
            measurement_time_ns: 5_000_000_000,
            noise_threshold: 0.01,
            nresamples: 100_000,
            sample_size: 100,
            plotting: plotting,
            significance_level: 0.05,
            warm_up_time_ns: 3_000_000_000,
        }
    }

    /// Changes the size of the sample
    ///
    /// A bigger sample should yield more accurate results, if paired with a "sufficiently" large
    /// measurement time, on the other hand, it also increases the analysis time
    ///
    /// # Failure
    ///
    /// Fails if set to zero
    #[experimental]
    pub fn sample_size(&mut self, n: uint) -> &mut Criterion {
        assert!(n > 0);

        self.sample_size = n;
        self
    }

    /// Changes the warm up time
    ///
    /// # Failure
    ///
    /// Fails if the warm up time is set to a non-positive value
    #[experimental]
    pub fn warm_up_time(&mut self, dur: Duration) -> &mut Criterion {
        let ns = dur.num_nanoseconds().expect("duration overflow");

        assert!(ns > 0);

        self.warm_up_time_ns = ns as u64;
        self
    }

    /// Changes the measurement time
    ///
    /// With a longer time, the measurement will become more resilient to transitory peak loads
    /// caused by external programs
    ///
    /// **Note**: If the measurement time is too "low", Criterion will automatically increase it
    ///
    /// # Failure
    ///
    /// Fails if the measurement time is set to a non-positive value
    #[experimental]
    pub fn measurement_time(&mut self, dur: Duration) -> &mut Criterion {
        let ns = dur.num_nanoseconds().expect("duration overflow");

        assert!(ns > 0);

        self.measurement_time_ns = ns as u64;
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
    /// # Failure
    ///
    /// Fails if the number of resamples is set to zero
    #[experimental]
    pub fn nresamples(&mut self, n: uint) -> &mut Criterion {
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
    /// # Failure
    ///
    /// Fails is the threshold is set to a negative value
    #[experimental]
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
    /// # Failure
    ///
    /// Fails if the confidence level is set to a value outside the `(0, 1)` range
    #[experimental]
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
    /// # Failure
    ///
    /// Fails if the significance level is set to a value outside the `(0, 1)` range
    #[experimental]
    pub fn significance_level(&mut self, sl: f64) -> &mut Criterion {
        assert!(sl > 0.0 && sl < 1.0);

        self.significance_level = sl;
        self
    }

    /// Enables plotting
    #[experimental]
    pub fn with_plots(&mut self) -> &mut Criterion {
        match self.plotting {
            NotAvailable => {},
            _ => self.plotting = Enabled,
        }

        self
    }

    /// Disabled plotting
    #[experimental]
    pub fn without_plots(&mut self) -> &mut Criterion {
        match self.plotting {
            NotAvailable => {},
            _ => self.plotting = Disabled,
        }

        self
    }

    /// Checks if plotting is possible
    pub fn can_plot(&self) -> bool {
        match self.plotting {
            NotAvailable => false,
            _ => true,
        }
    }

    /// Benchmarks a function
    ///
    /// The function under test must follow the setup - bench - teardown pattern:
    ///
    /// ``` ignore-test
    /// use criterion::{Bencher, Criterion};
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
    /// Criterion::default().bench("routine", routine);
    /// ```
    #[experimental]
    pub fn bench(&mut self, id: &str, f: |&mut Bencher|:'static) -> &mut Criterion {
        analysis::function(id, f, self);

        self
    }

    /// Benchmarks a function under various inputs
    ///
    /// This is a convenience method to execute several related benchmarks. Each benchmark will
    /// receive the id: `${id}/${input}`.
    ///
    /// ``` ignore-test
    /// use criterion::{Bencher, Criterion};
    ///
    /// Criterion::default().bench_with_inputs("from_elem", |b: &mut Bencher, &size: &uint| {
    ///     b.iter(|| Vec::from_elem(size, 0u8));
    /// }, [1024, 2048, 4096]);
    /// ```
    #[experimental]
    pub fn bench_with_inputs<I: Show>(
        &mut self,
        id: &str,
        f: |&mut Bencher, &I|:'static,
        inputs: &[I],
    ) -> &mut Criterion {
        analysis::function_with_inputs(id, f, inputs, self);

        self
    }

    /// Benchmarks an external program
    ///
    /// The external program must conform to the following specification:
    ///
    /// ``` ignore-test
    /// extern crate time;
    ///
    /// use std::io::stdio;
    ///
    /// fn main() {
    ///     // For each line in stdin
    ///     for line in stdio::stdin().lines() {
    ///         // Parse line as the number of iterations
    ///         let iters: u64 = from_str(line.unwrap().as_slice().trim()).unwrap();
    ///
    ///         // Setup
    ///
    ///         // Benchmark
    ///         let ns_start = time::precise_time_ns();
    ///         // Execute the routine "iters" times
    ///         for _ in range(0, iters) {
    ///             // Code to benchmark goes here
    ///         }
    ///         let ns_end = time::precise_time_ns();
    ///
    ///         // Teardown
    ///
    ///         // Report elapsed time in nanoseconds to stdout
    ///         println!("{}", ns_end - ns_start);
    ///     }
    /// }
    /// ```
    #[experimental]
    pub fn bench_program(&mut self, id: &str, program: &Command) -> &mut Criterion {
        analysis::program(id, program, self);

        self
    }

    /// Benchmarks an external program under various inputs
    ///
    /// This is a convenience method to execute several related benchmarks. Each benchmark will
    /// receive the id: `${id}/${input}`.
    #[experimental]
    pub fn bench_program_with_inputs<I: Show>(
        &mut self,
        id: &str,
        program: &Command,
        inputs: &[I],
    ) -> &mut Criterion {
        analysis::program_with_inputs(id, program, inputs, self);

        self
    }

    /// Summarize the results stored under the `.criterion/${id}` folder
    ///
    /// *Note:* The `bench_with_inputs` and `bench_program_with_inputs` functions internally call
    /// the `summarize` method
    #[experimental]
    pub fn summarize(&mut self, id: &str) -> &mut Criterion {
        analysis::summarize(id, self);

        self
    }
}
