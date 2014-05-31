use bencher::Bencher;
use clock::Clock;
use metrics::Metrics;
use sample::Sample;
use std::default::Default;
use std::fmt::Show;

pub struct Criterion {
    clock: Option<Clock>,
    config: CriterionConfig,
    metrics: Metrics,
}

impl Criterion {
    pub fn new() -> Criterion {
        Criterion {
            clock: None,
            config: Default::default(),
            metrics: Metrics::new(),
        }
    }

    pub fn set_config(&mut self, config: CriterionConfig) {
        self.config = config;
    }

    pub fn bench<N: Str + ToStr>(&mut self, name: N, f: |&mut Bencher|) {
        if self.clock.is_none() {
            self.clock = Some(Clock::new(&self.config));
        }

        println!("\nbenchmarking {}", name.as_slice());

        let sample = Sample::new(f, &self.config, self.clock);

        sample.outliers().report();

        let sample = sample.without_outliers();

        sample.estimate(&self.config);

        self.metrics.update(&name.to_str(), sample.into_data(), &self.config);
    }

    pub fn bench_group<G: Show, I: Clone + Show>(&mut self,
                                                 group: G,
                                                 inputs: &[I],
                                                 f: |&mut Bencher, I|) {
        for input in inputs.iter() {
            self.bench(format!("{}_{}", group, input), |x| {
                f(x, input.clone())
            });
        }
    }
}

pub struct CriterionConfig {
    pub confidence_level: f64,
    pub measurement_time: uint,
    pub nresamples: uint,
    pub sample_size: u64,
    pub significance_level: f64,
    pub warm_up_time: uint,
}

impl Default for CriterionConfig {
    fn default() -> CriterionConfig {
        CriterionConfig {
            confidence_level: 0.95,
            measurement_time: 10,
            nresamples: 100_000,
            sample_size: 100,
            significance_level: 0.05,
            warm_up_time: 500,
        }
    }
}
