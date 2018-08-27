use analysis;
use program::CommandFactory;
use report::{BenchmarkId, ReportContext};
use routine::{Function, Routine};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::Sized;
use std::process::Command;
use std::time::Duration;
use {Bencher, Criterion, DurationExt, PlotConfiguration, Throughput};

/// Struct containing all of the configuration options for a benchmark.
pub struct BenchmarkConfig {
    pub confidence_level: f64,
    pub measurement_time: Duration,
    pub noise_threshold: f64,
    pub nresamples: usize,
    pub sample_size: usize,
    pub significance_level: f64,
    pub warm_up_time: Duration,
}

/// Struct representing a partially-complete per-benchmark configuration.
struct PartialBenchmarkConfig {
    confidence_level: Option<f64>,
    measurement_time: Option<Duration>,
    noise_threshold: Option<f64>,
    nresamples: Option<usize>,
    sample_size: Option<usize>,
    significance_level: Option<f64>,
    warm_up_time: Option<Duration>,
    plot_config: PlotConfiguration,
}

impl Default for PartialBenchmarkConfig {
    fn default() -> Self {
        PartialBenchmarkConfig {
            confidence_level: None,
            measurement_time: None,
            noise_threshold: None,
            nresamples: None,
            sample_size: None,
            significance_level: None,
            warm_up_time: None,
            plot_config: PlotConfiguration::default(),
        }
    }
}

impl PartialBenchmarkConfig {
    fn to_complete(&self, defaults: &BenchmarkConfig) -> BenchmarkConfig {
        BenchmarkConfig {
            confidence_level: self.confidence_level.unwrap_or(defaults.confidence_level),
            measurement_time: self.measurement_time.unwrap_or(defaults.measurement_time),
            noise_threshold: self.noise_threshold.unwrap_or(defaults.noise_threshold),
            nresamples: self.nresamples.unwrap_or(defaults.nresamples),
            sample_size: self.sample_size.unwrap_or(defaults.sample_size),
            significance_level: self.significance_level
                .unwrap_or(defaults.significance_level),
            warm_up_time: self.warm_up_time.unwrap_or(defaults.warm_up_time),
        }
    }
}

pub struct NamedRoutine<T> {
    pub id: String,
    pub f: Box<RefCell<Routine<T>>>,
}

/// Structure representing a benchmark (or group of benchmarks)
/// which take one parameter.
pub struct ParameterizedBenchmark<T: Debug> {
    config: PartialBenchmarkConfig,
    values: Vec<T>,
    routines: Vec<NamedRoutine<T>>,
    throughput: Option<Box<Fn(&T) -> Throughput>>,
}

/// Structure representing a benchmark (or group of benchmarks)
/// which takes no parameters.
pub struct Benchmark {
    config: PartialBenchmarkConfig,
    routines: Vec<NamedRoutine<()>>,
    throughput: Option<Throughput>,
}

/// Common trait for `Benchmark` and `ParameterizedBenchmark`. Not intended to be
/// used outside of Criterion.rs.
pub trait BenchmarkDefinition: Sized {
    #[doc(hidden)]
    fn run(self, group_id: &str, c: &Criterion);
}

