use test::stats::Stats;
use time;

use bencher::Bencher;
use criterion::Criterion;
use sample::Sample;

pub struct Clock {
    cost: f64,
}

impl Clock {
    pub fn cost(&self) -> f64 {
        self.cost as f64
    }

    pub fn new(criterion: &Criterion) -> Clock {
        println!("estimating the cost of precise_time_ns()");

        // XXX Is it worth to do a more complex analysis of this sample?
        let sample = Sample::new(clock_cost, criterion);
        let median = sample.data().median();

        println!("> median: {}", median);

        Clock {
            cost: median,
        }
    }
}

fn clock_cost(b: &mut Bencher) {
    b.iter(|| time::precise_time_ns())
}
