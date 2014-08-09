use std::rand::{Rng, SeedableRng, XorShiftRng};
use std::rand::distributions::{IndependentSample,Range};

// Generates resamples from a sample, using resampling with replacement
pub struct Resampler<'a, T> {
    range: Range<uint>,
    rng: XorShiftRng,
    sample: &'a [T],
    stage: Option<Vec<T>>
}

impl<'a, T: Clone> Resampler<'a, T> {
    pub fn new(sample: &'a [T]) -> Resampler<'a, T> {
        use std::rand;

        let mut rng = rand::task_rng();
        let seed = [rng.next_u32(), rng.next_u32(), rng.next_u32(), rng.next_u32()];
        Resampler {
            range: Range::new(0, sample.len()),
            rng: SeedableRng::from_seed(seed),
            sample: sample,
            stage: None,
        }
    }

    pub fn next(&mut self) -> &[T] {
        let n = self.sample.len();
        let rng = &mut self.rng;

        match self.stage {
            None => {
                let mut stage = Vec::with_capacity(n);

                for _ in range(0, n) {
                    stage.push(self.sample[self.range.ind_sample(rng)].clone())
                }

                self.stage = Some(stage);
            },
            Some(ref mut stage) => {
                for elem in stage.mut_iter() {
                    *elem = self.sample[self.range.ind_sample(rng)].clone()
                }
            },
        }

        self.stage.get_ref().as_slice()
    }
}