macro_rules! benchmark_config {
    ($type:tt) => {

        /// Changes the size of the sample for this benchmark
        ///
        /// A bigger sample should yield more accurate results if paired with a sufficiently large
        /// measurement time.
        /// 
        /// Sample size must be at least 2.
        ///
        /// # Panics
        ///
        /// Panics if set to zero or one.
        pub fn sample_size(mut self, n: usize) -> Self {
            assert!(n >= 2);

            self.config.sample_size = Some(n);
            self
        }

        /// Changes the warm up time for this benchmark
        ///
        /// # Panics
        ///
        /// Panics if the input duration is zero
        pub fn warm_up_time(mut self, dur: Duration) -> Self {
            assert!(dur.to_nanos() > 0);

            self.config.warm_up_time = Some(dur);
            self
        }

        /// Changes the target measurement time for this benchmark. Criterion will attempt
        /// to spent approximately this amount of time measuring the benchmark.
        /// With a longer time, the measurement will become more resilient to transitory peak loads
        /// caused by external programs.
        ///
        /// # Panics
        ///
        /// Panics if the input duration in zero
        pub fn measurement_time(mut self, dur: Duration) -> Self {
            assert!(dur.to_nanos() > 0);

            self.config.measurement_time = Some(dur);
            self
        }

        /// Changes the number of resamples for this benchmark
        ///
        /// Number of resamples to use for the
        /// [bootstrap](http://en.wikipedia.org/wiki/Bootstrapping_(statistics)#Case_resampling)
        ///
        /// A larger number of resamples reduces the random sampling errors, which are inherent to the
        /// bootstrap method, but also increases the analysis time.
        ///
        /// # Panics
        ///
        /// Panics if the number of resamples is set to zero
        pub fn nresamples(mut self, n: usize) -> Self {
            assert!(n > 0);

            self.config.nresamples = Some(n);
            self
        }

        /// Changes the noise threshold for this benchmark
        ///
        /// This threshold is used to decide if an increase of `X%` in the execution time is considered
        /// significant or should be flagged as noise
        ///
        /// *Note:* A value of `0.02` is equivalent to `2%`
        ///
        /// # Panics
        ///
        /// Panics is the threshold is set to a negative value
        pub fn noise_threshold(mut self, threshold: f64) -> Self {
            assert!(threshold >= 0.0);

            self.config.noise_threshold = Some(threshold);
            self
        }

        /// Changes the confidence level for this benchmark
        ///
        /// The confidence level is used to calculate the
        /// [confidence intervals](https://en.wikipedia.org/wiki/Confidence_interval) of the estimated
        /// statistics
        ///
        /// # Panics
        ///
        /// Panics if the confidence level is set to a value outside the `(0, 1)` range
        pub fn confidence_level(mut self, cl: f64) -> Self {
            assert!(cl > 0.0 && cl < 1.0);

            self.config.confidence_level = Some(cl);
            self
        }

        /// Changes the [significance level](https://en.wikipedia.org/wiki/Statistical_significance)
        /// for this benchmark
        ///
        /// The significance level is used for
        /// [hypothesis testing](https://en.wikipedia.org/wiki/Statistical_hypothesis_testing)
        ///
        /// # Panics
        ///
        /// Panics if the significance level is set to a value outside the `(0, 1)` range
        pub fn significance_level(mut self, sl: f64) -> Self {
            assert!(sl > 0.0 && sl < 1.0);

            self.config.significance_level = Some(sl);
            self
        }

        /// Changes the plot configuration for this benchmark.
        pub fn plot_config(mut self, new_config: PlotConfiguration) -> Self {
            self.config.plot_config = new_config;
            self
        }

    }
}

impl Benchmark {
    benchmark_config!(Benchmark);

    /// Create a new benchmark group and adds the given function to it.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![macro_use] extern crate criterion;
    /// # use criterion::*;
    ///
    /// fn bench(c: &mut Criterion) {
    ///     // One-time setup goes here
    ///     c.bench(
    ///         "my_group",
    ///         Benchmark::new("my_function", |b| b.iter(|| {
    ///             // Code to benchmark goes here
    ///         })),
    ///     );
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    pub fn new<S, F>(id: S, f: F) -> Benchmark
    where
        S: Into<String>,
        F: FnMut(&mut Bencher) + 'static,
    {
        Benchmark {
            config: PartialBenchmarkConfig::default(),
            routines: vec![],
            throughput: None,
        }.with_function(id, f)
    }

    /// Create a new benchmark group and add the given program to it.
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
    pub fn new_external<S>(id: S, program: Command) -> Benchmark
    where
        S: Into<String>,
    {
        Benchmark {
            config: PartialBenchmarkConfig::default(),
            routines: vec![],
            throughput: None,
        }.with_program(id, program)
    }

    /// Add a function to the benchmark group.
    ///
    /// # Example:
    /// ```
    /// # use criterion::Benchmark;
    /// Benchmark::new("return 10", |b| b.iter(|| 10))
    ///     .with_function("return 20", |b| b.iter(|| 20));
    /// ```
    pub fn with_function<S, F>(mut self, id: S, mut f: F) -> Benchmark
    where
        S: Into<String>,
        F: FnMut(&mut Bencher) + 'static,
    {
        let routine = NamedRoutine {
            id: id.into(),
            f: Box::new(RefCell::new(Function::new(move |b, _| f(b)))),
        };
        self.routines.push(routine);
        self
    }

    /// Add an external program to the benchmark group.
    ///
    /// # Example:
    /// ```
    /// # use criterion::Benchmark;
    /// # use std::process::Command;
    /// Benchmark::new("internal", |b| b.iter(|| 10))
    ///     .with_program("external", Command::new("my_external_benchmark"));
    /// ```
    pub fn with_program<S>(mut self, id: S, program: Command) -> Benchmark
    where
        S: Into<String>,
    {
        let routine = NamedRoutine {
            id: id.into(),
            f: Box::new(RefCell::new(program)),
        };
        self.routines.push(routine);
        self
    }

