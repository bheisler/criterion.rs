use std::time::Duration;
use ::{DurationExt, Criterion, Bencher};
use routine::{Routine, Function};
use ::analysis;
use std::cell::RefCell;

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
            warm_up_time: None
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
            significance_level: self.significance_level.unwrap_or(defaults.significance_level),
            warm_up_time: self.warm_up_time.unwrap_or(defaults.warm_up_time),
        }
    }
}

/*pub struct ParameterizedBenchmark<T> {
    id: String,
    config: PartialBenchmarkConfig,
    values: Vec<T>,
}*/

/// Structure representing a benchmark (or group of benchmarks)
/// which takes no parameters.
pub struct Benchmark {
    id: String,
    config: PartialBenchmarkConfig,
    f: Box<RefCell<Routine>>,
}


macro_rules! benchmark_config {
    ($type:tt) => {

        /// Changes the size of the sample for this benchmark
        ///
        /// A bigger sample should yield more accurate results, if paired with a "sufficiently" large
        /// measurement time, on the other hand, it also increases the analysis time
        ///
        /// # Panics
        ///
        /// Panics if set to zero
        pub fn sample_size(mut self, n: usize) -> Self {
            assert!(n > 0);

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

        /// Changes the measurement time for this benchmark
        ///
        /// With a longer time, the measurement will become more resilient to transitory peak loads
        /// caused by external programs
        ///
        /// **Note**: If the measurement time is too "low", Criterion will automatically increase it
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
        /// bootstrap method, but also increases the analysis time
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
    }
}

impl Benchmark {
    benchmark_config!(Benchmark);

    /// Create a new Benchmark object given a benchmark ID and a function.
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
    /// Criterion::default()
    ///     .bench(Benchmark::new("routine", routine));
    /// ```
    pub fn new<S, F>(id: S, f: F) -> Benchmark
        where S: Into<String>, F: FnMut(&mut Bencher) + 'static {
        Benchmark {
            id: id.into(),
            config: Default::default(),
            f: Box::new(RefCell::new(Function(f))),
        }
    }

    pub(crate) fn run(&self, c: &Criterion) {
        let config = self.config.to_complete(&c.config);

        if c.filter_matches(&self.id) {
            analysis::common(&self.id, &mut *self.f.borrow_mut(), &config, c);

            println!();
        }
    }
}
/*impl<T> ParameterizedBenchmark<T> {
    benchmark_config!(ParameterizedBenchmark);
}*/
