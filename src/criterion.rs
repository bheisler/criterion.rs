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
    pub warm_up_time: uint,
}

impl Criterion {
    // XXX What would be a good default?
    // XXX Should this be named `new` or `default`?
    pub fn default() -> Criterion {
        Criterion {
            confidence_level: 0.95,
            measurement_time: 10,
            nresamples: 100_000,
            sample_size: 100,
            warm_up_time: 1_000,
        }
    }

    pub fn confidence_level<'a>(&'a mut self, cl: f64) -> &'a mut Criterion {
        self.confidence_level = cl;

        self
    }

    pub fn measurement_time<'a>(&'a mut self, t: uint) -> &'a mut Criterion {
        self.measurement_time = t;

        self
    }

    pub fn nresamples<'a>(&'a mut self, n: uint) -> &'a mut Criterion {
        self.nresamples = n;

        self
    }

    pub fn sample_size<'a>(&'a mut self, n: u64) -> &'a mut Criterion {
        self.sample_size = n;

        self
    }

    pub fn warm_up_time<'a>(&'a mut self, ms: uint) -> &'a mut Criterion {
        self.warm_up_time = ms;

        self
    }

    pub fn bench<'a,
                 N: Str + ToStr>(
                 &'a mut self,
                 name: N,
                 f: |&mut Bencher|)
                 -> &'a mut Criterion {
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

        self
    }

    pub fn bench_group<'a,
                       G: Show,
                       I: Clone + Show>(
                       &'a mut self,
                       group: G,
                       inputs: &[I],
                       f: |&mut Bencher, I|)
                       -> &'a mut Criterion {
        for input in inputs.iter() {
            self.bench(format!("{}_{}", group, input), |x| {
                f(x, input.clone())
            });
        }

        self
    }
}
