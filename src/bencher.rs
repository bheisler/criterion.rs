use clock::Clock;
use common::run_for_at_least;
use metrics::Metrics;
use sample::Sample;
use std::default::Default;
use std::fmt::Show;
use units::{AsTime,ToNanoSeconds};

pub struct Bencher {
    clock: Option<Clock>,
    config: BencherConfig,
    metrics: Metrics,
}

impl Bencher {
    pub fn new() -> Bencher {
        Bencher {
            clock: None,
            config: Default::default(),
            metrics: Metrics::new(),
        }
    }

    pub fn set_config(&mut self, config: BencherConfig) {
        self.config = config;
    }

    pub fn bench<T, N: Str + ToStr>(&mut self, name: N, action: || -> T) {
        if self.clock.is_none() {
            self.clock = Some(Clock::new());
        }

        let m_time = self.config.measurement_time as u64 * 1.ms();
        let size = self.config.sample_size;
        let wu_time = self.config.warm_up_time as u64 * 1.ms();

        println!("\nbenchmarking {}", name.as_slice());

        println!("> warming up for {} ms", self.config.warm_up_time);
        let (wu_ns, wu_iters, action) = run_for_at_least(wu_time, 1, action);

        let m_iters = (wu_iters as u64 * m_time / wu_time) as uint;
        let m_ns = wu_ns * m_time / wu_time * size as u64;

        println!("> collecting {} measurements, {} iters each in estimated {}",
                 size,
                 m_iters,
                 (m_ns as f64).as_time());

        let (sample, _) = Sample::new(size,
                                      action,
                                      m_iters,
                                      self.clock);

        sample.outliers().report();

        sample.without_outliers().estimate(&self.config);

        self.metrics.update(&name.to_str(), sample.into_data(), &self.config);
    }

    pub fn bench_group<G: Show, I: Clone + Show, O>(&mut self,
                                                    group: G,
                                                    inputs: &[I],
                                                    action: |I| -> O) {
        for input in inputs.iter() {
            self.bench(format!("{}_{}", group, input), || {
                action(input.clone())
            });
        }
    }
}

pub struct BencherConfig {
    pub confidence_level: f64,
    pub measurement_time: uint,
    pub nresamples: uint,
    pub sample_size: uint,
    pub significance_level: f64,
    pub warm_up_time: uint,
}

impl Default for BencherConfig {
    fn default() -> BencherConfig {
        BencherConfig {
            confidence_level: 0.95,
            measurement_time: 10,
            nresamples: 100_000,
            sample_size: 100,
            significance_level: 0.05,
            warm_up_time: 500,
        }
    }
}
