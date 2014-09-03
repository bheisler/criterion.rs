use std::rand::TaskRng;
use std::rand::distributions::{IndependentSample,Range};

use super::Sample;

// Generates resamples from a sample, using resampling with replacement
pub struct Resampler<'a> {
    range: Range<uint>,
    rng: TaskRng,
    sample: &'a [f64],
    stage: Vec<f64>
}

impl<'a> Resampler<'a> {
    pub fn new<V: Slice<f64>>(sample: &'a Sample<V>) -> Resampler<'a> {
        use std::rand;

        let sample = sample.as_slice();
        let length = sample.len();

        Resampler {
            range: Range::new(0, length),
            rng: rand::task_rng(),
            sample: sample,
            stage: Vec::from_elem(length, 0f64),
        }
    }

    pub fn next<'b>(&'b mut self) -> Sample<&'b [f64]> {
        for elem in self.stage.as_mut_slice().mut_iter() {
            *elem = self.sample[self.range.ind_sample(&mut self.rng)];
        }

        Sample::new(self.stage.as_slice())
    }
}

