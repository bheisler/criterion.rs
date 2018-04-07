use float::Float;
use rand::distributions::{IndependentSample, Range};
use rand::{Rng, XorShiftRng};

use bivariate::Data;

pub struct Resamples<'a, X, Y>
where
    X: 'a + Float,
    Y: 'a + Float,
{
    range: Range<usize>,
    rng: XorShiftRng,
    data: (&'a [X], &'a [Y]),
    stage: Option<(Vec<X>, Vec<Y>)>,
}

#[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
impl<'a, X, Y> Resamples<'a, X, Y>
where
    X: 'a + Float,
    Y: 'a + Float,
{
    pub fn new(data: Data<'a, X, Y>) -> Resamples<'a, X, Y> {
        Resamples {
            range: Range::new(0, data.0.len()),
            rng: ::rand::thread_rng().gen(),
            data: (data.x().as_slice(), data.y().as_slice()),
            stage: None,
        }
    }

    pub fn next(&mut self) -> Data<X, Y> {
        let n = self.data.0.len();
        let rng = &mut self.rng;

        match self.stage {
            None => {
                let mut stage = (Vec::with_capacity(n), Vec::with_capacity(n));

                for _ in 0..n {
                    let i = self.range.ind_sample(rng);

                    stage.0.push(self.data.0[i]);
                    stage.1.push(self.data.1[i]);
                }

                self.stage = Some(stage);
            }
            Some(ref mut stage) => for i in 0..n {
                let j = self.range.ind_sample(rng);

                stage.0[i] = self.data.0[j];
                stage.1[i] = self.data.1[j];
            },
        }

        if let Some((ref x, ref y)) = self.stage {
            Data(x, y)
        } else {
            unreachable!();
        }
    }
}
