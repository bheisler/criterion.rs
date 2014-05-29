use common::run_for_at_least;
use sample::Sample;
use time::precise_time_ns;
use units::{AsTime,ToNanoSeconds};

pub struct Clock {
    cost: f64,
}

impl Clock {
    pub fn cost(&self) -> f64 {
        self.cost as f64
    }

    pub fn new() -> Clock {
        let action = || precise_time_ns();
        let (_, iters, action) = run_for_at_least(10.ms(), 10_000, action);

        println!("estimating the cost of precise_time_ns()");
        let (sample, _) = Sample::new(100, action, iters, None);

        let mean = sample.mean();
        let median = sample.median();
        let std_dev = sample.std_dev();

        println!("> mean:    {}", mean.as_time());
        println!("> median:  {}", median.as_time());
        println!("> std dev: {}", std_dev.as_time());

        Clock {
            cost: median,
        }
    }
}
