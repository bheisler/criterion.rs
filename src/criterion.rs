use serialize::json;
use std::fmt::Show;

use analyze;
use bencher::Bencher;
use clock::Clock;
use file;
use fs;
use plot;
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

        let cl = self.confidence_level;
        let name = name.as_slice();
        let nresamples = self.nresamples;

        if clock.get().is_none() {
            clock.replace(Some(Clock::new(self)));
        }

        println!("\nbenchmarking {}", name);

        let sample = Sample::new(f, self);

        let base_dir = Path::new(".criterion").join(name);
        let new_dir = base_dir.join("new");
        let new_data = new_dir.join("data.json");
        let old_dir = base_dir.join("old");

        if old_dir.exists() { fs::rmrf(&old_dir) }

        if new_dir.exists() { fs::mv(&new_dir, &old_dir) }

        fs::mkdirp(&new_dir);
        sample.save(&new_data);

        plot::kde(sample.data(), &new_dir.join("dirty"));

        let outliers = sample.classify_outliers();
        outliers.save(&new_dir);
        outliers.report();

        // TODO plot outliers

        let sample = outliers.normal();
        let estimates = analyze::estimate_statistics(sample, nresamples, cl);

        file::write(&new_dir.join("statistics.json"),
                    json::encode(&estimates).as_slice());

        plot::kde(sample, &new_dir.join("clean"));

        if !old_dir.exists() {
            return self;
        }

        let old_sample =
            Sample::load(&old_dir.join("data.json")).classify_outliers();
        let old_sample = old_sample.normal();

        let estimates =
            analyze::compare_samples(old_sample, sample, nresamples, cl);

        let diff_dir = base_dir.join("diff");
        fs::mkdirp(&diff_dir);
        file::write(&diff_dir.join("statistics.json"),
                    json::encode(&estimates).as_slice());

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
            self.bench(format!("{}/{}", group, input), |x| {
                f(x, input.clone())
            });
        }

        // TODO Summary analysis

        self
    }
}
