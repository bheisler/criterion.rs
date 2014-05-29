use clock::Clock;
use common::run_for_at_least;
use sample::Sample;
use std::default::Default;
use std::fmt::Show;
use units::{AsTime,ToNanoSeconds};

pub struct Bencher {
    clock: Option<Clock>,
    config: BencherConfig,
}

impl Bencher {
    pub fn new() -> Bencher {
        Bencher {
            clock: None,
            config: Default::default(),
        }
    }

    pub fn set_config(&mut self, config: BencherConfig) {
        self.config = config;
    }

    pub fn bench<T, N: Show>(&mut self, name: N, action: || -> T) {
        if self.clock.is_none() {
            self.clock = Some(Clock::new(self.config.dump_clock));
        }

        let cl = self.config.confidence_level;
        let m_time = self.config.measurement_time as u64 * 1.ms();
        let nresamples = self.config.nresamples;
        let size = self.config.sample_size;
        let wu_time = self.config.warm_up_time as u64 * 1.ms();

        println!("\nbenchmarking {}", name);

        println!("> warming up...");
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

        if self.config.dump_sample {
            sample.dump(&Path::new(format!("{}.json", name)));
        }

        sample.without_outliers().bootstrap(nresamples, cl).report();
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
    pub dump_clock: bool,
    pub dump_sample: bool,
    pub measurement_time: uint,
    pub nresamples: uint,
    pub sample_size: uint,
    pub warm_up_time: uint,
}

impl Default for BencherConfig {
    fn default() -> BencherConfig {
        BencherConfig {
            confidence_level: 0.95,
            dump_clock: false,
            dump_sample: false,
            measurement_time: 10,
            nresamples: 100_000,
            sample_size: 100,
            warm_up_time: 500,
        }
    }
}
