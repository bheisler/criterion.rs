use std::time::{Duration, Instant};
use BenchmarkConfig;

use {Bencher, Criterion, DurationExt};
use std::marker::PhantomData;
use report::BenchmarkId;

/// PRIVATE
pub trait Routine<T> {
    /// PRIVATE
    fn warm_up(&mut self, how_long: Duration, parameter: &T)
        -> (u64, u64);

    /// PRIVATE
    fn sample(
        &mut self,
        id: &BenchmarkId,
        config: &BenchmarkConfig,
        criterion: &Criterion,
        parameter: &T,
    ) {
        let wu = config.warm_up_time;
        let m_ns = config.measurement_time.to_nanos();

        criterion
            .report
            .warmup(id, wu.to_nanos() as f64);

        let (wu_elapsed, wu_iters) = self.warm_up(wu, parameter);

        // Initial guess for the mean execution time
        let met = wu_elapsed as f64 / wu_iters as f64;

        let n = config.sample_size as u64;
        // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns
        let total_runs = n * (n + 1) / 2;
        let d = (m_ns as f64 / met / total_runs as f64).ceil() as u64;

        let m_iters = (1..(n + 1) as u64).map(|a| a * d).collect::<Vec<u64>>();

        let m_ns = total_runs as f64 * d as f64 * met;
        criterion
            .report
            .measurement_start(id, n, m_ns, m_iters.iter().sum());
    }
}

pub struct Function<F, T>
where
    F: FnMut(&mut Bencher, &T),
{
    f: F,
    _phantom: PhantomData<T>,
}
impl<F, T> Function<F, T>
where
    F: FnMut(&mut Bencher, &T),
{
    pub fn new(f: F) -> Function<F, T> {
        Function {
            f: f,
            _phantom: PhantomData,
        }
    }
}

impl<F, T> Routine<T> for Function<F, T>
where
    F: FnMut(&mut Bencher, &T),
{
    fn warm_up(
        &mut self,
        how_long: Duration,
        parameter: &T,
    ) -> (u64, u64) {
        let f = &mut self.f;
        let mut b = Bencher {
            iters: 1,
            elapsed: Duration::from_secs(0),
        };

        let mut total_iters = 0;
        let start = Instant::now();
        loop {
            (*f)(&mut b, parameter);

            total_iters += b.iters;
            let elapsed = start.elapsed();
            if elapsed > how_long {
                return (elapsed.to_nanos(), total_iters);
            }

            b.iters *= 2;
        }
    }
}
