use std::rand::TaskRng;
use std::rand::distributions::{IndependentSample,Range};
use std::rand;

pub struct Resamples<'a> {
    range: Range<uint>,
    rng: TaskRng,
    sample: &'a [f64],
    stage: Vec<f64>,
}

impl<'a> Resamples<'a> {
    pub fn new(sample: &'a [f64]) -> Resamples<'a> {
        let size = sample.len();

        Resamples {
            range: Range::new(0, size),
            rng: rand::task_rng(),
            sample: sample,
            stage: Vec::from_elem(size, 0f64),
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