    /// Set the input size for this benchmark group. Used for reporting the
    /// throughput.
    ///
    /// ```
    /// # use criterion::{Benchmark, Throughput};
    /// # use std::process::Command;
    /// Benchmark::new("strlen", |b| b.iter(|| "foo".len()))
    ///     .throughput(Throughput::Bytes(3));
    /// ```
    pub fn throughput(mut self, throughput: Throughput) -> Benchmark {
        self.throughput = Some(throughput);
        self
    }
}

impl BenchmarkDefinition for Benchmark {
    fn run(self, group_id: &str, c: &Criterion) {
        let report_context = ReportContext {
            output_directory: c.output_directory.clone(),
            plotting: c.plotting,
            plot_config: self.config.plot_config.clone(),
            test_mode: c.test_mode,
        };

        let config = self.config.to_complete(&c.config);
        let num_routines = self.routines.len();

        let mut all_ids = vec![];
        let mut any_matched = false;
        let mut all_directories = HashSet::new();

        for routine in self.routines {
            let function_id = if num_routines == 1 && group_id == routine.id {
                None
            } else {
                Some(routine.id)
            };

            let mut id = BenchmarkId::new(
                group_id.to_owned(),
                function_id,
                None,
                self.throughput.clone(),
            );

            id.ensure_directory_name_unique(&all_directories);
            all_directories.insert(id.as_directory_name().to_owned());

            if c.filter_matches(id.id()) {
                any_matched = true;
                analysis::common(
                    &id,
                    &mut *routine.f.borrow_mut(),
                    &config,
                    c,
                    &report_context,
                    &(),
                    self.throughput.clone(),
                );
            }

            all_ids.push(id);
        }

        if all_ids.len() > 1 && any_matched && !c.measure_only && !c.test_mode {
            c.report.summarize(&report_context, &all_ids);
        }
        if any_matched {
            println!();
        }
    }
}
impl<T> ParameterizedBenchmark<T>
where
    T: Debug + 'static,
{
    benchmark_config!(ParameterizedBenchmark);

    /// Create a new parameterized benchmark group and adds the given function
    /// to it.
    /// The function under test must follow the setup - bench - teardown pattern:
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![macro_use] extern crate criterion;
    /// # use criterion::*;
    ///
    /// fn bench(c: &mut Criterion) {
    ///     let parameters = vec![1u64, 2u64, 3u64];
    ///
    ///     // One-time setup goes here
    ///     c.bench(
    ///         "my_group",
    ///         ParameterizedBenchmark::new(
    ///             "my_function",
    ///             |b, param| b.iter(|| {
    ///                 // Code to benchmark using param goes here
    ///             }),
    ///             parameters
    ///         )
    ///     );
    /// }
    ///
    /// criterion_group!(benches, bench);
    /// criterion_main!(benches);
    /// ```
    pub fn new<S, F, I>(id: S, f: F, parameters: I) -> ParameterizedBenchmark<T>
    where
        S: Into<String>,
        F: FnMut(&mut Bencher, &T) + 'static,
        I: IntoIterator<Item = T>,
    {
        ParameterizedBenchmark {
            config: PartialBenchmarkConfig::default(),
            values: parameters.into_iter().collect(),
            routines: vec![],
            throughput: None,
        }.with_function(id, f)
    }

    /// Create a new parameterized benchmark group and add the given program to it.
    /// The program under test must implement the following protocol:
    ///
    /// * Read the number of iterations from stdin
    /// * Execute the routine to benchmark that many times
    /// * Print the elapsed time (in nanoseconds) to stdout
    ///
    /// You can pass the argument to the program in any way you choose.
    ///
    /// ```rust,no_run
    /// # use std::io::{self, BufRead};
    /// # use std::time::Instant;
    /// # use std::time::Duration;
    /// # trait DurationExt { fn to_nanos(&self) -> u64 { 0 } }
    /// # impl DurationExt for Duration {}
    /// # use std::env;
    /// // Example of an external program that implements this protocol
    ///
    /// fn main() {
    ///     let stdin = io::stdin();
    ///     let ref mut stdin = stdin.lock();
    ///
    ///     // You might opt to pass the parameter to the external command as
    ///     // an environment variable, command line argument, file on disk, etc.
    ///     let parameter = env::var("PARAMETER").unwrap();
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
    ///             // Code to benchmark using the parameter goes here
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
    pub fn new_external<S, F, I>(id: S, program: F, parameters: I) -> ParameterizedBenchmark<T>
    where
        S: Into<String>,
        F: FnMut(&T) -> Command + 'static,
        I: IntoIterator<Item = T>,
    {
        ParameterizedBenchmark {
            config: PartialBenchmarkConfig::default(),
            routines: vec![],
            values: parameters.into_iter().collect(),
            throughput: None,
        }.with_program(id, program)
    }

    pub(crate) fn with_functions(
        functions: Vec<NamedRoutine<T>>,
        parameters: Vec<T>,
    ) -> ParameterizedBenchmark<T> {
        ParameterizedBenchmark {
            config: PartialBenchmarkConfig::default(),
            values: parameters,
            routines: functions,
            throughput: None,
        }
    }

    /// Add a function to the benchmark group.
    ///
    /// # Example
    ///
    /// ```
    /// # use criterion::ParameterizedBenchmark;
    /// ParameterizedBenchmark::new("times 10", |b, i| b.iter(|| i * 10), vec![1, 2, 3])
    ///     .with_function("times 20", |b, i| b.iter(|| i * 20));
    /// ```
    pub fn with_function<S, F>(mut self, id: S, f: F) -> ParameterizedBenchmark<T>
    where
        S: Into<String>,
        F: FnMut(&mut Bencher, &T) + 'static,
    {
        let routine = NamedRoutine {
            id: id.into(),
            f: Box::new(RefCell::new(Function::new(f))),
        };
        self.routines.push(routine);
        self
    }

    /// Add an external program to the benchmark group.
    ///
    /// # Example
    ///
    /// ```
    /// # use criterion::ParameterizedBenchmark;
    /// # use std::process::Command;
    /// ParameterizedBenchmark::new("internal", |b, i| b.iter(|| i * 10), vec![1, 2, 3])
    ///     .with_program("external", |i| {
    ///         let mut command = Command::new("my_external_benchmark");
    ///         command.arg(format!("{:?}", i));
    ///         command
    ///     });
    /// ```
    pub fn with_program<S, F>(mut self, id: S, program: F) -> ParameterizedBenchmark<T>
    where
        S: Into<String>,
        F: FnMut(&T) -> Command + 'static,
    {
        let factory = CommandFactory::new(program);
        let routine = NamedRoutine {
            id: id.into(),
            f: Box::new(RefCell::new(factory)),
        };
        self.routines.push(routine);
        self
    }

    /// Use the given function to calculate the input size for a given input.
    ///
    /// # Example
    ///
    /// ```
    /// # use criterion::{ParameterizedBenchmark, Throughput};
    /// # use std::process::Command;
    /// ParameterizedBenchmark::new("strlen", |b, s| b.iter(|| s.len()), vec!["foo", "lorem ipsum"])
    ///     .throughput(|s| Throughput::Bytes(s.len() as u32));
    /// ```
    pub fn throughput<F>(mut self, throughput: F) -> ParameterizedBenchmark<T>
    where
        F: Fn(&T) -> Throughput + 'static,
    {
        self.throughput = Some(Box::new(throughput));
        self
    }
}
impl<T> BenchmarkDefinition for ParameterizedBenchmark<T>
where
    T: Debug + 'static,
{
    fn run(self, group_id: &str, c: &Criterion) {
        let report_context = ReportContext {
            output_directory: c.output_directory.clone(),
            plotting: c.plotting,
            plot_config: self.config.plot_config.clone(),
            test_mode: c.test_mode,
        };

        let config = self.config.to_complete(&c.config);
        let num_parameters = self.values.len();
        let num_routines = self.routines.len();

        let mut all_ids = vec![];
        let mut any_matched = false;
        let mut all_directories = HashSet::new();

        for routine in self.routines {
            for value in &self.values {
                let function_id = if num_routines == 1 && group_id == routine.id {
                    None
                } else {
                    Some(routine.id.clone())
                };

                let value_str = if num_parameters == 1 {
                    None
                } else {
                    Some(format!("{:?}", value))
                };

                let throughput = self.throughput.as_ref().map(|func| func(value));
                let mut id = BenchmarkId::new(
                    group_id.to_owned(),
                    function_id,
                    value_str,
                    throughput.clone(),
                );

                id.ensure_directory_name_unique(&all_directories);
                all_directories.insert(id.as_directory_name().to_owned());

                if c.filter_matches(id.id()) {
                    any_matched = true;

                    analysis::common(
                        &id,
                        &mut *routine.f.borrow_mut(),
                        &config,
                        c,
                        &report_context,
                        value,
                        throughput,
                    );
                }

                all_ids.push(id);
            }
        }

        if all_ids.len() > 1 && any_matched && !c.measure_only && !c.test_mode {
            c.report.summarize(&report_context, &all_ids);
        }
        if any_matched {
            println!();
        }
    }
}
