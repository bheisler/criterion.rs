use std::time::{Duration, Instant};
use {Bencher, DurationExt};

/// PRIVATE
pub trait Routine {
    /// PRIVATE
    fn warm_up(&mut self, how_long: Duration)
        -> (u64, u64);
}

pub struct Function<F>
where
    F: FnMut(&mut Bencher),
{
    f: F,
}
impl<F> Function<F>
where
    F: FnMut(&mut Bencher),
{
    pub fn new(f: F) -> Function<F> {
        Function {
            f: f,
        }
    }
}

impl<F> Routine for Function<F>
where
    F: FnMut(&mut Bencher),
{
    fn warm_up(
        &mut self,
        how_long: Duration,
    ) -> (u64, u64) {
        let f = &mut self.f;
        let mut b = Bencher {
            iters: 1,
            elapsed: Duration::from_secs(0),
        };

        let mut total_iters = 0;
        let start = Instant::now();
        loop {
            (*f)(&mut b);

            total_iters += b.iters;
            let elapsed = start.elapsed();
            if elapsed > how_long {
                return (elapsed.to_nanos(), total_iters);
            }

            b.iters *= 2;
        }
    }
}
