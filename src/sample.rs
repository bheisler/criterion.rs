use serialize::json;
use std::cmp;

use bencher::Bencher;
use common::run_for_at_least;
use criterion::Criterion;
use file;
use outlier::Outliers;
use units::{AsTime,ToNanoSeconds};

pub struct Sample {
    data: Vec<f64>,
}

impl Sample {
    pub fn new(f: |&mut Bencher|,
               criterion: &Criterion)
        -> Sample
    {
        // XXX `m_time` should have a lower limit based on clock_cost
        let m_time = criterion.measurement_time as u64 * 1.ms();
        let size = criterion.sample_size;
        let wu_time = criterion.warm_up_time as u64 * 1.ms();

        let mut b = Bencher::new();
        println!("> warming up for {} ms", criterion.warm_up_time);
        let (wu_elapsed, wu_iters) = run_for_at_least(wu_time, 1, |x| f(x));

        let m_iters = cmp::max(wu_iters * m_time / wu_time, 1);
        let s_elapsed = (wu_elapsed * m_iters * size) as f64 / wu_iters as f64;

        println!("> collecting {} measurements, {} iters each in estimated {}",
                 size, m_iters, s_elapsed.as_time());

        let mut sample = Vec::from_elem(size as uint, 0f64);
        for measurement in sample.mut_iter() {
            b.bench_n(m_iters, |x| f(x));
            *measurement = b.ns_per_iter();
        }

        Sample {
            data: sample,
        }
    }

    pub fn load(path: &Path) -> Sample {
        match json::decode(file::read(path).as_slice()) {
            Err(e) => fail!("Couldn't decode {}: {}", path.display(), e),
            Ok(v) => Sample { data: v },
        }
    }

    pub fn data<'a>(&'a self) -> &'a [f64] {
        self.data.as_slice()
    }

    pub fn classify_outliers(&self) -> Outliers {
        Outliers::new(self.data())
    }

    pub fn save(&self, path: &Path) {
        file::write(path, json::encode(&self.data()).as_slice());
    }
}
