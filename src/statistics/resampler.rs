use std::rand::{Rng, SeedableRng, XorShiftRng};
use std::rand::distributions::{IndependentSample,Range};

use super::Sample;

// Generates resamples from a sample, using resampling with replacement
pub struct Resampler<'a> {
    range: Range<uint>,
    rng: XorShiftRng,
    sample: &'a [f64],
    stage: Vec<f64>
}

impl<'a> Resampler<'a> {
    pub fn new<V: Vector<f64>>(sample: &'a Sample<V>) -> Resampler<'a> {
        use std::rand;

        let sample = sample.as_slice();
        let length = sample.len();

        let mut rng = rand::task_rng();
        let seed = [rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32()];
        Resampler {
            range: Range::new(0, length),
            rng: SeedableRng::from_seed(seed),
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

