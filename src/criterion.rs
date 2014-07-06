use std::fmt::Show;
use std::io::{fs,UserRWX};

use bencher::Bencher;
use bootstrap;
use clock::Clock;
use sample::Sample;

pub struct Criterion {
    pub confidence_level: f64,
    pub measurement_time: uint,
    pub nresamples: uint,
    pub sample_size: u64,
    pub significance_level: f64,
    pub warm_up_time: uint,
}

impl Criterion {
    pub fn new() -> Criterion {
        Criterion {
            confidence_level: 0.95,
            measurement_time: 10,
            nresamples: 100_000,
            sample_size: 100,
            significance_level: 0.05,
            warm_up_time: 1_000,
        }
    }

    pub fn bench<N: Str + ToStr>(&mut self, name: N, f: |&mut Bencher|) {
        local_data_key!(clock: Clock);

        if clock.get().is_none() {
            clock.replace(Some(Clock::new(self)));
        }

        println!("\nbenchmarking {}", name.as_slice());

        let sample = Sample::new(f, self);

        sample.outliers().report();

        let sample = sample.without_outliers();

        sample.estimate(self);

        let dir = Path::new(".criterion").join(name.as_slice());

        if !dir.exists() {
            match fs::mkdir_recursive(&dir, UserRWX) {
                Err(e) => fail!("`mkdir -p {}`: {}", dir.display(), e),
                Ok(_) => {},
            }
        }

        let new_path = dir.join("new.json");
        if new_path.exists() {
            let old_sample = match Sample::load(&new_path) {
                Err(e) => fail!("{}", e),
                Ok(s) => s,
            };

            bootstrap::compare(old_sample.data(), sample.data(), self);

            // TODO add regression test here, fail if regressed

            old_sample.save(&dir.join("old.json"));
            sample.save(&new_path);
        } else {
            match sample.save(&new_path) {
                Err(e) => fail!("Couldn't store sample: {}", e),
                Ok(_) => {},
            }
        }
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
