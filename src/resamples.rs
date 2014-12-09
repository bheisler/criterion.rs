use std::rand::{Rng, XorShiftRng, mod};
use std::rand::distributions::{IndependentSample, Range};

pub struct Resamples<'a, A> where A: 'a {
    range: Range<uint>,
    rng: XorShiftRng,
    sample: &'a [A],
    stage: Option<Vec<A>>,
}

impl <'a, A> Resamples<'a, A> where A: Clone {
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
                for elem in stage.iter_mut() {
                    *elem = self.sample[self.range.ind_sample(rng)].clone()
                }
            },
        }

        self.stage.as_ref().unwrap()[]
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::collections::TreeSet;

    use super::Resamples;
    use test;

    // Check that the resample is a subset of the sample
    #[quickcheck]
    fn subset(size: uint, nresamples: uint) -> TestResult {
        if let Some(sample) = test::vec::<int>(size) {
            let mut resamples = Resamples::new(sample[]);
            let sample = sample.iter().map(|&x| x).collect::<TreeSet<_>>();

            TestResult::from_bool(range(0, nresamples).all(|_| {
                let resample = resamples.next().iter().map(|&x| x).collect::<TreeSet<_>>();

                resample.is_subset(&sample)
            }))
        } else {
            TestResult::discard()
        }
    }

    // XXX Perhaps add a check that the resamples are different
}
