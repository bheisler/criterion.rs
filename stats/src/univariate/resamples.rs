use std::mem;

use float::Float;
use rand::distributions::{Distribution, Range};
use rand::{rngs::SmallRng, FromEntropy};

use univariate::Sample;

pub struct Resamples<'a, A>
where
    A: 'a + Float,
{
    range: Range<usize>,
    rng: SmallRng,
    sample: &'a [A],
    stage: Option<Vec<A>>,
}

impl<'a, A> Resamples<'a, A>
where
    A: 'a + Float,
{
    pub fn new(sample: &'a Sample<A>) -> Resamples<'a, A> {
        let slice = sample;

        Resamples {
            range: Range::new(0, slice.len()),
            rng: SmallRng::from_entropy(),
            sample: slice,
            stage: None,
        }
    }

    pub fn next(&mut self) -> &Sample<A> {
        let n = self.sample.len();
        let rng = &mut self.rng;

        match self.stage {
            None => {
                let mut stage = Vec::with_capacity(n);

                for _ in 0..n {
                    stage.push(self.sample[self.range.sample(rng)])
                }

                self.stage = Some(stage);
            }
            Some(ref mut stage) => for elem in stage.iter_mut() {
                *elem = self.sample[self.range.sample(rng)]
            },
        }

        if let Some(ref v) = self.stage {
            unsafe { mem::transmute::<&[_], _>(v) }
        } else {
            unreachable!();
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::collections::HashSet;

    use univariate::resamples::Resamples;
    use univariate::Sample;

    // FIXME
    // Check that the resample is a subset of the sample
    quickcheck!{
        fn subset(size: usize, nresamples: usize) -> TestResult {
            if size > 1 {
                let v: Vec<_> = (0..size).map(|i| i as f32).collect();
                let sample = Sample::new(&v);
                let mut resamples = Resamples::new(sample);
                let sample = v.iter().map(|&x| x as i64).collect::<HashSet<_>>();

                TestResult::from_bool((0..nresamples).all(|_| {
                    let resample = resamples.next()

                        .iter()
                        .map(|&x| x as i64)
                        .collect::<HashSet<_>>();

                    resample.is_subset(&sample)
                }))
            } else {
                TestResult::discard()
            }
        }
    }

    // XXX Perhaps add a check that the resamples are different
}
