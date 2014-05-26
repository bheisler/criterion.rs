use rand::distributions::IndependentSample;
use rand::distributions::range::Range;
use rand::{TaskRng,task_rng};
use sample::Sample;
use test::stats::Stats;
use units::AsTime;

// XXX for debugging purposes, remove later
//use std::io::{File,Truncate,Write};
//use serialize::json::ToJson;

pub struct Estimate {
    confidence_level: f64,
    lower_bound: f64,
    point: f64,
    standard_error: f64,
    upper_bound: f64,
}

impl Estimate {
    fn new(point: f64, bootstrap: &[f64], cl: f64) -> Estimate {
        let standard_error = bootstrap.std_dev();
        let lower_bound = bootstrap.percentile(50.0 * (1.0 - cl));
        let upper_bound = bootstrap.percentile(50.0 * (1.0 + cl));

        Estimate {
            confidence_level: cl,
            lower_bound: lower_bound,
            point: point,
            standard_error: standard_error,
            upper_bound: upper_bound,
        }
    }
}

impl Estimate {
    fn report(&self) -> String {
        format!("{} ± {} [{} {}] {}% CI",
                self.point.as_time(),
                self.standard_error.as_time(),
                self.lower_bound.as_time(),
                self.upper_bound.as_time(),
                self.confidence_level * 100.0)
    }
}

pub struct Bootstrap {
    iters: uint,
    mean: Estimate,
    median: Estimate,
    nresamples: uint,
    sample_size: uint,
    std_dev: Estimate,
}

impl Bootstrap {
    pub fn new(sample: &Sample,
               nresamples: uint,
               cl: f64)
        -> Bootstrap
    {
        assert!(cl > 0.0 && cl < 1.0,
                "confidence level must be between 0.0 and 1.0");

        println!("> bootstrapping sample with {} resamples", nresamples);


        let mut means = Vec::with_capacity(nresamples);
        let mut medians = Vec::with_capacity(nresamples);
        let mut std_devs = Vec::with_capacity(nresamples);

        let mut resamples = Resamples::new(sample.data());
        for _ in range(0, nresamples) {
            let resample = resamples.next();

            means.push(resample.mean());
            medians.push(resample.median());
            std_devs.push(resample.std_dev());
        }

        // XXX for debugging purposes, remove later
        //match File::open_mode(&Path::new("b-mean.json"), Truncate, Write) {
            //Err(_) => fail!("couldn't open b-mean.json"),
            //Ok(mut file) => {
                //match file.write_str(means.to_json().to_str().as_slice()) {
                    //Err(_) => fail!("couldn't write b-mean.json"),
                    //Ok(_) => {},
                //}
            //}
        //}

        let mean = Estimate::new(sample.mean(), means.as_slice(), cl);
        let median = Estimate::new(sample.median(), medians.as_slice(), cl);
        let std_dev = Estimate::new(sample.std_dev(), std_devs.as_slice(), cl);

        Bootstrap {
            iters: sample.iters(),
            mean: mean,
            median: median,
            nresamples: nresamples,
            sample_size: sample.len(),
            std_dev: std_dev,
        }
    }

    pub fn report(&self) {
        println!("  > mean:    {}", self.mean.report());
        println!("  > median:  {}", self.median.report());
        println!("  > std_dev: {}", self.std_dev.report());
    }
}

struct Resamples<'a> {
    range: Range<uint>,
    rng: TaskRng,
    sample: &'a [f64],
    stage: Vec<f64>,
}

impl<'a> Resamples<'a> {
    pub fn new(sample: &'a [f64]) -> Resamples<'a> {
        let size = sample.len();

        Resamples {
            range: Range::new(0, size - 1),
            rng: task_rng(),
            sample: sample,
            stage: Vec::from_elem(size, 0.0),
        }
    }

    pub fn next<'b>(&'b mut self) -> &'b [f64] {
        let size = self.sample.len();

        // resampling *with* replacement
        for i in range(0, size) {
            let j = self.range.ind_sample(&mut self.rng);

            self.stage.as_mut_slice()[i] = self.sample[j];
        }

        self.stage.as_slice()
    }
}
