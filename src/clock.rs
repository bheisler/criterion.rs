use time::precise_time_ns;

use bencher::Bencher;
use criterion::CriterionConfig;
use sample::Sample;

pub struct Clock {
    cost: f64,
}

impl Clock {
    pub fn cost(&self) -> f64 {
        self.cost as f64
    }

    pub fn new(config: &CriterionConfig) -> Clock {
        println!("estimating the cost of precise_time_ns()");

        let sample = Sample::new(clock_cost, config);

        sample.outliers().report();

        let sample = sample.without_outliers();

        sample.estimate(config);

        Clock {
            cost: sample.median(),
        }
    }
}

fn clock_cost(b: &mut Bencher) {
    b.iter(|| precise_time_ns())
}
