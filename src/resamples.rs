use std::rand::{Rng, XorShiftRng, mod};
use std::rand::distributions::{IndependentSample, Range};

pub struct Resamples<'a, A> {
    range: Range<uint>,
    rng: XorShiftRng,
    sample: &'a [A],
    stage: Option<Vec<A>>,
}

impl <'a, A: Clone> Resamples<'a, A> {
    pub fn new(sample: &'a [A]) -> Resamples<'a, A> {
        let mut rng = rand::task_rng();

        Resamples {
            range: Range::new(0, sample.len()),
            rng: rng.gen(),
            sample: sample,
            stage: None,
        }
    }

    pub fn next(&mut self) -> &[A] {
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

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::collections::TreeSet;
    use std::rand::{Rng, mod};

    use super::Resamples;

    // Check that the resample is a subset of the sample
    #[quickcheck]
    fn subset(sample_size: uint, nresamples: uint) -> TestResult {
        let sample = if sample_size > 1 {
            let mut rng = rand::task_rng();

            Vec::from_fn(sample_size, |_| rng.gen::<int>())
        } else {
            return TestResult::discard();
        };

        let mut resamples = Resamples::new(sample.as_slice());
        let sample: TreeSet<int> = sample.iter().map(|&x| x).collect();

        TestResult::from_bool(range(0, nresamples).all(|_| {
            let resample: TreeSet<int> = resamples.next().iter().map(|&x| x).collect();

            resample.is_subset(&sample)
        }))
    }

    // XXX Perhaps add a check that the resamples are different
}
