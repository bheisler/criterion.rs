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

        let min_time = 10.ms();
        let size = self.config.sample_size;
        let nresamples = self.config.nresamples;
        let cl = self.config.confidence_level;

        println!("\nbenchmarking {}", name);
        let (elapsed, iters, action) = run_for_at_least(min_time, 1, action);

        println!("> collecting {} measurements, {} iters each in estimated {}",
                 size,
                 iters,
                 (elapsed as f64 * size as f64).as_time());

        let (sample, _) = Sample::new(size,
                                      action,
                                      iters,
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
    pub nresamples: uint,
    pub sample_size: uint,
}

impl Default for BencherConfig {
    fn default() -> BencherConfig {
        BencherConfig {
            confidence_level: 0.95,
            dump_clock: false,
            dump_sample: false,
            nresamples: 100_000,
            sample_size: 100,
        }
    }
}
