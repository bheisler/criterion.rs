use std::time::{Duration, Instant};
use benchmark::BenchmarkConfig;

use {Bencher, Criterion, DurationExt};

/// PRIVATE
pub trait Routine {
    /// PRIVATE
    fn bench(&mut self, iters: &Vec<u64>) -> Vec<f64>;
    /// PRIVATE
    fn warm_up(&mut self, how_long: Duration) -> (u64, u64);

    /// PRIVATE
    fn sample(&mut self, id: &str, config: &BenchmarkConfig, criterion: &Criterion) -> (Box<[f64]>, Box<[f64]>) {
        let wu = config.warm_up_time;
        let m_ns = config.measurement_time.to_nanos();

        criterion.report.warmup(id, wu.to_nanos() as f64);

        let (wu_elapsed, wu_iters) = self.warm_up(wu);

        // Initial guess for the mean execution time
        let met = wu_elapsed as f64 / wu_iters as f64;

        let n = config.sample_size as u64;
        // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns
        let total_runs = n * (n + 1) / 2;
        let d = (m_ns as f64 / met / total_runs as f64).ceil() as u64;

        let m_iters = (1..(n+1) as u64).map(|a| a*d).collect::<Vec<u64>>();

        let m_ns = total_runs as f64 * d as f64 * met;
        criterion.report.measurement_start(id, n, m_ns, m_iters.iter().sum());
        let m_elapsed = self.bench(&m_iters);

        let m_iters_f: Vec<f64> = m_iters.iter().map(|&x| x as f64).collect();

        (m_iters_f.into_boxed_slice(), m_elapsed.into_boxed_slice())
    }
}

pub struct Function<F>(pub F) where F: FnMut(&mut Bencher);

impl<F> Routine for Function<F> where F: FnMut(&mut Bencher) {
    fn bench(&mut self, iters: &Vec<u64>) -> Vec<f64> {
        let Function(ref mut f) = *self;

        let mut b = Bencher { iters: 0, elapsed: Duration::from_secs(0) };

        iters.iter().map(|iters| {
            b.iters = *iters;
            (*f)(&mut b);
            b.elapsed.to_nanos() as f64
        }).collect()
    }

    fn warm_up(&mut self, how_long: Duration) -> (u64, u64) {
        let Function(ref mut f) = *self;
        let mut b = Bencher { iters: 1, elapsed: Duration::from_secs(0) };

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
