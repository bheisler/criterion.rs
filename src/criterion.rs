use serialize::json;
use std::fmt::Show;
use std::io::{fs,File,UserRWX};

use bencher::Bencher;
use bootstrap;
use clock::Clock;
use math;
use sample::Sample;
use simplot::Figure;

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

        let name = name.as_slice();

        if clock.get().is_none() {
            clock.replace(Some(Clock::new(self)));
        }

        println!("\nbenchmarking {}", name);

        let sample = Sample::new(f, self);

        // FIXME Don't remove outliers for now
        //let sample = sample.without_outliers();

        let base_dir = Path::new(".criterion").join(name);
        let new_dir = base_dir.join("new");
        let new_dir_ = new_dir.display();
        let new_data = new_dir.join("data.json");
        let new_data_ = new_data.display();

        if new_dir.exists() {
            let old_sample = match Sample::load(&new_data) {
                Err(e) => fail!("{}", e),
                Ok(s) => s,
            };

            bootstrap::compare(old_sample.data(), sample.data(), self);

            let old_dir = base_dir.join("old");
            let old_dir_ = old_dir.display();

            if old_dir.exists() {
                match fs::rmdir_recursive(&old_dir) {
                    Err(e) => fail!("`rm -rf {}: {}`", old_dir_, e),
                    Ok(_) => {},
                }
            }

            match fs::rename(&new_dir, &old_dir) {
                Err(e) => fail!("`mv {} {}`: {}", new_dir_, old_dir_, e),
                Ok(_) => {},
            }

            // TODO add regression test here, fail if regressed
        }

        match fs::mkdir_recursive(&new_dir, UserRWX) {
            Err(e) => fail!("`mkdir -p {}`: {}", new_dir_, e),
            Ok(_) => {},
        }

        match sample.save(&new_data) {
            Err(e) => fail!("Couldn't save {}: {}", new_data_, e),
            Ok(_) => {},
        }

        sample.outliers().report();

        let estimates = sample.estimate(self);

        let new_estimates = new_dir.join("estimates.json");
        let new_estimates_ = new_estimates.display();
        match File::create(&new_estimates) {
            Err(e) => fail!("Couldn't create {}: {}", new_estimates_, e),
            Ok(mut f) => match f.write_str(json::encode(&estimates).as_slice())
            {
                Err(e) => fail!("Couldn't write {}: {}", new_estimates_, e),
                Ok(_) => {},
            },
        }

        let (xs, ys) = math::kde(sample.data());
        // XXX should the size of the image be configurable?
        Figure::new().
            set_output_file(new_dir.join("kde.png")).
            set_size((1366, 768)).
            plot(xs.iter(), ys.iter()).
            draw();

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

        self
    }
}
