use bencher::Bencher;
use bootstrap;
use clock::Clock;
use common::run_for_at_least;
use criterion::CriterionConfig;
use outlier::Outliers;
use std::cmp;
use test::stats::Stats;
use units::{AsTime,ToNanoSeconds};

#[deriving(Encodable)]
pub struct Sample {
    data: Vec<f64>,
}

impl Sample {
    pub fn new(f: |&mut Bencher|,
               config: &CriterionConfig,
               clock: Option<Clock>)
        -> Sample
    {
        let m_time = config.measurement_time as u64 * 1.ms();
        let size = config.sample_size;
        let wu_time = config.warm_up_time as u64 * 1.ms();

        let mut b = Bencher::new(clock);
        println!("> warming up for {} ms", config.warm_up_time);
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

    pub fn estimate(&self, config: &CriterionConfig) {
        bootstrap::estimate(self, config.nresamples, config.confidence_level)
    }

    pub fn data<'a>(&'a self) -> &'a [f64] {
        self.data.as_slice()
    }

    pub fn into_data(self) -> Vec<f64> {
        self.data
    }

    pub fn median(&self) -> f64 {
        self.data.as_slice().median()
    }

    pub fn outliers(&self) -> Outliers {
        Outliers::new(self)
    }

    pub fn quartiles(&self) -> (f64, f64, f64) {
        self.data.as_slice().quartiles()
    }

    // remove severe outliers using the IQR criteria
    pub fn without_outliers(&self) -> Sample {
        let (q1, _, q3) = self.quartiles();
        let iqr = q3 - q1;
        let (lb, ub) = (q1 - 3.0 * iqr, q3 + 3.0 * iqr);

        let data: Vec<f64> = self.data.iter().filter_map(|&x| {
            if x > lb && x < ub {
                Some(x)
            } else {
                None
            }
        }).collect();

        Sample {
            data: data,
        }
    }
}

impl Collection for Sample {
    fn len(&self) -> uint {
        self.data.len()
    }
}
