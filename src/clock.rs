use bencher::BencherConfig;
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

    pub fn new(config: &BencherConfig) -> Clock {
        println!("estimating the cost of precise_time_ns()");

        let m_time = config.measurement_time as u64 * 1.ms();
        let size = config.sample_size;
        let wu_time = config.warm_up_time as u64 * 1.ms();

        println!("> warming up for {} ms", config.warm_up_time);
        let action = || precise_time_ns();
        let (_, wu_iters, action) = run_for_at_least(wu_time, 10_000, action);

        let m_iters = (wu_iters as u64 * m_time / wu_time) as uint;
        let (sample, _) = Sample::new(size, action, m_iters, None);

        let median = sample.median();

        println!("> mean:   {}", sample.mean().as_time());
        println!("> SD:     {}", sample.std_dev().as_time());
        println!("> median: {}", median.as_time());
        println!("> MAD:    {}", sample.median_abs_dev().as_time());

        Clock {
            cost: median,
        }
    }
}
